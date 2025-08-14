//! File size filtering for performance and memory management

use crate::scan_v3::filters::{Filter, FilterDecision};
use anyhow::Result;
use smallvec::SmallVec;
use std::fs;
use std::path::Path;

/// Filter files based on size to prevent memory issues
/// and skip files that are likely not source code
#[derive(Clone, Copy)]
pub struct SizeFilter {
    /// Maximum file size in bytes
    max_size_bytes: u64,
}

impl SizeFilter {
    /// Create a new size filter with max size in megabytes
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            max_size_bytes: (max_size_mb as u64) * 1024 * 1024,
        }
    }
    
}

impl Filter for SizeFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Get file metadata efficiently
        match fs::metadata(path) {
            Ok(metadata) => {
                let size = metadata.len();
                
                if size > self.max_size_bytes {
                    tracing::debug!(
                        "Skipping large file {}: {} MB > {} MB",
                        path.display(),
                        size / (1024 * 1024),
                        self.max_size_bytes / (1024 * 1024)
                    );
                    return Ok(FilterDecision::Skip("file too large"));
                }
                
                // Also skip empty files
                if size == 0 {
                    return Ok(FilterDecision::Skip("empty file"));
                }
                
                Ok(FilterDecision::Process)
            }
            Err(e) => {
                // If we can't read metadata, skip the file
                tracing::warn!("Cannot read metadata for {}: {}", path.display(), e);
                Ok(FilterDecision::Skip("cannot read metadata"))
            }
        }
    }
    
    fn name(&self) -> &'static str {
        "SizeFilter"
    }
    
    fn get_stats(&self) -> SmallVec<[(String, String); 8]> {
        smallvec::smallvec![
            ("Max file size (MB)".to_string(), (self.max_size_bytes / (1024 * 1024)).to_string()),
        ]
    }
}

