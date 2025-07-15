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
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,

    /// Auto-install missing tools instead of failing
    #[arg(long)]
    pub auto_install: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Guardy in current repository
    Init {
        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Show system status
    Status,
    /// MCP server commands
    #[command(subcommand)]
    Mcp(McpCommands),
    /// Git hooks management
    #[command(subcommand)]
    Hooks(HooksCommands),
    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),
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
    Install {
        /// Force overwrite existing hooks
        #[arg(short, long)]
        force: bool,
    },
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

impl Cli {
    /// Execute the CLI command
    pub async fn run(self) -> Result<()> {
        // Initialize output handler
        let output = Output::new(self.debug);

        // Handle the command
        match self.command {
            Some(Commands::Init { yes }) => commands::init::execute(yes, &output).await,
            Some(Commands::Status) => commands::status::execute(&output).await,
            Some(Commands::Mcp(cmd)) => commands::mcp::execute(cmd, &output).await,
            Some(Commands::Hooks(cmd)) => commands::hooks::execute(cmd, &output).await,
            Some(Commands::Config(cmd)) => commands::config::execute(cmd, &output).await,
            None => {
                // Show help when no command is provided
                let mut cmd = Cli::command();
                cmd.print_help()?;
                Ok(())
            }
        }
    }
}
