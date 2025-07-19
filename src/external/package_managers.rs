//! Package manager utilities
//!
//! This module provides functionality for managing dependencies and validating
//! lockfiles across different package managers (npm, pnpm, yarn, cargo).

use crate::cli::Output;
use crate::config::GuardyConfig;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Package manager configuration
struct PackageManager {
    name: &'static str,
    install_args: &'static [&'static str],
    validate_args: &'static [&'static str],
    lockfile: &'static str,
    sync_message: &'static str,
    error_message: &'static str,
    install_command: &'static str,
}

const PACKAGE_MANAGERS: &[PackageManager] = &[
    PackageManager {
        name: "pnpm",
        install_args: &["install"],
        validate_args: &["install", "--frozen-lockfile"],
        lockfile: "pnpm-lock.yaml",
        sync_message: "pnpm lockfile is synchronized",
        error_message: "pnpm lockfile is out of sync",
        install_command: "pnpm install",
    },
    PackageManager {
        name: "npm",
        install_args: &["install"],
        validate_args: &["ci", "--dry-run"],
        lockfile: "package-lock.json",
        sync_message: "npm lockfile is synchronized",
        error_message: "npm lockfile is out of sync",
        install_command: "npm install",
    },
    PackageManager {
        name: "yarn",
        install_args: &["install"],
        validate_args: &["install", "--frozen-lockfile"],
        lockfile: "yarn.lock",
        sync_message: "yarn lockfile is synchronized",
        error_message: "yarn lockfile is out of sync",
        install_command: "yarn install",
    },
    PackageManager {
        name: "cargo",
        install_args: &["check"],
        validate_args: &["check", "--locked"],
        lockfile: "Cargo.lock",
        sync_message: "Cargo lockfile is synchronized",
        error_message: "Cargo lockfile is out of sync",
        install_command: "cargo update",
    },
];

/// Run dependency installation command
pub fn run_dependency_install(package_manager: &str, args: &[&str], current_dir: &Path, output: &Output) -> Result<bool> {
    let mut cmd = Command::new(package_manager);
    cmd.args(args)
        .current_dir(current_dir);
    
    if output.is_verbose() {
        output.info(&format!("Running: {} {}", package_manager, args.join(" ")));
    }
    
    let result = cmd.output()?;
    
    if output.is_verbose() {
        if !result.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            output.info(&format!("Output: {}", stdout));
        }
        if !result.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            output.info(&format!("Stderr: {}", stderr));
        }
    }
    
    Ok(result.status.success())
}

/// Validate lockfile to ensure dependencies are synchronized
pub async fn validate_lockfile(current_dir: &Path, output: &Output) -> Result<bool> {
    // Check if configuration enables lockfile validation
    let config_result = GuardyConfig::load_from_file(&current_dir.join("guardy.yml"));
    let enable_lockfile_validation = if let Ok(_config) = &config_result {
        // TODO: Add configuration flag support when config schema is updated
        // For now, default to enabled
        true
    } else {
        // Default to enabled if no config found
        true
    };
    
    if !enable_lockfile_validation {
        output.info("Lockfile validation is disabled");
        return Ok(true);
    }
    
    // Find the appropriate package manager
    for pm in PACKAGE_MANAGERS {
        if current_dir.join(pm.lockfile).exists() {
            return validate_package_manager_lockfile(pm, current_dir, output).await;
        }
    }
    
    output.info("No lockfile detected, skipping validation");
    Ok(true)
}

/// Validate a specific package manager's lockfile
async fn validate_package_manager_lockfile(pm: &PackageManager, current_dir: &Path, output: &Output) -> Result<bool> {
    output.info(&format!("Validating {} lockfile synchronization...", pm.name));
    
    let result = Command::new(pm.name)
        .args(pm.validate_args)
        .current_dir(current_dir)
        .output();
    
    match result {
        Ok(output_result) => {
            if output_result.status.success() {
                output.success(pm.sync_message);
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                output.error(pm.error_message);
                if output.is_verbose() {
                    output.indent(&format!("Error: {}", stderr));
                }
                output.indent(&format!("Run '{}' to synchronize dependencies", pm.install_command));
                Ok(false)
            }
        }
        Err(e) => {
            output.warning(&format!("{} command not found or failed to execute", pm.name));
            if output.is_verbose() {
                output.indent(&format!("Error: {}", e));
            }
            // Don't fail validation if package manager is not available
            Ok(true)
        }
    }
}

/// Legacy functions for backward compatibility - these delegate to the new implementations

pub async fn validate_pnpm_lockfile(current_dir: &Path, output: &Output) -> Result<bool> {
    let pm = &PACKAGE_MANAGERS[0]; // pnpm
    validate_package_manager_lockfile(pm, current_dir, output).await
}

pub async fn validate_npm_lockfile(current_dir: &Path, output: &Output) -> Result<bool> {
    let pm = &PACKAGE_MANAGERS[1]; // npm
    validate_package_manager_lockfile(pm, current_dir, output).await
}

pub async fn validate_yarn_lockfile(current_dir: &Path, output: &Output) -> Result<bool> {
    let pm = &PACKAGE_MANAGERS[2]; // yarn
    validate_package_manager_lockfile(pm, current_dir, output).await
}

pub async fn validate_cargo_lockfile(current_dir: &Path, output: &Output) -> Result<bool> {
    let pm = &PACKAGE_MANAGERS[3]; // cargo
    validate_package_manager_lockfile(pm, current_dir, output).await
}