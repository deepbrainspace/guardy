//! Static pattern library for secret detection
//!
//! This provides a global, shared pattern library that is compiled once
//! and shared across all threads via Arc for zero-copy access.
//!
//! Adapted from scan-v3 implementation for optimal performance.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
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

/// YAML pattern definition for deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct YamlPattern {
    name: String,
    regex: String,
    description: String,
    classification: String,
    keywords: Vec<String>,
    priority: u8,
}

/// YAML patterns file structure
#[derive(Debug, Serialize, Deserialize)]
struct PatternsConfig {
    patterns: Vec<YamlPattern>,
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
        
        // Step 1: Load base patterns from embedded YAML
        let base_patterns = Self::load_embedded_patterns()?;
        tracing::info!("Loaded {} base patterns", base_patterns.len());
        
        // Step 2: Try to load custom patterns (optional)
        let custom_patterns = Self::load_custom_patterns().unwrap_or_else(|e| {
            tracing::warn!("Failed to load custom patterns: {}", e);
            Vec::new()
        });
        if !custom_patterns.is_empty() {
            tracing::info!("Loaded {} custom patterns", custom_patterns.len());
        }
        
        // Step 3: Merge and compile all patterns
        let mut all_patterns = Vec::new();
        let mut keywords = Vec::new();
        let mut pattern_map = HashMap::new();
        
        // Process base patterns
        for (index, yaml_pattern) in base_patterns.into_iter().enumerate() {
            let compiled = Self::compile_pattern(index, yaml_pattern)?;
            keywords.extend(compiled.keywords.clone());
            let arc_pattern = Arc::new(compiled.clone());
            pattern_map.insert(index, arc_pattern);
            all_patterns.push(compiled);
        }
        
        // Process custom patterns (continue numbering)
        let base_count = all_patterns.len();
        for (offset, yaml_pattern) in custom_patterns.into_iter().enumerate() {
            let index = base_count + offset;
            let compiled = Self::compile_pattern(index, yaml_pattern)?;
            keywords.extend(compiled.keywords.clone());
            let arc_pattern = Arc::new(compiled.clone());
            pattern_map.insert(index, arc_pattern);
            all_patterns.push(compiled);
        }
        
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
    
    /// Load embedded patterns from YAML
    fn load_embedded_patterns() -> Result<Vec<YamlPattern>> {
        const EMBEDDED_PATTERNS: &str = include_str!("../../../assets/patterns.yaml");
        
        let config: PatternsConfig = serde_yaml_bw::from_str(EMBEDDED_PATTERNS)
            .context("Failed to parse embedded patterns YAML")?;
        
        Ok(config.patterns)
    }
    
    /// Load custom patterns from scanner configuration
    fn load_custom_patterns() -> Result<Vec<YamlPattern>> {
        // For now return empty - patterns can be added via config later
        // TODO: Integrate with GuardyConfig when custom patterns are needed
        Ok(Vec::new())
    }
    
    /// Compile a YAML pattern into a CompiledPattern
    fn compile_pattern(index: usize, yaml: YamlPattern) -> Result<CompiledPattern> {
        let regex = Regex::new(&yaml.regex)
            .with_context(|| format!("Failed to compile regex for pattern '{}'", yaml.name))?;
        
        Ok(CompiledPattern {
            index,
            name: Arc::from(yaml.name.as_str()),
            regex,
            description: Arc::from(yaml.description.as_str()),
            keywords: yaml.keywords,
            priority: yaml.priority,
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