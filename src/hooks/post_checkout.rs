//! Post-checkout hook implementation
//!
//! This hook runs after branch checkouts and performs dependency management.

use super::HookContext;
use crate::cli::Output;
use crate::external::package_managers::run_dependency_install;
use anyhow::Result;

/// Execute post-checkout hook
pub async fn execute(_context: HookContext) -> Result<()> {
    let output = Output::new(false, false); // Default to non-verbose, non-quiet
    let current_dir = std::env::current_dir()?;
    
    let step1_start = std::time::Instant::now();
    
    // 1. Check if configuration enables post-checkout dependency management
    // TODO: Add configuration flag support when config schema is updated
    // For now, default to enabled
    let enable_dependency_management = true;
    
    if !enable_dependency_management {
        output.success("Post-checkout dependency management is disabled");
        return Ok(());
    }
    
    let step1_duration = step1_start.elapsed();
    output.workflow_step_timed(1, 3, "Checking dependency management settings", "⚙️", step1_duration);
    
    let step2_start = std::time::Instant::now();
    
    // 2. Detect package files and check if dependencies might have changed
    let package_files = [
        "package.json",
        "pnpm-workspace.yaml", 
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "go.sum"
    ];
    
    let mut found_package_files = Vec::new();
    for file in &package_files {
        let file_path = current_dir.join(file);
        if file_path.exists() {
            found_package_files.push(file.to_string());
        }
    }
    
    if found_package_files.is_empty() {
        output.info("No package files detected, skipping dependency management");
        return Ok(());
    }
    
    output.success(&format!("Detected package files: {}", found_package_files.join(", ")));
    
    let step2_duration = step2_start.elapsed();
    output.workflow_step_timed(2, 3, "Detecting package files", "📦", step2_duration);
    
    let step3_start = std::time::Instant::now();
    
    // 3. Check if we should run dependency installation
    let should_install = if current_dir.join("pnpm-lock.yaml").exists() {
        output.info("pnpm workspace detected, running dependency sync...");
        true
    } else if current_dir.join("package-lock.json").exists() {
        output.info("npm project detected, running dependency sync...");
        true
    } else if current_dir.join("yarn.lock").exists() {
        output.info("Yarn project detected, running dependency sync...");
        true
    } else if current_dir.join("Cargo.lock").exists() {
        output.info("Rust project detected, checking Cargo dependencies...");
        true
    } else {
        output.info("No lockfile detected, skipping automatic dependency installation");
        false
    };
    
    if should_install {
        // Run the appropriate package manager
        let install_result = if current_dir.join("pnpm-lock.yaml").exists() {
            run_dependency_install("pnpm", &["install"], &current_dir, &output)
        } else if current_dir.join("package-lock.json").exists() {
            run_dependency_install("npm", &["install"], &current_dir, &output)
        } else if current_dir.join("yarn.lock").exists() {
            run_dependency_install("yarn", &["install"], &current_dir, &output)
        } else if current_dir.join("Cargo.lock").exists() {
            run_dependency_install("cargo", &["check"], &current_dir, &output)
        } else {
            Ok(true)
        };
        
        match install_result {
            Ok(success) => {
                if success {
                    output.success("Dependencies synchronized successfully");
                } else {
                    output.warning("Dependency synchronization completed with warnings");
                }
            }
            Err(e) => {
                output.error(&format!("Dependency installation failed: {}", e));
                output.info("You may need to run the package manager manually");
                // Don't fail the hook for dependency issues
            }
        }
    } else {
        output.info("No dependency installation required");
    }
    
    let step3_duration = step3_start.elapsed();
    output.workflow_step_timed(3, 3, "Managing dependencies", "🔄", step3_duration);
    
    Ok(())
}