use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::config::GuardyConfig;
use super::{SyncConfig, SyncStatus, SyncRepo};

pub struct SyncManager {
    config: SyncConfig,
    cache_dir: PathBuf,
}

impl SyncManager {
    pub fn new(guardy_config: &GuardyConfig) -> Result<Self> {
        // Try to load sync configuration from guardy config
        let sync_config = match guardy_config.get_section("sync") {
            Ok(sync_section) => {
                serde_json::from_value(sync_section).unwrap_or_default()
            }
            Err(_) => SyncConfig::default(),
        };

        // Create cache directory in .guardy/cache/
        let cache_dir = PathBuf::from(".guardy/cache");
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            config: sync_config,
            cache_dir,
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

        Ok(Self {
            config: sync_config,
            cache_dir,
        })
    }

    pub fn check_sync_status(&self) -> Result<SyncStatus> {
        if self.config.repos.is_empty() {
            return Ok(SyncStatus::NotConfigured);
        }

        // For now, just return InSync - we'll implement the actual checking later
        // TODO: Implement actual file comparison logic
        tracing::info!("Checking sync status for {} repositories", self.config.repos.len());
        
        Ok(SyncStatus::InSync)
    }

    pub fn update_all_repos(&self, force: bool) -> Result<()> {
        if self.config.repos.is_empty() {
            return Err(anyhow!("No repositories configured for sync"));
        }

        tracing::info!("Updating {} repositories (force: {})", self.config.repos.len(), force);

        // For now, just log what we would do
        for repo in &self.config.repos {
            tracing::info!("Would sync repo '{}' from '{}' version '{}'", 
                         repo.name, repo.repo, repo.version);
        }

        // TODO: Implement actual sync logic with parallel module
        Ok(())
    }

    pub fn show_status(&self) -> Result<String> {
        let mut output = String::new();
        
        output.push_str(&format!("Sync Configuration:\n"));
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

    /// Get the sync configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }
}