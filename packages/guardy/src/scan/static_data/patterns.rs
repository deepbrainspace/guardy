//! Static pattern library for secret detection
//!
//! This provides a global, shared pattern library that is compiled once
//! and shared across all threads via Arc for zero-copy access.
//!
//! Adapted from scan-v3 implementation for optimal performance.

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

/// A compiled pattern ready for matching
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    /// Pattern index (for Aho-Corasick mapping)
    pub index: usize,
    /// Human-readable name
    pub name: Arc<str>,
    /// Compiled regex
    pub regex: Regex,
    /// Description of what this detects
    pub description: Arc<str>,
    /// Keywords for Aho-Corasick prefiltering
    pub keywords: Vec<String>,
    /// Priority (1-10, higher = run first)
    pub priority: u8,
}


/// The pattern library containing all compiled patterns
pub struct PatternLibrary {
    /// All compiled patterns
    patterns: Vec<CompiledPattern>,
    /// Keywords for Aho-Corasick prefiltering
    keywords: Vec<String>,
    /// Map from pattern index to Arc reference (for zero-copy)
    pattern_map: HashMap<usize, Arc<CompiledPattern>>,
    // Count computed dynamically from patterns.len()
}

impl PatternLibrary {
    /// Create a new pattern library from base and custom patterns
    fn new() -> Result<Self> {
        let start = std::time::Instant::now();
        
        // Step 1: Compile base patterns directly from native Rust data
        let mut all_patterns = Vec::new();
        let mut keywords = Vec::new();
        let mut pattern_map = HashMap::new();
        
        use super::base_patterns::BASE_PATTERNS;
        
        // Process base patterns directly (no YAML conversion)
        for (index, base_pattern) in BASE_PATTERNS.iter().enumerate() {
            let compiled = Self::compile_base_pattern(index, base_pattern)?;
            keywords.extend(compiled.keywords.clone());
            let arc_pattern = Arc::new(compiled.clone());
            pattern_map.insert(index, arc_pattern);
            all_patterns.push(compiled);
        }
        
        tracing::info!("Compiled {} base patterns", BASE_PATTERNS.len());
        
        // Step 2: Process custom patterns (if any) - for future config integration
        // TODO: Load custom patterns from config system when needed
        
        // Sort patterns by priority (higher first)
        all_patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Remove duplicate keywords
        keywords.sort();
        keywords.dedup();
        
        tracing::info!(
            "Pattern library initialized with {} patterns in {:?}",
            all_patterns.len(),
            start.elapsed()
        );
        
        Ok(Self {
            patterns: all_patterns,
            keywords,
            pattern_map,
        })
    }
    
    /// Compile a base pattern directly into a CompiledPattern (zero YAML overhead)
    fn compile_base_pattern(index: usize, base: &super::base_patterns::BasePattern) -> Result<CompiledPattern> {
        let regex = Regex::new(base.regex)
            .with_context(|| format!("Failed to compile regex for pattern '{}'", base.name))?;
        
        Ok(CompiledPattern {
            index,
            name: Arc::from(base.name),
            regex,
            description: Arc::from(base.description),
            keywords: base.keywords.iter().map(|&s| s.to_string()).collect(),
            priority: base.priority,
        })
    }
    
    
    /// Get all patterns
    pub fn patterns(&self) -> &[CompiledPattern] {
        &self.patterns
    }
    
    /// Get all keywords for Aho-Corasick
    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }
    
    /// Get a pattern by index (zero-copy via Arc)
    pub fn get_pattern(&self, index: usize) -> Option<Arc<CompiledPattern>> {
        self.pattern_map.get(&index).cloned()
    }
    
    /// Get total pattern count
    pub fn count(&self) -> usize {
        self.patterns.len()
    }
}

/// Global shared pattern library - compiled once, shared everywhere
pub static PATTERN_LIBRARY: LazyLock<Arc<PatternLibrary>> = LazyLock::new(|| {
    Arc::new(
        PatternLibrary::new()
            .expect("Failed to initialize pattern library - this is fatal")
    )
});

/// Get the global pattern library
pub fn get_pattern_library() -> Arc<PatternLibrary> {
    PATTERN_LIBRARY.clone()
}