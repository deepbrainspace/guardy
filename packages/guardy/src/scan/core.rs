use crate::parallel::ExecutionStrategy;
use crate::scan::{
    directory::Directory,
    file::File,
    pattern::Pattern,
    progress::{Progress, ProgressSnapshot},
    strategy::Strategy,
    types::{ScannerConfig, ScanResult, ScanStats, SecretMatch},
    filters::{
        directory::{PathFilter, SizeFilter, BinaryFilter},
        content::{ContextPrefilter, CommentFilter, EntropyFilter}
    }
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Core - Main orchestrator that coordinates all scanning phases
///
/// Following proper OOP principles, Core delegates to specialized objects:
/// - Directory: file system operations
/// - Strategy: execution strategies & threading  
/// - Progress: visual feedback & statistics
/// - Pattern: secret patterns & regex
/// - File: individual file processing
/// - Filters: Two-tier filtering system (directory + content level)
///
/// ## Scanning Pipeline
/// 
/// The core orchestrates a sophisticated scanning pipeline:
/// 
/// ```text
/// Input Paths → Directory Traversal → Directory Filters → File Processing → Content Filters → Results
///      ↓              ↓                    ↓                  ↓                ↓
///   Path List    File Collection    Path/Size/Binary    Pattern Matching   Context/Comment/Entropy
/// ```
/// 
/// ## Performance Optimizations
/// 
/// - **Directory Filters**: Applied before file I/O to eliminate 70-90% of files
/// - **Context Prefiltering**: Aho-Corasick keyword matching provides ~5x speedup
/// - **Shared Data Structures**: Arc<LazyLock> patterns for zero-copy sharing
/// - **Parallel Execution**: Adaptive worker allocation based on file count
pub struct Scanner {
    config: ScannerConfig,
    // Core components (lazy-initialized for performance)
    directory: Directory,
    strategy: Strategy,
    file_processor: File,
}

impl Scanner {
    /// Create a new scanner with the given configuration
    /// 
    /// Initializes all core components and validates configuration.
    /// Components are created eagerly but expensive operations (like pattern
    /// compilation) are deferred to first use via Arc<LazyLock>.
    pub fn new(config: ScannerConfig) -> Result<Self> {
        let directory = Directory::new(&config);
        let strategy = Strategy::new(&config);
        let file_processor = File::new(&config);
        
        Ok(Self { 
            config, 
            directory,
            strategy, 
            file_processor,
        })
    }

    /// Main entry point: Scan with full progress and configuration options
    /// 
    /// This is the complete scanning pipeline that provides:
    /// - Progress reporting with indicatif integration
    /// - Adaptive parallel/sequential execution
    /// - Comprehensive error handling and recovery
    /// - Full filtering pipeline application
    /// - Statistics collection and reporting
    /// 
    /// # Arguments
    /// - `paths`: Input paths to scan (files or directories)
    /// - `verbose_level`: Verbosity level (0=quiet, 1=normal, 2=verbose, 3=debug)
    /// - `quiet`: Suppress progress bars and non-essential output
    /// 
    /// # Returns
    /// Complete ScanResult with matches, statistics, and metadata
    pub fn scan_with_progress(&self, paths: &[String], verbose_level: u8, quiet: bool) -> Result<ScanResult> {
        let start_time = Instant::now();
        
        // Phase 1: Directory Analysis and Path Collection
        tracing::info!("Starting scan with {} input paths", paths.len());
        
        // Analyze input paths for warnings and recommendations
        let warnings = Directory::analyze_paths(paths)
            .with_context(|| "Failed to analyze input paths")?;
        
        // Collect all file paths through directory traversal
        let file_paths = Directory::collect_file_paths(paths, &self.config)
            .with_context(|| "Failed to collect file paths for scanning")?;
        
        if file_paths.is_empty() {
            tracing::warn!("No files found to scan in the provided paths");
            return Ok(ScanResult {
                matches: Vec::new(),
                stats: ScanStats::default(),
                warnings,
            });
        }
        
        tracing::info!("Collected {} files for scanning", file_paths.len());
        
        // Phase 2: Strategy Calculation and Progress Setup
        let execution_strategy = Strategy::calculate(&self.config, file_paths.len())
            .with_context(|| "Failed to calculate optimal execution strategy")?;
        
        tracing::debug!("Using execution strategy: {:?}", execution_strategy);
        
        // Create progress reporter (unless quiet mode)
        let progress_reporter = if !quiet {
            let progress = match execution_strategy {
                ExecutionStrategy::Sequential => {
                    Progress::new_sequential(file_paths.len())?
                }
                ExecutionStrategy::Parallel { workers } => {
                    Progress::new_parallel(file_paths.len(), workers)?
                }
            };
            Some(Arc::new(progress))
        } else {
            None
        };
        
        // Phase 3: Apply Directory-Level Filters
        let filtered_file_paths = self.apply_directory_filters(file_paths, &progress_reporter)
            .with_context(|| "Failed to apply directory-level filters")?;
        
        if filtered_file_paths.is_empty() {
            tracing::info!("All files filtered out by directory-level filters");
            return Ok(ScanResult {
                matches: Vec::new(),
                stats: ScanStats::default(),
                warnings,
            });
        }
        
        tracing::info!("Directory filters passed {} files", filtered_file_paths.len());
        
        // Phase 4: Execute File Processing with Chosen Strategy
        let matches = self.execute_file_processing(
            filtered_file_paths,
            &execution_strategy,
            progress_reporter.clone(),
            verbose_level,
        ).with_context(|| "Failed to execute file processing")?;
        
        // Phase 5: Finalize Progress and Collect Statistics
        if let Some(progress) = &progress_reporter {
            progress.finish();
        }
        
        let scan_duration = start_time.elapsed();
        tracing::info!("Scan completed in {:?}, found {} secrets", scan_duration, matches.len());
        
        // Calculate proper statistics from progress tracking
        let (files_scanned, files_skipped) = if let Some(progress) = &progress_reporter {
            let stats_snapshot = progress.stats().get_snapshot();
            (stats_snapshot.files_scanned, stats_snapshot.files_skipped)
        } else {
            (filtered_file_paths.len(), 0)
        };
        
        // Build scan statistics with proper tracking
        let stats = ScanStats {
            files_scanned,
            files_skipped,
            total_matches: matches.len(),
            scan_duration_ms: scan_duration.as_millis() as u64,
        };
        
        // Build final result with comprehensive metadata
        Ok(ScanResult {
            matches,
            stats,
            warnings,
        })
    }

    /// Simplified interface for basic scanning (used by CLI and testing)
    /// 
    /// This provides a streamlined interface without progress reporting,
    /// suitable for CLI usage, testing, and integration scenarios.
    /// 
    /// # Arguments
    /// - `paths`: Input paths to scan (files or directories)
    /// 
    /// # Returns
    /// Vector of SecretMatch objects found during scanning
    pub fn scan(&self, paths: &[String]) -> Result<Vec<SecretMatch>> {
        // Delegate to full scanning pipeline with minimal options
        let result = self.scan_with_progress(paths, 0, true)?; // verbose=0, quiet=true
        Ok(result.matches)
    }
    
    /// Apply directory-level filters to reduce file I/O operations
    /// 
    /// This is where the major performance improvements come from - filtering
    /// out files before any content loading occurs.
    /// 
    /// Filters applied in order:
    /// 1. Path Filter - gitignore patterns, explicit exclusions
    /// 2. Size Filter - file size limits (metadata-only check)  
    /// 3. Binary Filter - extension + content inspection
    fn apply_directory_filters(
        &self,
        file_paths: Vec<PathBuf>,
        progress_reporter: &Option<Arc<Progress>>,
    ) -> Result<Vec<PathBuf>> {
        tracing::debug!("Applying directory-level filters to {} files", file_paths.len());
        
        // Initialize filters (these use Arc<LazyLock> for shared data)
        let path_filter = PathFilter::new(&self.config)?;
        let size_filter = SizeFilter::new(&self.config)?;
        let binary_filter = BinaryFilter::new(&self.config)?;
        
        let mut filtered_paths = Vec::new();
        let mut stats_filtered = 0;
        
        for file_path in file_paths {
            // Note: Directory filtering is fast, so we don't need per-file progress updates
            
            // Apply filters in order of performance (fastest first)
            
            // 1. Path Filter - O(1) HashSet lookup
            if path_filter.should_ignore(&file_path) {
                stats_filtered += 1;
                continue;
            }
            
            // 2. Size Filter - O(1) metadata check
            if size_filter.should_filter(&file_path)? {
                stats_filtered += 1;
                continue;
            }
            
            // 3. Binary Filter - O(1) extension + O(512 bytes) content fallback
            if binary_filter.should_filter(&file_path)? {
                stats_filtered += 1;
                continue;
            }
            
            // File passed all directory filters
            filtered_paths.push(file_path);
        }
        
        tracing::debug!("Directory filters eliminated {} files, {} remain", 
                       stats_filtered, filtered_paths.len());
        
        Ok(filtered_paths)
    }
    
    /// Execute file processing using the chosen execution strategy
    /// 
    /// This is where the actual file content processing happens, coordinated
    /// by the Strategy module to use either sequential or parallel execution.
    fn execute_file_processing(
        &self,
        file_paths: Vec<PathBuf>,
        execution_strategy: &ExecutionStrategy,
        progress_reporter: Option<Arc<Progress>>,
        verbose_level: u8,
    ) -> Result<Vec<SecretMatch>> {
        tracing::debug!("Executing file processing with strategy: {:?}", execution_strategy);
        
        // Create scanner Arc for worker threads
        let scanner_arc = Arc::new(Self {
            config: self.config.clone(),
            directory: Directory::new(&self.config),
            strategy: Strategy::new(&self.config),
            file_processor: File::new(&self.config),
        });
        
        // Execute using Strategy module's delegation to parallel framework
        let matches = Strategy::execute(
            execution_strategy,
            file_paths,
            scanner_arc,
            progress_reporter,
            verbose_level,
        ).with_context(|| "Failed to execute file processing strategy")?;
        
        Ok(matches)
    }
    
    /// Process a single file - used by Strategy execution workers
    /// 
    /// This is the core file processing method that gets called for each file
    /// by the parallel execution framework. It applies the full content-level
    /// filtering pipeline.
    /// 
    /// # Pipeline
    /// 1. Load file content with size/encoding handling
    /// 2. Context prefiltering (Aho-Corasick keyword matching)
    /// 3. Pattern matching (regex execution on filtered content)
    /// 4. Comment filtering (guardy:allow directives)
    /// 5. Entropy filtering (statistical validation)
    /// 
    /// # Arguments
    /// - `file_path`: Path to the file to process
    /// 
    /// # Returns
    /// Vector of validated SecretMatch objects
    pub fn process_file(&self, file_path: &PathBuf) -> Result<Vec<SecretMatch>> {
        // Delegate to File module for complete processing pipeline
        File::process_single_file(file_path, &self.config)
            .with_context(|| format!("Failed to process file: {}", file_path.display()))
    }
}