//! Configuration command implementations
//!
//! Commands for managing Guardy configuration.

use crate::cli::ConfigCommands;
use crate::cli::Output;
use crate::config::GuardyConfig;
use crate::utils::{get_current_dir, detect_project_type};
use anyhow::Result;
use std::fs;

/// Execute config commands
pub async fn execute(cmd: ConfigCommands, output: &Output) -> Result<()> {
    match cmd {
        ConfigCommands::Init => init(output).await,
        ConfigCommands::Validate => validate(output).await,
        ConfigCommands::Show => show(output).await,
    }
}

async fn init(output: &Output) -> Result<()> {
    output.header("🔧 Initializing Configuration");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    // Check if config already exists
    if config_path.exists() {
        output.warning("Configuration file already exists");
        if !output.confirm("Do you want to overwrite it?") {
            output.info("Configuration initialization cancelled");
            return Ok(());
        }
    }
    
    // Detect project type
    let project_type = detect_project_type(&current_dir);
    output.info(&format!("Detected project type: {:?}", project_type));
    
    // Create default configuration
    let config = GuardyConfig::default();
    
    // Save configuration
    config.save_to_file(&config_path)?;
    
    output.success("Configuration file created successfully");
    output.table_row("Config file", &config_path.display().to_string());
    output.info("Edit guardy.yml to customize your settings");
    
    Ok(())
}

async fn validate(output: &Output) -> Result<()> {
    output.header("✅ Validating Configuration");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    if !config_path.exists() {
        output.error("Configuration file not found");
        output.indent("Run 'guardy config init' to create a configuration file");
        return Ok(());
    }
    
    match GuardyConfig::load_from_file(&config_path) {
        Ok(config) => {
            output.success("Configuration is valid");
            output.blank_line();
            
            // Show configuration summary
            output.step("Configuration Summary");
            output.table_row("Security patterns", &config.security.patterns.len().to_string());
            output.table_row("Tool integrations", &(config.tools.formatters.len() + config.tools.linters.len()).to_string());
            output.table_row("MCP server enabled", &config.mcp.enabled.to_string());
            
            if config.mcp.enabled {
                output.table_row("MCP server port", &config.mcp.port.to_string());
            }
            
            // Validate security patterns
            output.blank_line();
            output.step("Security Patterns");
            for pattern in &config.security.patterns {
                if pattern.enabled {
                    output.success(&format!("✓ {} ({})", pattern.name, pattern.severity));
                } else {
                    output.info(&format!("○ {} (disabled)", pattern.name));
                }
            }
            
            // Validate tool configurations
            output.blank_line();
            output.step("Tool Integrations");
            
            // Show formatters
            for formatter in &config.tools.formatters {
                output.success(&format!("✓ {} (formatter)", formatter.name));
            }
            
            // Show linters
            for linter in &config.tools.linters {
                output.success(&format!("✓ {} (linter)", linter.name));
            }
            
            if config.tools.formatters.is_empty() && config.tools.linters.is_empty() {
                output.info("○ No tools configured");
            }
        }
        Err(err) => {
            output.error("Configuration file is invalid");
            output.indent(&format!("Error: {}", err));
        }
    }
    
    Ok(())
}

async fn show(output: &Output) -> Result<()> {
    output.header("📄 Current Configuration");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    if !config_path.exists() {
        output.error("Configuration file not found");
        output.indent("Run 'guardy config init' to create a configuration file");
        return Ok(());
    }
    
    // Read and display the raw configuration file
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            output.success("Configuration file content:");
            output.blank_line();
            output.separator();
            println!("{}", content);
            output.separator();
            output.blank_line();
            output.table_row("Config file", &config_path.display().to_string());
        }
        Err(err) => {
            output.error("Failed to read configuration file");
            output.indent(&format!("Error: {}", err));
        }
    }
    
    Ok(())
}
