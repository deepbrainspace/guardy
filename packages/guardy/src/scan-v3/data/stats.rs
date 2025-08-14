//! Hierarchical statistics for scan operations

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};


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

/// Thread-safe statistics collector for parallel scanning
#[derive(Debug)]
pub struct StatsCollector {
    pub files_scanned: AtomicUsize,
    pub files_skipped: AtomicUsize,
    pub files_failed: AtomicUsize,
    pub directories_traversed: AtomicUsize,
    pub total_files_discovered: AtomicUsize,
    pub total_matches: AtomicUsize,
    pub total_bytes_processed: AtomicU64,
    pub total_lines_processed: AtomicUsize,
    pub files_filtered_by_size: AtomicUsize,
    pub files_filtered_by_binary: AtomicUsize,
    pub files_filtered_by_path: AtomicUsize,
    pub matches_filtered_by_comments: AtomicUsize,
    pub matches_filtered_by_entropy: AtomicUsize,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            files_scanned: AtomicUsize::new(0),
            files_skipped: AtomicUsize::new(0),
            files_failed: AtomicUsize::new(0),
            directories_traversed: AtomicUsize::new(0),
            total_files_discovered: AtomicUsize::new(0),
            total_matches: AtomicUsize::new(0),
            total_bytes_processed: AtomicU64::new(0),
            total_lines_processed: AtomicUsize::new(0),
            files_filtered_by_size: AtomicUsize::new(0),
            files_filtered_by_binary: AtomicUsize::new(0),
            files_filtered_by_path: AtomicUsize::new(0),
            matches_filtered_by_comments: AtomicUsize::new(0),
            matches_filtered_by_entropy: AtomicUsize::new(0),
        }
    }
    
    pub fn increment_files_discovered(&self) {
        self.total_files_discovered.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_files_filtered_by_size(&self) {
        self.files_filtered_by_size.fetch_add(1, Ordering::Relaxed);
        self.files_skipped.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_files_filtered_by_binary(&self) {
        self.files_filtered_by_binary.fetch_add(1, Ordering::Relaxed);
        self.files_skipped.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_files_filtered_by_path(&self) {
        self.files_filtered_by_path.fetch_add(1, Ordering::Relaxed);
        self.files_skipped.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_files_scanned(&self) {
        self.files_scanned.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_files_failed(&self) {
        self.files_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_directories_traversed(&self) {
        self.directories_traversed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn add_bytes_processed(&self, bytes: u64) {
        self.total_bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn add_lines_processed(&self, lines: usize) {
        self.total_lines_processed.fetch_add(lines, Ordering::Relaxed);
    }
    
    pub fn add_matches(&self, count: usize) {
        self.total_matches.fetch_add(count, Ordering::Relaxed);
    }
    
    pub fn add_matches_filtered_by_comments(&self, count: usize) {
        self.matches_filtered_by_comments.fetch_add(count, Ordering::Relaxed);
    }
    
    pub fn add_matches_filtered_by_entropy(&self, count: usize) {
        self.matches_filtered_by_entropy.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Convert to final ScanStats with scan duration
    pub fn to_scan_stats(&self, scan_duration_ms: u64) -> ScanStats {
        ScanStats {
            files_scanned: self.files_scanned.load(Ordering::Relaxed),
            files_skipped: self.files_skipped.load(Ordering::Relaxed),
            files_failed: self.files_failed.load(Ordering::Relaxed),
            directories_traversed: self.directories_traversed.load(Ordering::Relaxed),
            total_files_discovered: self.total_files_discovered.load(Ordering::Relaxed),
            total_matches: self.total_matches.load(Ordering::Relaxed),
            total_bytes_processed: self.total_bytes_processed.load(Ordering::Relaxed),
            total_lines_processed: self.total_lines_processed.load(Ordering::Relaxed),
            scan_duration_ms,
            files_filtered_by_size: self.files_filtered_by_size.load(Ordering::Relaxed),
            files_filtered_by_binary: self.files_filtered_by_binary.load(Ordering::Relaxed),
            files_filtered_by_path: self.files_filtered_by_path.load(Ordering::Relaxed),
            matches_filtered_by_comments: self.matches_filtered_by_comments.load(Ordering::Relaxed),
            matches_filtered_by_entropy: self.matches_filtered_by_entropy.load(Ordering::Relaxed),
        }
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

