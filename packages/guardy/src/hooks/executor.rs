use anyhow::{Context, Result, anyhow};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::PathBuf;
use std::process::Command;

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::git::GitRepo;
use crate::scan_v1::Scanner;

use super::config::{CustomCommand, HookConfig};

pub struct HookExecutor {
    config: GuardyConfig,
}

impl HookExecutor {
    pub fn new(config: GuardyConfig) -> Self {
        Self { config }
    }

    pub async fn execute(&self, hook_name: &str, args: &[String]) -> Result<()> {
        let hook_config_value = self.config.get_section("hooks")?;
        let hook_config: HookConfig = serde_json::from_value(hook_config_value)?;

        let hook = hook_config
            .hooks
            .get(hook_name)
            .ok_or_else(|| anyhow!("Hook '{}' not found in configuration", hook_name))?;

        if !hook.enabled {
            output::info!(&format!("Hook '{hook_name}' is disabled"));
            return Ok(());
        }

        output::info!(&format!("Executing {hook_name} hook..."));

        // Execute builtin commands
        for builtin in &hook.builtin {
            self.execute_builtin(builtin, hook_name, args).await?;
        }

        // Execute custom commands - either in parallel or sequentially
        if hook.parallel {
            self.execute_custom_parallel(&hook.custom, hook_name)
                .await?;
        } else {
            self.execute_custom_sequential(&hook.custom, hook_name)
                .await?;
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
                self.validate_commit_msg(&args[0]).await
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

            if scan_result.stats.total_matches > 5 {
                println!("  ... and {} more", scan_result.stats.total_matches - 5);
            }

            println!("\nCommit aborted. Remove secrets before committing.");
            return Err(anyhow!("Secrets detected in staged files"));
        }

        output::success!(&format!(
            "✅ Scanned {} files - no secrets found",
            scan_result.stats.files_scanned
        ));
        Ok(())
    }

    async fn validate_commit_msg(&self, commit_file: &str) -> Result<()> {
        output::info!("Validating commit message format...");

        let commit_msg =
            std::fs::read_to_string(commit_file).context("Failed to read commit message file")?;

        // Remove comments and trailing whitespace
        let commit_msg = commit_msg
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        if commit_msg.is_empty() {
            return Err(anyhow!("Empty commit message"));
        }

        // Use git-conventional to parse and validate
        match git_conventional::Commit::parse(&commit_msg) {
            Ok(commit) => {
                let scope_str = commit.scope().map(|s| s.as_str()).unwrap_or("no scope");

                output::success!(&format!(
                    "✅ Valid conventional commit: {} ({})",
                    commit.type_(),
                    scope_str
                ));

                // Optional: Add custom validation rules
                if commit.type_() == git_conventional::Type::FEAT && commit.scope().is_none() {
                    output::warning!("Consider adding a scope to feat commits");
                }

                Ok(())
            }
            Err(e) => {
                output::error!(&format!("❌ Invalid conventional commit format: {e}"));
                output::info!("Expected format: <type>(<scope>): <description>");
                output::info!("Examples:");
                output::info!("  feat(auth): add login functionality");
                output::info!("  fix(ui): correct button alignment");
                output::info!("  docs: update README");
                Err(anyhow!(
                    "Commit message does not follow conventional commits format"
                ))
            }
        }
    }

    async fn execute_custom_sequential(
        &self,
        commands: &[CustomCommand],
        hook_name: &str,
    ) -> Result<()> {
        for cmd in commands {
            self.execute_custom_command(cmd, hook_name).await?;
        }
        Ok(())
    }

    async fn execute_custom_parallel(
        &self,
        commands: &[CustomCommand],
        hook_name: &str,
    ) -> Result<()> {
        use crate::profiling::{ProfilingConfig, WorkloadProfiler};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        // Profile the workload to determine optimal parallelism
        let profiling_config = ProfilingConfig {
            max_threads: 0, // No limit
            thread_percentage: 75,
            min_items_for_parallel: 2, // Low threshold for hook commands
        };

        // Use custom adapter for hook-specific optimization
        let strategy = WorkloadProfiler::profile_with_adapter(
            commands.len(),
            &profiling_config,
            |count, max_workers| {
                // Start with standard workload adaptation
                let base_workers = WorkloadProfiler::adapt_workers_to_workload(count, max_workers);

                // Apply hook-specific constraints: commands may involve I/O, so be more conservative
                // and never exceed command count (no point in more workers than commands)
                let hook_optimized = std::cmp::min(base_workers, count);

                // Cap at reasonable limit for hook commands (they're usually not CPU-intensive)
                std::cmp::min(hook_optimized, 8)
            },
        );

        // If profiling suggests sequential, fall back to sequential execution
        if matches!(strategy, crate::parallel::ExecutionStrategy::Sequential) {
            return self.execute_custom_sequential(commands, hook_name).await;
        }

        // Extract worker count from strategy
        let max_concurrent =
            if let crate::parallel::ExecutionStrategy::Parallel { workers } = strategy {
                workers
            } else {
                4 // Fallback default
            };

        output::info!(&format!(
            "Running {} commands in parallel (max {max_concurrent} concurrent)",
            commands.len()
        ));

        let errors = Arc::new(Mutex::new(Vec::new()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut handles = Vec::new();

        for cmd in commands {
            let cmd = cmd.clone();
            let hook_name = hook_name.to_string();
            let errors = errors.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            // Run each command in its own task with concurrency limit
            let handle = tokio::spawn(async move {
                // Execute the command directly without needing self
                let result = execute_single_command(&cmd, &hook_name).await;
                drop(permit); // Release semaphore permit
                if let Err(e) = result {
                    let mut errs = errors.lock().await;
                    errs.push(e);
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await?;
        }

        // Check if there were any errors
        let errs = errors.lock().await;
        if !errs.is_empty() {
            let error_msg = errs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(anyhow!("Parallel execution failed:\n{}", error_msg));
        }

        Ok(())
    }

    async fn execute_custom_command(&self, cmd: &CustomCommand, hook_name: &str) -> Result<()> {
        output::info!(&cmd.description);

        // Get files to operate on
        let files = self.get_files_for_command(cmd, hook_name)?;

        // Build the command with file substitution
        let command_str = if files.is_empty() {
            // No files to process, run command as-is
            cmd.command.clone()
        } else {
            // Replace {files} placeholder with actual file list
            let files_str = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ");

            cmd.command.replace("{files}", &files_str)
        };

        // Execute the command
        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &command_str]);
            c
        };

        let output = command.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if cmd.fail_on_error {
                output::error!(&format!("✗ {}", cmd.description));
                return Err(anyhow!("Command failed: {}", stderr));
            } else {
                output::warning!(&format!("⚠ {} (non-fatal)", cmd.description));
            }
        } else {
            output::success!(&format!("✓ {}", cmd.description));

            // If stage_fixed is enabled, stage any modified files
            if cmd.stage_fixed && !files.is_empty() {
                self.stage_modified_files(&files)?;
            }
        }

        Ok(())
    }

    fn get_files_for_command(&self, cmd: &CustomCommand, hook_name: &str) -> Result<Vec<PathBuf>> {
        let repo = GitRepo::discover()?;

        // Get base file list
        let mut files = if cmd.all_files {
            // Get all files in repository matching the glob patterns
            if cmd.glob.is_empty() {
                return Err(anyhow!("all_files requires glob patterns to be specified"));
            }
            self.get_all_files_matching_globs(&cmd.glob)?
        } else {
            // Default to staged files for pre-commit
            if hook_name == "pre-commit" {
                repo.get_staged_files()?
            } else {
                vec![]
            }
        };

        // Apply glob filtering if specified
        if !cmd.glob.is_empty() && !cmd.all_files {
            files = self.filter_by_globs(&files, &cmd.glob)?;
        }

        Ok(files)
    }

    fn get_all_files_matching_globs(&self, globs: &[String]) -> Result<Vec<PathBuf>> {
        let glob_set = self.build_glob_set(globs)?;
        let mut matching_files = Vec::new();

        // Walk the current directory recursively
        for entry in walkdir::WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if glob_set.is_match(path) {
                matching_files.push(path.to_path_buf());
            }
        }

        Ok(matching_files)
    }

    fn filter_by_globs(&self, files: &[PathBuf], globs: &[String]) -> Result<Vec<PathBuf>> {
        let glob_set = self.build_glob_set(globs)?;

        Ok(files
            .iter()
            .filter(|path| glob_set.is_match(path.as_path()))
            .cloned()
            .collect())
    }

    fn build_glob_set(&self, globs: &[String]) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for pattern in globs {
            builder.add(Glob::new(pattern)?);
        }
        Ok(builder.build()?)
    }

    fn stage_modified_files(&self, files: &[PathBuf]) -> Result<()> {
        let files_to_stage: Vec<String> = files
            .iter()
            .filter(|path| path.exists())
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        if files_to_stage.is_empty() {
            return Ok(());
        }

        output::info!(&format!(
            "Staging {} modified files...",
            files_to_stage.len()
        ));

        let mut command = Command::new("git");
        command.arg("add");
        command.args(&files_to_stage);

        let output = command.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            output::warning!(&format!("Failed to stage some files: {stderr}"));
        } else {
            output::success!(&format!("Staged {} files", files_to_stage.len()));
        }

        Ok(())
    }
}

// Standalone function for parallel execution
async fn execute_single_command(cmd: &CustomCommand, hook_name: &str) -> Result<()> {
    use crate::git::GitRepo;

    output::info!(&cmd.description);

    // Get files to operate on
    let repo = GitRepo::discover()?;
    let mut files = if cmd.all_files {
        // Get all files in repository matching the glob patterns
        if cmd.glob.is_empty() {
            return Err(anyhow!("all_files requires glob patterns to be specified"));
        }
        get_all_files_matching_globs(&cmd.glob)?
    } else {
        // Default to staged files for pre-commit
        if hook_name == "pre-commit" {
            repo.get_staged_files()?
        } else {
            vec![]
        }
    };

    // Apply glob filtering if specified
    if !cmd.glob.is_empty() && !cmd.all_files {
        files = filter_by_globs(&files, &cmd.glob)?;
    }

    // Build the command with file substitution
    let command_str = if files.is_empty() {
        cmd.command.clone()
    } else {
        let files_str = files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        cmd.command.replace("{files}", &files_str)
    };

    // Execute the command
    let mut command = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", &command_str]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &command_str]);
        c
    };

    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if cmd.fail_on_error {
            output::error!(&format!("✗ {}", &cmd.description));
            return Err(anyhow!("Command failed: {}", stderr));
        } else {
            output::warning!(&format!("⚠ {} (non-fatal)", &cmd.description));
        }
    } else {
        output::success!(&format!("✓ {}", &cmd.description));

        // If stage_fixed is enabled, stage any modified files
        if cmd.stage_fixed && !files.is_empty() {
            stage_modified_files(&files)?;
        }
    }

    Ok(())
}

fn get_all_files_matching_globs(globs: &[String]) -> Result<Vec<PathBuf>> {
    let glob_set = build_glob_set(globs)?;
    let mut matching_files = Vec::new();

    for entry in walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if glob_set.is_match(path) {
            matching_files.push(path.to_path_buf());
        }
    }

    Ok(matching_files)
}

fn filter_by_globs(files: &[PathBuf], globs: &[String]) -> Result<Vec<PathBuf>> {
    let glob_set = build_glob_set(globs)?;

    Ok(files
        .iter()
        .filter(|path| glob_set.is_match(path.as_path()))
        .cloned()
        .collect())
}

fn build_glob_set(globs: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in globs {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn stage_modified_files(files: &[PathBuf]) -> Result<()> {
    let files_to_stage: Vec<String> = files
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.to_string_lossy().to_string())
        .collect();

    if files_to_stage.is_empty() {
        return Ok(());
    }

    output::info!(&format!(
        "Staging {} modified files...",
        files_to_stage.len()
    ));

    let mut command = Command::new("git");
    command.arg("add");
    command.args(&files_to_stage);

    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        output::warning!(&format!("Failed to stage some files: {stderr}"));
    } else {
        output::success!(&format!("Staged {} files", files_to_stage.len()));
    }

    Ok(())
}
