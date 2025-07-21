use std::path::{Path, PathBuf};
use std::time::Instant;
use std::sync::Arc;
use anyhow::Result;
use console;
use crate::parallel::{ExecutionStrategy, progress::{factories, ProgressReporter}};
use super::types::{ScanStats, Warning, ScanResult, Scanner, ScanFileResult};

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
/// ```rust,no_run
/// use guardy::scanner::types::ScanMode;
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
/// ```rust,no_run
/// use guardy::scanner::types::ScanMode;
/// use guardy::parallel::ExecutionStrategy;
/// use guardy::scanner::directory::DirectoryHandler;
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
/// ```rust,no_run
/// use guardy::scanner::types::ScanMode;
/// use guardy::parallel::ExecutionStrategy;
/// use guardy::scanner::directory::DirectoryHandler;
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

impl DirectoryHandler {
    /// Get the default directory handler
    pub fn default() -> Self {
        Self {
            rust: &["target"],
            nodejs: &["node_modules", "dist", "build", ".next", ".nuxt"],
            python: &["__pycache__", ".pytest_cache", "venv", ".venv", "env", ".env"],
            go: &["vendor"],
            java: &["out"],
            generic: &["cache", ".cache", "tmp", ".tmp", "temp", ".temp"],
            vcs: &[".git", ".svn", ".hg"],
            ide: &[".vscode", ".idea", ".vs"],
            coverage: &["coverage", ".nyc_output"],
        }
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
    /// use guardy::scanner::directory::DirectoryHandler;
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
    ///    use guardy::scanner::directory::DirectoryHandler;
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
    /// use guardy::scanner::directory::DirectoryHandler;
    /// use guardy::scanner::Scanner;
    /// use guardy::config::GuardyConfig;
    /// 
    /// # fn example() -> anyhow::Result<()> {
    /// let config = GuardyConfig::load(None, None::<&()>)?;
    /// let scanner = Scanner::new(&config)?;
    /// let directory_handler = DirectoryHandler::default();
    /// let result = directory_handler.scan(
    ///     Arc::new(scanner), 
    ///     Path::new("/tmp"), 
    ///     None  // Use config-based strategy
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn scan(&self, scanner: Arc<Scanner>, path: &Path, strategy: Option<ExecutionStrategy>) -> Result<ScanResult> {
        let start_time = Instant::now();
        let mut warnings: Vec<Warning> = Vec::new();

        // Determine execution strategy (smart mode by default)
        let execution_strategy = strategy.unwrap_or_else(|| {
            let file_count = scanner.fast_count_files(path).unwrap_or(0);
            
            match &scanner.config.mode {
                super::types::ScanMode::Sequential => ExecutionStrategy::Sequential,
                
                super::types::ScanMode::Parallel | super::types::ScanMode::Auto => {
                    // Calculate optimal workers based on available system resources
                    let max_workers_by_resources = ExecutionStrategy::calculate_optimal_workers(
                        scanner.config.max_threads,
                        scanner.config.thread_percentage,
                    );
                    
                    // Apply domain-specific file count adaptation
                    let optimal_workers = Self::adapt_workers_for_file_count(file_count, max_workers_by_resources);
                    
                    match &scanner.config.mode {
                        super::types::ScanMode::Parallel => ExecutionStrategy::Parallel { 
                            workers: optimal_workers
                        },
                        super::types::ScanMode::Auto => ExecutionStrategy::auto(
                            file_count, 
                            scanner.config.min_files_for_parallel,
                            optimal_workers
                        ),
                        _ => unreachable!(), // Already handled Sequential above
                    }
                }
            }
        });

        // Common setup: file counting and directory analysis
        let file_count = scanner.fast_count_files(path)?;
        let (mode_icon, mode_text) = match &execution_strategy {
            ExecutionStrategy::Sequential => ("🔍", String::new()),
            ExecutionStrategy::Parallel { workers } => ("⚡", format!(" using {} workers", workers)),
        };
        
        println!("{} Scanning {} files{}...", mode_icon, file_count, mode_text);
        
        // Analyze directories and display results
        let analysis = self.analyze_directories(path);
        analysis.display();

        // Collect all file paths using unified walker logic
        let file_paths = self.collect_file_paths(&scanner, path, &mut warnings)?;
        
        // Create progress reporter based on strategy with proper icons and frequency
        let progress_reporter = match &execution_strategy {
            ExecutionStrategy::Sequential => Some(factories::sequential_reporter("files").with_icon("⏳").with_frequency(10)),
            ExecutionStrategy::Parallel { .. } => Some(factories::parallel_reporter("files").with_icon("⚡").with_frequency(5)),
        };

        // Execute file scanning using the generic parallel framework with Arc
        let scan_results = execution_strategy.execute(
            file_paths,
            {
                let scanner = scanner.clone();
                move |file_path: &PathBuf| -> ScanFileResult {
                    match scanner.scan_single_path(file_path) {
                        Ok(matches) => ScanFileResult {
                            matches,
                            file_path: file_path.to_string_lossy().to_string(),
                            success: true,
                            error: None,
                        },
                        Err(e) => ScanFileResult {
                            matches: Vec::new(),
                            file_path: file_path.to_string_lossy().to_string(),
                            success: false,
                            error: Some(e.to_string()),
                        },
                    }
                }
            },
            progress_reporter.clone().map(|reporter| {
                move |current: usize, total: usize, worker_id: usize| {
                    reporter.report(current, total, worker_id);
                }
            }),
        )?;

        // Clear progress line using proper method
        if !scan_results.is_empty() {
            // Use the reporter's clear method if we have one
            if let Some(ref reporter) = progress_reporter {
                reporter.clear();
            }
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

        // Show timing summary
        let summary_icon = match &execution_strategy {
            ExecutionStrategy::Sequential => "⏱️",
            ExecutionStrategy::Parallel { workers: _ } => "⚡",
        };
        let mode_info = match &execution_strategy {
            ExecutionStrategy::Sequential => String::new(),
            ExecutionStrategy::Parallel { workers } => format!(" ({} workers)", workers),
        };

        println!("{}  Scan completed in {:.2}s ({} files scanned, {} matches found{})", 
                 summary_icon,
                 scan_duration.as_secs_f64(), 
                 stats.files_scanned, 
                 stats.total_matches,
                 mode_info);

        Ok(ScanResult {
            matches: all_matches,
            stats,
            warnings,
        })
    }

    /// Collect file paths from directory walker (shared by both modes)
    fn collect_file_paths(&self, scanner: &Arc<Scanner>, path: &Path, warnings: &mut Vec<Warning>) -> Result<Vec<PathBuf>> {
        let walker = scanner.build_directory_walker(path).build();
        let mut file_paths = Vec::new();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        file_paths.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    warnings.push(Warning {
                        message: format!("Walk error: {}", e),
                    });
                }
            }
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
                gitignore_content.lines()
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
                let dir_with_slash = format!("{}/", dir_name);
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
        println!("📁 Discovered {} director{}:", 
                 total_dirs, 
                 if total_dirs == 1 { "y" } else { "ies" });
        
        // Show properly ignored directories
        for (dir, description) in &self.properly_ignored {
            println!("   ✅ {} ({})", 
                console::style(dir).green(),
                console::style(description).dim()
            );
        }
        
        // Show directories that need gitignore rules
        for (dir, description) in &self.needs_gitignore {
            println!("   ⚠️  {} ({})", 
                console::style(dir).yellow(),
                console::style(description).dim()
            );
        }
        
        // Only show gitignore recommendations for directories that need them
        if !self.needs_gitignore.is_empty() {
            let patterns: Vec<&str> = self.needs_gitignore.iter().map(|(dir, _)| dir.as_str()).collect();
            println!("💡 Consider adding to .gitignore: {}", 
                     console::style(patterns.join(", ")).cyan());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_directory_handler() {
        let handler = DirectoryHandler::default();
        
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
        let handler = DirectoryHandler::default();
        let analyzable = handler.analyzable_directories();
        
        // Should include common build directories
        assert!(analyzable.iter().any(|(name, _)| *name == "target"));
        assert!(analyzable.iter().any(|(name, _)| *name == "node_modules"));
        assert!(analyzable.iter().any(|(name, _)| *name == "__pycache__"));
    }

    #[test]
    fn test_directory_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let handler = DirectoryHandler::default();
        
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
        let handler = DirectoryHandler::default();
        
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