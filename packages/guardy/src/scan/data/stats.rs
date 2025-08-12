//! Hierarchical statistics for scan operations

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Thread-safe statistics for a single file
#[derive(Debug, Default)]
pub struct FileStats {
    pub lines_processed: AtomicUsize,
    pub bytes_processed: AtomicU64,
    pub matches_found: AtomicUsize,
    pub scan_time_ms: AtomicU64,
}

impl FileStats {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Thread-safe statistics for a directory
#[derive(Debug, Default)]
pub struct DirectoryStats {
    pub files_discovered: AtomicUsize,
    pub files_filtered: AtomicUsize,
    pub directories_traversed: AtomicUsize,
    pub total_size_bytes: AtomicU64,
}

impl DirectoryStats {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Overall scan statistics
#[derive(Debug, Clone)]
pub struct ScanStats {
    // File processing stats
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub files_failed: usize,
    
    // Directory stats
    pub directories_traversed: usize,
    pub total_files_discovered: usize,
    
    // Match stats
    pub total_matches: usize,
    pub high_severity_matches: usize,
    pub critical_matches: usize,
    
    // Performance stats
    pub total_bytes_processed: u64,
    pub total_lines_processed: usize,
    pub scan_duration_ms: u64,
    
    // Filter stats
    pub files_filtered_by_size: usize,
    pub files_filtered_by_binary: usize,
    pub files_filtered_by_path: usize,
    pub matches_filtered_by_comments: usize,
    pub matches_filtered_by_entropy: usize,
}

impl ScanStats {
    /// Create empty stats
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            files_skipped: 0,
            files_failed: 0,
            directories_traversed: 0,
            total_files_discovered: 0,
            total_matches: 0,
            high_severity_matches: 0,
            critical_matches: 0,
            total_bytes_processed: 0,
            total_lines_processed: 0,
            scan_duration_ms: 0,
            files_filtered_by_size: 0,
            files_filtered_by_binary: 0,
            files_filtered_by_path: 0,
            matches_filtered_by_comments: 0,
            matches_filtered_by_entropy: 0,
        }
    }
    
    /// Calculate throughput in MB/s
    pub fn throughput_mb_per_sec(&self) -> f64 {
        if self.scan_duration_ms == 0 {
            return 0.0;
        }
        let mb = self.total_bytes_processed as f64 / (1024.0 * 1024.0);
        let seconds = self.scan_duration_ms as f64 / 1000.0;
        mb / seconds
    }
    
    /// Calculate files per second
    pub fn files_per_sec(&self) -> f64 {
        if self.scan_duration_ms == 0 {
            return 0.0;
        }
        let seconds = self.scan_duration_ms as f64 / 1000.0;
        self.files_scanned as f64 / seconds
    }
    
    /// Get filter efficiency (percentage of files filtered)
    pub fn filter_efficiency(&self) -> f64 {
        if self.total_files_discovered == 0 {
            return 0.0;
        }
        let filtered = self.files_skipped as f64;
        let total = self.total_files_discovered as f64;
        (filtered / total) * 100.0
    }
}

impl Default for ScanStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for accumulating stats during parallel processing
#[derive(Debug)]
pub struct StatsAccumulator {
    pub directory: Arc<DirectoryStats>,
    pub file: Arc<FileStats>,
}

impl StatsAccumulator {
    pub fn new() -> Self {
        Self {
            directory: Arc::new(DirectoryStats::new()),
            file: Arc::new(FileStats::new()),
        }
    }
    
    /// Increment files discovered
    pub fn add_file_discovered(&self) {
        self.directory.files_discovered.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increment files filtered
    pub fn add_file_filtered(&self) {
        self.directory.files_filtered.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Add bytes processed
    pub fn add_bytes_processed(&self, bytes: u64) {
        self.file.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
        self.directory.total_size_bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    
    /// Add lines processed
    pub fn add_lines_processed(&self, lines: usize) {
        self.file.lines_processed.fetch_add(lines, Ordering::Relaxed);
    }
    
    /// Add match found
    pub fn add_match_found(&self) {
        self.file.matches_found.fetch_add(1, Ordering::Relaxed);
    }
}