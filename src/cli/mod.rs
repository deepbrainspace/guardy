//! Command-line interface for Guardy
//!
//! This module provides the main CLI structure and command handling for Guardy.
//! It uses clap for argument parsing and provides a clean, user-friendly interface.

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod commands;
mod output;

pub use output::Output;

/// Guardy - Intelligent Git Workflows for Modern Developers
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<String>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable quiet output (minimal)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Auto-install missing tools instead of failing
    #[arg(long, global = true)]
    pub auto_install: bool,

    /// Show what would be done without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Output format (text, json, yaml)
    #[arg(long, default_value = "text", global = true)]
    pub format: String,

    /// Force overwrite without prompting
    #[arg(short, long, global = true)]
    pub force: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Guardy in current repository
    Init,
    /// Show system status
    Status,
    /// Show version information
    Version,
    /// Uninstall Guardy from current repository
    Uninstall,
    /// MCP server commands
    #[command(subcommand)]
    Mcp(McpCommands),
    /// Git hooks management
    #[command(subcommand)]
    Hooks(HooksCommands),
    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),
    /// Security scanning and validation
    #[command(subcommand)]
    Security(SecurityCommands),
}

/// MCP server subcommands
#[derive(Subcommand)]
pub enum McpCommands {
    /// Setup MCP server configuration
    Setup,
    /// Start MCP server daemon
    Start {
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Stop MCP server daemon
    Stop,
    /// Show MCP server status
    Status,
    /// Show MCP server logs
    Logs,
}

/// Git hooks subcommands
#[derive(Subcommand)]
pub enum HooksCommands {
    /// Install git hooks
    Install,
    /// Remove git hooks
    Remove,
    /// List available hooks
    List,
    /// Run specific hook
    Run {
        /// Hook name to run
        hook: String,
    },
}

/// Configuration subcommands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize configuration
    Init,
    /// Validate configuration
    Validate,
    /// Show current configuration
    Show,
}

/// Security subcommands
#[derive(Subcommand)]
pub enum SecurityCommands {
    /// Scan for secrets in files
    Scan {
        /// Specific files to scan (comma-separated or multiple -i flags, supports glob patterns)
        #[arg(short = 'i', long, value_delimiter = ',')]
        files: Vec<String>,
        /// Scan specific directory
        #[arg(short, long)]
        directory: Option<String>,
    },
    /// Validate branch protection settings
    Validate,
    /// Check staging area for security issues
    Check,
}

impl Cli {
    /// Execute the CLI command
    pub async fn run(self) -> Result<()> {
        // Initialize output handler with global verbose and quiet settings
        let output = Output::new(self.verbose, self.quiet);

        // Handle the command
        match self.command {
            Some(Commands::Init) => commands::init::execute(self.force, &output).await,
            Some(Commands::Status) => commands::status::execute(&output).await,
            Some(Commands::Version) => commands::version::execute(&output).await,
            Some(Commands::Uninstall) => commands::uninstall::execute(self.force, &output).await,
            Some(Commands::Mcp(cmd)) => commands::mcp::execute(cmd, &output).await,
            Some(Commands::Hooks(cmd)) => commands::hooks::execute(cmd, self.force, &output).await,
            Some(Commands::Config(cmd)) => commands::config::execute(cmd, &output).await,
            Some(Commands::Security(cmd)) => commands::security::execute(cmd, &self.format, &output).await,
            None => {
                // Show help when no command is provided
                let mut cmd = Cli::command();
                cmd.print_help()?;
                Ok(())
            }
        }
    }
}
