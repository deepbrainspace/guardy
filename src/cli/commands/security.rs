//! Security command implementations
//!
//! Commands for security scanning, validation, and protection.

use crate::cli::{SecurityCommands, Output};
use crate::config::GuardyConfig;
use crate::security::SecretScanner;
use crate::utils::get_current_dir;
use anyhow::Result;
use std::path::Path;

/// Execute security commands
pub async fn execute(cmd: SecurityCommands, output: &Output) -> Result<()> {
    match cmd {
        SecurityCommands::Scan { files, directory, format } => {
            scan(files, directory, format, output).await
        }
        SecurityCommands::Validate => validate(output).await,
        SecurityCommands::Check => check(output).await,
    }
}

/// Scan for secrets in files or directories
async fn scan(
    files: Vec<String>,
    directory: Option<String>,
    format: String,
    output: &Output,
) -> Result<()> {
    output.header("🔍 Security Scanning");

    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");

    // Load configuration
    let config = if config_path.exists() {
        GuardyConfig::load_from_file(&config_path)?
    } else {
        output.warning("No configuration file found, using defaults");
        GuardyConfig::default()
    };

    if !config.security.secret_detection {
        output.warning("Secret detection is disabled in configuration");
        output.info("Enable it in guardy.yml or run 'guardy config init'");
        return Ok(());
    }

    // Create scanner
    let scanner = SecretScanner::from_config(&config)?;

    let mut all_matches = Vec::new();

    if !files.is_empty() {
        // Scan specific files
        output.step("Scanning specified files");
        for file_path in &files {
            let path = Path::new(file_path);
            if path.exists() {
                let matches = scanner.scan_file(path)?;
                all_matches.extend(matches);
                output.info(&format!("Scanned: {}", file_path));
            } else {
                output.warning(&format!("File not found: {}", file_path));
            }
        }
    } else if let Some(dir) = directory {
        // Scan specific directory
        output.step(&format!("Scanning directory: {}", dir));
        let dir_path = Path::new(&dir);
        if dir_path.exists() && dir_path.is_dir() {
            let matches = scanner.scan_directory(dir_path)?;
            all_matches.extend(matches);
        } else {
            output.error(&format!("Directory not found or not a directory: {}", dir));
            return Ok(());
        }
    } else {
        // Scan current directory
        output.step("Scanning current directory");
        let matches = scanner.scan_directory(&current_dir)?;
        all_matches.extend(matches);
    }

    // Display results
    output.blank_line();
    display_scan_results(&all_matches, &format, output)?;

    Ok(())
}

/// Display scan results in the specified format
fn display_scan_results(
    matches: &[crate::security::SecurityMatch],
    format: &str,
    output: &Output,
) -> Result<()> {
    match format {
        "json" => {
            let json_output = serde_json::to_string_pretty(matches)?;
            println!("{}", json_output);
        }
        _ => {
            if matches.is_empty() {
                output.success("No security issues found");
            } else {
                output.warning(&format!("Found {} security issues", matches.len()));
                output.blank_line();

                for (i, security_match) in matches.iter().enumerate() {
                    let _severity_color = match security_match.severity {
                        crate::security::Severity::Critical => "red",
                        crate::security::Severity::Info => "yellow",
                    };

                    output.error(&format!(
                        "{}. [{}] {} in {}:{}:{}",
                        i + 1,
                        security_match.severity,
                        security_match.pattern_name,
                        security_match.file_path,
                        security_match.line_number,
                        security_match.column
                    ));
                    output.indent(&format!("Content: {}", security_match.content));
                    output.blank_line();
                }

                output.separator();
                output.error("Security scan completed with issues");
                output.info("Review the findings above and take appropriate action");
            }
        }
    }

    Ok(())
}

/// Validate branch protection settings
async fn validate(output: &Output) -> Result<()> {
    output.header("🛡️ Branch Protection Validation");

    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");

    // Load configuration
    let config = if config_path.exists() {
        GuardyConfig::load_from_file(&config_path)?
    } else {
        output.warning("No configuration file found, using defaults");
        GuardyConfig::default()
    };

    output.step("Checking branch protection settings");

    // Get current branch
    let current_branch = if let Ok(branch_output) = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
    {
        String::from_utf8_lossy(&branch_output.stdout).trim().to_string()
    } else {
        output.error("Could not determine current branch");
        return Ok(());
    };

    // Check if current branch is protected
    if config.security.protected_branches.contains(&current_branch) {
        output.success(&format!("Current branch '{}' is protected", current_branch));
    } else {
        output.warning(&format!("Current branch '{}' is not protected", current_branch));
    }

    output.blank_line();
    output.step("Protected branches configuration");
    for branch in &config.security.protected_branches {
        output.list_item(&format!("{} (protected)", branch));
    }

    if config.security.protected_branches.is_empty() {
        output.info("No protected branches configured");
        output.indent("Add protected branches to guardy.yml");
    }

    // Check git-crypt integration
    output.blank_line();
    output.step("Git-crypt integration");
    if config.security.git_crypt {
        output.info("Git-crypt integration enabled");
        
        // Check if git-crypt is installed
        if let Ok(output_cmd) = std::process::Command::new("git-crypt").arg("--version").output() {
            if output_cmd.status.success() {
                output.success("Git-crypt is installed");
            } else {
                output.warning("Git-crypt command failed");
            }
        } else {
            output.error("Git-crypt is not installed");
            output.indent("Install git-crypt: https://github.com/AGWA/git-crypt");
        }
        
        // Check if git-crypt is initialized
        if Path::new(".git-crypt").exists() {
            output.success("Git-crypt is properly configured");
            
            // Check for .gitattributes file
            if Path::new(".gitattributes").exists() {
                output.success("Git attributes file exists");
            } else {
                output.warning("No .gitattributes file found");
                output.indent("Create .gitattributes to specify encrypted files");
            }
        } else {
            output.warning("Git-crypt enabled but not initialized");
            output.indent("Run 'git-crypt init' to initialize");
        }
    } else {
        output.info("Git-crypt integration disabled");
        output.indent("Enable in guardy.yml for file encryption support");
    }

    Ok(())
}

/// Check staging area for security issues
async fn check(output: &Output) -> Result<()> {
    output.header("🔍 Staging Area Security Check");

    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");

    // Load configuration
    let config = if config_path.exists() {
        GuardyConfig::load_from_file(&config_path)?
    } else {
        output.warning("No configuration file found, using defaults");
        GuardyConfig::default()
    };

    if !config.security.secret_detection {
        output.warning("Secret detection is disabled in configuration");
        return Ok(());
    }

    // Get staged files
    let staged_files_output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()?;

    let staged_files: Vec<String> = String::from_utf8_lossy(&staged_files_output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if staged_files.is_empty() {
        output.info("No files staged for commit");
        return Ok(());
    }

    output.step(&format!("Checking {} staged files", staged_files.len()));

    // Create scanner
    let scanner = SecretScanner::from_config(&config)?;
    let mut all_matches = Vec::new();

    for file_path in &staged_files {
        let path = Path::new(file_path);
        if path.exists() {
            let matches = scanner.scan_file(path)?;
            all_matches.extend(matches);
        }
    }

    // Display results
    output.blank_line();
    if all_matches.is_empty() {
        output.success("No security issues found in staged files");
    } else {
        output.error(&format!("Found {} security issues in staged files", all_matches.len()));
        output.blank_line();

        for (i, security_match) in all_matches.iter().enumerate() {
            output.error(&format!(
                "{}. [{}] {} in {}:{}:{}",
                i + 1,
                security_match.severity,
                security_match.pattern_name,
                security_match.file_path,
                security_match.line_number,
                security_match.column
            ));
            output.indent(&format!("Content: {}", security_match.content));
            output.blank_line();
        }

        output.separator();
        output.error("Staging area contains security issues");
        output.info("Fix the issues above before committing");
        output.info("Use 'git reset' to unstage files if needed");
    }

    Ok(())
}