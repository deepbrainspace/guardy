use crate::parallel::ExecutionStrategy;
use crate::scan::types::{ScannerConfig, SecretMatch};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Strategy - Manages execution strategies and threading coordination
///
/// Responsibilities:
/// - Calculate optimal execution strategy (Sequential/Parallel)
/// - Coordinate with the existing parallel module's ExecutionStrategy
/// - Adapt worker count based on file count
/// - Bridge between scan module and parallel module
pub struct Strategy {
    config: ScannerConfig,
}

impl Strategy {
    /// Create a new Strategy handler with configuration
    pub fn new(config: &ScannerConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Calculate optimal execution strategy based on file count and config
    pub fn calculate(config: &ScannerConfig, file_count: usize) -> Result<ExecutionStrategy> {
        // Determine strategy based on config mode
        match config.mode.as_str() {
            "sequential" => Ok(ExecutionStrategy::Sequential),
            "parallel" => {
                let workers = Self::calculate_worker_count(config, file_count);
                Ok(ExecutionStrategy::Parallel { workers })
            }
            "auto" | _ => {
                // Auto mode: choose based on file count threshold
                if file_count < config.min_files_for_parallel {
                    Ok(ExecutionStrategy::Sequential)
                } else {
                    let workers = Self::calculate_worker_count(config, file_count);
                    Ok(ExecutionStrategy::Parallel { workers })
                }
            }
        }
    }

    /// Calculate optimal worker count based on system resources and file count
    fn calculate_worker_count(config: &ScannerConfig, file_count: usize) -> usize {
        // Get system profile (cached - computed once per program run)
        let profile = system_profile::SystemProfile::get();

        // Calculate max workers based on config
        let max_workers = profile.calculate_workers_with_limit(
            config.thread_percentage,
            config.max_threads
        );

        // Apply domain-specific adaptation based on file count
        profile.adapt_workers_for_workload(file_count, max_workers)
    }

    /// Execute scanning with the chosen strategy
    ///
    /// This delegates to the existing parallel module's ExecutionStrategy implementation,
    /// similar to how the existing scanner does it in directory.rs:465-537
    pub fn execute(
        strategy: &ExecutionStrategy,
        file_paths: Vec<PathBuf>,
        scanner: Arc<crate::scan::core::Scanner>,
        progress_reporter: Option<Arc<crate::scan::progress::Progress>>,
        verbose_level: u8,
    ) -> Result<Vec<SecretMatch>> {
        // Get statistics reference for tracking
        let stats = progress_reporter.as_ref().map(|p| p.stats());

        // Structure to hold results from file scanning
        #[derive(Debug)]
        struct ScanFileResult {
            matches: Vec<SecretMatch>,
            file_path: String,
            success: bool,
            error: Option<String>,
        }

        // Execute file scanning using the generic parallel framework
        // This is the same pattern as directory.rs:466-537
        let scan_results = strategy.execute(
            file_paths,
            {
                let scanner = scanner.clone();
                let stats = stats.clone();
                let progress_for_worker = progress_reporter.clone();

                move |file_path: &PathBuf, worker_id: usize| -> ScanFileResult {
                    // Update worker bar with current file (if parallel)
                    if let Some(ref progress) = progress_for_worker {
                        if progress.is_parallel {
                            progress.update_worker_file(worker_id, &file_path.to_string_lossy());
                        }
                    }

                    // Check if this is a binary file first
                    if !scanner.config.include_binary &&
                       crate::scan::filters::directory::binary::is_binary_file(file_path, &scanner.config.binary_extensions) {
                        // Update statistics for binary files
                        if let Some(ref stats) = stats {
                            stats.increment_binary();
                        }
                        return ScanFileResult {
                            matches: Vec::new(),
                            file_path: file_path.to_string_lossy().to_string(),
                            success: true,
                            error: None,
                        };
                    }

                    // Process the file
                    match crate::scan::file::File::process_single_file(file_path, &scanner.config) {
                        Ok(matches) => {
                            // Update statistics - file was successfully scanned
                            if let Some(ref stats) = stats {
                                stats.increment_scanned();
                                if !matches.is_empty() {
                                    stats.increment_with_secrets();
                                }
                            }
                            ScanFileResult {
                                matches,
                                file_path: file_path.to_string_lossy().to_string(),
                                success: true,
                                error: None,
                            }
                        }
                        Err(e) => {
                            // Update statistics for errors
                            if let Some(ref stats) = stats {
                                stats.increment_skipped();
                            }
                            // Log error in verbose mode
                            if verbose_level > 0 {
                                tracing::debug!("Error scanning {}: {}", file_path.display(), e);
                            }
                            ScanFileResult {
                                matches: Vec::new(),
                                file_path: file_path.to_string_lossy().to_string(),
                                success: false,
                                error: Some(e.to_string()),
                            }
                        }
                    }
                }
            },
            // Progress callback
            progress_reporter.as_ref().map(|progress| {
                let progress = progress.clone();
                move |current: usize, total: usize, _worker_id: usize| {
                    // Update overall progress only
                    progress.update_overall(current, total);
                }
            }),
        )?;

        // Collect all matches from results
        let mut all_matches = Vec::new();
        for result in scan_results {
            if result.success && !result.matches.is_empty() {
                all_matches.extend(result.matches);
            }
        }

        // Finish progress reporting
        if let Some(ref progress) = progress_reporter {
            progress.finish();
        }

        Ok(all_matches)
    }
}