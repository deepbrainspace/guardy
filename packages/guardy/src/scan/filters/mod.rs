//! Filters for file and content processing
//!
//! This module provides a consistent interface for filtering files and content
//! during the scanning process. All filters implement the Filter trait for
//! composability and testability.

pub mod directory;

use anyhow::Result;
use smallvec::SmallVec;

/// Common trait for all filters
pub trait Filter {
    /// Input type for the filter
    type Input: ?Sized;
    /// Output type for the filter
    type Output;
    
    /// Apply the filter to the input
    fn filter(&self, input: &Self::Input) -> Result<Self::Output>;
    
    /// Get the name of this filter for debugging/logging
    fn name(&self) -> &'static str;
    
    /// Get statistics about this filter's performance
    /// Returns key-value pairs for metrics
    fn get_stats(&self) -> SmallVec<[(String, String); 8]> {
        SmallVec::new() // Default: no stats
    }
}

/// Decision enum for directory-level filters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    /// Process this file/directory
    Process,
    /// Skip this file/directory with reason
    Skip(&'static str),
}