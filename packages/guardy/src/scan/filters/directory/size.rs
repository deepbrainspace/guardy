//! File size filtering

use crate::scan::filters::{DirectoryFilter, Filter, FilterDecision};
use anyhow::Result;
use std::path::Path;

/// Filter files based on size
pub struct SizeFilter {
    max_size_bytes: u64,
}

impl SizeFilter {
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            max_size_bytes: (max_size_mb * 1024 * 1024) as u64,
        }
    }
}

impl Filter for SizeFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Placeholder - will check file metadata
        Ok(FilterDecision::Process)
    }
    
    fn name(&self) -> &'static str {
        "SizeFilter"
    }
}

impl DirectoryFilter for SizeFilter {}