//! Efficient data aggregation for report generation

use super::ReportConfig;
use crate::scan_v3::data::{ScanResult, SecretMatch};
use std::sync::Arc;

/// Efficient data aggregation for report generation
/// Uses zero-copy techniques and pre-computed statistics
pub struct ReportDataAggregator<'a> {
    result: &'a ScanResult,
    config: &'a ReportConfig,
}

impl<'a> ReportDataAggregator<'a> {
    pub fn new(result: &'a ScanResult, config: &'a ReportConfig) -> Self {
        Self { result, config }
    }
    
    /// Get matches grouped by secret type (zero-copy)
    pub fn matches_by_type(&self) -> Vec<(Arc<str>, Vec<&'a SecretMatch>)> {
        use std::collections::HashMap;
        
        let mut groups: HashMap<Arc<str>, Vec<&'a SecretMatch>> = HashMap::new();
        let limit = if self.config.max_matches > 0 {
            self.config.max_matches
        } else {
            self.result.matches.len()
        };
        
        for secret in self.result.matches.iter().take(limit) {
            groups
                .entry(secret.secret_type.clone())
                .or_default()
                .push(secret);
        }
        
        let mut result: Vec<_> = groups.into_iter().collect();
        // Sort by count descending, then by type name for consistent output
        result.sort_by(|a, b| {
            b.1.len().cmp(&a.1.len())
                .then_with(|| a.0.as_ref().cmp(b.0.as_ref()))
        });
        
        result
    }
    
    /// Get file performance statistics (uses FileResult timing data)
    pub fn file_performance_stats(&self) -> FilePerformanceStats {
        if !self.config.include_file_timing {
            return FilePerformanceStats::empty();
        }
        
        let mut total_scan_time = 0u64;
        let mut total_lines = 0usize;
        let mut slowest_files = Vec::new();
        let mut largest_files = Vec::new();
        
        // Collect timing data from successful file results
        for file_result in &self.result.file_results {
            if !file_result.success {
                continue;
            }
            
            total_scan_time += file_result.scan_time_ms;
            total_lines += file_result.lines_processed;
            
            // Track top 10 slowest files
            slowest_files.push((file_result.file_path.clone(), file_result.scan_time_ms));
            if slowest_files.len() > 10 {
                slowest_files.sort_by(|a, b| b.1.cmp(&a.1));
                slowest_files.truncate(10);
            }
            
            // Track top 10 largest files
            largest_files.push((file_result.file_path.clone(), file_result.file_size));
            if largest_files.len() > 10 {
                largest_files.sort_by(|a, b| b.1.cmp(&a.1));
                largest_files.truncate(10);
            }
        }
        
        // Final sort
        slowest_files.sort_by(|a, b| b.1.cmp(&a.1));
        largest_files.sort_by(|a, b| b.1.cmp(&a.1));
        
        FilePerformanceStats {
            total_file_scan_time_ms: total_scan_time,
            total_lines_processed: total_lines,
            slowest_files,
            largest_files,
            files_with_matches: self.result.file_results
                .iter()
                .filter(|f| f.success && f.has_matches())
                .map(|f| (f.file_path.clone(), f.matches.len()))
                .collect(),
        }
    }
    
    /// Get matches grouped by file (zero-copy)
    pub fn matches_by_file(&self) -> Vec<(Arc<str>, Vec<&'a SecretMatch>)> {
        use std::collections::HashMap;
        
        let mut groups: HashMap<Arc<str>, Vec<&'a SecretMatch>> = HashMap::new();
        let limit = if self.config.max_matches > 0 {
            self.config.max_matches
        } else {
            self.result.matches.len()
        };
        
        for secret in self.result.matches.iter().take(limit) {
            groups
                .entry(Arc::from(secret.file_path()))
                .or_default()
                .push(secret);
        }
        
        let mut result: Vec<_> = groups.into_iter().collect();
        // Sort by filename for consistent output
        result.sort_by(|a, b| a.0.as_ref().cmp(b.0.as_ref()));
        
        result
    }
}

/// File performance statistics derived from FileResult data
#[derive(Debug)]
pub struct FilePerformanceStats {
    pub total_file_scan_time_ms: u64,
    pub total_lines_processed: usize,
    pub slowest_files: Vec<(Arc<str>, u64)>, // (file_path, scan_time_ms)
    pub largest_files: Vec<(Arc<str>, u64)>, // (file_path, file_size)
    pub files_with_matches: Vec<(Arc<str>, usize)>, // (file_path, match_count)
}

impl FilePerformanceStats {
    pub fn empty() -> Self {
        Self {
            total_file_scan_time_ms: 0,
            total_lines_processed: 0,
            slowest_files: Vec::new(),
            largest_files: Vec::new(),
            files_with_matches: Vec::new(),
        }
    }
    
    /// Calculate lines per second throughput
    pub fn lines_per_second(&self, total_duration_ms: u64) -> f64 {
        if total_duration_ms == 0 {
            0.0
        } else {
            (self.total_lines_processed as f64) / (total_duration_ms as f64 / 1000.0)
        }
    }
}