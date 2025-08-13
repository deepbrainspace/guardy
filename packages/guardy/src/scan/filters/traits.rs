//! Core filter traits for the scanning pipeline
//!
//! These traits define the interface for all filters in the system,
//! allowing for clean composition and testing.

use anyhow::Result;
use smallvec::SmallVec;

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
    
    /// Get performance statistics for this filter (key, value pairs)
    /// Default implementation returns empty stats for filters without performance tracking
    /// Uses SmallVec to avoid heap allocation for small collections (typical 5-10 items)
    fn get_stats(&self) -> SmallVec<[(String, String); 8]> {
        SmallVec::new()
    }
}

/// Decision for whether to process or skip a file/directory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    /// Continue processing this item
    Process,
    /// Skip this item with a reason
    Skip(&'static str),
}

