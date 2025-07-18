//! Version command implementation
//!
//! Displays version information about Guardy in an engaging format.

use crate::cli::Output;
use anyhow::Result;

/// Execute the version command
pub async fn execute(output: &Output) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let homepage = env!("CARGO_PKG_HOMEPAGE");
    let repository = env!("CARGO_PKG_REPOSITORY");
    
    output.header("🚀 Guardy Version Information");
    
    // Main version info
    output.status_indicator("VERSION", &format!("{} v{}", name, version), true);
    output.blank_line();
    
    // Description
    output.category("About");
    output.key_value("Description:", description, false);
    output.key_value("Authors:", authors, false);
    output.blank_line();
    
    // Links
    output.category("Links");
    output.key_value("Homepage:", homepage, false);
    output.key_value("Repository:", repository, false);
    output.blank_line();
    
    // Build info
    output.category("Build Information");
    output.key_value("Rust edition:", "2024", false);
    output.key_value("Target:", std::env::consts::ARCH, false);
    output.key_value("Profile:", if cfg!(debug_assertions) { "debug" } else { "release" }, false);
    
    // Build timestamp (if available)
    if let Ok(timestamp) = std::env::var("BUILD_TIMESTAMP") {
        output.key_value("Built at:", &timestamp, false);
    }
    
    output.blank_line();
    output.success("💡 Run 'guardy --help' for usage information");
    
    Ok(())
}