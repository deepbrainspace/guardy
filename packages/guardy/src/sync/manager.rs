use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::fs;

use crate::config::GuardyConfig;
use crate::git::remote::RemoteOperations;
use super::{SyncConfig, SyncStatus, SyncRepo};
use super::protection::ProtectionManager;

pub struct SyncManager {
    config: SyncConfig,
    cache_dir: PathBuf,
    remote_ops: RemoteOperations,
    pub protection_manager: ProtectionManager,
}

impl SyncManager {
    pub fn with_config(sync_config: SyncConfig) -> Result<Self> {
        // Create cache directory in .guardy/cache/
        let cache_dir = PathBuf::from(".guardy/cache");
        std::fs::create_dir_all(&cache_dir)?;

        let remote_ops = RemoteOperations::new(cache_dir.clone());
        let protection_manager = ProtectionManager::new(sync_config.protection.clone())?;

        Ok(Self {
            config: sync_config,
            cache_dir,
            remote_ops,
            protection_manager,
        })
    }

    /// Bootstrap sync from a repository URL and version (for initial setup)
    pub fn bootstrap(repo_url: &str, version: &str) -> Result<Self> {
        // For bootstrap, we create a minimal config
        let sync_repo = SyncRepo {
            name: "bootstrap".to_string(),
            repo: repo_url.to_string(),
            version: version.to_string(),
            source_path: ".".to_string(),
            dest_path: ".".to_string(),
            include: vec!["*".to_string()],
            exclude: vec![".git/".to_string(), "target/".to_string()],
            protected: true,
        };

        let sync_config = SyncConfig {
            repos: vec![sync_repo],
            protection: Default::default(),
        };

        let cache_dir = PathBuf::from(".guardy/cache");
        std::fs::create_dir_all(&cache_dir)?;

        let remote_ops = RemoteOperations::new(cache_dir.clone());
        let protection_manager = ProtectionManager::new(sync_config.protection.clone())?;

        Ok(Self {
            config: sync_config,
            cache_dir,
            remote_ops,
            protection_manager,
        })
    }

    pub fn check_sync_status(&self) -> Result<SyncStatus> {
        if self.config.repos.is_empty() {
            return Ok(SyncStatus::NotConfigured);
        }

        tracing::info!("Checking sync status for {} repositories", self.config.repos.len());
        
        let mut changed_files = Vec::new();
        
        for repo in &self.config.repos {
            // Check if repository exists in cache
            let repo_name = self.extract_repo_name(&repo.repo);
            let repo_path = self.cache_dir.join(&repo_name);
            
            if !repo_path.exists() {
                // Repository not cached - all files are out of sync
                let dest_path = Path::new(&repo.dest_path);
                if dest_path.exists() {
                    // Find files that would be synced from this repo
                    self.collect_sync_files(repo, dest_path, &mut changed_files)?;
                }
                continue;
            }
            
            // Compare files between cache and destination
            self.check_repo_sync_status(repo, &repo_path, &mut changed_files)?;
        }
        
        if changed_files.is_empty() {
            Ok(SyncStatus::InSync)
        } else {
            Ok(SyncStatus::OutOfSync { changed_files })
        }
    }

    pub fn update_all_repos(&mut self, force: bool) -> Result<()> {
        if self.config.repos.is_empty() {
            return Err(anyhow!("No repositories configured for sync"));
        }

        tracing::info!("Updating {} repositories (force: {})", self.config.repos.len(), force);
        
        // Check if we should block modifications to protected files
        if !force && self.protection_manager.should_block_modifications() {
            // Collect all files that will be modified
            let mut files_to_modify = Vec::new();
            for repo in &self.config.repos {
                let repo_path = self.remote_ops.clone_or_fetch(&repo.repo, &repo.version)?;
                let potential_overwrites = self.get_files_to_overwrite(repo, &repo_path)?;
                files_to_modify.extend(potential_overwrites);
            }
            
            // Validate that no protected files will be modified
            self.protection_manager.validate_modifications(&files_to_modify)?;
        }

        for repo in &self.config.repos {
            tracing::info!("Syncing repo '{}' from '{}' version '{}'", 
                         repo.name, repo.repo, repo.version);
            
            // Clone/fetch the repository to cache
            let repo_path = self.remote_ops.clone_or_fetch(&repo.repo, &repo.version)?;
            
            tracing::info!("Repository cached at: {}", repo_path.display());
            
            // Backup existing files before overwriting (if force mode)
            if force {
                let dest_path = Path::new(&repo.dest_path);
                if dest_path.exists() {
                    let files_to_backup = self.get_files_to_overwrite(repo, &repo_path)?;
                    if !files_to_backup.is_empty() {
                        tracing::info!("Backing up {} files before sync", files_to_backup.len());
                        self.protection_manager.backup_before_sync(&files_to_backup)?;
                    }
                }
            }
            
            // Copy files from source to destination and protect them
            let synced_files = self.copy_repo_files(repo, &repo_path)?;
            
            // Protect synced files if configured
            self.protection_manager.protect_synced_files(repo, synced_files)?;
        }

        Ok(())
    }

    pub fn show_status(&self) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("Sync Configuration:\n");
        output.push_str(&format!("  Repositories: {}\n", self.config.repos.len()));
        output.push_str(&format!("  Cache Directory: {}\n", self.cache_dir.display()));
        output.push_str(&format!("  Auto Protect: {}\n", self.config.protection.auto_protect_synced));
        
        if self.config.repos.is_empty() {
            output.push_str("\n❌ No repositories configured\n");
            output.push_str("Run 'guardy sync update --repo=<url> --version=<version>' to bootstrap\n");
        } else {
            output.push_str("\nConfigured Repositories:\n");
            for repo in &self.config.repos {
                output.push_str(&format!("  • {} ({}@{})\n", repo.name, repo.repo, repo.version));
            }
        }

        Ok(output)
    }


    /// Copy files from repository to destination with include/exclude patterns
    fn copy_repo_files(&self, repo: &SyncRepo, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let source_path = repo_path.join(&repo.source_path);
        let dest_path = Path::new(&repo.dest_path);

        tracing::info!("Copying files from {} to {}", 
                      source_path.display(), dest_path.display());

        if !source_path.exists() {
            return Err(anyhow!("Source path '{}' does not exist in repository", 
                              repo.source_path));
        }

        // Build globset for include/exclude patterns
        let include_set = self.build_globset(&repo.include)?;
        let exclude_set = self.build_globset(&repo.exclude)?;

        // Track synced files
        let mut synced_files = Vec::new();

        // Walk through source directory and copy matching files
        self.copy_directory_recursive(&source_path, dest_path, &include_set, &exclude_set, &mut synced_files)?;

        Ok(synced_files)
    }

    /// Build a globset from pattern strings
    fn build_globset(&self, patterns: &[String]) -> Result<globset::GlobSet> {
        let mut builder = globset::GlobSetBuilder::new();
        
        for pattern in patterns {
            let glob = globset::Glob::new(pattern)
                .map_err(|e| anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
            builder.add(glob);
        }
        
        builder.build()
           .map_err(|e| anyhow!("Failed to build globset: {}", e))
    }

    /// Recursively copy directory contents with pattern matching
    fn copy_directory_recursive(
        &self,
        source: &Path,
        dest: &Path,
        include_set: &globset::GlobSet,
        exclude_set: &globset::GlobSet,
        synced_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if source.is_file() {
            return self.copy_single_file(source, dest, include_set, exclude_set, synced_files);
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest.join(&file_name);

            if entry_path.is_dir() {
                // Create destination directory if it doesn't exist
                if !dest_path.exists() {
                    fs::create_dir_all(&dest_path)?;
                }
                
                // Recursively copy directory contents
                self.copy_directory_recursive(&entry_path, &dest_path, include_set, exclude_set, synced_files)?;
            } else {
                self.copy_single_file(&entry_path, &dest_path, include_set, exclude_set, synced_files)?;
            }
        }

        Ok(())
    }

    /// Copy a single file if it matches include/exclude patterns
    fn copy_single_file(
        &self,
        source: &Path,
        dest: &Path,
        include_set: &globset::GlobSet,
        exclude_set: &globset::GlobSet,
        synced_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let file_name = source.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Check exclude patterns first
        if !exclude_set.is_empty() && exclude_set.is_match(file_name) {
            tracing::debug!("Excluding file: {}", source.display());
            return Ok(());
        }

        // Check include patterns (if any are specified)
        if !include_set.is_empty() && !include_set.is_match(file_name) {
            tracing::debug!("File not included: {}", source.display());
            return Ok(());
        }

        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the file
        tracing::debug!("Copying file: {} -> {}", source.display(), dest.display());
        fs::copy(source, dest)?;
        
        // Track the synced file
        synced_files.push(dest.to_path_buf());

        Ok(())
    }

    pub fn parse_sync_config(guardy_config: &GuardyConfig) -> Result<SyncConfig> {
        // Try to load sync configuration from guardy config
        let sync_config = match guardy_config.get_section("sync") {
            Ok(sync_section) => {
                serde_json::from_value(sync_section).unwrap_or_default()
            }
            Err(_) => SyncConfig::default(),
        };

        Ok(sync_config)
    }

    /// Get list of files that will be overwritten during sync
    fn get_files_to_overwrite(&self, repo: &SyncRepo, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let source_path = repo_path.join(&repo.source_path);
        let dest_path = Path::new(&repo.dest_path);
        
        if !source_path.exists() || !dest_path.exists() {
            return Ok(Vec::new());
        }
        
        let include_set = self.build_globset(&repo.include)?;
        let exclude_set = self.build_globset(&repo.exclude)?;
        
        let mut existing_files = Vec::new();
        find_existing_files(&source_path, dest_path, &include_set, &exclude_set, &mut existing_files)?;
        
        Ok(existing_files)
    }

    /// Extract repository name from URL for caching
    fn extract_repo_name(&self, repo_url: &str) -> String {
        repo_url
            .trim_end_matches(".git")
            .split('/')
            .next_back()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Check sync status for a single repository
    fn check_repo_sync_status(&self, repo: &SyncRepo, repo_path: &Path, changed_files: &mut Vec<PathBuf>) -> Result<()> {
        let source_path = repo_path.join(&repo.source_path);
        let dest_path = Path::new(&repo.dest_path);

        if !source_path.exists() {
            return Ok(()); // Nothing to sync
        }

        // Build pattern matchers
        let include_set = self.build_globset(&repo.include)?;
        let exclude_set = self.build_globset(&repo.exclude)?;

        // Compare files
        self.compare_directories(&source_path, dest_path, &include_set, &exclude_set, changed_files)?;

        Ok(())
    }

    /// Collect files that would be synced from a repository
    fn collect_sync_files(&self, repo: &SyncRepo, dest_path: &Path, changed_files: &mut Vec<PathBuf>) -> Result<()> {
        if !dest_path.exists() {
            return Ok(());
        }

        // Build pattern matchers
        let include_set = self.build_globset(&repo.include)?;
        let exclude_set = self.build_globset(&repo.exclude)?;

        // Walk through destination and collect files that would be synced
        collect_matching_files(dest_path, &include_set, &exclude_set, changed_files)?;
        
        Ok(())
    }

    /// Compare source and destination directories for differences
    fn compare_directories(
        &self,
        source: &Path,
        dest: &Path,
        include_set: &globset::GlobSet,
        exclude_set: &globset::GlobSet,
        changed_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if source.is_file() {
            return self.compare_single_file(source, dest, include_set, exclude_set, changed_files);
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest.join(&file_name);

            if entry_path.is_dir() {
                if dest_path.exists() {
                    self.compare_directories(&entry_path, &dest_path, include_set, exclude_set, changed_files)?;
                } else {
                    // Directory doesn't exist in destination - mark as changed
                    changed_files.push(dest_path);
                }
            } else {
                self.compare_single_file(&entry_path, &dest_path, include_set, exclude_set, changed_files)?;
            }
        }

        Ok(())
    }

    /// Compare a single file between source and destination
    fn compare_single_file(
        &self,
        source: &Path,
        dest: &Path,
        include_set: &globset::GlobSet,
        exclude_set: &globset::GlobSet,
        changed_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let file_name = source.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Check exclude patterns first
        if !exclude_set.is_empty() && exclude_set.is_match(file_name) {
            return Ok(());
        }

        // Check include patterns (if any are specified)
        if !include_set.is_empty() && !include_set.is_match(file_name) {
            return Ok(());
        }

        // Check if file exists and has different content
        if !dest.exists() {
            changed_files.push(dest.to_path_buf());
        } else {
            // Compare file contents using metadata first (size and modification time)
            let source_metadata = fs::metadata(source)?;
            let dest_metadata = fs::metadata(dest)?;

            if source_metadata.len() != dest_metadata.len() {
                changed_files.push(dest.to_path_buf());
            } else {
                // For more accurate comparison, we could compare file contents
                // but for now, size comparison is sufficient for most cases
                // TODO: Add optional content hash comparison for accuracy
            }
        }

        Ok(())
    }
}

/// Find files that already exist in destination (free function for recursion)
fn find_existing_files(
    source: &Path,
    dest: &Path,
    include_set: &globset::GlobSet,
    exclude_set: &globset::GlobSet,
    existing_files: &mut Vec<PathBuf>,
) -> Result<()> {
    if source.is_file() {
        if dest.exists() {
            existing_files.push(dest.to_path_buf());
        }
        return Ok(());
    }
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);
        
        if entry_path.is_dir() {
            if dest_path.exists() {
                find_existing_files(&entry_path, &dest_path, include_set, exclude_set, existing_files)?;
            }
        } else {
            let file_name_str = file_name.to_str().unwrap_or("");
            
            // Check patterns
            if !exclude_set.is_empty() && exclude_set.is_match(file_name_str) {
                continue;
            }
            if !include_set.is_empty() && !include_set.is_match(file_name_str) {
                continue;
            }
            
            if dest_path.exists() {
                existing_files.push(dest_path);
            }
        }
    }
    
    Ok(())
}

/// Collect files matching include/exclude patterns (free function for recursion)
fn collect_matching_files(
    path: &Path,
    include_set: &globset::GlobSet,
    exclude_set: &globset::GlobSet,
    changed_files: &mut Vec<PathBuf>,
) -> Result<()> {
    if path.is_file() {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Check if file matches patterns
        if !exclude_set.is_empty() && exclude_set.is_match(file_name) {
            return Ok(());
        }

        if !include_set.is_empty() && !include_set.is_match(file_name) {
            return Ok(());
        }

        changed_files.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_matching_files(&entry.path(), include_set, exclude_set, changed_files)?;
        }
    }

    Ok(())
}