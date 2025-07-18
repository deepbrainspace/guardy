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
    output.header("📊 Guardy Status");

    let current_dir = get_current_dir()?;
    
    // Check if we're in a git repository
    check_git_status(&current_dir, output);
    
    // Check configuration status
    check_config_status(&current_dir, output);
    
    // Check git hooks status
    check_hooks_status(&current_dir, output);
    
    // Check MCP server status
    check_mcp_status(output);
    
    // Check security status
    check_security_status(&current_dir, output);
    
    Ok(())
}

/// Check git repository status
fn check_git_status(current_dir: &Path, output: &Output) {
    output.category("Git Repository");
    
    if FileUtils::is_git_repository(current_dir) {
        output.status_indicator("DETECTED", "Git repository found", true);
        
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
        output.status_indicator("FOUND", "Configuration file found", true);
        output.key_value("Config file:", &config_path.display().to_string(), false);
        
        // Try to load and validate config
        match GuardyConfig::load_from_file(&config_path) {
            Ok(config) => {
                output.status_indicator("VALID", "Configuration is valid", true);
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
    output.blank_line();
    output.step("Git Hooks");
    
    let hooks_dir = current_dir.join(".git/hooks");
    
    if hooks_dir.exists() {
        let hooks = ["pre-commit", "commit-msg", "pre-push"];
        let mut installed_count = 0;
        
        for hook in &hooks {
            let hook_path = hooks_dir.join(hook);
            if hook_path.exists() {
                // Check if it's a Guardy hook
                if let Ok(content) = fs::read_to_string(&hook_path) {
                    if content.contains("guardy") {
                        output.success(&format!("{} hook installed", hook));
                        installed_count += 1;
                    } else {
                        output.warning(&format!("{} hook exists but not managed by Guardy", hook));
                    }
                } else {
                    output.warning(&format!("{} hook exists but cannot be read", hook));
                }
            } else {
                output.warning(&format!("{} hook not installed", hook));
            }
        }
        
        output.table_row("Installed hooks", &format!("{}/{}", installed_count, hooks.len()));
        
        if installed_count == 0 {
            output.indent("Run 'guardy hooks install' to install git hooks");
        }
    } else {
        output.error("Git hooks directory not found");
        output.indent("This might not be a git repository");
    }
}

/// Check MCP server status
fn check_mcp_status(output: &Output) {
    output.blank_line();
    output.step("MCP Server");
    
    // TODO: Implement actual MCP server status checking
    output.info("MCP server status checking not yet implemented");
    output.indent("This will show:");
    output.indent("• Server running status");
    output.indent("• Port configuration");
    output.indent("• Connected clients");
    output.indent("• Available tools");
}

/// Check security status
fn check_security_status(current_dir: &Path, output: &Output) {
    output.blank_line();
    output.step("Security Status");
    
    let config_path = current_dir.join("guardy.yml");
    
    if let Ok(config) = GuardyConfig::load_from_file(&config_path) {
        // Check secret detection
        if config.security.secret_detection {
            output.success("Secret detection enabled");
        } else {
            output.warning("Secret detection disabled");
        }
        
        // Check protected branches
        if !config.security.protected_branches.is_empty() {
            output.success(&format!("{} protected branches configured", config.security.protected_branches.len()));
        } else {
            output.warning("No protected branches configured");
        }
        
        // Check git-crypt
        if config.security.git_crypt {
            if Path::new(".git-crypt").exists() {
                output.success("Git-crypt enabled and initialized");
            } else {
                output.warning("Git-crypt enabled but not initialized");
            }
        } else {
            output.info("Git-crypt integration disabled");
        }
    } else {
        output.warning("Security configuration not available");
    }
}
