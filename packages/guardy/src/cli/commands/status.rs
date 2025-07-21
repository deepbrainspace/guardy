use anyhow::Result;
use clap::Args;

#[derive(Args, Default)]
pub struct StatusArgs {
    // Add status-specific options here
}

pub async fn execute(_args: StatusArgs) -> Result<()> {
    use crate::cli::output::*;
    use crate::git::GitRepo;
    use crate::config::GuardyConfig;
    use crate::scanner::SecretPatterns;
    
    info("Checking guardy status...");
    
    // Check if we're in a git repository
    let repo = match GitRepo::discover() {
        Ok(repo) => {
            success("✅ Git repository detected");
            let branch = repo.current_branch()?;
            println!("  Current branch: {}", branch);
            repo
        }
        Err(_) => {
            warning("❌ Not in a git repository");
            return Ok(());
        }
    };
    
    // Check configuration
    match GuardyConfig::load(None, None::<&()>) {
        Ok(config) => {
            success("✅ Configuration loaded");
            
            // Check patterns
            match SecretPatterns::new(&config) {
                Ok(patterns) => {
                    success(&format!("✅ Scanner ready with {} patterns", patterns.pattern_count()));
                }
                Err(e) => {
                    error(&format!("❌ Pattern loading failed: {}", e));
                }
            }
        }
        Err(e) => {
            warning(&format!("⚠️  Configuration issues: {}", e));
        }
    }
    
    // Check hook installation
    let hooks_dir = repo.git_dir().join("hooks");
    let hook_names = ["pre-commit", "commit-msg", "post-checkout", "pre-push"];
    let mut installed_hooks = Vec::new();
    let mut missing_hooks = Vec::new();
    
    for hook_name in &hook_names {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() {
            // Check if it's a guardy hook
            if let Ok(content) = std::fs::read_to_string(&hook_path) {
                if content.contains("guardy run") {
                    installed_hooks.push(*hook_name);
                } else {
                    println!("  ⚠️  {} exists but not managed by guardy", hook_name);
                }
            }
        } else {
            missing_hooks.push(*hook_name);
        }
    }
    
    if !installed_hooks.is_empty() {
        success(&format!("✅ Installed hooks: {}", installed_hooks.join(", ")));
    }
    
    if !missing_hooks.is_empty() {
        warning(&format!("❌ Missing hooks: {}", missing_hooks.join(", ")));
        info("Run 'guardy install' to install missing hooks");
    }
    
    if installed_hooks.len() == hook_names.len() {
        success("🎉 Guardy is fully configured and ready!");
    } else {
        info("Status: Partially configured");
    }
    
    Ok(())
}