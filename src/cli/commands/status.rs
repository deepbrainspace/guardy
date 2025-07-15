//! Show system status
//!
//! This command displays the current status of Guardy configuration,
//! git hooks, and MCP server.

use crate::cli::Output;
use anyhow::Result;

/// Execute the status command
pub async fn execute(output: &Output) -> Result<()> {
    output.header("📊 Guardy Status");

    // TODO: Implement status checking logic
    output.info("Status checking not yet implemented");
    output.info("This will show:");
    output.list_item("Configuration file status");
    output.list_item("Git hooks installation status");
    output.list_item("MCP server status");
    output.list_item("Repository information");
    output.list_item("Security check results");

    Ok(())
}
