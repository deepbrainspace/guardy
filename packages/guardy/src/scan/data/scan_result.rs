//! Overall scan result with hierarchical statistics

use super::{FileResult, ScanStats, SecretMatch};

/// Complete result of a scanning operation
#[derive(Debug)]
pub struct ScanResult {
    /// All matches found across all files
    pub matches: Vec<SecretMatch>,
    
    /// Detailed statistics
    pub stats: ScanStats,
    
    /// Results per file (for detailed reporting)
    pub file_results: Vec<FileResult>,
    
    /// Any warnings generated during scanning
    pub warnings: Vec<String>,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(
        matches: Vec<SecretMatch>,
        stats: ScanStats,
        file_results: Vec<FileResult>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            matches,
            stats,
            file_results,
            warnings,
        }
    }
    
    /// Check if the scan found any secrets
    pub fn has_secrets(&self) -> bool {
        !self.matches.is_empty()
    }
    
    /// Get count of critical severity matches
    pub fn critical_count(&self) -> usize {
        self.matches
            .iter()
            .filter(|m| m.severity == super::MatchSeverity::Critical)
            .count()
    }
    
    /// Get count of high severity matches
    pub fn high_severity_count(&self) -> usize {
        self.matches
            .iter()
            .filter(|m| m.severity == super::MatchSeverity::High)
            .count()
    }
    
    /// Get files with matches
    pub fn files_with_secrets(&self) -> Vec<&str> {
        use std::collections::HashSet;
        let mut files = HashSet::new();
        for match_ in &self.matches {
            files.insert(match_.file_path.as_ref());
        }
        let mut result: Vec<&str> = files.into_iter().collect();
        result.sort();
        result
    }
    
    /// Get a summary string of the scan
    pub fn summary(&self) -> String {
        format!(
            "Scanned {} files in {:.2}s, found {} secrets in {} files (throughput: {:.1} MB/s)",
            self.stats.files_scanned,
            self.stats.scan_duration_ms as f64 / 1000.0,
            self.matches.len(),
            self.files_with_secrets().len(),
            self.stats.throughput_mb_per_sec()
        )
    }
}