//! Formatter utilities
//!
//! This module provides functionality for running code formatters
//! and checking formatting in various programming languages.

use crate::cli::Output;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Run formatter in check mode to see if files need formatting
pub fn run_formatter_check(command: &str, files: &[String], current_dir: &Path, output: &Output) -> Result<bool> {
    // Different formatters have different check modes
    let check_command = if command.contains("cargo fmt") {
        "cargo fmt -- --check".to_string()
    } else if command.contains("prettier") {
        format!("{} --check", command)
    } else if command.contains("black") {
        format!("{} --check", command)
    } else if command.contains("ruff format") {
        format!("{} --check", command)
    } else if command.contains("gofmt") {
        format!("{} -d", command)
    } else {
        // For other formatters, try common check patterns
        format!("{} --check", command)
    };
    
    // Run the check command
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(&check_command)
        .current_dir(current_dir);
    
    // Add files as arguments if the formatter supports it
    if !command.contains("cargo fmt") {
        for file in files {
            cmd.arg(file);
        }
    }
    
    let result = cmd.output()?;
    
    if output.is_verbose() {
        if !result.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            output.info(&format!("Formatter output: {}", stdout));
        }
        if !result.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            output.info(&format!("Formatter stderr: {}", stderr));
        }
    }
    
    // Non-zero exit code usually means formatting is needed
    Ok(!result.status.success())
}