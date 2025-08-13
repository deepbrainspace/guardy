//! Core filter traits for the scanning pipeline
//!
//! These traits define the interface for all filters in the system,
//! allowing for clean composition and testing.

use anyhow::Result;
use std::path::Path;

/// Base filter trait that all filters implement
pub trait Filter {
    /// The input type this filter processes
    type Input: ?Sized;
    /// The output type this filter produces
    type Output;
    
    /// Apply the filter to the input
    fn filter(&self, input: &Self::Input) -> Result<Self::Output>;
    
    /// Get the name of this filter for debugging/logging
    fn name(&self) -> &'static str;
}

/// Decision for whether to process or skip a file/directory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    /// Continue processing this item
    Process,
    /// Skip this item with a reason
    Skip(&'static str),
}

/// Directory-level filter for path-based filtering
pub trait DirectoryFilter: Filter<Input = Path, Output = FilterDecision> {
    /// Check if a directory should be filtered (skipped)
    /// Returns true if the directory should be skipped
    fn should_skip_directory(&self, path: &Path) -> bool {
        matches!(self.filter(path), Ok(FilterDecision::Skip(_)))
    }
    
    /// Check if a file should be filtered (skipped)
    /// Returns true if the file should be skipped
    fn should_skip_file(&self, path: &Path) -> bool {
        matches!(self.filter(path), Ok(FilterDecision::Skip(_)))
    }
}

/// Content-level filter for analyzing file contents
pub trait ContentFilter: Filter {
    /// Pre-process check to see if this filter applies to the file
    /// Can be used to skip expensive content filtering based on file metadata
    fn applies_to(&self, _path: &Path) -> bool {
        true // By default, apply to all files
    }
}