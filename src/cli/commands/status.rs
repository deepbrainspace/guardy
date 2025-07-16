//! Show system status
//!
//! This command displays the current status of Guardy configuration,
//! git hooks, and MCP server.

use crate::cli::Output;
use crate::config::GuardyConfig;
use crate::utils::{get_current_dir, is_git_repository};
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
    
    Ok(())
}

/// Check git repository status
fn check_git_status(current_dir: &Path, output: &Output) {
    output.blank_line();
    output.step("Git Repository");
    
    if is_git_repository(current_dir) {
        output.success("Git repository detected");
        
        // Get current branch
        if let Ok(branch) = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
        {
            if let Ok(branch_name) = String::from_utf8(branch.stdout) {
                output.table_row("Current branch", branch_name.trim());
            }
        }
        
        // Get repository root
        if let Ok(root) = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            if let Ok(root_path) = String::from_utf8(root.stdout) {
                output.table_row("Repository root", root_path.trim());
            }
        }
    } else {
        output.error("Not a git repository");
        output.indent("Run 'git init' to initialize a git repository");
    }
}

/// Check configuration file status
fn check_config_status(current_dir: &Path, output: &Output) {
    output.blank_line();
    output.step("Configuration");
    
    let config_path = current_dir.join("guardy.yml");
    
    if config_path.exists() {
        output.success("Configuration file found");
        output.table_row("Config file", &config_path.display().to_string());
        
        // Try to load and validate config
        match GuardyConfig::load_from_file(&config_path) {
            Ok(config) => {
                output.success("Configuration is valid");
                output.table_row("Security patterns", &config.security.patterns.len().to_string());
                output.table_row("Tool integrations", &(config.tools.formatters.len() + config.tools.linters.len()).to_string());
                output.table_row("MCP server enabled", &config.mcp.enabled.to_string());
            }
            Err(err) => {
                output.error("Configuration file is invalid");
                output.indent(&format!("Error: {}", err));
            }
        }
    } else {
        output.warning("No configuration file found");
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
