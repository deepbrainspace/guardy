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
    use crate::git::GitRepo;
    use std::fs;
    
    // Check if we're in a git repository
    let repo = match GitRepo::discover() {
        Ok(repo) => repo,
        Err(_) => {
            error!("Not in a git repository");
            return Ok(());
        }
    };
    
    let hooks_dir = repo.git_dir().join("hooks");
    let hook_names = ["pre-commit", "commit-msg", "post-checkout", "pre-push"];
    
    // Find guardy hooks
    let mut guardy_hooks = Vec::new();
    for hook_name in &hook_names {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() {
            if let Ok(content) = fs::read_to_string(&hook_path) {
                if content.contains("guardy run") {
                    guardy_hooks.push((hook_name, hook_path));
                }
            }
        }
    }
    
    if guardy_hooks.is_empty() {
        info!("No guardy hooks found to remove");
        return Ok(());
    }
    
    if !args.yes {
        warning!(&format!("This will remove {} guardy hooks:", guardy_hooks.len()));
        for (hook_name, _) in &guardy_hooks {
            println!("  - {hook_name}");
        }
        
        // Prompt for confirmation
        print!("Are you sure you want to continue? [y/N]: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input != "y" && input != "yes" {
            info!("Uninstall cancelled");
            return Ok(());
        }
    }
    
    info!("Removing guardy hooks...");
    
    let mut removed_count = 0;
    for (hook_name, hook_path) in guardy_hooks {
        match fs::remove_file(&hook_path) {
            Ok(_) => {
                success!(&format!("Removed '{hook_name}' hook"));
                removed_count += 1;
            }
            Err(e) => {
                error!(&format!("Failed to remove '{hook_name}' hook: {e}"));
            }
        }
    }
    
    if removed_count > 0 {
        success!(&format!("Successfully removed {removed_count} guardy hooks"));
    } else {
        warning!("No hooks were removed");
    }
    
    Ok(())
}