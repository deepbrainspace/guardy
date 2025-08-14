//! Directory traversal and file discovery pipeline

use crate::scan_v3::{
    config::ScannerConfig,
    data::StatsCollector,
    filters::{
        directory::{BinaryFilter, PathFilter, SizeFilter}, Filter, FilterDecision,
    },
};
use anyhow::Result;
use ignore::{WalkBuilder, WalkState};
use smallvec::SmallVec;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use system_profile::SYSTEM;

/// Combined filter statistics for performance analysis
#[derive(Debug, Clone)]
pub struct FilterStats {
    /// Binary filter performance stats (key-value pairs)
    pub binary_filter_stats: SmallVec<[(String, String); 8]>,
    /// Path filter statistics (key-value pairs)
    pub path_filter_stats: SmallVec<[(String, String); 8]>,
    /// Size filter statistics (key-value pairs)
    pub size_filter_stats: SmallVec<[(String, String); 8]>,
}

/// Pipeline for directory traversal and file filtering
pub struct DirectoryPipeline {
    config: Arc<ScannerConfig>,
    path_filter: PathFilter,
    size_filter: SizeFilter,
    binary_filter: BinaryFilter,
    /// Computed thread count (calculated once, reused everywhere)
    thread_count: usize,
}

impl DirectoryPipeline {
    /// Create a new directory pipeline
    pub fn new(config: Arc<ScannerConfig>) -> Result<Self> {
        let path_filter = PathFilter::new(config.ignore_paths.clone());
        let size_filter = SizeFilter::new(config.max_file_size_mb);
        let binary_filter = BinaryFilter::new(config.skip_binary_files);
        
        // Calculate thread count for directory walking
        // Rayon thread pool is already configured globally in static_data::init_config
        let thread_count = if let Some(override_threads) = config.max_threads {
            override_threads
        } else {
            let cpu_count = SYSTEM.cpu_count;
            let calculated = (cpu_count * config.max_cpu_percentage as usize) / 100;
            std::cmp::max(1, calculated)
        };
        
        Ok(Self {
            config,
            path_filter,
            size_filter,
            binary_filter,
            thread_count,
        })
    }
    
    /// Discover all files to scan from a path using the ignore crate
    /// This respects .gitignore files and provides efficient parallel walking
    pub fn discover_files(
        &self, 
        path: &Path, 
        stats: Arc<StatsCollector>
    ) -> Result<Vec<PathBuf>> {
        // Verify path exists
        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path.display());
        }
        
        // Use Arc<Mutex<Vec>> for thread-safe collection
        let files = Arc::new(Mutex::new(Vec::new()));
        let files_clone = files.clone();
        
        // Build the walker with optimized settings
        let mut builder = WalkBuilder::new(path);
        builder
            .follow_links(self.config.follow_symlinks)
            .git_ignore(true)           // Respect .gitignore files
            .git_global(true)           // Respect global gitignore
            .git_exclude(true)          // Respect .git/info/exclude
            .hidden(false)              // Don't skip hidden files by default
            .parents(true)              // Check parent .gitignore files
            .ignore(true)               // Respect .ignore files
            .require_git(false)         // Work in non-git directories too
            .max_depth(None);           // No depth limit
        
        // Use pre-calculated thread count (computed once in constructor)
        builder.threads(self.thread_count);
        
        // Add custom ignore patterns
        for pattern in &self.config.ignore_paths {
            builder.add_custom_ignore_filename(pattern);
        }
        
        // Clone filters and stats once to move into the closure (cheap - just Arc ref count)
        // Each filter uses Arc internally, so cloning is just incrementing ref count
        let path_filter = self.path_filter.clone();
        let size_filter = self.size_filter;
        let binary_filter = self.binary_filter.clone();
        let stats_collector = stats.clone();
        let follow_symlinks = self.config.follow_symlinks;
        
        // Walk the directory tree in parallel
        builder.build_parallel().run(move || {
            // This closure is called once per worker thread
            // Clone again for each thread's visitor (still cheap - Arc increment)
            let files = files_clone.clone();
            let path_filter = path_filter.clone();
            let size_filter = size_filter;
            let binary_filter = binary_filter.clone();
            let stats = stats_collector.clone();
            
            Box::new(move |result| {
                // This closure processes each file/directory entry
                match result {
                    Ok(entry) => {
                        let path = entry.path();
                        
                        // Skip directories, but count them
                        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                            stats.increment_directories_traversed();
                            return WalkState::Continue;
                        }
                        
                        // Count discovered files
                        stats.increment_files_discovered();
                        
                        // Skip symlinks if not following them
                        if entry.file_type().is_some_and(|ft| ft.is_symlink()) 
                            && !follow_symlinks {
                            return WalkState::Continue;
                        }
                        
                        // Apply filters - these are read-only operations
                        if let Ok(FilterDecision::Skip(_)) = path_filter.filter(path) {
                            stats.increment_files_filtered_by_path();
                            return WalkState::Continue;
                        }
                        
                        if let Ok(FilterDecision::Skip(_)) = size_filter.filter(path) {
                            stats.increment_files_filtered_by_size();
                            return WalkState::Continue;
                        }
                        
                        if let Ok(FilterDecision::Skip(_)) = binary_filter.filter(path) {
                            stats.increment_files_filtered_by_binary();
                            return WalkState::Continue;
                        }
                        
                        // Add file to collection
                        if let Ok(mut files_guard) = files.lock() {
                            files_guard.push(path.to_path_buf());
                        }
                        
                        WalkState::Continue
                    }
                    Err(err) => {
                        // Log error but continue walking
                        tracing::warn!("Error walking directory: {}", err);
                        WalkState::Continue
                    }
                }
            })
        });
        
        // Extract the collected files
        let mut files = Arc::try_unwrap(files)
            .map(|mutex| mutex.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone());
        
        // Sort files for consistent ordering
        files.sort();
        
        tracing::info!("Discovered {} files to scan", files.len());
        Ok(files)
    }
    
    /// Process files in parallel using ignore crate's parallel walker
    /// Get filter statistics for performance analysis
    pub fn get_filter_stats(&self) -> FilterStats {
        FilterStats {
            binary_filter_stats: self.binary_filter.get_stats(),
            path_filter_stats: self.path_filter.get_stats(),
            size_filter_stats: self.size_filter.get_stats(),
        }
    }
    
    
    /// Get path filter statistics for trace-level debugging
    pub fn path_filter_stats(&self) -> SmallVec<[(String, String); 8]> {
        self.path_filter.get_stats()
    }
}