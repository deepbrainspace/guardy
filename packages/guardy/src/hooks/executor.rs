use anyhow::{Result, anyhow};
use std::process::Command;

use super::config::HookConfig;
use crate::cli::output;
use crate::config::GuardyConfig;
use crate::git::GitRepo;
use crate::scanner::Scanner;

pub struct HookExecutor {
    config: GuardyConfig,
    hook_config: HookConfig,
}

impl HookExecutor {
    pub fn new(config: GuardyConfig) -> Result<Self> {
        // Parse hook configuration from config
        let hook_config = Self::parse_hook_config(&config)?;

        Ok(Self {
            config,
            hook_config,
        })
    }

    fn parse_hook_config(config: &GuardyConfig) -> Result<HookConfig> {
        // Try to get hooks section from config
        if let Ok(hooks_value) = config.get_section("hooks") {
            serde_json::from_value(hooks_value)
                .map_err(|e| anyhow!("Failed to parse hooks config: {}", e))
        } else {
            // Use default if no hooks config
            Ok(HookConfig::default())
        }
    }

    pub async fn execute(&self, hook_name: &str, args: Vec<String>) -> Result<()> {
        // Get hook definition
        let hook_def = self
            .hook_config
            .hooks
            .get(hook_name)
            .ok_or_else(|| anyhow!("Unknown hook: {}", hook_name))?;

        if !hook_def.enabled {
            output::info!(&format!("Hook '{hook_name}' is disabled"));
            return Ok(());
        }

        output::info!(&format!("Executing {hook_name} hook..."));

        // Execute built-in commands
        for builtin_cmd in &hook_def.builtin {
            self.execute_builtin(builtin_cmd, hook_name, &args).await?;
        }

        // Execute custom commands
        for custom_cmd in &hook_def.custom {
            self.execute_custom(custom_cmd, &args)?;
        }

        output::success!("Hook execution completed!");
        Ok(())
    }

    async fn execute_builtin(&self, builtin: &str, hook_name: &str, args: &[String]) -> Result<()> {
        match builtin {
            "scan_secrets" => {
                if hook_name != "pre-commit" {
                    return Ok(()); // Only valid for pre-commit
                }
                self.scan_secrets().await
            }
            "validate_commit_msg" => {
                if hook_name != "commit-msg" || args.is_empty() {
                    return Ok(()); // Only valid for commit-msg with args
                }
                // Future implementation for commit message validation
                output::info!(&format!(
                    "Commit message validation would check: {}",
                    args[0]
                ));
                Ok(())
            }
            unknown => {
                output::warning!(&format!("Unknown builtin command: {unknown}"));
                Ok(())
            }
        }
    }

    async fn scan_secrets(&self) -> Result<()> {
        output::info!("Scanning for secrets...");

        let repo = GitRepo::discover()?;
        let staged_files = repo.get_staged_files()?;

        if staged_files.is_empty() {
            output::info!("No staged files to check");
            return Ok(());
        }

        let scanner = Scanner::new(&self.config)?;
        let scan_result = scanner.scan_paths(&staged_files)?;

        if scan_result.stats.total_matches > 0 {
            output::error!(&format!(
                "❌ Found {} secrets in staged files",
                scan_result.stats.total_matches
            ));

            for secret_match in scan_result.matches.iter().take(5) {
                println!(
                    "  🔍 {}:{} [{}]",
                    secret_match.file_path, secret_match.line_number, secret_match.secret_type
                );
            }

            if scan_result.matches.len() > 5 {
                println!("  ... and {} more", scan_result.matches.len() - 5);
            }

            println!("\nCommit aborted. Remove secrets before committing.");
            std::process::exit(1);
        } else {
            output::success!(&format!(
                "✅ Scanned {} files - no secrets found",
                scan_result.stats.files_scanned
            ));
        }

        Ok(())
    }

    fn execute_custom(&self, custom: &super::config::CustomCommand, args: &[String]) -> Result<()> {
        if !custom.description.is_empty() {
            output::info!(&custom.description);
        }

        // Build command with argument substitution
        let mut command_str = custom.command.clone();

        // Replace $1, $2, etc. with actual arguments
        for (i, arg) in args.iter().enumerate() {
            command_str = command_str.replace(&format!("${}", i + 1), arg);
        }

        // Execute command using shell
        let output = Command::new("sh").arg("-c").arg(&command_str).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if custom.fail_on_error {
                output::error!(&format!("Command failed: {command_str}"));
                if !stderr.is_empty() {
                    println!("{stderr}");
                }
                std::process::exit(1);
            } else {
                output::warning!(&format!("Command failed (continuing): {command_str}"));
                if !stderr.is_empty() {
                    println!("{stderr}");
                }
            }
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                print!("{stdout}");
            }
            output::success!(&format!(
                "✓ {}",
                if custom.description.is_empty() {
                    &command_str
                } else {
                    &custom.description
                }
            ));
        }

        Ok(())
    }
}
