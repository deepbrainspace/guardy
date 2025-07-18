//! Show system status
//!
//! This command displays the current status of Guardy configuration,
//! git hooks, and MCP server.

use crate::cli::Output;
use crate::config::GuardyConfig;
use crate::utils::{get_current_dir, FileUtils};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Execute the status command
pub async fn execute(output: &Output) -> Result<()> {
    let start_time = std::time::Instant::now();
    
    output.header("📊 Guardy Status");

    let current_dir = get_current_dir()?;
    
    // Use workflow steps for better progress indication
    if output.is_verbose() {
        output.workflow_step(1, 5, "Checking git repository", "🔍");
    }
    check_git_status(&current_dir, output);
    
    if output.is_verbose() {
        output.workflow_step(2, 5, "Validating configuration", "⚙️");
    }
    check_config_status(&current_dir, output);
    
    if output.is_verbose() {
        output.workflow_step(3, 5, "Inspecting git hooks", "🪝");
    }
    check_hooks_status(&current_dir, output);
    
    if output.is_verbose() {
        output.workflow_step(4, 5, "Checking MCP server", "🔧");
    }
    check_mcp_status(output);
    
    if output.is_verbose() {
        output.workflow_step(5, 5, "Analyzing security status", "🔒");
    }
    check_security_status(&current_dir, output);
    
    // Show completion summary with timing
    let duration = start_time.elapsed();
    if output.is_verbose() {
        output.completion_summary("Status check", duration, true);
    }
    
    Ok(())
}

/// Check git repository status
fn check_git_status(current_dir: &Path, output: &Output) {
    output.category("Git Repository");
    
    if FileUtils::is_git_repository(current_dir) {
        output.status_indicator("DETECTED", "Git repository found 🎯", true);
        
        // Get current branch
        if let Ok(branch) = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
        {
            if let Ok(branch_name) = String::from_utf8(branch.stdout) {
                output.key_value("Current branch:", branch_name.trim(), true);
            }
        }
        
        // Get repository root
        if let Ok(root) = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            if let Ok(root_path) = String::from_utf8(root.stdout) {
                output.key_value("Repository root:", root_path.trim(), false);
            }
        }
    } else {
        output.status_indicator("NOT FOUND", "Git repository not found", false);
        output.indent("Run 'git init' to initialize a git repository");
    }
}

/// Check configuration file status
fn check_config_status(current_dir: &Path, output: &Output) {
    output.category("Configuration");
    
    let config_path = current_dir.join("guardy.yml");
    
    if config_path.exists() {
        output.status_indicator("FOUND", "Configuration file found 📋", true);
        output.key_value("Config file:", &config_path.display().to_string(), false);
        
        // Try to load and validate config
        match GuardyConfig::load_from_file(&config_path) {
            Ok(config) => {
                output.status_indicator("VALID", "Configuration is valid ✅", true);
                output.key_value("Security patterns:", &config.security.patterns.len().to_string(), false);
                output.key_value("Tool integrations:", &(config.tools.formatters.len() + config.tools.linters.len()).to_string(), false);
                output.key_value("MCP server enabled:", &config.mcp.enabled.to_string(), config.mcp.enabled);
            }
            Err(err) => {
                output.status_indicator("INVALID", "Configuration file is invalid", false);
                output.indent(&format!("Error: {}", err));
            }
        }
    } else {
        output.status_indicator("NOT FOUND", "No configuration file found", false);
        output.indent("Run 'guardy init' to create a configuration file");
    }
}

/// Check git hooks status
fn check_hooks_status(current_dir: &Path, output: &Output) {
    output.category("Git Hooks");
    
    let hooks_dir = current_dir.join(".git/hooks");
    
    if hooks_dir.exists() {
        let hooks = ["pre-commit", "commit-msg", "pre-push"];
        let mut installed_hooks = Vec::new();
        let mut external_hooks = Vec::new();
        let mut missing_hooks = Vec::new();
        
        for hook in &hooks {
            let hook_path = hooks_dir.join(hook);
            if hook_path.exists() {
                // Check if it's a Guardy hook
                if let Ok(content) = fs::read_to_string(&hook_path) {
                    if content.contains("guardy") {
                        installed_hooks.push(hook);
                    } else {
                        external_hooks.push(hook);
                    }
                } else {
                    external_hooks.push(hook);
                }
            } else {
                missing_hooks.push(hook);
            }
        }
        
        // Show overall status
        if installed_hooks.len() == hooks.len() {
            output.status_indicator("COMPLETE", "All git hooks installed", true);
        } else if !installed_hooks.is_empty() {
            output.status_indicator("PARTIAL", "Some git hooks installed", false);
        } else {
            output.status_indicator("NONE", "No git hooks installed", false);
        }
        
        // Show details
        if !installed_hooks.is_empty() {
            let installed_list: Vec<String> = installed_hooks.iter().map(|s| s.to_string()).collect();
            output.key_value("Installed:", &installed_list.join(", "), true);
        }
        if !external_hooks.is_empty() {
            let external_list: Vec<String> = external_hooks.iter().map(|s| s.to_string()).collect();
            output.key_value("External:", &external_list.join(", "), false);
        }
        if !missing_hooks.is_empty() {
            let missing_list: Vec<String> = missing_hooks.iter().map(|s| s.to_string()).collect();
            output.key_value("Missing:", &missing_list.join(", "), false);
        }
        
        output.key_value("Status:", &format!("{}/{} hooks active", installed_hooks.len(), hooks.len()), installed_hooks.len() > 0);
        
        if installed_hooks.is_empty() {
            output.indent("Run 'guardy hooks install' to install git hooks");
        }
    } else {
        output.status_indicator("NOT FOUND", "Git hooks directory not found", false);
        output.indent("This might not be a git repository");
    }
}

/// Check MCP server status
fn check_mcp_status(output: &Output) {
    output.category("MCP Server");
    
    // TODO: Implement actual MCP server status checking
    output.status_indicator("NOT IMPLEMENTED", "MCP server status checking not yet implemented", false);
    output.indent("This will show:");
    output.indent("• Server running status");
    output.indent("• Port configuration");
    output.indent("• Connected clients");
    output.indent("• Available tools");
}

/// Check security status
fn check_security_status(current_dir: &Path, output: &Output) {
    output.category("Security Status");
    
    let config_path = current_dir.join("guardy.yml");
    
    if let Ok(config) = GuardyConfig::load_from_file(&config_path) {
        // Check secret detection
        if config.security.secret_detection {
            output.status_indicator("ENABLED", "Secret detection enabled", true);
        } else {
            output.status_indicator("DISABLED", "Secret detection disabled", false);
        }
        
        // Check protected branches
        if !config.security.protected_branches.is_empty() {
            output.status_indicator("CONFIGURED", &format!("{} protected branches configured", config.security.protected_branches.len()), true);
        } else {
            output.status_indicator("NONE", "No protected branches configured", false);
        }
        
        // Check git-crypt
        if config.security.git_crypt {
            if Path::new(".git-crypt").exists() {
                output.status_indicator("INITIALIZED", "Git-crypt enabled and initialized", true);
            } else {
                output.status_indicator("NOT INITIALIZED", "Git-crypt enabled but not initialized", false);
            }
        } else {
            output.status_indicator("DISABLED", "Git-crypt integration disabled", false);
        }
    } else {
        output.status_indicator("UNAVAILABLE", "Security configuration not available", false);
    }
}
