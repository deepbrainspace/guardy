use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::sync::{manager::SyncManager, status::StatusDisplay};

#[derive(Parser)]
#[command(about = "File synchronization from remote repositories")]
pub struct SyncArgs {
    #[command(subcommand)]
    pub command: Option<SyncSubcommand>,

    /// Force update, bypass interactive mode and update all changes without prompting
    #[arg(long)]
    pub force: bool,

    /// Bootstrap from a specific repository (initial setup)
    #[arg(long)]
    pub repo: Option<String>,

    /// Specific version to sync (tag, branch, or commit)
    #[arg(long)]
    pub version: Option<String>,
}

#[derive(Subcommand)]
pub enum SyncSubcommand {
    /// Show sync status and configuration
    Status,

    /// Update files from configured repositories (interactive by default)
    Update {
        /// Force update, bypass interactive mode and update all changes without prompting
        #[arg(long)]
        force: bool,

        /// Bootstrap from a specific repository (initial setup)
        #[arg(long)]
        repo: Option<String>,

        /// Specific version to sync (tag, branch, or commit)
        #[arg(long)]
        version: Option<String>,
    },

    /// Show differences between local and remote files (what has drifted)
    Diff,
}

pub async fn execute(args: SyncArgs, config_path: Option<&str>) -> Result<()> {
    match args.command {
        Some(SyncSubcommand::Status) => execute_status(config_path).await,
        Some(SyncSubcommand::Update {
            force,
            repo,
            version,
        }) => {
            // Prefer subcommand args over main args
            let final_force = force || args.force;
            let final_repo = repo.or(args.repo);
            let final_version = version.or(args.version);
            execute_update(final_force, final_repo, final_version, config_path).await
        }
        Some(SyncSubcommand::Diff) => execute_diff(config_path).await,
        // Default to update behavior when no subcommand is provided, using main args
        None => execute_update(args.force, args.repo, args.version, config_path).await,
    }
}

async fn execute_status(config_path: Option<&str>) -> Result<()> {
    let manager = create_sync_manager(config_path)?;
    let status_display = StatusDisplay::new(&manager);
    status_display.show_detailed_status()
}

async fn execute_diff(config_path: Option<&str>) -> Result<()> {
    let mut manager = create_sync_manager(config_path)?;

    // Check if we have any configuration
    if manager.config.repos.is_empty() {
        output::styled!("{} No sync configuration found", ("⚠️", "warning_symbol"));
        return Ok(());
    }

    output::styled!("{} Checking for differences...", ("🔍", "info_symbol"));

    // Use the dedicated diff-only method (no interactive prompts)
    manager.show_all_diffs().await?;

    Ok(())
}

async fn execute_update(
    force: bool,
    repo: Option<String>,
    version: Option<String>,
    config_path: Option<&str>,
) -> Result<()> {
    // Handle bootstrap case
    if let (Some(repo_url), Some(version_str)) = (repo, version) {
        output::styled!(
            "{} Bootstrapping sync from {} @ {}",
            ("🚀", "info_symbol"),
            (&repo_url, "property"),
            (&version_str, "id_value")
        );

        let mut manager = SyncManager::bootstrap(&repo_url, &version_str)?;
        let updated_files = manager.update_all_repos(false).await?; // Bootstrap is always non-interactive

        if !updated_files.is_empty() {
            output::styled!(
                "{} Synced {} files:",
                ("📝", "info_symbol"),
                (updated_files.len().to_string(), "property")
            );
            for file in &updated_files {
                println!("  • {}", output::file_path(file.display().to_string()));
            }
        }
        output::styled!("{} Bootstrap complete", ("✅", "success_symbol"));
        return Ok(());
    }

    // Regular update case
    let mut manager = create_sync_manager(config_path)?;

    // Check if we have any configuration (without doing full status check)
    if manager.config.repos.is_empty() {
        output::styled!("{} No sync configuration found", ("⚠️", "warning_symbol"));
        output::styled!(
            "Run {} to bootstrap",
            (
                "guardy sync update --repo=<url> --version=<version>",
                "property"
            )
        );
        return Ok(());
    }

    // Perform the update (interactive by default, force bypasses)
    let interactive = !force;

    let updated_files = manager.update_all_repos(interactive).await?;

    // Show results for force mode
    if force {
        if updated_files.is_empty() {
            output::styled!("<info>  No files were updated");
        } else {
            output::styled!(
                "{} Successfully updated {} files:",
                ("✅", "success_symbol"),
                (updated_files.len().to_string(), "property")
            );

            for file in &updated_files {
                println!("  • {}", output::file_path(file.display().to_string()));
            }
        }
    }
    // Interactive mode shows its own summary

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
