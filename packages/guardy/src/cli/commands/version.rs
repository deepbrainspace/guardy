use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct VersionArgs {
    /// Show detailed version information
    #[arg(long)]
    pub detailed: bool,
}

pub async fn execute(args: VersionArgs) -> Result<()> {
    if args.detailed {
        println!("guardy {}", env!("CARGO_PKG_VERSION"));
        println!("Rust Edition: 2024");
        println!("Built with: git2 0.20.2, clap 4.5.41, tokio 1.46.1");
        println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        println!("License: {}", env!("CARGO_PKG_LICENSE"));
        println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    } else {
        // Detailed output by default for the command (unlike the --version flag)
        println!("guardy {} - Fast, secure git hooks in Rust with MCP server integration", env!("CARGO_PKG_VERSION"));
        println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    }
    Ok(())
}