//! Path-based filtering using globset for efficient pattern matching

use crate::scan::filters::{Filter, FilterDecision};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use smallvec::SmallVec;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

/// Filter based on path patterns and gitignore
/// Uses globset for O(n) matching where n is the number of patterns
#[derive(Clone)]
pub struct PathFilter {
    /// Compiled glob patterns for efficient matching
    ignore_set: Arc<GlobSet>,
    /// Original patterns for debugging and stats
    patterns: Arc<Vec<String>>,
    /// Usage statistics per pattern (atomic for thread safety)
    pattern_usage: Arc<Vec<AtomicUsize>>,
}

impl PathFilter {
    /// Create a new path filter with the given ignore patterns
    pub fn new(ignore_patterns: Arc<Vec<String>>) -> Self {
        let mut builder = GlobSetBuilder::new();
        
        // Add each pattern to the glob set
        for pattern in ignore_patterns.iter() {
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
        
        // Initialize usage counters for each pattern
        let pattern_usage: Vec<AtomicUsize> = (0..ignore_patterns.len())
            .map(|_| AtomicUsize::new(0))
            .collect();
        
        tracing::debug!("[PathFilter] Initialized with {} patterns", ignore_patterns.len());
        
        Self {
            ignore_set: Arc::new(ignore_set),
            patterns: ignore_patterns,
            pattern_usage: Arc::new(pattern_usage),
        }
    }
    
    /// Get current statistics
    pub fn get_statistics(&self) -> PathFilterStats {
        let total_patterns = self.patterns.len();
        let mut active_patterns = 0;
        let mut total_usage = 0;
        
        for counter in self.pattern_usage.iter() {
            let usage = counter.load(Ordering::Relaxed);
            if usage > 0 {
                active_patterns += 1;
            }
            total_usage += usage;
        }
        
        PathFilterStats {
            total_patterns,
            active_patterns,
            total_usage,
        }
    }
}

/// Statistics for path filter performance
#[derive(Debug, Clone)]
pub struct PathFilterStats {
    pub total_patterns: usize,
    pub active_patterns: usize,
    pub total_usage: usize,
}

impl Filter for PathFilter {
    type Input = Path;
    type Output = FilterDecision;
    
    fn filter(&self, path: &Path) -> Result<FilterDecision> {
        // Check if path matches any ignore pattern and get which ones
        let matches: Vec<_> = self.ignore_set.matches(path);
        if !matches.is_empty() {
            // Increment usage counter for each matching pattern (atomic, thread-safe)
            for match_idx in &matches {
                if let Some(counter) = self.pattern_usage.get(*match_idx) {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            }
            
            return Ok(FilterDecision::Skip("matched ignore pattern"));
        }
        
        // All directory filtering is now handled by globset patterns above
        
        Ok(FilterDecision::Process)
    }
    
    fn name(&self) -> &'static str {
        "PathFilter"
    }
    
    fn get_stats(&self) -> SmallVec<[(String, String); 8]> {
        let mut stats = SmallVec::new();
        let total_patterns = self.patterns.len();
        let mut active_patterns = 0;
        
        // First collect active pattern stats
        let mut pattern_stats: SmallVec<[(String, String); 8]> = SmallVec::new();
        for (pattern, counter) in self.patterns.iter().zip(self.pattern_usage.iter()) {
            let count = counter.load(Ordering::Relaxed);
            if count > 0 {
                pattern_stats.push((format!("Pattern: {pattern}"), count.to_string()));
                active_patterns += 1;
            }
        }
        
        // Build final stats with summary first, then pattern details
        stats.push(("Total patterns".to_string(), total_patterns.to_string()));
        stats.push(("Active patterns".to_string(), active_patterns.to_string()));
        stats.extend(pattern_stats);
        
        stats
    }
}