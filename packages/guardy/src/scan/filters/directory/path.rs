//! Path-based filtering

use crate::scan::filters::{DirectoryFilter, Filter, FilterDecision};
use anyhow::Result;
use std::path::Path;

/// Filter based on path patterns and gitignore
pub struct PathFilter {
    ignore_patterns: Vec<String>,
}

impl PathFilter {
    pub fn new(ignore_patterns: Vec<String>) -> Self {
        Self { ignore_patterns }
    }
}

impl Filter for PathFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Placeholder - will implement with ignore crate
        Ok(FilterDecision::Process)
    }
    
    fn name(&self) -> &'static str {
        "PathFilter"
    }
}

impl DirectoryFilter for PathFilter {}