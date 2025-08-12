//! Binary file filtering

use crate::scan::filters::{DirectoryFilter, Filter, FilterDecision};
use anyhow::Result;
use std::path::Path;

/// Filter binary files based on extension
pub struct BinaryFilter {
    binary_extensions: Vec<String>,
}

impl BinaryFilter {
    pub fn new(binary_extensions: Vec<String>) -> Self {
        Self { binary_extensions }
    }
}

impl Filter for BinaryFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Placeholder - will check file extension
        Ok(FilterDecision::Process)
    }
    
    fn name(&self) -> &'static str {
        "BinaryFilter"
    }
}

impl DirectoryFilter for BinaryFilter {}