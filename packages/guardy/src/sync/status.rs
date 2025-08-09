use anyhow::Result;
use crate::cli::output;
use crate::sync::{SyncStatus, manager::SyncManager};

pub struct StatusDisplay<'a> {
    manager: &'a SyncManager,
}

impl<'a> StatusDisplay<'a> {
    pub fn new(manager: &'a SyncManager) -> Self {
        Self { manager }
    }
    
    pub fn show_detailed_status(&self) -> Result<()> {
        self.show_configuration_overview();
        self.show_repositories()?;
        self.show_sync_status()?;
        Ok(())
    }
    
    fn show_configuration_overview(&self) {
        output::styled!("{} Sync Configuration", ("📋", "info_symbol"));
        println!("  Repositories: {}", self.manager.get_config().repos.len());
        println!("  Cache Directory: {}", self.manager.get_cache_dir().display());
        println!();
    }
    
    fn show_repositories(&self) -> Result<()> {
        if self.manager.get_config().repos.is_empty() {
            output::styled!("{} No repositories configured", ("⚠️", "warning_symbol"));
            output::styled!("Run {} to bootstrap", 
                ("guardy sync update --repo=<url> --version=<version>", "property"));
            return Ok(());
        }
        
        for repo in &self.manager.get_config().repos {
            self.show_repository_details(repo);
        }
        
        Ok(())
    }
    
    fn show_repository_details(&self, repo: &crate::sync::SyncRepo) {
        output::styled!("{} Repository: {}", 
            ("🔗", "info_symbol"), 
            (&repo.name, "property"));
        
        println!("  URL: {}", output::file_path(repo.repo.clone()));
        println!("  Version: {}", output::property_name(repo.version.clone()));
        println!("  Source Path: {}", output::file_path(repo.source_path.clone()));
        println!("  Destination Path: {}", output::file_path(repo.dest_path.clone()));
        
        self.show_patterns(repo);
        self.show_protection_status(repo);
        self.show_protected_files_for_repo();
        
        println!();
    }
    
    fn show_patterns(&self, repo: &crate::sync::SyncRepo) {
        if !repo.include.is_empty() {
            println!("  Include Patterns: {}", repo.include.join(", "));
        }
        if !repo.exclude.is_empty() {
            println!("  Exclude Patterns: {}", repo.exclude.join(", "));
        }
    }
    
    fn show_protection_status(&self, repo: &crate::sync::SyncRepo) {
        println!("  Protected: {}", if repo.protected { "Yes" } else { "No" });
    }
    
    fn show_protected_files_for_repo(&self) {
        let all_protected_files = self.manager.protection_manager.list_protected_files();
        
        // Filter out files that match exclude patterns (like .git)
        let repo_protected_files: Vec<_> = all_protected_files.iter()
            .filter(|file| {
                // Check if file should be excluded based on repository patterns
                for repo in &self.manager.get_config().repos {
                    // Simple pattern matching - if file path contains any exclude pattern, filter it out
                    for exclude_pattern in &repo.exclude {
                        if file.to_string_lossy().contains(exclude_pattern) {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();
        
        if !repo_protected_files.is_empty() {
            println!("  Protected Files ({}):", repo_protected_files.len());
            for file in repo_protected_files {
                // Convert to relative path from current directory
                let display_path = if let Ok(current_dir) = std::env::current_dir() {
                    file.strip_prefix(&current_dir)
                        .map(|p| format!("./{}", p.display()))
                        .unwrap_or_else(|_| file.display().to_string())
                } else {
                    file.display().to_string()
                };
                println!("    • {}", output::file_path(display_path));
            }
        } else {
            println!("  Protected Files: None");
        }
    }
    
    fn show_sync_status(&self) -> Result<()> {
        match self.manager.check_sync_status()? {
            SyncStatus::InSync => {
                output::styled!("{} All files are in sync", ("✅", "success_symbol"));
            },
            SyncStatus::OutOfSync { changed_files } => {
                output::styled!("{} {} files are out of sync:", 
                    ("❌", "error_symbol"),
                    (changed_files.len().to_string(), "property"));
                
                for file in &changed_files {
                    let protection_status = if self.manager.protection_manager.is_protected(file) {
                        " 🔒"
                    } else {
                        ""
                    };
                    println!("  • {}{}", output::file_path(file.display().to_string()), protection_status);
                }
            },
            SyncStatus::NotConfigured => {
                output::styled!("{} Sync not configured", ("⚠️", "warning_symbol"));
            }
        }
        
        Ok(())
    }
}