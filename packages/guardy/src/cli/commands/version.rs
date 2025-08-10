use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct VersionArgs {
    /// Show detailed version information
    #[arg(short = 'v', long = "verbose")]
    pub detailed: bool,
}

pub async fn execute(args: VersionArgs) -> Result<()> {
    // Get the git SHA from build time
    let git_sha = option_env!("GIT_SHA").unwrap_or("unknown");
    let git_branch = option_env!("GIT_BRANCH").unwrap_or("unknown");

    if args.detailed {
        println!("guardy {} ({})", env!("CARGO_PKG_VERSION"), git_sha);
        println!("Branch: {git_branch}");
        println!("Rust Edition: 2024");
        println!("Built with: clap 4.5.41, tokio 1.46.1, dialoguer 0.11.0");
        println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        println!("License: {}", env!("CARGO_PKG_LICENSE"));
        println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    } else {
        // Show version with SHA
        println!("guardy {} ({})", env!("CARGO_PKG_VERSION"), git_sha);
    }
    Ok(())
}
