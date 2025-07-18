use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod git;
mod hooks;
mod mcp;
mod security;
mod tools;
mod utils;

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
