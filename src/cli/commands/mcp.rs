//! MCP server command implementations
//!
//! Commands for managing the built-in MCP server.

use crate::cli::McpCommands;
use crate::cli::Output;
use crate::config::GuardyConfig;
use crate::shared::get_current_dir;
use anyhow::Result;

/// Execute MCP commands
pub async fn execute(cmd: McpCommands, output: &Output) -> Result<()> {
    match cmd {
        McpCommands::Setup => setup(output).await,
        McpCommands::Start { port } => start(port, output).await,
        McpCommands::Stop => stop(output).await,
        McpCommands::Status => status(output).await,
        McpCommands::Logs => logs(output).await,
    }
}

async fn setup(output: &Output) -> Result<()> {
    output.header("🔧 MCP Server Setup");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    // Check if configuration exists
    if !config_path.exists() {
        output.error("Configuration file not found");
        output.indent("Run 'guardy init' first to create configuration");
        return Ok(());
    }
    
    // Load configuration
    let config = GuardyConfig::load_from_file(&config_path)?;
    
    output.info("MCP Server Configuration:");
    output.table_row("Enabled", &config.mcp.enabled.to_string());
    output.table_row("Port", &config.mcp.port.to_string());
    output.table_row("Host", &config.mcp.host);
    
    if config.mcp.enabled {
        output.success("MCP server is enabled in configuration");
        output.blank_line();
        
        // Generate Claude Desktop configuration
        generate_claude_config(&config, output)?;
        
        output.blank_line();
        output.success("MCP server setup complete");
        output.info("Next steps:");
        output.list_item("Restart Claude Desktop to load the new configuration");
        output.list_item("Run 'guardy mcp start' to start the server");
        output.list_item("Test the connection with 'guardy mcp status'");
    } else {
        output.warning("MCP server is disabled in configuration");
        output.indent("Edit guardy.yml to enable MCP server");
    }
    
    Ok(())
}

async fn start(port: u16, output: &Output) -> Result<()> {
    output.header("🚀 Starting MCP Server");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    // Check if configuration exists
    if !config_path.exists() {
        output.error("Configuration file not found");
        output.indent("Run 'guardy init' first to create configuration");
        return Ok(());
    }
    
    // Load configuration
    let config = GuardyConfig::load_from_file(&config_path)?;
    
    if !config.mcp.enabled {
        output.error("MCP server is disabled in configuration");
        output.indent("Edit guardy.yml to enable MCP server");
        return Ok(());
    }
    
    let server_port = if port != 3000 { port } else { config.mcp.port };
    
    output.info(&format!("Starting MCP server on {}:{}", config.mcp.host, server_port));
    output.info("Server will provide:");
    output.list_item("Git repository analysis tools");
    output.list_item("Security scanning capabilities");
    output.list_item("Configuration management");
    output.list_item("Hook execution and validation");
    
    output.blank_line();
    output.warning("MCP server implementation not yet complete");
    output.info("This will be implemented in Phase 1.6");
    
    Ok(())
}

async fn stop(output: &Output) -> Result<()> {
    output.header("🛑 Stopping MCP Server");
    
    // TODO: Implement server process management
    output.info("Looking for running MCP server processes...");
    output.warning("MCP server stop not yet implemented");
    output.info("This will:");
    output.list_item("Find running MCP server processes");
    output.list_item("Send shutdown signal to server");
    output.list_item("Clean up server resources");
    output.list_item("Update process status");
    
    Ok(())
}

async fn status(output: &Output) -> Result<()> {
    output.header("📊 MCP Server Status");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    // Check configuration
    if config_path.exists() {
        let config = GuardyConfig::load_from_file(&config_path)?;
        
        output.step("Configuration");
        output.table_row("Enabled", &config.mcp.enabled.to_string());
        output.table_row("Port", &config.mcp.port.to_string());
        output.table_row("Host", &config.mcp.host);
        
        if config.mcp.enabled {
            output.blank_line();
            output.step("Server Status");
            output.warning("Server status checking not yet implemented");
            output.info("This will show:");
            output.list_item("Server process status (running/stopped)");
            output.list_item("Active connections and clients");
            output.list_item("Available tools and capabilities");
            output.list_item("Recent activity and logs");
        } else {
            output.blank_line();
            output.warning("MCP server is disabled in configuration");
        }
    } else {
        output.error("Configuration file not found");
        output.indent("Run 'guardy init' first to create configuration");
    }
    
    Ok(())
}

async fn logs(output: &Output) -> Result<()> {
    output.header("📄 MCP Server Logs");
    
    // TODO: Implement log file reading and display
    output.info("Looking for MCP server logs...");
    output.warning("MCP server logs not yet implemented");
    output.info("This will show:");
    output.list_item("Recent server activity");
    output.list_item("Client connections and disconnections");
    output.list_item("Tool execution logs");
    output.list_item("Error messages and warnings");
    output.list_item("Performance metrics");
    
    output.blank_line();
    output.info("Log files will be stored in:");
    output.indent("~/.guardy/logs/mcp-server.log");
    output.indent("~/.guardy/logs/mcp-errors.log");
    
    Ok(())
}

/// Generate Claude Desktop MCP configuration
fn generate_claude_config(config: &GuardyConfig, output: &Output) -> Result<()> {
    output.step("Claude Desktop Configuration");
    
    let claude_config = format!(r#"{{
  "mcpServers": {{
    "guardy": {{
      "command": "guardy",
      "args": ["mcp", "start", "--port", "{}"]
    }}
  }}
}}
"#, config.mcp.port);
    
    output.info("Add this configuration to your Claude Desktop settings:");
    output.blank_line();
    output.separator();
    println!("{}", claude_config);
    output.separator();
    output.blank_line();
    
    // Try to detect Claude Desktop config location
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let claude_config_path = home.join(".config/claude/claude_desktop_config.json");
        output.info("Claude Desktop config location:");
        output.indent(&claude_config_path.display().to_string());
    }
    
    Ok(())
}
