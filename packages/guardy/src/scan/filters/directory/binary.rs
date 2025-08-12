//! Binary file filtering using efficient HashSet lookups

use crate::scan::filters::{DirectoryFilter, Filter, FilterDecision};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

/// Filter binary files based on extension
/// Uses Arc<HashSet> for zero-copy sharing and O(1) lookups
#[derive(Clone)]
pub struct BinaryFilter {
    /// Shared set of binary extensions for O(1) lookup
    binary_extensions: Arc<HashSet<String>>,
    /// Whether to skip binary files
    skip_binary: bool,
}

impl BinaryFilter {
    /// Create a new binary filter with shared extension set
    pub fn new(binary_extensions: Arc<HashSet<String>>, skip_binary: bool) -> Self {
        Self {
            binary_extensions,
            skip_binary,
        }
    }
    
    /// Check if a file has a binary extension
    fn is_binary_extension(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                // Check lowercase version for case-insensitive matching
                return self.binary_extensions.contains(&ext_str.to_lowercase());
            }
        }
        false
    }
}

impl Filter for BinaryFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Skip if configured to skip binary files AND file has binary extension
        if self.skip_binary && self.is_binary_extension(path) {
            tracing::trace!("Skipping binary file: {}", path.display());
            return Ok(FilterDecision::Skip("binary file"));
        }
        
        Ok(FilterDecision::Process)
    }
    
    fn name(&self) -> &'static str {
        "BinaryFilter"
    }
}

impl DirectoryFilter for BinaryFilter {}