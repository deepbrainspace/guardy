use anyhow::Result;

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

#[tokio::main]
async fn main() -> Result<()> {
    cli::init().run().await
}
