use anyhow::Result;
use clap::Parser;

mod cli;
mod hooks;
mod git;
mod security;
mod external;
mod config;
mod mcp;
mod shared;

use cli::commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}