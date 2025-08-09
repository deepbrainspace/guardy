use anyhow::Result;
use crate::cli::output;
use super::{SyncStatus, manager::SyncManager};

pub struct StatusDisplay<'a> {
    manager: &'a SyncManager,
}

impl<'a> StatusDisplay<'a> {
    pub fn new(manager: &'a SyncManager) -> Self {
        Self { manager }
    }

    pub fn show_detailed_status(&self) -> Result<()> {
        // Show configured repositories
        if self.manager.config.repos.is_empty() {
            output::styled!("{} No sync repositories configured", 
                ("⚠️", "warning_symbol")
            );
            return Ok(());
        }

        output::styled!("{} Sync Configuration", 
            ("📋", "info_symbol")
        );
        println!();

        // Show each repository configuration
        for repo in &self.manager.config.repos {
            output::styled!("  {} {}", 
                ("📦", "info_symbol"),
                (&repo.name, "property")
            );
            println!("      Repository: {}", output::file_path(repo.repo.clone()));
            println!("      Version:    {}", output::property_name(repo.version.clone()));
            println!("      Source:     {}", repo.source_path);
            println!("      Dest:       {}", repo.dest_path);
            
            if !repo.include.is_empty() {
                println!("      Include:    {:?}", repo.include);
            }
            if !repo.exclude.is_empty() {
                println!("      Exclude:    {:?}", repo.exclude);
            }
            println!();
        }

        // Check sync status
        let status = self.manager.check_sync_status()?;
        
        match status {
            SyncStatus::InSync => {
                output::styled!("{} All files are in sync", 
                    ("✅", "success_symbol")
                );
            },
            SyncStatus::OutOfSync { changed_files } => {
                output::styled!("{} {} files are out of sync:", 
                    ("⚠️", "warning_symbol"),
                    (changed_files.len().to_string(), "property")
                );
                for file in &changed_files {
                    println!("      • {}", output::file_path(file.display().to_string()));
                }
                println!();
                output::styled!("  Run {} to update", 
                    ("guardy sync update", "property")
                );
            },
            SyncStatus::NotConfigured => {
                // Already handled above
            }
        }

        Ok(())
    }
}