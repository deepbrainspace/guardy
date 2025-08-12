//! Path-based filtering using globset for efficient pattern matching

use crate::scan::filters::{DirectoryFilter, Filter, FilterDecision};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use std::sync::Arc;

/// Filter based on path patterns and gitignore
/// Uses globset for O(n) matching where n is the number of patterns
#[derive(Clone)]
pub struct PathFilter {
    /// Compiled glob patterns for efficient matching
    ignore_set: Arc<GlobSet>,
    /// Original patterns for debugging
    patterns: Arc<Vec<String>>,
}

impl PathFilter {
    /// Create a new path filter with the given ignore patterns
    pub fn new(ignore_patterns: Vec<String>) -> Self {
        let mut builder = GlobSetBuilder::new();
        
        // Add each pattern to the glob set
        for pattern in &ignore_patterns {
            // Handle both glob patterns and simple directory names
            let glob_pattern = if pattern.contains('*') || pattern.contains('?') {
                pattern.clone()
            } else {
                // Convert directory name to glob pattern
                format!("**/{}", pattern.trim_matches('/'))
            };
            
            if let Ok(glob) = Glob::new(&glob_pattern) {
                builder.add(glob);
            } else {
                tracing::warn!("Invalid glob pattern: {}", pattern);
            }
        }
        
        let ignore_set = builder.build().unwrap_or_else(|e| {
            tracing::error!("Failed to build glob set: {}", e);
            GlobSet::empty()
        });
        
        Self {
            ignore_set: Arc::new(ignore_set),
            patterns: Arc::new(ignore_patterns),
        }
    }
}

impl Filter for PathFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Check if path matches any ignore pattern
        if self.ignore_set.is_match(path) {
            return Ok(FilterDecision::Skip("matched ignore pattern"));
        }
        
        // Also check individual path components for common patterns
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                // Skip common build/dependency directories
                match name {
                    "node_modules" | ".git" | "target" | "dist" | "build" | 
                    ".next" | "out" | "coverage" | ".nyc_output" | 
                    ".pytest_cache" | "__pycache__" | ".tox" |
                    "vendor" | "deps" | ".bundle" => {
                        return Ok(FilterDecision::Skip("common ignore directory"));
                    }
                    _ => {}
                }
            }
        }
        
        Ok(FilterDecision::Process)
    }
    
    fn name(&self) -> &'static str {
        "PathFilter"
    }
}

impl DirectoryFilter for PathFilter {}