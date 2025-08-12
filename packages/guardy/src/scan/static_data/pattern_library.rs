//! Pattern library for secret detection
//!
//! This provides a global, shared pattern library that is compiled once
//! and shared across all threads via Arc for zero-copy access.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

/// Pattern classification for optimization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternClass {
    /// High-specificity patterns with reliable keywords (e.g., "sk_live_")
    #[serde(rename = "specific")]
    Specific,
    /// Patterns needing context analysis (e.g., generic API keys)
    #[serde(rename = "contextual")]
    Contextual,
    /// Patterns without reliable keywords (e.g., entropy-only)
    #[serde(rename = "always_run")]
    AlwaysRun,
}

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
    /// Classification for optimization
    pub class: PatternClass,
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
    /// Total pattern count
    count: usize,
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
        
        let count = all_patterns.len();
        tracing::info!(
            "Pattern library initialized with {} patterns in {:?}",
            count,
            start.elapsed()
        );
        
        Ok(Self {
            patterns: all_patterns,
            keywords,
            pattern_map,
            count,
        })
    }
    
    /// Load embedded patterns from YAML
    fn load_embedded_patterns() -> Result<Vec<YamlPattern>> {
        const EMBEDDED_PATTERNS: &str = include_str!("../../../assets/patterns.yaml");
        
        let config: PatternsConfig = serde_yaml_bw::from_str(EMBEDDED_PATTERNS)
            .context("Failed to parse embedded patterns YAML")?;
        
        Ok(config.patterns)
    }
    
    /// Load custom patterns from configuration
    fn load_custom_patterns() -> Result<Vec<YamlPattern>> {
        // TODO: Implement loading from:
        // - ~/.config/guardy/patterns.yaml
        // - Environment variable GUARDY_CUSTOM_PATTERNS
        // - CLI argument --patterns-file
        
        // For now, return empty
        Ok(Vec::new())
    }
    
    /// Compile a YAML pattern into a CompiledPattern
    fn compile_pattern(index: usize, yaml: YamlPattern) -> Result<CompiledPattern> {
        let regex = Regex::new(&yaml.regex)
            .with_context(|| format!("Failed to compile regex for pattern '{}'", yaml.name))?;
        
        let class = match yaml.classification.as_str() {
            "specific" => PatternClass::Specific,
            "contextual" => PatternClass::Contextual,
            "always_run" => PatternClass::AlwaysRun,
            other => {
                tracing::warn!("Unknown pattern class '{}', defaulting to contextual", other);
                PatternClass::Contextual
            }
        };
        
        Ok(CompiledPattern {
            index,
            name: Arc::from(yaml.name.as_str()),
            regex,
            description: Arc::from(yaml.description.as_str()),
            class,
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
    
    /// Get patterns filtered by class
    pub fn patterns_by_class(&self, class: PatternClass) -> Vec<&CompiledPattern> {
        self.patterns
            .iter()
            .filter(|p| p.class == class)
            .collect()
    }
    
    /// Get total pattern count
    pub fn count(&self) -> usize {
        self.count
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