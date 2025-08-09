use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::fs;
use ignore::WalkBuilder;

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
        let cache_dir = PathBuf::from(".guardy/cache");
        std::fs::create_dir_all(&cache_dir)?;
        let remote_ops = RemoteOperations::new(cache_dir.clone());
        let protection_manager = ProtectionManager::new(sync_config.protection.clone())?;

        Ok(Self { config: sync_config, cache_dir, remote_ops, protection_manager })
    }

    pub fn bootstrap(repo_url: &str, version: &str) -> Result<Self> {
        let sync_repo = SyncRepo {
            name: "bootstrap".to_string(), repo: repo_url.to_string(), version: version.to_string(),
            source_path: ".".to_string(), dest_path: ".".to_string(),
            include: vec!["*".to_string()], exclude: vec![".git".to_string()], protected: true,
        };
        Self::with_config(SyncConfig { repos: vec![sync_repo], protection: Default::default() })
    }

    /// Core abstraction: Get files matching patterns using ignore crate  
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
            if let Some(parent) = dst_file.parent() { fs::create_dir_all(parent)?; }
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
            } else { None }
        }).collect()
    }

    fn existing_files(&self, files: &[PathBuf], dst: &Path) -> Vec<PathBuf> {
        files.iter().map(|f| dst.join(f)).filter(|f| f.exists()).collect()
    }

    /// Public interface - compose the abstractions
    fn copy_repo_files(&self, repo: &SyncRepo, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let src = repo_path.join(&repo.source_path);
        let dst = Path::new(&repo.dest_path);
        let files = self.get_files(&src, repo)?;
        self.copy_files(&files, &src, dst)
    }

    fn copy_changed_repo_files(&self, repo: &SyncRepo, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let src = repo_path.join(&repo.source_path);
        let dst = Path::new(&repo.dest_path);
        let all_files = self.get_files(&src, repo)?;
        let changed_files = self.files_differ(&all_files, &src, dst);
        
        // Only copy the files that actually differ
        let relative_changed_files: Vec<PathBuf> = changed_files.iter()
            .filter_map(|abs_path| abs_path.strip_prefix(dst).ok().map(|p| p.to_path_buf()))
            .collect();
            
        if !relative_changed_files.is_empty() {
            self.copy_files(&relative_changed_files, &src, dst)
        } else {
            Ok(vec![])
        }
    }

    fn check_repo_sync_status(&self, repo: &SyncRepo, repo_path: &Path, changed: &mut Vec<PathBuf>) -> Result<()> {
        let src = repo_path.join(&repo.source_path);
        let dst = Path::new(&repo.dest_path);
        let files = self.get_files(&src, repo)?;
        changed.extend(self.files_differ(&files, &src, dst));
        Ok(())
    }

    fn get_files_to_overwrite(&self, repo: &SyncRepo, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let src = repo_path.join(&repo.source_path);
        let dst = Path::new(&repo.dest_path);
        let files = self.get_files(&src, repo)?;
        Ok(self.existing_files(&files, dst))
    }

    pub fn check_sync_status(&self) -> Result<SyncStatus> {
        if self.config.repos.is_empty() { return Ok(SyncStatus::NotConfigured); }
        
        let mut changed_files = Vec::new();
        for repo in &self.config.repos {
            let repo_path = self.cache_dir.join(self.extract_repo_name(&repo.repo));
            if repo_path.exists() {
                self.check_repo_sync_status(repo, &repo_path, &mut changed_files)?;
            }
        }
        
        if changed_files.is_empty() { Ok(SyncStatus::InSync) } 
        else { Ok(SyncStatus::OutOfSync { changed_files }) }
    }

    pub fn update_all_repos(&mut self, force: bool) -> Result<Vec<PathBuf>> {
        if self.config.repos.is_empty() { return Err(anyhow!("No repositories configured")); }

        // Protection check
        if !force && self.protection_manager.should_block_modifications() {
            let files: Vec<_> = self.config.repos.iter()
                .map(|repo| {
                    let repo_path = self.remote_ops.clone_or_fetch(&repo.repo, &repo.version)?;
                    self.get_files_to_overwrite(repo, &repo_path)
                }).collect::<Result<Vec<_>, _>>()?
                .into_iter().flatten().collect();
            self.protection_manager.validate_modifications(&files)?;
        }

        let mut all_synced_files = Vec::new();
        
        // Sync repositories
        for repo in &self.config.repos {
            let repo_path = self.remote_ops.clone_or_fetch(&repo.repo, &repo.version)?;
            if force {
                let files = self.get_files_to_overwrite(repo, &repo_path)?;
                if !files.is_empty() { self.protection_manager.backup_before_sync(&files)?; }
            }
            
            // For bootstrap (initial setup), copy all files. For regular updates, only copy changed files.
            let synced = if repo.name == "bootstrap" {
                self.copy_repo_files(repo, &repo_path)?
            } else {
                self.copy_changed_repo_files(repo, &repo_path)?
            };
            
            all_synced_files.extend(synced.clone());
            self.protection_manager.protect_synced_files(repo, synced)?;
        }
        Ok(all_synced_files)
    }


    pub fn parse_sync_config(config: &GuardyConfig) -> Result<SyncConfig> {
        match config.get_section("sync") {
            Ok(section) => serde_json::from_value(section).map_err(|e| anyhow!("Parse error: {}", e)),
            Err(_) => Ok(SyncConfig::default())
        }
    }

    pub fn extract_repo_name(&self, url: &str) -> String {
        url.trim_end_matches(".git").split('/').next_back().unwrap_or("unknown").to_string()
    }
    
    pub fn get_config(&self) -> &SyncConfig {
        &self.config
    }
    
    pub fn get_cache_dir(&self) -> &std::path::Path {
        &self.cache_dir
    }
}