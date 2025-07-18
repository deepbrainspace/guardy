//! Security command implementations
//!
//! Commands for security scanning, validation, and protection.

use crate::cli::{SecurityCommands, Output};
use crate::config::GuardyConfig;
use crate::security::SecretScanner;
use crate::utils::{get_current_dir, PathUtils, glob::{expand_file_patterns, is_glob_pattern}};
use anyhow::Result;
use std::path::Path;

/// Execute security commands
pub async fn execute(cmd: SecurityCommands, format: &str, output: &Output) -> Result<()> {
    match cmd {
        SecurityCommands::Scan { files, directory } => {
            scan(files, directory, format.to_string(), output).await
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
    if output.is_verbose() {
        output.header("🔍 Security Scanning");
    }

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
    let scanner = SecretScanner::from_config(&config, output)?;

    let mut all_matches = Vec::new();
    let mut files_scanned = 0;
    let mut files_excluded = 0;

    if !files.is_empty() {
        // Scan specific files (including glob patterns)
        if output.is_verbose() {
            output.step("Analyzing file patterns");
        }
        
        // Expand file patterns (including globs) using utility function
        let current_dir = get_current_dir()?;
        let valid_paths = match expand_file_patterns(&files, &current_dir) {
            Ok(paths) => paths,
            Err(e) => {
                output.error(&format!("Error expanding file patterns: {}", e));
                return Ok(());
            }
        };
        
        // Report any missing files
        for file_pattern in &files {
            if !is_glob_pattern(file_pattern) {
                // Only check literal paths for existence warnings
                let path = Path::new(file_pattern);
                if !path.exists() {
                    output.warning(&format!("File not found: {}", file_pattern));
                }
            }
        }
        
        // Use scan_files for better verbose output
        if !valid_paths.is_empty() {
            let path_refs: Vec<&Path> = valid_paths.iter().map(|p| p.as_path()).collect();
            let (matches, scanned, excluded) = scanner.scan_files(&path_refs)?;
            all_matches.extend(matches);
            files_scanned = scanned;
            files_excluded = excluded;
        }
    } else if let Some(dir) = directory {
        // Scan specific directory
        if output.is_verbose() {
            output.step(&format!("Analyzing directory: {}", dir));
        }
        let dir_path = Path::new(&dir);
        if dir_path.exists() && dir_path.is_dir() {
            let (matches, scanned, excluded) = scanner.scan_directory(dir_path)?;
            all_matches.extend(matches);
            files_scanned = scanned;
            files_excluded = excluded;
        } else {
            output.error(&format!("Directory not found or not a directory: {}", dir));
            return Ok(());
        }
    } else {
        // Scan current directory
        if output.is_verbose() {
            output.step("Analyzing current directory");
        }
        let (matches, scanned, excluded) = scanner.scan_directory(&current_dir)?;
        all_matches.extend(matches);
        files_scanned = scanned;
        files_excluded = excluded;
    }

    // Display results
    if !output.is_quiet() {
        output.blank_line();
    }
    display_scan_results(&all_matches, files_scanned, files_excluded, &format, output)?;

    Ok(())
}

/// Display scan results in the specified format
fn display_scan_results(
    matches: &[crate::security::SecurityMatch],
    files_scanned: usize,
    files_excluded: usize,
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
                let summary = if files_excluded > 0 {
                    format!("Scan completed successfully: scanned {} files. {} files excluded", files_scanned, files_excluded)
                } else {
                    format!("Scan completed successfully: scanned {} files", files_scanned)
                };
                output.success(&summary);
            } else {
                if !output.is_quiet() {
                    output.warning(&format!("⚠ Found {} security issues", matches.len()));
                    output.blank_line();
                }

                // Group matches by pattern type for cleaner display
                let mut grouped_matches: std::collections::HashMap<String, Vec<&crate::security::SecurityMatch>> = std::collections::HashMap::new();
                for security_match in matches {
                    grouped_matches.entry(security_match.pattern_name.clone()).or_insert_with(Vec::new).push(security_match);
                }

                for (pattern_name, pattern_matches) in grouped_matches.iter() {
                    let severity = &pattern_matches[0].severity;
                    let _severity_color = match severity {
                        crate::security::Severity::Critical => "red",
                        crate::security::Severity::Info => "yellow",
                    };

                    output.error(&format!(
                        "[{}] {} ({} occurrence{})",
                        severity,
                        pattern_name,
                        pattern_matches.len(),
                        if pattern_matches.len() == 1 { "" } else { "s" }
                    ));

                    for security_match in pattern_matches {
                        // Show relative path instead of full path
                        let relative_path = PathUtils::to_relative_path(&security_match.file_path);
                        output.indent(&format!("• {}:{}", relative_path, security_match.line_number));
                        if output.is_verbose() {
                            output.indent(&format!("  Content: {}", security_match.content));
                        }
                    }
                    
                    if !output.is_quiet() {
                        output.blank_line();
                    }
                }

                if !output.is_quiet() {
                    output.separator();
                }
                let summary = if files_excluded > 0 {
                    format!("Security scan completed with {} issues: scanned {} files. {} files excluded", matches.len(), files_scanned, files_excluded)
                } else {
                    format!("Security scan completed with {} issues: scanned {} files", matches.len(), files_scanned)
                };
                output.error(&format!("✖ {}", summary));
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
    let scanner = SecretScanner::from_config(&config, output)?;
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