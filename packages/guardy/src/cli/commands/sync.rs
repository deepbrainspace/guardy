use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::sync::{SyncStatus, manager::SyncManager};

#[derive(Parser)]
#[command(about = "Protected file synchronization")]
pub struct SyncArgs {
    #[command(subcommand)]
    pub command: SyncSubcommand,
}

#[derive(Subcommand)]
pub enum SyncSubcommand {
    /// Check if files are in sync with configured repositories
    Check,
    
    /// Update files from configured repositories
    Update {
        /// Force update, overwriting local changes
        #[arg(long)]
        force: bool,
        
        /// Bootstrap from a specific repository (initial setup)
        #[arg(long)]
        repo: Option<String>,
        
        /// Specific version to sync (tag, branch, or commit)
        #[arg(long)]
        version: Option<String>,
    },
    
    /// Show sync configuration and current status
    Show,
    
    /// Unprotect specific files
    Unprotect {
        /// Files to unprotect (can use glob patterns)
        files: Vec<String>,
        
        /// Unprotect all synced files
        #[arg(long)]
        all: bool,
    },
    
    /// List all protected files
    Protected,
    
    /// Restore files from a backup
    Restore {
        /// Path to backup directory to restore from
        backup_path: String,
    },
}

pub async fn execute(args: SyncArgs, config_path: Option<&str>) -> Result<()> {
    match args.command {
        SyncSubcommand::Check => execute_check(config_path).await,
        SyncSubcommand::Update { force, repo, version } => {
            execute_update(force, repo, version, config_path).await
        },
        SyncSubcommand::Show => execute_show(config_path).await,
        SyncSubcommand::Unprotect { files, all } => {
            execute_unprotect(files, all, config_path).await
        },
        SyncSubcommand::Protected => execute_list_protected(config_path).await,
        SyncSubcommand::Restore { backup_path } => {
            execute_restore(backup_path, config_path).await
        },
    }
}

async fn execute_check(config_path: Option<&str>) -> Result<()> {
    let manager = create_sync_manager(config_path)?;
    
    let status = manager.check_sync_status()?;
    
    match status {
        SyncStatus::InSync => {
            output::styled!("{} All files are in sync", 
                ("✅", "success_symbol")
            );
            Ok(())
        },
        SyncStatus::OutOfSync { changed_files } => {
            output::styled!("{} Files are out of sync:", 
                ("❌", "error_symbol")
            );
            for file in &changed_files {
                let protection_status = if manager.protection_manager.is_protected(file) {
                    " 🔒"
                } else {
                    ""
                };
                println!("  • {}{}", output::file_path(file.display().to_string()), protection_status);
            }
            std::process::exit(1);
        },
        SyncStatus::NotConfigured => {
            output::styled!("{} No sync configuration found", 
                ("⚠️", "warning_symbol")
            );
            output::styled!("Run {} to bootstrap", 
                ("guardy sync update --repo=<url> --version=<version>", "property")
            );
            std::process::exit(1);
        }
    }
}

async fn execute_update(force: bool, repo: Option<String>, version: Option<String>, config_path: Option<&str>) -> Result<()> {
    // Handle bootstrap case
    if let (Some(repo_url), Some(version_str)) = (repo, version) {
        output::styled!("{} Bootstrapping sync from {} @ {}", 
            ("🚀", "info_symbol"),
            (&repo_url, "property"),
            (&version_str, "id_value")
        );
        let mut manager = SyncManager::bootstrap(&repo_url, &version_str)?;
        manager.update_all_repos(force)?;
        output::styled!("{} Bootstrap complete", 
            ("✅", "success_symbol")
        );
        return Ok(());
    }

    // Regular update case
    let mut manager = create_sync_manager(config_path)?;
    
    if force {
        output::styled!("{} Force updating all repositories...", 
            ("⚡", "info_symbol")
        );
    } else {
        output::styled!("{} Updating all repositories...", 
            ("📥", "info_symbol")
        );
    }
    
    manager.update_all_repos(force)?;
    output::styled!("{} All repositories updated", 
        ("✅", "success_symbol")
    );
    
    Ok(())
}

async fn execute_show(config_path: Option<&str>) -> Result<()> {
    let manager = create_sync_manager(config_path)?;
    let status_output = manager.show_status()?;
    println!("{status_output}");
    Ok(())
}

async fn execute_unprotect(files: Vec<String>, all: bool, config_path: Option<&str>) -> Result<()> {
    let mut manager = create_sync_manager(config_path)?;
    
    if all {
        output::styled!("{} Removing protection from all files...", 
            ("🔓", "info_symbol")
        );
        manager.protection_manager.clear_all_protections()?;
        output::styled!("{} All file protections removed", 
            ("✅", "success_symbol")
        );
    } else if !files.is_empty() {
        output::styled!("{} Unprotecting {} files...", 
            ("🔓", "info_symbol"),
            (files.len().to_string(), "property")
        );
        
        let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        manager.protection_manager.unprotect_files(paths)?;
        
        output::styled!("{} Files unprotected", 
            ("✅", "success_symbol")
        );
    } else {
        return Err(anyhow!("Specify files to unprotect or use --all flag"));
    }
    
    Ok(())
}

async fn execute_list_protected(config_path: Option<&str>) -> Result<()> {
    let manager = create_sync_manager(config_path)?;
    let protected_files = manager.protection_manager.list_protected_files();
    
    if protected_files.is_empty() {
        output::styled!("{} No files are currently protected", 
            ("ℹ️", "info_symbol")
        );
    } else {
        output::styled!("{} Protected files ({} total):", 
            ("🔒", "info_symbol"),
            (protected_files.len().to_string(), "property")
        );
        
        for file in protected_files {
            println!("  • {}", output::file_path(file.display().to_string()));
        }
    }
    
    Ok(())
}

async fn execute_restore(backup_path: String, config_path: Option<&str>) -> Result<()> {
    let manager = create_sync_manager(config_path)?;
    
    output::styled!("{} Restoring files from backup: {}", 
        ("📂", "info_symbol"),
        (&backup_path, "property")
    );
    
    let backup_path = PathBuf::from(backup_path);
    manager.protection_manager.restore_from_backup(&backup_path)?;
    
    output::styled!("{} Files restored successfully", 
        ("✅", "success_symbol")
    );
    
    Ok(())
}

fn create_sync_manager(config_path: Option<&str>) -> Result<SyncManager> {
    let config = GuardyConfig::load::<()>(config_path, None, 0)
        .map_err(|e| anyhow!("Failed to load configuration: {}", e))?;
    
    // Extract sync config using the proper parsing method
    let sync_config = SyncManager::parse_sync_config(&config)?;
    
    // Create sync manager with parsed config
    SyncManager::with_config(sync_config)
}