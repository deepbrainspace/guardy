use super::filters::directory::BinaryFilter;
use super::types::{ScanFileResult, ScanResult, ScanStats, Scanner, Warning};
use crate::cli::output;
use crate::parallel::{ExecutionStrategy, progress::factories};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Check if a file should be treated as binary using configured extensions
pub(crate) fn is_binary_file_by_extension(path: &Path, binary_extensions: &[String]) -> bool {
    if let Some(extension) = path.extension()
        && let Some(ext_str) = extension.to_str()
    {
        let ext_lower = ext_str.to_lowercase();
        return binary_extensions.contains(&ext_lower);
    }
    false
}

/// Check if a file is binary using content inspection
pub(crate) fn is_binary_file_by_content(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;

    // Try to read a small sample of the file
    if let Ok(mut file) = File::open(path) {
        let mut buffer = vec![0; 512]; // Read first 512 bytes
        if let Ok(bytes_read) = file.read(&mut buffer) {
            buffer.truncate(bytes_read);
            content_inspector::inspect(&buffer).is_binary()
        } else {
            false
        }
    } else {
        false
    }
}

/// Hybrid binary file detection: fast extension check first, then content inspection
pub(crate) fn is_binary_file(path: &Path, binary_extensions: &[String]) -> bool {
    // Fast extension-based check first
    if is_binary_file_by_extension(path, binary_extensions) {
        return true;
    }

    // For unknown extensions, use content inspection as fallback
    is_binary_file_by_content(path)
}

/// Directory handling for the scanner - combines filtering and analysis logic
///
/// # Architecture Responsibilities
///
/// The DirectoryHandler is responsible for **domain-specific logic** related to file scanning:
///
/// ## What This Module Does:
/// - **File Discovery**: Walks directory trees and collects file paths
/// - **Directory Filtering**: Skips build/cache directories (node_modules, target, etc.)
/// - **Workload Analysis**: Analyzes file counts and suggests gitignore improvements
/// - **Domain Adaptation**: Adapts parallel worker counts based on scanning workload characteristics
/// - **Execution Coordination**: Orchestrates the parallel scanning process
///
/// ## Worker Count Adaptation Strategy
///
/// This module implements **domain-specific worker adaptation** based on file scanning characteristics:
///
/// ```text
/// File Count Ranges → Worker Adaptation
/// ≤10 files    → Use min(2, max_workers)        # Minimal parallelism
/// ≤50 files    → Use 50% of max_workers        # Conservative
/// ≤100 files   → Use 75% of max_workers        # Moderate
/// >100 files   → Use 100% of max_workers       # Aggressive
/// ```
///
/// **Rationale**: Small file counts have overhead that outweighs parallel benefits,
/// while large file counts benefit from full parallelization.
///
/// # Integration with Parallel Module
///
/// This module works in coordination with the parallel module:
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │                   Execution Strategy Flow                   │
/// └─────────────────────────────────────────────────────────────┘
///
/// 1. Scanner Config       →  2. Resource Calculation      →  3. Domain Adaptation
///    ┌─────────────────┐      ┌──────────────────────────┐     ┌─────────────────────┐
///    │ • max_threads   │      │ CPU cores: 16            │     │ File count: 36      │
///    │ • thread_%: 75% │  ──▶ │ 16 * 75% = 12 workers    │ ──▶ │ ≤50 → 12/2 = 6     │
///    │ • mode: auto    │      │ (system resource limit)  │     │ (domain adaptation)  │
///    └─────────────────┘      └──────────────────────────┘     └─────────────────────┘
///                                                                        │
/// 4. Strategy Decision                          ← ← ← ← ← ← ← ← ← ← ← ← ← ←
///    ┌─────────────────────────────────────────┐
///    │ auto(file_count=36, threshold=50, workers=6) │
///    │ → 36 < 50 → ExecutionStrategy::Sequential    │
///    └─────────────────────────────────────────────┘
/// ```
///
/// # Mode-Specific Behavior
///
/// ## Sequential Mode
/// ```ignore
/// use guardy::scan_v1::types::ScanMode;
/// use guardy::parallel::ExecutionStrategy;
/// # let mode = ScanMode::Sequential;
/// let strategy = match mode {
///     ScanMode::Sequential => ExecutionStrategy::Sequential,
///     _ => unreachable!(),
/// };
/// ```
/// - No worker calculation
/// - Single-threaded execution
/// - No resource overhead
///
/// ## Parallel Mode
/// ```ignore
/// use guardy::scan_v1::types::ScanMode;
/// use guardy::parallel::ExecutionStrategy;
/// use guardy::scan_v1::directory::DirectoryHandler;
/// # let mode = ScanMode::Parallel;
/// # let files = 100;
/// # struct Config { max_threads: usize, thread_percentage: u8 }
/// # let config = Config { max_threads: 0, thread_percentage: 75 };
/// let strategy = match mode {
///     ScanMode::Parallel => {
///         let max_workers = ExecutionStrategy::calculate_optimal_workers(config.max_threads, config.thread_percentage);     // Resource-based
///         let optimal_workers = DirectoryHandler::adapt_workers_for_file_count(files, max_workers); // Domain-adaptive
///         ExecutionStrategy::Parallel { workers: optimal_workers }
///     },
///     _ => unreachable!(),
/// };
/// ```
/// - Always uses parallelism
/// - Worker count adapted to file count
///
/// ## Auto Mode
/// ```ignore
/// use guardy::scan_v1::types::ScanMode;
/// use guardy::parallel::ExecutionStrategy;
/// use guardy::scan_v1::directory::DirectoryHandler;
/// # let mode = ScanMode::Auto;
/// # let files = 36;
/// # let threshold = 50;
/// # struct Config { max_threads: usize, thread_percentage: u8 }
/// # let config = Config { max_threads: 0, thread_percentage: 75 };
/// let strategy = match mode {
///     ScanMode::Auto => {
///         let max_workers = ExecutionStrategy::calculate_optimal_workers(config.max_threads, config.thread_percentage);     // Resource-based
///         let optimal_workers = DirectoryHandler::adapt_workers_for_file_count(files, max_workers); // Domain-adaptive
///         ExecutionStrategy::auto(files, threshold, optimal_workers)  // Threshold decision
///     },
///     _ => unreachable!(),
/// };
/// ```
/// - Uses file count threshold to decide sequential vs parallel
/// - If parallel chosen, uses domain-adapted worker count
///
/// # Key Methods
///
/// ## Worker Adaptation
/// `adapt_workers_for_file_count()` - Adapts worker count based on file scanning domain knowledge
///
/// ## Unified Scanning
/// [`DirectoryHandler::scan()`] - Main entry point that orchestrates the entire scanning process
#[derive(Debug)]
pub struct DirectoryHandler {
    /// Rust-specific directories
    pub rust: &'static [&'static str],
    /// Node.js/JavaScript directories
    pub nodejs: &'static [&'static str],
    /// Python directories
    pub python: &'static [&'static str],
    /// Go directories
    pub go: &'static [&'static str],
    /// Java directories
    pub java: &'static [&'static str],
    /// Generic build/cache directories
    pub generic: &'static [&'static str],
    /// Version control directories
    pub vcs: &'static [&'static str],
    /// IDE directories
    pub ide: &'static [&'static str],
    /// Test coverage directories
    pub coverage: &'static [&'static str],
}

impl Default for DirectoryHandler {
    fn default() -> Self {
        Self {
            rust: &["target"],
            nodejs: &["node_modules", "dist", "build", ".next", ".nuxt"],
            python: &[
                "__pycache__",
                ".pytest_cache",
                "venv",
                ".venv",
                "env",
                ".env",
            ],
            go: &["vendor"],
            java: &["out"],
            generic: &["cache", ".cache", "tmp", ".tmp", "temp", ".temp"],
            vcs: &[".git", ".svn", ".hg"],
            ide: &[".vscode", ".idea", ".vs"],
            coverage: &["coverage", ".nyc_output"],
        }
    }
}

impl DirectoryHandler {
    /// Create a new directory handler with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all directory names that should be filtered during scanning
    pub fn all_filtered_directories(&self) -> Vec<&'static str> {
        let mut dirs = Vec::new();
        dirs.extend_from_slice(self.rust);
        dirs.extend_from_slice(self.nodejs);
        dirs.extend_from_slice(self.python);
        dirs.extend_from_slice(self.go);
        dirs.extend_from_slice(self.java);
        dirs.extend_from_slice(self.generic);
        dirs.extend_from_slice(self.vcs);
        dirs.extend_from_slice(self.ide);
        dirs.extend_from_slice(self.coverage);
        dirs
    }

    /// Check if a directory name should be filtered (skipped) during scanning
    pub fn should_filter_directory(&self, dir_name: &str) -> bool {
        self.all_filtered_directories().contains(&dir_name)
    }

    /// Get directories that should be analyzed for gitignore patterns
    /// These are typically build/cache directories that projects generate
    pub fn analyzable_directories(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            // Rust
            ("target", "Rust build directory"),
            // Node.js
            ("node_modules", "Node.js dependencies"),
            ("dist", "Build output directory"),
            ("build", "Build output directory"),
            // Python
            ("__pycache__", "Python cache directory"),
            ("venv", "Python virtual environment"),
            (".venv", "Python virtual environment"),
            // Go
            ("vendor", "Go dependencies"),
        ]
    }

    /// Adapt worker count based on file count (domain-specific logic)
    ///
    /// This method implements file scanning domain knowledge to optimize parallel execution:
    ///
    /// # Parameters
    /// - `file_count`: Number of files to be scanned
    /// - `max_workers`: Maximum workers available (from resource calculation)
    ///
    /// # Returns
    /// Optimal worker count for the given file count, constrained by max_workers
    ///
    /// # Adaptation Strategy
    /// ```text
    /// ≤10 files    → min(2, max_workers)         # Minimal overhead
    /// ≤50 files    → min(max_workers/2, max_workers)   # Conservative scaling
    /// ≤100 files   → min(max_workers*3/4, max_workers) # Moderate scaling
    /// >100 files   → max_workers                 # Full utilization
    /// ```
    ///
    /// # Rationale
    /// - **Small workloads**: Parallel overhead exceeds benefits
    /// - **Medium workloads**: Conservative parallelism provides good balance
    /// - **Large workloads**: Full parallelism maximizes throughput
    ///
    /// # Example
    /// ```rust
    /// use guardy::scan_v1::directory::DirectoryHandler;
    ///
    /// // System has 16 cores, config allows 12 workers
    /// let workers = DirectoryHandler::adapt_workers_for_file_count(36, 12);
    /// assert_eq!(workers, 6); // 36 ≤ 50, so 12/2 = 6
    /// ```
    pub fn adapt_workers_for_file_count(file_count: usize, max_workers: usize) -> usize {
        if file_count <= 10 {
            // Very small workloads: use minimal parallelism
            std::cmp::min(2, max_workers)
        } else if file_count <= 50 {
            // Small workloads: use conservative parallelism
            std::cmp::min(max_workers / 2, max_workers)
        } else if file_count <= 100 {
            // Medium workloads: use moderate parallelism
            std::cmp::min((max_workers * 3) / 4, max_workers)
        } else {
            // Large workloads: use full available parallelism
            max_workers
        }
        // The result is capped by file count in the parallel executor
    }

    /// Unified directory scanning method that orchestrates the entire scanning process
    ///
    /// This is the main entry point for file scanning that coordinates between system resource
    /// management and domain-specific scanning logic.
    ///
    /// # Parameters
    /// - `scanner`: Arc-wrapped scanner instance for thread-safe sharing
    /// - `path`: Directory path to scan
    /// - `strategy`: Optional execution strategy override (None uses config-based decision)
    ///
    /// # Returns
    /// Complete scan results including matches, statistics, and warnings
    ///
    /// # Execution Flow
    ///
    /// 1. **Strategy Determination**: Decides execution strategy based on config mode:
    ///    - `Sequential`: Always single-threaded
    ///    - `Parallel`: Always multi-threaded with domain-adapted worker count
    ///    - `Auto`: Uses file count threshold to decide + domain-adapted workers
    ///
    /// 2. **Resource Calculation**: For parallel modes, calculates optimal workers:
    ///    ```rust,no_run
    ///    use guardy::parallel::ExecutionStrategy;
    ///    # struct Config { max_threads: usize, thread_percentage: u8 }
    ///    # struct Scanner { config: Config }
    ///    # let scanner = Scanner { config: Config { max_threads: 0, thread_percentage: 75 } };
    ///    let max_workers = ExecutionStrategy::calculate_optimal_workers(
    ///        scanner.config.max_threads,      // User limit
    ///        scanner.config.thread_percentage // CPU percentage
    ///    );
    ///    ```
    ///
    /// 3. **Domain Adaptation**: Adapts workers based on file count:
    ///    ```rust,no_run
    ///    use guardy::scan_v1::directory::DirectoryHandler;
    ///    # let file_count = 36;
    ///    # let max_workers = 12;
    ///    let optimal_workers = DirectoryHandler::adapt_workers_for_file_count(file_count, max_workers);
    ///    ```
    ///
    /// 4. **Directory Analysis**: Analyzes directories and suggests gitignore improvements
    ///
    /// 5. **File Collection**: Walks directory tree collecting file paths
    ///
    /// 6. **Parallel Execution**: Executes file scanning using chosen strategy
    ///
    /// 7. **Result Aggregation**: Combines results and generates statistics
    ///
    /// # Configuration Integration
    ///
    /// The method respects all scanner configuration:
    /// - `mode`: Sequential/Parallel/Auto execution strategy
    /// - `max_threads`: Hard limit on worker count (0 = no limit)
    /// - `thread_percentage`: Percentage of CPU cores to use
    /// - `min_files_for_parallel`: Threshold for auto mode decision
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use std::path::Path;
    /// use guardy::scan_v1::directory::DirectoryHandler;
    /// use guardy::scan_v1::Scanner;
    /// use guardy::config::GuardyConfig;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let config = GuardyConfig::load(None, None::<&()>, 0)?;
    /// let scanner = Scanner::new(&config)?;
    /// let directory_handler = DirectoryHandler::new();
    /// let result = directory_handler.scan(
    ///     Arc::new(scanner),
    ///     Path::new("/tmp"),
    ///     None  // Use config-based strategy
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn scan(
        &self,
        scanner: Arc<Scanner>,
        path: &Path,
        strategy: Option<ExecutionStrategy>,
    ) -> Result<ScanResult> {
        let start_time = Instant::now();
        let mut warnings: Vec<Warning> = Vec::new();

        // Determine execution strategy - always use parallel for best performance
        // (parallel executor automatically adapts to file count)
        let execution_strategy = strategy.unwrap_or_else(|| {
            match &scanner.config.mode {
                super::types::ScanMode::Sequential => ExecutionStrategy::Sequential,
                
                super::types::ScanMode::Parallel | super::types::ScanMode::Auto => {
                    // Calculate optimal workers based on available system resources
                    let optimal_workers = ExecutionStrategy::calculate_optimal_workers(
                        scanner.config.max_threads,
                        scanner.config.thread_percentage,
                    );

                    ExecutionStrategy::Parallel {
                        workers: optimal_workers,
                    }
                }
            }
        });

        // Note: Scanning message now shown by discovery progress bar in collect_file_paths()

        // Analyze directories and display results
        let analysis = self.analyze_directories(path);
        analysis.display();

        // Collect all file paths using unified walker logic
        let file_paths = self.collect_file_paths(&scanner, path, &mut warnings)?;

        // Now show scanning strategy message with worker count
        match &execution_strategy {
            ExecutionStrategy::Sequential => {
                output::styled!(
                    "{} Scanning {} files sequentially...",
                    ("🔍", "info_symbol"),
                    (file_paths.len().to_string(), "accent")
                );
            }
            ExecutionStrategy::Parallel { workers } => {
                output::styled!(
                    "{} Scanning {} files using {} workers...",
                    ("⚡", "info_symbol"),
                    (file_paths.len().to_string(), "accent"),
                    (workers.to_string(), "accent")
                );
            }
        };

        // Create enhanced progress reporter based on strategy
        let enhanced_progress = match &execution_strategy {
            ExecutionStrategy::Sequential => {
                Some(factories::enhanced_sequential_reporter(file_paths.len()))
            }
            ExecutionStrategy::Parallel { workers } => Some(factories::enhanced_parallel_reporter(
                file_paths.len(),
                *workers,
            )),
        };

        // Get statistics reference for tracking
        let stats = enhanced_progress.as_ref().map(|p| p.stats());

        // Execute file scanning using the generic parallel framework with enhanced progress
        let scan_results = execution_strategy.execute(
            file_paths,
            {
                let scanner = scanner.clone();
                let stats = stats.clone();
                let enhanced_progress_for_worker = enhanced_progress.clone();
                move |file_path: &PathBuf, worker_id: usize| -> ScanFileResult {
                    // Update worker bar with current file
                    if let Some(ref progress) = enhanced_progress_for_worker
                        && progress.is_parallel
                    {
                        progress.update_worker_file(worker_id, &file_path.to_string_lossy());
                    }

                    // All filtering now happens during directory walk - no need to re-filter here
                    // This eliminates the performance regression caused by repeated filter creation
                    match scanner.scan_single_path(file_path) {
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
            enhanced_progress.as_ref().map(|progress| {
                let progress = progress.clone();
                move |current: usize, total: usize, _worker_id: usize| {
                    // Update overall progress only
                    progress.update_overall(current, total);
                }
            }),
        )?;

        // Clear progress display using enhanced reporter
        if let Some(ref progress) = enhanced_progress {
            progress.finish();
        }

        // Aggregate results
        let mut all_matches = Vec::new();
        let mut files_scanned = 0;
        let mut files_skipped = 0;

        for result in scan_results {
            if result.success {
                files_scanned += 1;
                all_matches.extend(result.matches);
            } else {
                files_skipped += 1;
                if let Some(error) = result.error {
                    warnings.push(Warning {
                        message: format!("Failed to scan {}: {}", result.file_path, error),
                    });
                }
            }
        }

        let scan_duration = start_time.elapsed();
        let stats = ScanStats {
            files_scanned,
            files_skipped,
            total_matches: all_matches.len(),
            scan_duration_ms: scan_duration.as_millis() as u64,
        };

        // Binary files are tracked internally but not displayed to users

        // Show detailed filter statistics if in debug mode
        if tracing::enabled!(tracing::Level::DEBUG) {
            let binary_filter = BinaryFilter::new(!scanner.config.include_binary);
            let path_filter = super::filters::directory::PathFilter::new(scanner.config.ignore_paths.clone());
            
            // Get detailed statistics
            let binary_detailed_stats = binary_filter.get_statistics();
            let path_detailed_stats = path_filter.get_statistics();
            
            // Show detailed binary filter stats
            if binary_detailed_stats.files_checked > 0 {
                tracing::debug!("Binary Filter Performance:");
                tracing::debug!("  Files checked: {}", binary_detailed_stats.files_checked);
                tracing::debug!("  Binary by extension: {} ({:.1}%)", 
                    binary_detailed_stats.files_binary_by_extension,
                    (binary_detailed_stats.files_binary_by_extension as f64 / binary_detailed_stats.files_checked as f64) * 100.0
                );
                tracing::debug!("  Binary by content: {}", binary_detailed_stats.files_binary_by_content);
                tracing::debug!("  Text confirmed: {}", binary_detailed_stats.files_text_confirmed);
                tracing::debug!("  Extension cache hits: {}", binary_detailed_stats.extension_cache_hits);
                tracing::debug!("  Content inspections: {}", binary_detailed_stats.content_inspections_performed);
                tracing::debug!("  Content inspections skipped: {}", binary_detailed_stats.files_content_inspection_skipped);
            }
            
            // Show detailed path filter stats
            if path_detailed_stats.total_usage > 0 {
                tracing::debug!("Path Filter Performance:");
                tracing::debug!("  Total patterns: {}", path_detailed_stats.total_patterns);
                tracing::debug!("  Active patterns: {}", path_detailed_stats.active_patterns);
                tracing::debug!("  Total matches: {}", path_detailed_stats.total_usage);
                tracing::debug!("  Pattern efficiency: {:.1}%", 
                    (path_detailed_stats.active_patterns as f64 / path_detailed_stats.total_patterns as f64) * 100.0
                );
            }
            
            // Show size filter stats using trait method
            use super::filters::Filter;
            let size_filter = super::filters::directory::SizeFilter::new(scanner.config.max_file_size_mb);
            let size_stats = size_filter.get_stats();
            if !size_stats.is_empty() {
                tracing::debug!("Size Filter Configuration:");
                for (key, value) in size_stats {
                    tracing::debug!("  {}: {}", key, value);
                }
            }
        }

        // Show timing summary
        let (summary_icon, mode_info) = match &execution_strategy {
            ExecutionStrategy::Sequential => (output::symbols::STOPWATCH, String::new()),
            ExecutionStrategy::Parallel { workers } => {
                (output::symbols::LIGHTNING, format!(" ({workers} workers)"))
            }
        };

        output::styled!(
            "{} Scan completed in {}s ({} files scanned, {} matches found{})",
            (summary_icon, "success_symbol"),
            (format!("{:.2}", scan_duration.as_secs_f64()), "time"),
            (stats.files_scanned.to_string(), "number"),
            (stats.total_matches.to_string(), "accent"),
            (mode_info, "muted")
        );

        Ok(ScanResult {
            matches: all_matches,
            stats,
            warnings,
        })
    }

    /// Collect file paths from directory walker with all filtering applied during walk
    /// This matches scan-v3's architecture where filtering happens during discovery, not processing
    fn collect_file_paths(
        &self,
        scanner: &Arc<Scanner>,
        path: &Path,
        warnings: &mut Vec<Warning>,
    ) -> Result<Vec<PathBuf>> {
        use indicatif::{ProgressBar, ProgressStyle};
        
        let discovery_start = Instant::now();
        
        // Show progress bar only if not in debug/trace mode to prevent scrolling issues
        let show_progress_bar = !tracing::enabled!(tracing::Level::DEBUG);
        let discovery_bar = if show_progress_bar {
            let bar = ProgressBar::new_spinner();
            bar.set_style(
                ProgressStyle::with_template(
                    "📁 [{elapsed_precise}] {spinner} Discovering files... {msg}"
                )
                .unwrap()
                .tick_strings(&["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"])
            );
            bar.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(bar)
        } else {
            // In debug mode, just show the static message once
            output::styled!("📁 Discovering files...");
            None
        };
        
        // Create counter for PathFilter statistics
        let path_filter_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let walker = scanner.build_directory_walker(path, path_filter_counter.clone()).build();
        let mut file_paths = Vec::new();
        
        // Track filter statistics during walk
        let mut files_filtered_by_size = 0;
        let mut files_filtered_by_binary = 0;
        let mut files_discovered = 0;

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        let file_path = entry.path();
                        files_discovered += 1;
                        
                        // Update discovery progress every 100 files to avoid too much overhead
                        if files_discovered % 100 == 0
                            && let Some(ref bar) = discovery_bar {
                                bar.set_message(format!("Found {files_discovered} files"));
                            }
                        
                        // Apply file-level filters (path filtering already done during directory walk)
                        // This prevents filtered files from ever entering the work queue
                        
                        // 1. Size filter - using cached filter from Scanner
                        {
                            use super::filters::Filter;
                            match scanner.size_filter.filter(file_path) {
                                Ok(super::filters::FilterDecision::Skip(reason)) => {
                                    tracing::trace!("Filter '{}' skipped {}: {}", scanner.size_filter.name(), file_path.display(), reason);
                                    files_filtered_by_size += 1;
                                    continue;
                                }
                                Ok(super::filters::FilterDecision::Process) => {
                                    tracing::trace!("Filter '{}' allowed {}", scanner.size_filter.name(), file_path.display());
                                }
                                Err(e) => {
                                    tracing::warn!("Filter '{}' failed for {}: {}", scanner.size_filter.name(), file_path.display(), e);
                                    // If size filter fails, continue processing
                                }
                            }
                        }
                        
                        // 2. Binary filter - using cached filter from Scanner
                        {
                            use super::filters::Filter;
                            match scanner.binary_filter.filter(file_path) {
                                Ok(super::filters::FilterDecision::Skip(reason)) => {
                                    tracing::trace!("Filter '{}' skipped {}: {}", scanner.binary_filter.name(), file_path.display(), reason);
                                    files_filtered_by_binary += 1;
                                    continue;
                                }
                                Ok(super::filters::FilterDecision::Process) => {
                                    tracing::trace!("Filter '{}' allowed {}", scanner.binary_filter.name(), file_path.display());
                                }
                                Err(e) => {
                                    tracing::warn!("Filter '{}' failed for {}: {}", scanner.binary_filter.name(), file_path.display(), e);
                                    // If binary filter fails, continue processing
                                }
                            }
                        }
                        
                        // If we reach here, file passed all filters - add to work queue
                        file_paths.push(file_path.to_path_buf());
                    }
                }
                Err(e) => {
                    warnings.push(Warning {
                        message: format!("Walk error: {e}"),
                    });
                }
            }
        }
        
        // Finish discovery progress bar with final statistics
        let discovery_duration = discovery_start.elapsed();
        let files_filtered_by_path = path_filter_counter.load(std::sync::atomic::Ordering::Relaxed);
        
        // Format timing - use milliseconds for fast operations
        let timing_str = if discovery_duration.as_millis() < 1000 {
            format!("{}ms", discovery_duration.as_millis())
        } else {
            format!("{:.1}s", discovery_duration.as_secs_f64())
        };
        
        let final_message = format!(
            "Discovered {files_discovered} files in {timing_str}, selected {} for scanning (filtered: {files_filtered_by_path} path, {files_filtered_by_size} size, {files_filtered_by_binary} binary)",
            file_paths.len()
        );
        
        if let Some(bar) = discovery_bar {
            bar.finish_with_message(final_message);
        } else {
            // In debug mode, show the final summary
            output::styled!("📁 {final_message}");
        }
        
        // Log filter statistics like scan-v3
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!("Files discovered: {}", files_discovered);
            tracing::debug!("Files filtered by path: {}", files_filtered_by_path);
            tracing::debug!("Files filtered by size: {}", files_filtered_by_size);
            tracing::debug!("Files filtered by binary: {}", files_filtered_by_binary);
            tracing::debug!("Files selected for processing: {}", file_paths.len());
        }

        Ok(file_paths)
    }

    /// Analyze directories in the given path and return analysis results
    pub fn analyze_directories(&self, path: &Path) -> DirectoryAnalysis {
        let mut properly_ignored = Vec::new();
        let mut needs_gitignore = Vec::new();

        // Helper function to check if a pattern exists in gitignore
        let check_gitignore_pattern = |pattern: &str| -> bool {
            if let Ok(gitignore_content) = std::fs::read_to_string(path.join(".gitignore")) {
                gitignore_content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .any(|line| line == pattern || line == pattern.trim_end_matches('/'))
            } else {
                false
            }
        };

        // Check all analyzable directories
        for (dir_name, description) in self.analyzable_directories() {
            if path.join(dir_name).exists() {
                let dir_with_slash = format!("{dir_name}/");
                if check_gitignore_pattern(&dir_with_slash) || check_gitignore_pattern(dir_name) {
                    properly_ignored.push((dir_with_slash, description.to_string()));
                } else {
                    needs_gitignore.push((dir_with_slash, description.to_string()));
                }
            }
        }

        DirectoryAnalysis {
            properly_ignored,
            needs_gitignore,
        }
    }
}

/// Directory analysis results
#[derive(Debug)]
pub struct DirectoryAnalysis {
    pub properly_ignored: Vec<(String, String)>,
    pub needs_gitignore: Vec<(String, String)>,
}

impl DirectoryAnalysis {
    /// Display the analysis results to the user
    pub fn display(&self) {
        if self.properly_ignored.is_empty() && self.needs_gitignore.is_empty() {
            return;
        }

        let total_dirs = self.properly_ignored.len() + self.needs_gitignore.len();
        output::styled!(
            "{} Discovered {} director{}:",
            ("📁", "info_symbol"),
            (total_dirs.to_string(), "number"),
            (if total_dirs == 1 { "y" } else { "ies" }, "primary")
        );

        // Show properly ignored directories
        for (dir, description) in &self.properly_ignored {
            output::styled!(
                "   {} {} ({})",
                ("✔", "success_symbol"),
                (dir, "file_path"),
                (description, "muted")
            );
        }

        // Show directories that need gitignore rules
        for (dir, description) in &self.needs_gitignore {
            output::styled!(
                "   {} {} ({})",
                ("⚠️", "warning_symbol"),
                (dir, "file_path"),
                (description, "muted")
            );
        }

        // Only show gitignore recommendations for directories that need them
        if !self.needs_gitignore.is_empty() {
            let patterns: Vec<&str> = self
                .needs_gitignore
                .iter()
                .map(|(dir, _)| dir.as_str())
                .collect();
            output::info!(
                &format!(
                    "Consider adding to .gitignore: {}",
                    output::property_name(patterns.join(", "))
                ),
                output::symbols::LIGHTBULB
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_directory_handler() {
        let handler = DirectoryHandler::new();

        // Test some known directories
        assert!(handler.should_filter_directory("target"));
        assert!(handler.should_filter_directory("node_modules"));
        assert!(handler.should_filter_directory("__pycache__"));
        assert!(handler.should_filter_directory(".git"));

        // Test non-filtered directory
        assert!(!handler.should_filter_directory("src"));
        assert!(!handler.should_filter_directory("lib"));
    }

    #[test]
    fn test_analyzable_directories() {
        let handler = DirectoryHandler::new();
        let analyzable = handler.analyzable_directories();

        // Should include common build directories
        assert!(analyzable.iter().any(|(name, _)| *name == "target"));
        assert!(analyzable.iter().any(|(name, _)| *name == "node_modules"));
        assert!(analyzable.iter().any(|(name, _)| *name == "__pycache__"));
    }

    #[test]
    fn test_directory_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let handler = DirectoryHandler::new();

        // Create a target directory (not in gitignore)
        fs::create_dir(temp_dir.path().join("target")).unwrap();

        let analysis = handler.analyze_directories(temp_dir.path());

        // Should find target directory that needs gitignore
        assert_eq!(analysis.properly_ignored.len(), 0);
        assert_eq!(analysis.needs_gitignore.len(), 1);
        assert_eq!(analysis.needs_gitignore[0].0, "target/");
        assert_eq!(analysis.needs_gitignore[0].1, "Rust build directory");
    }

    #[test]
    fn test_directory_analysis_with_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let handler = DirectoryHandler::new();

        // Create target directory and gitignore file
        fs::create_dir(temp_dir.path().join("target")).unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "target/\n").unwrap();

        let analysis = handler.analyze_directories(temp_dir.path());

        // Should find properly ignored target directory
        assert_eq!(analysis.properly_ignored.len(), 1);
        assert_eq!(analysis.needs_gitignore.len(), 0);
        assert_eq!(analysis.properly_ignored[0].0, "target/");
        assert_eq!(analysis.properly_ignored[0].1, "Rust build directory");
    }
}
