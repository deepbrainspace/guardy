use anyhow::Result;
use clap::Parser;

mod cli;
mod hooks;
mod git;
mod scanner;
mod external;
mod config;
mod mcp;
mod shared;
mod sync;
mod parallel;
mod reports;

use cli::commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}