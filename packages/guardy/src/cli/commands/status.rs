use anyhow::Result;
use clap::Args;

#[derive(Args, Clone, Default)]
pub struct StatusArgs {
    // Add status-specific options here
}

pub async fn execute(_args: StatusArgs) -> Result<()> {
    use crate::cli::output::*;
    use crate::config::CONFIG;
    use crate::git::GitRepo;
    use crate::scan::static_data::patterns::get_pattern_library;

    styled!("Checking {} status...", ("guardy", "primary"));

    // Check if we're in a git repository
    let repo = match GitRepo::discover() {
        Ok(repo) => {
            styled!("{} Git repository detected", ("✅", "success_symbol"));

            let branch = repo.current_branch()?;
            styled!("  Current branch: {}", (branch, "branch"));
            repo
        }
        Err(_) => {
            styled!("{} Not in a git repository", ("❌", "error_symbol"));
            return Ok(());
        }
    };

    // Show configuration details (CONFIG is always valid)
    styled!("{} Configuration loaded", ("✅", "success_symbol"));
    
    // Show key configuration details using direct field access
    styled!("  Max file size: {}MB", (CONFIG.scanner.max_file_size_mb.to_string(), "number"));
    styled!("  Include binary files: {}", (CONFIG.scanner.include_binary.to_string(), "property"));
    styled!("  Entropy analysis: {}", (CONFIG.scanner.enable_entropy_analysis.to_string(), "property"));

    // Check patterns
    let pattern_lib = get_pattern_library();
    styled!(
        "{} Scanner ready with {} patterns",
        ("✅", "success_symbol"),
        (pattern_lib.count().to_string(), "number")
    );

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
                    styled!(
                        "  {} {} exists but not managed by guardy",
                        ("⚠️", "warning_symbol"),
                        (hook_name, "property")
                    );
                }
            }
        } else {
            missing_hooks.push(*hook_name);
        }
    }

    if !installed_hooks.is_empty() {
        styled!(
            "{} Installed hooks: {}",
            ("✅", "success_symbol"),
            (installed_hooks.join(", "), "property")
        );
    }

    if !missing_hooks.is_empty() {
        styled!(
            "{} Missing hooks: {}",
            ("❌", "error_symbol"),
            (missing_hooks.join(", "), "property")
        );
        styled!(
            "Run {} to install missing hooks",
            ("'guardy install'", "command")
        );
    }

    if installed_hooks.len() == hook_names.len() {
        styled!(
            "{} Guardy is fully configured and ready!",
            ("🎉", "success_symbol")
        );
    } else {
        styled!("Status: {}", ("Partially configured", "warning"));
    }

    Ok(())
}
