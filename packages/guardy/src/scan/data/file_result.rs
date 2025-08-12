//! Result from scanning a single file

use super::SecretMatch;
use std::sync::Arc;

/// Result from scanning a single file
#[derive(Debug, Clone)]
pub struct FileResult {
    /// Path to the file
    pub file_path: Arc<str>,
    
    /// All matches found in this file
    pub matches: Vec<SecretMatch>,
    
    /// Whether the scan completed successfully
    pub success: bool,
    
    /// Error message if scan failed
    pub error: Option<String>,
    
    /// Number of lines processed
    pub lines_processed: usize,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// Time taken to scan in milliseconds
    pub scan_time_ms: u64,
}

impl FileResult {
    /// Create a successful file result
    pub fn success(
        file_path: Arc<str>,
        matches: Vec<SecretMatch>,
        lines_processed: usize,
        file_size: u64,
        scan_time_ms: u64,
    ) -> Self {
        Self {
            file_path,
            matches,
            success: true,
            error: None,
            lines_processed,
            file_size,
            scan_time_ms,
        }
    }
    
    /// Create a failed file result
    pub fn failure(file_path: Arc<str>, error: String) -> Self {
        Self {
            file_path,
            matches: Vec::new(),
            success: false,
            error: Some(error),
            lines_processed: 0,
            file_size: 0,
            scan_time_ms: 0,
        }
    }
    
    /// Check if this file has any matches
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }
    
    /// Get the highest severity match in this file
    pub fn highest_severity(&self) -> Option<super::MatchSeverity> {
        self.matches
            .iter()
            .map(|m| m.severity)
            .max()
    }
}