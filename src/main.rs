use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod external;
mod git;
mod hooks;
mod shared;
mod mcp;
mod security;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Execute the command
    cli.run().await?;

    Ok(())
}
