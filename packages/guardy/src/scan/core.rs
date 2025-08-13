//! Main Scanner implementation - Optimized with zero-copy patterns and OOP design
//!
//! This module implements the Scanner class with optimal Rust patterns:
//! - Arc for zero-copy sharing across threads
//! - LazyLock for one-time initialization
//! - Rayon for parallel processing
//! - Indicatif for progress tracking
//! - Good OOP encapsulation and trait boundaries

use crate::scan::{
    config::ScannerConfig,
    data::{FileResult, ScanResult, ScanStats, MatchSeverity},
    pipeline::{DirectoryPipeline, FilePipeline},
    static_data,
    tracking::ProgressTracker,
};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::{info, warn};

/// Scanner orchestrator - Main entry point for secret scanning
/// 
/// This class encapsulates the entire scanning pipeline with:
/// - Optimal memory management using Arc for shared immutable data
/// - Zero-copy string sharing across threads
/// - Parallel file processing with rayon
/// - Real-time progress tracking with indicatif
/// 
/// # Design Patterns
/// - **Builder Pattern**: Configuration through ScannerConfig
/// - **Pipeline Pattern**: DirectoryPipeline -> FilePipeline
/// - **Strategy Pattern**: Pluggable filters in pipelines
/// - **Facade Pattern**: Simple public API hiding complex internals
#[derive(Clone)]
pub struct Scanner {
    /// Shared configuration (immutable after creation)
    config: Arc<ScannerConfig>,
    /// Directory traversal and filtering pipeline
    directory_pipeline: Arc<DirectoryPipeline>,
    /// File content analysis pipeline
    file_pipeline: Arc<FilePipeline>,
    /// Statistics collector (thread-safe)
    stats_collector: Arc<StatsCollector>,
}

/// Thread-safe statistics collector
/// Uses interior mutability pattern for concurrent updates
struct StatsCollector {
    /// Mutex-protected stats for thread-safe updates
    inner: Mutex<ScanStats>,
}

impl StatsCollector {
    fn new() -> Self {
        Self {
            inner: Mutex::new(ScanStats::new()),
        }
    }
    
    /// Update stats with a file result (thread-safe)
    fn update(&self, result: &FileResult) {
        if let Ok(mut stats) = self.inner.lock() {
            if result.success {
                stats.files_scanned += 1;
                stats.total_bytes_processed += result.file_size;
                stats.total_lines_processed += result.lines_processed;
                stats.total_matches += result.matches.len();
                
                // Count severity levels
                for match_ in &result.matches {
                    match match_.severity {
                        MatchSeverity::Critical => stats.critical_matches += 1,
                        MatchSeverity::High => stats.high_severity_matches += 1,
                        _ => {}
                    }
                }
            } else {
                stats.files_failed += 1;
            }
        }
    }
    
    /// Get final stats
    fn finalize(&self, duration_ms: u64) -> ScanStats {
        let mut stats = self.inner.lock().unwrap().clone();
        stats.scan_duration_ms = duration_ms;
        stats
    }
}

impl Scanner {
    /// Create a new scanner with the given configuration
    /// 
    /// # Performance Optimizations
    /// - Configuration is Arc-wrapped for zero-copy sharing
    /// - Pipelines are pre-initialized and cached
    /// - Global thread pool is configured once via static_data
    pub fn new(config: ScannerConfig) -> Result<Self> {
        // Initialize global configuration if not already done
        // This sets up the rayon thread pool and other global state
        if !static_data::is_initialized() {
            static_data::init_config(config.clone());
        }
        
        let config = Arc::new(config);
        
        // Initialize pipelines with Arc for cheap cloning
        let directory_pipeline = Arc::new(
            DirectoryPipeline::new(config.clone())
                .context("Failed to initialize directory pipeline")?
        );
        let file_pipeline = Arc::new(
            FilePipeline::new(config.clone())
                .context("Failed to initialize file pipeline")?
        );
        
        let stats_collector = Arc::new(StatsCollector::new());
        
        Ok(Self {
            config,
            directory_pipeline,
            file_pipeline,
            stats_collector,
        })
    }
    
    /// Create a new scanner with custom pipelines (for testing/customization)
    /// 
    /// # Design Pattern: Dependency Injection
    /// Allows injecting custom pipelines for testing or specialized use cases
    pub fn with_pipelines(
        config: ScannerConfig,
        directory_pipeline: DirectoryPipeline,
        file_pipeline: FilePipeline,
    ) -> Result<Self> {
        if !static_data::is_initialized() {
            static_data::init_config(config.clone());
        }
        
        Ok(Self {
            config: Arc::new(config),
            directory_pipeline: Arc::new(directory_pipeline),
            file_pipeline: Arc::new(file_pipeline),
            stats_collector: Arc::new(StatsCollector::new()),
        })
    }
    
    /// Create a scanner using the global configuration
    /// 
    /// # Performance Note
    /// This reuses the globally initialized configuration,
    /// avoiding redundant initialization overhead
    pub fn from_global_config() -> Result<Self> {
        let config = static_data::get_config();
        let config_owned = (*config).clone();
        Self::new(config_owned)
    }
    
    /// Get scanner configuration (immutable reference)
    pub fn config(&self) -> &ScannerConfig {
        &self.config
    }
    
    /// Check if scanner is properly initialized
    pub fn is_ready(&self) -> bool {
        static_data::is_initialized()
    }
    
    /// Scan a path (file or directory) for secrets
    /// 
    /// # Performance Optimizations
    /// - Parallel file discovery with optimized thread count
    /// - Parallel file processing with rayon work-stealing
    /// - Zero-copy string sharing with Arc<str>
    /// - Lock-free atomic counters for progress tracking
    /// 
    /// # Error Handling
    /// - Continues on individual file errors
    /// - Collects warnings for non-fatal issues
    /// - Returns partial results even on some failures
    pub fn scan(&self, path: &Path) -> Result<ScanResult> {
        self.scan_with_progress(path, None)
    }
    
    /// Scan with custom progress tracker
    pub fn scan_with_progress(
        &self,
        path: &Path,
        external_progress: Option<Arc<ProgressTracker>>,
    ) -> Result<ScanResult> {
        let start = Instant::now();
        
        // Create or use provided progress tracker
        let progress = external_progress.unwrap_or_else(|| {
            Arc::new(ProgressTracker::new_with_indicatif(
                self.config.show_progress
            ))
        });
        
        // Phase 1: File Discovery
        progress.start_discovery();
        let files = self.directory_pipeline
            .discover_files(path)
            .context("Failed to discover files")?;
        let total_files = files.len();
        progress.finish_discovery(total_files);
        
        info!("Discovered {} files to scan", total_files);
        
        if files.is_empty() {
            return Ok(ScanResult::empty());
        }
        
        // Phase 2: Parallel File Processing
        progress.start_scanning(total_files);
        
        // Clone Arc references for the parallel closure (cheap operation)
        let file_pipeline = self.file_pipeline.clone();
        let stats_collector = self.stats_collector.clone();
        let progress_clone = progress.clone();
        
        // Process files in parallel using rayon
        // This automatically uses the globally configured thread pool
        let file_results: Vec<FileResult> = files
            .par_iter()
            .map(|file_path| {
                // Update progress
                progress_clone.increment_files_processed();
                
                // Process file
                let result = match file_pipeline.process_file(file_path) {
                    Ok(result) => result,
                    Err(e) => {
                        warn!("Failed to process {}: {}", file_path.display(), e);
                        FileResult::failure(
                            Arc::from(file_path.to_string_lossy().as_ref()),
                            e.to_string(),
                        )
                    }
                };
                
                // Update statistics atomically
                stats_collector.update(&result);
                
                // Update progress with details
                if result.success {
                    progress_clone.update_scan_details(
                        result.matches.len(),
                        result.file_size,
                    );
                }
                
                result
            })
            .collect();
        
        progress.finish_scanning();
        
        // Phase 3: Results Aggregation
        progress.start_aggregation();
        
        // Collect all matches and warnings
        let mut all_matches = Vec::with_capacity(
            file_results.iter()
                .filter(|r| r.success)
                .map(|r| r.matches.len())
                .sum()
        );
        
        let mut warnings = Vec::new();
        
        for result in &file_results {
            if result.success {
                all_matches.extend(result.matches.clone());
            } else if let Some(ref error) = result.error {
                warnings.push(format!("{}: {}", result.file_path, error));
            }
        }
        
        // Finalize statistics
        let duration_ms = start.elapsed().as_millis() as u64;
        let stats = self.stats_collector.finalize(duration_ms);
        
        progress.finish_aggregation();
        
        // Log summary
        info!(
            "Scan complete: {} files, {} matches, {} errors in {}ms",
            stats.files_scanned,
            stats.total_matches,
            stats.files_failed,
            duration_ms
        );
        
        Ok(ScanResult::new(
            all_matches,
            stats,
            file_results,
            warnings,
        ))
    }
    
    /// Scan a single file (convenience method)
    pub fn scan_file(&self, file_path: &Path) -> Result<FileResult> {
        if !file_path.exists() {
            anyhow::bail!("File does not exist: {}", file_path.display());
        }
        
        if !file_path.is_file() {
            anyhow::bail!("Path is not a file: {}", file_path.display());
        }
        
        self.file_pipeline.process_file(file_path)
    }
    
    /// Scan multiple paths in parallel
    pub fn scan_multiple(&self, paths: &[PathBuf]) -> Result<Vec<ScanResult>> {
        paths
            .par_iter()
            .map(|path| self.scan(path))
            .collect()
    }
}