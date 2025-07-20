use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct InstallArgs {
    /// Specify which hooks to install (default: all)
    #[arg(long, value_delimiter = ',')]
    pub hooks: Option<Vec<String>>,
    
    /// Overwrite existing hooks
    #[arg(long)]
    pub force: bool,
}

pub async fn execute(args: InstallArgs) -> Result<()> {
    use crate::cli::output::*;
    use crate::git::GitRepo;
    use crate::config::GuardyConfig;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    
    info("Installing guardy hooks...");
    
    // Check if we're in a git repository
    let repo = match GitRepo::discover() {
        Ok(repo) => repo,
        Err(_) => {
            error("Not in a git repository. Run 'git init' first.");
            return Ok(());
        }
    };
    
    let hooks_dir = repo.git_dir().join("hooks");
    
    // Validate .git/hooks directory exists
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)?;
        info("Created .git/hooks directory");
    }
    
    // Parse guardy.toml configuration
    let _config = GuardyConfig::load()?;
    
    if args.force {
        warning("Force mode enabled - will overwrite existing hooks");
    }
    
    // Determine which hooks to install
    let hooks_to_install = args.hooks.unwrap_or_else(|| {
        vec!["pre-commit".to_string(), "commit-msg".to_string(), "post-checkout".to_string(), "pre-push".to_string()]
    });
    
    // Install each hook
    for hook_name in hooks_to_install {
        let hook_path = hooks_dir.join(&hook_name);
        
        // Check if hook exists and handle based on force flag
        if hook_path.exists() && !args.force {
            warning(&format!("Hook '{}' already exists. Use --force to overwrite.", hook_name));
            continue;
        }
        
        // Create hook script that calls guardy
        let hook_script = format!(
            "#!/bin/sh\n# Guardy hook: {}\nexec guardy run {}\n",
            hook_name, hook_name
        );
        
        fs::write(&hook_path, hook_script)?;
        
        // Make hook executable
        let mut permissions = fs::metadata(&hook_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions)?;
        
        success(&format!("Installed '{}' hook", hook_name));
    }
    
    success("Hook installation completed!");
    
    // Show next steps
    info("Next steps:");
    println!("  - Run 'guardy status' to verify installation");
    println!("  - Run 'guardy run pre-commit' to test hooks manually");
    println!("  - Configure patterns in guardy.toml if needed");
    
    Ok(())
}