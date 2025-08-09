use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::fs;
use ignore::WalkBuilder;

use crate::config::GuardyConfig;
use crate::git::remote::RemoteOperations;
use super::{SyncConfig, SyncStatus, SyncRepo};

pub struct SyncManager {
    pub config: SyncConfig,
    cache_dir: PathBuf,
    remote_ops: RemoteOperations,
}

impl SyncManager {
    pub fn with_config(sync_config: SyncConfig) -> Result<Self> {
        let cache_dir = PathBuf::from(".guardy/cache");
        std::fs::create_dir_all(&cache_dir)?;
        let remote_ops = RemoteOperations::new(cache_dir.clone());

        Ok(Self { 
            config: sync_config, 
            cache_dir, 
            remote_ops 
        })
    }

    pub fn bootstrap(repo_url: &str, version: &str) -> Result<Self> {
        let sync_repo = SyncRepo {
            name: "bootstrap".to_string(), 
            repo: repo_url.to_string(), 
            version: version.to_string(),
            source_path: ".".to_string(), 
            dest_path: ".".to_string(),
            include: vec!["*".to_string()], 
            exclude: vec![".git".to_string()],
        };
        Self::with_config(SyncConfig { 
            repos: vec![sync_repo]
        })
    }

    /// Parse sync config from GuardyConfig
    pub fn parse_sync_config(config: &GuardyConfig) -> Result<SyncConfig> {
        let sync_value = config.get_section("sync")
            .map_err(|_| anyhow!("No sync configuration found"))?;
        
        let sync_config: SyncConfig = serde_json::from_value(sync_value)
            .map_err(|e| anyhow!("Failed to parse sync configuration: {}", e))?;
        
        Ok(sync_config)
    }

    /// Get files matching patterns using ignore crate  
    fn get_files(&self, source: &Path, repo: &SyncRepo) -> Result<Vec<PathBuf>> {
        let mut builder = WalkBuilder::new(source);
        
        // Disable automatic ignore file discovery - only use our custom patterns
        builder.standard_filters(false);
        
        // Create syncignore file in .guardy/ directory for patterns
        let syncignore_file = if !repo.exclude.is_empty() {
            let ignore_file = self.cache_dir.join(".syncignore");
            fs::write(&ignore_file, repo.exclude.join("\n"))?;
            // Copy the syncignore file to the source directory temporarily
            let source_ignore = source.join(".syncignore");
            fs::copy(&ignore_file, &source_ignore)?;
            builder.add_custom_ignore_filename(".syncignore");
            Some((ignore_file, source_ignore))
        } else {
            None
        };
        
        let result = builder.build()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| entry.path().strip_prefix(source).ok().map(|p| p.to_path_buf()))
            .filter(|path| path.file_name() != Some(".syncignore".as_ref())) // Filter out temp file
            .collect();
            
        // Cleanup syncignore files
        if let Some((ignore_file, source_ignore)) = syncignore_file {
            let _ = fs::remove_file(ignore_file);
            let _ = fs::remove_file(source_ignore);
        }
        
        Ok(result)
    }

    /// Simple file operations
    fn copy_files(&self, files: &[PathBuf], src: &Path, dst: &Path) -> Result<Vec<PathBuf>> {
        files.iter().map(|f| {
            let src_file = src.join(f);
            let dst_file = dst.join(f);
            if let Some(parent) = dst_file.parent() { 
                fs::create_dir_all(parent)?; 
            }
            fs::copy(&src_file, &dst_file)?;
            Ok(dst_file)
        }).collect()
    }

    fn files_differ(&self, files: &[PathBuf], src: &Path, dst: &Path) -> Vec<PathBuf> {
        files.iter().filter_map(|f| {
            let src_file = src.join(f);
            let dst_file = dst.join(f);
            if !dst_file.exists() || 
               fs::metadata(&src_file).ok()?.len() != fs::metadata(&dst_file).ok()?.len() {
                Some(dst_file)
            } else { 
                None 
            }
        }).collect()
    }

    /// Update cache from remote repository using git pull
    fn update_cache(&self, repo: &SyncRepo) -> Result<PathBuf> {
        let repo_name = self.extract_repo_name(&repo.repo);
        let repo_path = self.cache_dir.join(&repo_name);

        if !repo_path.exists() {
            // Clone if doesn't exist
            self.remote_ops.clone_repository(&repo.repo, &repo_name)?;
        }

        // Always fetch and reset to match remote
        self.remote_ops.fetch_and_reset(&repo_name, &repo.version)?;
        
        Ok(repo_path)
    }

    /// Copy repository files from cache to destination
    fn copy_repo_files(&self, repo: &SyncRepo, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let src = repo_path.join(&repo.source_path);
        let dst = Path::new(&repo.dest_path);
        let files = self.get_files(&src, repo)?;
        self.copy_files(&files, &src, dst)
    }

    /// Check if repository is in sync
    fn check_repo_sync_status(&self, repo: &SyncRepo, repo_path: &Path, changed: &mut Vec<PathBuf>) -> Result<()> {
        let src = repo_path.join(&repo.source_path);
        let dst = Path::new(&repo.dest_path);
        let files = self.get_files(&src, repo)?;
        changed.extend(self.files_differ(&files, &src, dst));
        Ok(())
    }

    /// Check sync status of all repositories
    pub fn check_sync_status(&self) -> Result<SyncStatus> {
        if self.config.repos.is_empty() { 
            return Ok(SyncStatus::NotConfigured); 
        }
        
        let mut changed_files = Vec::new();
        for repo in &self.config.repos {
            let repo_path = self.cache_dir.join(self.extract_repo_name(&repo.repo));
            if repo_path.exists() {
                self.check_repo_sync_status(repo, &repo_path, &mut changed_files)?;
            }
        }
        
        if changed_files.is_empty() {
            Ok(SyncStatus::InSync)
        } else {
            Ok(SyncStatus::OutOfSync { changed_files })
        }
    }

    /// Update all repositories
    pub fn update_all_repos(&mut self, force: bool) -> Result<Vec<PathBuf>> {
        let mut all_updated_files = Vec::new();

        for repo in &self.config.repos {
            tracing::info!("Updating repository: {}", repo.name);
            
            // Update cache from remote
            let repo_path = self.update_cache(repo)?;
            
            // Check what will change
            let src = repo_path.join(&repo.source_path);
            let dst = Path::new(&repo.dest_path);
            let files = self.get_files(&src, repo)?;
            let changed_files = self.files_differ(&files, &src, dst);
            
            if !changed_files.is_empty() && !force {
                // In non-force mode, we could add prompting here if needed
                tracing::warn!("Files will be overwritten. Use --force to proceed.");
                continue;
            }
            
            // Copy files from cache to destination
            let updated = self.copy_repo_files(repo, &repo_path)?;
            all_updated_files.extend(updated);
        }

        Ok(all_updated_files)
    }


    /// Extract repository name from URL
    fn extract_repo_name(&self, repo_url: &str) -> String {
        repo_url
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .split('/')
            .next_back()
            .unwrap_or("unknown")
            .to_string()
    }
}