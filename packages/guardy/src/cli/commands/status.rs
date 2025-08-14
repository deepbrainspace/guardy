use anyhow::Result;
use clap::Args;

#[derive(Args, Default)]
pub struct StatusArgs {
    // Add status-specific options here
}

pub async fn execute(_args: StatusArgs, verbosity_level: u8) -> Result<()> {
    use crate::cli::output::*;
    use crate::config::GuardyConfig;
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

    // Check configuration
    match GuardyConfig::load(None, None::<&()>, verbosity_level) {
        Ok(config) => {
            styled!("{} Configuration loaded", ("✅", "success_symbol"));
            
            // Show key configuration details
            if let Ok(scanner_section) = config.get_section("scanner") {
                if let Some(max_size) = scanner_section.get("max_file_size_mb").and_then(|v| v.as_u64()) {
                    styled!("  Max file size: {}MB", (max_size.to_string(), "number"));
                }
                if let Some(include_binary) = scanner_section.get("include_binary").and_then(|v| v.as_bool()) {
                    styled!("  Include binary files: {}", (include_binary.to_string(), "property"));
                }
                if let Some(entropy_enabled) = scanner_section.get("enable_entropy_analysis").and_then(|v| v.as_bool()) {
                    styled!("  Entropy analysis: {}", (entropy_enabled.to_string(), "property"));
                }
            }

            // Check patterns
            let pattern_lib = get_pattern_library();
            styled!(
                "{} Scanner ready with {} patterns",
                ("✅", "success_symbol"),
                (pattern_lib.count().to_string(), "number")
            );
        }
        Err(e) => {
            styled!(
                "{} Configuration issues: {}",
                ("⚠️", "warning_symbol"),
                (e.to_string(), "warning")
            );
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
