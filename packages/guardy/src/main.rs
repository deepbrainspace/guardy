use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod external;
mod git;
mod hooks;
mod parallel;
mod profiling;
mod reports;
mod scan;
mod shared;
mod sync;

use cli::commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
