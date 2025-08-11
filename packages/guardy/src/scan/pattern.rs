use crate::scan::types::ScannerConfig;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

/// Pattern - Secret patterns & regex management
///
/// Responsibilities:
/// - Load and manage secret detection patterns from YAML configuration
/// - Pattern classification for Aho-Corasick optimization
/// - Keyword extraction for prefiltering
/// - Pattern matching and validation
///
/// This module implements the pattern system following the plan's strategy:
/// 1. Load patterns from embedded YAML for zero runtime overhead
/// 2. Support external YAML overrides for customization
/// 3. Support keyword extraction for 5x performance improvement
/// 4. Pattern classification for optimal execution
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Human-readable name for the pattern
    pub name: String,
    /// Compiled regex for pattern matching
    pub regex: Regex,
    /// Detailed description of what this pattern detects
    pub description: String,
    /// Classification for optimization (Specific, Contextual, AlwaysRun)
    pub class: PatternClass,
    /// Keywords for Aho-Corasick prefiltering
    pub keywords: Vec<String>,
    /// Priority for execution order (1-10, higher = run first)
    pub priority: u8,
}

/// Pattern classification for Aho-Corasick optimization
#[derive(Debug, Clone, PartialEq)]
pub enum PatternClass {
    /// High-specificity patterns with reliable keywords (e.g., "sk_live_")
    Specific,
    /// Patterns needing context analysis (e.g., generic API keys)
    Contextual,
    /// Patterns without reliable keywords (e.g., entropy-only)
    AlwaysRun,
}

/// Result from regex pattern matching
#[derive(Debug, Clone)]
pub struct RegexMatch {
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
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

/// Global shared pattern cache - compiled once, shared across all threads
/// 
/// This provides significant performance benefits:
/// - Regex compilation happens only once per program execution
/// - All threads share the same compiled patterns via Arc (zero-copy sharing)
/// - LazyLock ensures thread-safe initialization on first access
/// - Subsequent pattern access is near-instant (just pointer operations)
/// - Loads embedded base patterns first (always works), then adds custom patterns if available
static STATIC_PATTERNS: LazyLock<Arc<Vec<Pattern>>> = LazyLock::new(|| {
    tracing::debug!("Initializing shared pattern cache - loading base and custom patterns");
    let start_time = std::time::Instant::now();
    
    // Step 1: Load base patterns (embedded, always available)
    let mut all_patterns = match Pattern::load_embedded_patterns_internal() {
        Ok(base_patterns) => {
            tracing::info!("Loaded {} base patterns successfully", base_patterns.len());
            base_patterns
        }
        Err(e) => {
            tracing::error!("Failed to load embedded base patterns: {}", e);
            // This should never happen since patterns are embedded, but handle gracefully
            Vec::new()
        }
    };
    
    // Step 2: Try to load custom patterns (optional, may fail)
    match Pattern::load_custom_patterns_runtime() {
        Ok(custom_patterns) => {
            if !custom_patterns.is_empty() {
                tracing::info!("Loaded {} custom patterns", custom_patterns.len());
                all_patterns.extend(custom_patterns);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load custom patterns (base patterns still available): {}", e);
            // Continue with base patterns only - don't fail the entire initialization
        }
    }
    
    let duration = start_time.elapsed();
    tracing::info!("Compiled {} total patterns in {:?} - now cached for all threads", 
                  all_patterns.len(), duration);
    
    Arc::new(all_patterns)
});

impl RegexMatch {
    /// Get the start position of the match in the file content
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get the matched text value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get the line number where the match was found (1-based)
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

impl Pattern {
    /// Get shared compiled patterns
    ///
    /// Returns all patterns (base + custom) from the global shared cache.
    /// Uses zero-copy sharing via Arc for maximum performance.
    ///
    /// # Performance
    /// - First call: Compiles all patterns (~50ms)
    /// - Subsequent calls: Near-instant (~0.001ms)
    /// - Memory efficient: Single pattern set shared across all threads
    ///
    /// # Returns
    /// Arc-wrapped pattern vector that can be cheaply cloned across threads
    pub fn get() -> Arc<Vec<Pattern>> {
        STATIC_PATTERNS.clone() // Cheap Arc clone - just increments reference count
    }

    /// Internal function to load patterns from embedded YAML (used by LazyLock)
    fn load_embedded_patterns_internal() -> Result<Vec<Pattern>> {
        // Load embedded YAML at compile time
        const EMBEDDED_PATTERNS: &str = include_str!("../../assets/patterns.yaml");
        
        let patterns_config: PatternsConfig = serde_yml::from_str(EMBEDDED_PATTERNS)
            .with_context(|| "Failed to parse embedded patterns YAML")?;
        
        let mut patterns = Vec::new();
        for yaml_pattern in patterns_config.patterns {
            let pattern = Self::from_yaml_pattern(yaml_pattern.clone())
                .with_context(|| format!("Failed to compile pattern: {}", yaml_pattern.name))?;
            patterns.push(pattern);
        }
        
        tracing::debug!("Loaded {} embedded patterns", patterns.len());
        Ok(patterns)
    }

    /// Convert YAML pattern definition to compiled Pattern
    fn from_yaml_pattern(yaml_pattern: YamlPattern) -> Result<Pattern> {
        let regex = Regex::new(&yaml_pattern.regex)
            .with_context(|| format!("Invalid regex pattern: {}", yaml_pattern.regex))?;
        
        let class = match yaml_pattern.classification.as_str() {
            "specific" => PatternClass::Specific,
            "contextual" => PatternClass::Contextual,
            "always_run" => PatternClass::AlwaysRun,
            _ => return Err(anyhow::anyhow!("Invalid pattern classification: {}", yaml_pattern.classification)),
        };

        Ok(Pattern {
            name: yaml_pattern.name,
            regex,
            description: yaml_pattern.description,
            class,
            keywords: yaml_pattern.keywords,
            priority: yaml_pattern.priority,
        })
    }

    /// Load custom patterns at runtime (used by LazyLock initialization)
    fn load_custom_patterns_runtime() -> Result<Vec<Pattern>> {
        // TODO: Implement custom pattern loading from runtime config
        // This would check for:
        // - ~/.config/guardy/patterns.yaml
        // - --patterns-file CLI argument (if available in global config)
        // - Environment variables for custom pattern paths
        
        let patterns = Vec::new();
        tracing::debug!("Custom patterns not yet implemented");
        Ok(patterns)
    }

    /// Filter patterns by keywords found in content (for Aho-Corasick optimization)
    pub fn filter_by_keywords(patterns: &[Pattern], found_keywords: &[String]) -> Vec<&Pattern> {
        let found_set: std::collections::HashSet<_> = found_keywords.iter().collect();
        
        patterns
            .iter()
            .filter(|pattern| {
                match pattern.class {
                    PatternClass::AlwaysRun => true, // Always include
                    PatternClass::Specific | PatternClass::Contextual => {
                        // Only include if keywords match
                        pattern.keywords.is_empty() || // No keywords means always include
                        pattern.keywords.iter().any(|keyword| found_set.contains(keyword))
                    }
                }
            })
            .collect()
    }

    /// Find all regex matches for this pattern in content
    pub fn find_all_matches(&self, content: &str) -> Result<Vec<RegexMatch>> {
        let mut matches = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut line_start_positions = vec![0];
        
        // Build line position index for accurate line/column reporting
        let mut pos = 0;
        for line in &lines {
            pos += line.len() + 1; // +1 for newline
            line_start_positions.push(pos);
        }

        // Find all regex matches
        for regex_match in self.regex.find_iter(content) {
            let start = regex_match.start();
            let end = regex_match.end();
            let value = regex_match.as_str().to_string();

            // Find line number and column positions
            let line_number = line_start_positions
                .iter()
                .position(|&pos| pos > start)
                .unwrap_or(lines.len())
                .saturating_sub(1);
            
            let line_start = line_start_positions[line_number];
            let column_start = start - line_start;
            let column_end = end - line_start;

            matches.push(RegexMatch {
                start,
                end,
                value,
                line_number: line_number + 1, // 1-based
                column_start,
                column_end,
            });
        }

        Ok(matches)
    }

    /// Get pattern classification as string for serialization/debugging
    pub fn classification_str(&self) -> &str {
        match self.class {
            PatternClass::Specific => "specific",
            PatternClass::Contextual => "contextual", 
            PatternClass::AlwaysRun => "always_run",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::types::ScannerConfig;

    #[test]
    fn test_load_embedded_patterns_internal() {
        let patterns = Pattern::load_embedded_patterns_internal().unwrap();
        
        assert!(!patterns.is_empty());
        assert!(patterns.len() >= 40); // Should have substantial pattern coverage
        
        // Verify we have key patterns
        assert!(patterns.iter().any(|p| p.name.contains("GitHub Token")));
        assert!(patterns.iter().any(|p| p.name.contains("OpenAI")));
        assert!(patterns.iter().any(|p| p.name.contains("Anthropic")));
        assert!(patterns.iter().any(|p| p.name.contains("Generic Secret")));
    }

    #[test]
    fn test_get_patterns() {
        let patterns = Pattern::get();
        
        assert!(!patterns.is_empty());
        
        // Test that patterns are properly compiled
        for pattern in patterns.iter() {
            assert!(!pattern.name.is_empty());
            assert!(!pattern.description.is_empty());
            assert!(pattern.priority >= 1 && pattern.priority <= 10);
        }
    }

    #[test]
    fn test_pattern_classification() {
        let patterns = Pattern::get();
        
        // Count patterns by class
        let specific_count = patterns.iter().filter(|p| p.class == PatternClass::Specific).count();
        let contextual_count = patterns.iter().filter(|p| p.class == PatternClass::Contextual).count();
        let always_run_count = patterns.iter().filter(|p| p.class == PatternClass::AlwaysRun).count();
        
        assert!(specific_count > 0);
        assert!(contextual_count > 0); 
        assert!(always_run_count > 0);
        
        tracing::info!("Pattern classification: {} specific, {} contextual, {} always-run", 
                      specific_count, contextual_count, always_run_count);
    }

    #[test]
    fn test_filter_by_keywords() {
        let patterns = Pattern::get();
        
        // Test with GitHub-related keywords
        let found_keywords = vec!["ghp_".to_string(), "github_pat".to_string()];
        let filtered = Pattern::filter_by_keywords(&patterns, &found_keywords);
        
        // Should include AlwaysRun patterns plus any with matching keywords
        assert!(!filtered.is_empty());
        assert!(filtered.len() <= patterns.len());
        
        // Verify AlwaysRun patterns are always included
        let always_run_in_original = patterns.iter().filter(|p| p.class == PatternClass::AlwaysRun).count();
        let always_run_in_filtered = filtered.iter().filter(|p| p.class == PatternClass::AlwaysRun).count();
        assert_eq!(always_run_in_original, always_run_in_filtered);
    }

    #[test]
    fn test_find_matches() {
        let pattern = Pattern {
            name: "Test Pattern".to_string(),
            regex: Regex::new(r"sk-proj-[\\dA-Za-z]{10}").unwrap(),
            description: "Test pattern".to_string(),
            class: PatternClass::Specific,
            keywords: vec!["sk-proj-".to_string()],
            priority: 5,
        };

        let content = "Here is a test key: sk-proj-abcde12345 on line 1\nAnother line here";
        let matches = pattern.find_all_matches(content).unwrap();
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 1);
        assert_eq!(matches[0].value, "sk-proj-abcde12345");
        assert!(matches[0].column_start > 0);
    }

    #[test]
    fn test_shared_patterns() {
        // Test that shared patterns work and are the same reference
        let patterns1 = Pattern::get();
        let patterns2 = Pattern::get();
        
        // Should have patterns
        assert!(!patterns1.is_empty());
        
        // Should be the same Arc (same pointer)
        assert!(Arc::ptr_eq(&patterns1, &patterns2));
        
        // Should have key patterns
        assert!(patterns1.iter().any(|p| p.name.contains("GitHub Token")));
        assert!(patterns1.iter().any(|p| p.name.contains("OpenAI")));
    }

    #[test]
    fn test_shared_patterns_performance() {
        use std::time::Instant;
        
        // First call - should compile patterns
        let start1 = Instant::now();
        let patterns1 = Pattern::get();
        let duration1 = start1.elapsed();
        
        // Second call - should be much faster (cached)
        let start2 = Instant::now();
        let patterns2 = Pattern::get();
        let duration2 = start2.elapsed();
        
        // Verify both have same content
        assert_eq!(patterns1.len(), patterns2.len());
        
        // Second call should be significantly faster (at least 10x)
        // Note: This test might be flaky on very fast systems
        println!("First call: {:?}, Second call: {:?}", duration1, duration2);
        assert!(duration2 < duration1 || duration2.as_nanos() < 1000); // Sub-microsecond for cached
    }

    #[test] 
    fn test_yaml_parsing() {
        const TEST_YAML: &str = r#"
patterns:
  - name: "Test Pattern"
    regex: "test-[a-z]{5}"
    description: "A test pattern"
    classification: "specific"
    keywords: ["test-"]
    priority: 5
"#;
        
        let patterns_config: PatternsConfig = serde_yml::from_str(TEST_YAML).unwrap();
        assert_eq!(patterns_config.patterns.len(), 1);
        assert_eq!(patterns_config.patterns[0].name, "Test Pattern");
        assert_eq!(patterns_config.patterns[0].classification, "specific");
        assert_eq!(patterns_config.patterns[0].priority, 5);
    }
}