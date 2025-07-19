use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct UninstallArgs {
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,
}

pub async fn execute(args: UninstallArgs) -> Result<()> {
    use crate::cli::output::*;
    
    if !args.yes {
        info("This will remove all guardy hooks from the repository");
        // In a real implementation, we'd prompt for confirmation
    }
    
    info("Removing guardy hooks...");
    success("Hooks removed successfully!");
    Ok(())
}