//! Main Scanner implementation - Optimized with zero-copy patterns and OOP design
//!
//! This module implements the Scanner class with optimal Rust patterns:
//! - Arc for zero-copy sharing across threads
//! - LazyLock for one-time initialization
//! - ExecutionStrategy for parallel processing with crossbeam channels
//! - Enhanced progress tracking via parallel module
//! - Good OOP encapsulation and trait boundaries


use crate::scan_v3::{
    config::ScannerConfig,
    data::{FileResult, ScanResult, StatsCollector},
    pipeline::{DirectoryPipeline, FilePipeline},
    static_data,
};
use anyhow::{Context, Result};
use crate::parallel::ExecutionStrategy;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
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
/// - **Strategy Pattern**: Pluggable filters and execution strategies
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


impl Scanner {
    /// Create a new scanner with the given configuration
    /// 
    /// # Performance Optimizations
    /// - Configuration is Arc-wrapped for zero-copy sharing
    /// - Pipelines are pre-initialized and cached
    /// - ExecutionStrategy manages worker threads efficiently
    pub fn new(config: ScannerConfig) -> Result<Self> {
        // Initialize global configuration if not already done
        // This stores the configuration for static data access
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
    
    /// Get scanner configuration (immutable reference)
    pub fn config(&self) -> &ScannerConfig {
        &self.config
    }
    
    /// Check if scanner is properly initialized
    pub fn is_ready(&self) -> bool {
        static_data::is_initialized()
    }
    
    /// Get filter performance statistics
    pub fn get_filter_stats(&self) -> crate::scan_v3::pipeline::directory::FilterStats {
        self.directory_pipeline.get_filter_stats()
    }
    
    /// Scan a path (file or directory) for secrets
    /// 
    /// # Performance Optimizations
    /// - Parallel file discovery with optimized thread count
    /// - Parallel file processing with ExecutionStrategy and crossbeam channels
    /// - Zero-copy string sharing with Arc<str>
    /// - Lock-free atomic counters for progress tracking
    /// 
    /// # Error Handling
    /// - Continues on individual file errors
    /// - Collects warnings for non-fatal issues
    /// - Returns partial results even on some failures
    pub fn scan(&self, path: &Path) -> Result<ScanResult> {
        self.scan_with_progress(path)
    }
    
    /// Scan with custom progress tracker
    pub fn scan_with_progress(
        &self,
        path: &Path,
    ) -> Result<ScanResult> {
        let start = Instant::now();
        
        // We now use the parallel module's progress system exclusively
        
        // Phase 1: File Discovery
        let files = self.directory_pipeline
            .discover_files(path, self.stats_collector.clone())
            .context("Failed to discover files")?;
        let total_files = files.len();
        
        info!("Discovered {} files to scan", total_files);
        
        if files.is_empty() {
            return Ok(ScanResult::empty());
        }
        
        // Debug trace: Show first few files discovered
        tracing::trace!("First 5 files discovered:");
        for (i, file) in files.iter().take(5).enumerate() {
            tracing::trace!("  {}: {}", i + 1, file.display());
        }
        
        // Phase 2: Parallel File Processing using the same approach as scan_v1
        
        // Determine execution strategy similar to scan_v1
        let max_workers_by_resources = ExecutionStrategy::calculate_optimal_workers(
            self.config.max_threads.unwrap_or(0),  // 0 means auto-detect
            self.config.max_cpu_percentage,
        );
        
        // Use auto strategy with file count threshold
        let execution_strategy = ExecutionStrategy::auto(
            total_files,
            50, // min files for parallel - same as scan_v1 default
            max_workers_by_resources,
        );
        
        tracing::trace!("Execution strategy: {:?}", execution_strategy);
        tracing::trace!("Max workers by resources: {}", max_workers_by_resources);
        
        // Create enhanced progress reporter based on strategy like scan_v1
        use crate::parallel::progress::factories;
        let enhanced_progress = match &execution_strategy {
            ExecutionStrategy::Sequential => {
                Some(factories::enhanced_sequential_reporter(total_files))
            }
            ExecutionStrategy::Parallel { workers } => Some(factories::enhanced_parallel_reporter(
                total_files,
                *workers,
            )),
        };
        
        // Get statistics reference for tracking
        let stats = enhanced_progress.as_ref().map(|p| p.stats());
        
        // Clone Arc references for the parallel closure (cheap operation)
        let file_pipeline = self.file_pipeline.clone();
        let stats_collector = self.stats_collector.clone();
        
        // Process files using ExecutionStrategy exactly like scan_v1
        let file_results = execution_strategy.execute(
            files,
            {
                let file_pipeline = file_pipeline.clone();
                let stats_collector = stats_collector.clone();
                let enhanced_progress_for_worker = enhanced_progress.clone();
                let stats = stats.clone();
                move |file_path: &PathBuf, worker_id: usize| -> FileResult {
                    tracing::trace!("Worker {} processing file: {}", worker_id, file_path.display());
                    
                    // Update worker bar with current file like scan_v1
                    if let Some(ref progress) = enhanced_progress_for_worker
                        && progress.is_parallel
                    {
                        progress.update_worker_file(worker_id, &file_path.to_string_lossy());
                    }
                    
                    // Process file with stats collection
                    match file_pipeline.process_file(file_path, stats_collector.clone()) {
                        Ok(result) => {
                            // Update statistics like scan_v1
                            if let Some(ref stats) = stats {
                                stats.increment_scanned();
                                if !result.matches.is_empty() {
                                    stats.increment_with_secrets();
                                }
                            }
                            result
                        }
                        Err(e) => {
                            warn!("Failed to process {}: {}", file_path.display(), e);
                            // Update statistics for errors
                            if let Some(ref stats) = stats {
                                stats.increment_skipped();
                            }
                            stats_collector.increment_files_failed();
                            FileResult::failure(
                                Arc::from(file_path.to_string_lossy().as_ref()),
                                e.to_string(),
                            )
                        }
                    }
                }
            },
            enhanced_progress.as_ref().map(|progress| {
                let progress = progress.clone();
                move |current: usize, total: usize, _worker_id: usize| {
                    // Update overall progress like scan_v1
                    progress.update_overall(current, total);
                }
            }),
        )?;
        
        // Clear progress display using enhanced reporter like scan_v1
        if let Some(ref progress) = enhanced_progress {
            progress.finish();
        }
        
        // Phase 3: Results Aggregation
        
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
        let stats = self.stats_collector.to_scan_stats(duration_ms);
        
        // Log summary
        info!(
            "Scan complete: {} files, {} matches, {} errors in {}ms",
            stats.files_scanned,
            stats.total_matches,
            stats.files_failed,
            duration_ms
        );
        
        // Show detailed filter statistics in trace mode
        if tracing::enabled!(tracing::Level::TRACE) {
            // Scanner configuration info
            tracing::trace!("Scanner configuration:");
            tracing::trace!("  Ready: {}", self.is_ready());
            let config = self.config();
            tracing::trace!("  Max file size: {} MB", config.max_file_size_mb);
            tracing::trace!("  CPU usage: {}%", config.max_cpu_percentage);
            tracing::trace!("  Follow symlinks: {}", config.follow_symlinks);
            tracing::trace!("  Entropy analysis: {}", config.enable_entropy_analysis);
            tracing::trace!("  Entropy threshold: {}", config.min_entropy_threshold);
            tracing::trace!("  Skip binary files: {}", config.skip_binary_files);
            tracing::trace!("  Show progress: {}", config.show_progress);
            if !config.ignore_paths.is_empty() {
                tracing::trace!("  Ignore patterns: {:?}", config.ignore_paths);
            }
            
            // Path filter statistics
            let path_stats = self.directory_pipeline.path_filter_stats();
            if !path_stats.is_empty() {
                tracing::trace!("Path filter statistics:");
                for (pattern, count) in path_stats {
                    tracing::trace!("  '{}': {} files filtered", pattern, count);
                }
            }
        }
        
        Ok(ScanResult::new(
            all_matches,
            stats,
            file_results,
            warnings,
        ))
    }
}