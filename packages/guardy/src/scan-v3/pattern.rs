use crate::scan_v3::types::{ScannerConfig, SecretPattern, SecretPatterns};
use crate::scan_v3::file::RegexMatch;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;

/// Pattern - Secret patterns & regex management
///
/// Responsibilities:  
/// - Load and manage secret detection patterns
/// - Compile regex patterns with caching
/// - Filter patterns by Aho-Corasick keywords
/// - Execute pattern matching against content
/// - Pattern classification and optimization
pub struct Pattern {
    /// Compiled regex for efficient matching
    regex: Regex,
    /// Original pattern metadata
    pattern: SecretPattern,
    /// Keywords that should trigger this pattern (for Aho-Corasick prefiltering)
    keywords: Vec<String>,
}

impl Pattern {
    /// Create a new Pattern instance from SecretPattern
    pub fn new(pattern: SecretPattern) -> Result<Self> {
        let regex = Regex::new(&pattern.regex)
            .with_context(|| format!("Failed to compile regex for pattern '{}': {}", pattern.description, pattern.regex))?;

        // Extract keywords from the pattern for Aho-Corasick prefiltering
        let keywords = Self::extract_keywords_from_regex(&pattern.regex);

        Ok(Self {
            regex,
            pattern,
            keywords,
        })
    }

    /// Load all patterns from the existing Guardy pattern configuration
    ///
    /// This method maintains 100% compatibility with the existing scanner
    /// by using the same pattern loading mechanism from the legacy implementation.
    ///
    /// # Parameters
    /// - `config`: Scanner configuration specifying pattern sources
    ///
    /// # Returns
    /// Vector of compiled Pattern instances ready for matching
    pub fn load_patterns(config: &ScannerConfig) -> Result<Vec<Pattern>> {
        // Load patterns using the existing mechanism from the legacy scanner
        let secret_patterns = crate::scanner::types::SecretPatterns::load(config)
            .context("Failed to load secret patterns using legacy scanner pattern loader")?;

        let mut compiled_patterns = Vec::new();

        // Convert each SecretPattern to our compiled Pattern struct
        for pattern in secret_patterns.patterns {
            match Pattern::new(pattern) {
                Ok(compiled_pattern) => compiled_patterns.push(compiled_pattern),
                Err(e) => {
                    // Log pattern compilation errors but continue with other patterns
                    tracing::warn!("Skipping invalid pattern: {}", e);
                }
            }
        }

        Ok(compiled_patterns)
    }

    /// Filter patterns by Aho-Corasick keywords to optimize pattern matching
    ///
    /// This method implements the critical performance optimization that provides ~5x speed improvement.
    /// Only patterns whose keywords were found by Aho-Corasick prefiltering are returned for regex execution.
    ///
    /// # Parameters
    /// - `patterns`: All available patterns
    /// - `found_keywords`: Keywords found by Aho-Corasick prefiltering
    ///
    /// # Returns
    /// Filtered vector containing only patterns that might match based on keyword presence
    pub fn filter_by_keywords(patterns: &[Pattern], found_keywords: &[String]) -> Vec<&Pattern> {
        patterns.iter()
            .filter(|pattern| {
                // Check if any of this pattern's keywords were found
                pattern.keywords.iter().any(|keyword| found_keywords.contains(keyword))
            })
            .collect()
    }

    /// Find all matches of this pattern in the given content
    ///
    /// This method executes the compiled regex against the content and returns
    /// detailed match information including line/column positions.
    ///
    /// # Parameters
    /// - `content`: File content to search within
    ///
    /// # Returns
    /// Vector of RegexMatch instances with position and context information
    pub fn find_all_matches(&self, content: &str) -> Result<Vec<RegexMatch>> {
        let mut matches = Vec::new();

        for regex_match in self.regex.find_iter(content) {
            let match_start = regex_match.start();
            let match_end = regex_match.end();
            let match_value = regex_match.as_str().to_string();

            // Calculate line and column information
            let (line_number, column_start, column_end) = Self::calculate_line_column_info(content, match_start, match_end)?;

            matches.push(RegexMatch {
                start: match_start,
                end: match_end,
                value: match_value,
                line_number,
                column_start,
                column_end,
            });
        }

        Ok(matches)
    }

    /// Get the pattern description for reporting
    pub fn description(&self) -> &str {
        &self.pattern.description
    }

    /// Get the pattern ID for categorization
    pub fn id(&self) -> &str {
        &self.pattern.id
    }

    /// Get the raw regex string
    pub fn regex(&self) -> &str {
        &self.pattern.regex
    }

    /// Get the entropy validation requirements
    pub fn entropy(&self) -> &Option<crate::scan_v3::types::EntropyConfig> {
        &self.pattern.entropy
    }

    /// Get the keywords used for Aho-Corasick prefiltering
    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    /// Extract keywords from a regex pattern for Aho-Corasick prefiltering
    ///
    /// This method analyzes regex patterns to extract literal strings that can be used
    /// for fast keyword-based prefiltering. The goal is to find common substrings that
    /// appear in matches of this pattern.
    ///
    /// # Algorithm
    /// 1. Look for literal string sequences in the regex
    /// 2. Extract common prefixes/suffixes (like "sk-", "ghp_", etc.)
    /// 3. Extract character class patterns (like sequences of base64 characters)
    /// 4. Return keywords that are at least 3 characters long for efficiency
    ///
    /// # Parameters
    /// - `regex_str`: The regex pattern to analyze
    ///
    /// # Returns
    /// Vector of keyword strings for Aho-Corasick matching
    fn extract_keywords_from_regex(regex_str: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // Extract literal strings from the regex (simplified heuristic)
        // This is a basic implementation that looks for common patterns

        // Look for literal prefixes like "sk-", "ghp_", "xoxb-", etc.
        if let Some(captures) = regex::Regex::new(r#"^[^\\]*?([a-zA-Z0-9\-_]{3,})"#).unwrap().captures(regex_str) {
            if let Some(prefix) = captures.get(1) {
                let prefix_str = prefix.as_str().to_string();
                if prefix_str.len() >= 3 {
                    keywords.push(prefix_str);
                }
            }
        }

        // Look for quoted literal strings
        for captures in regex::Regex::new(r#"["']([^"']{3,})["']"#).unwrap().captures_iter(regex_str) {
            if let Some(literal) = captures.get(1) {
                keywords.push(literal.as_str().to_string());
            }
        }

        // Look for common secret patterns
        let common_patterns = [
            ("sk-", "Stripe"),
            ("pk_", "Stripe"),
            ("ghp_", "GitHub"),
            ("gho_", "GitHub"),
            ("ghu_", "GitHub"),
            ("ghs_", "GitHub"),
            ("xoxb-", "Slack"),
            ("xoxa-", "Slack"),
            ("xoxp-", "Slack"),
            ("AIza", "Google API"),
            ("AKIA", "AWS"),
            ("ya29", "Google OAuth"),
        ];

        for (prefix, _description) in common_patterns {
            if regex_str.contains(prefix) {
                keywords.push(prefix.to_string());
            }
        }

        // Remove duplicates and return
        keywords.sort();
        keywords.dedup();
        keywords
    }

    /// Calculate line and column information for a match position
    ///
    /// This method converts byte positions to human-readable line/column coordinates
    /// for better error reporting and match context.
    ///
    /// # Parameters
    /// - `content`: The full file content
    /// - `match_start`: Byte position where match starts
    /// - `match_end`: Byte position where match ends
    ///
    /// # Returns
    /// Tuple of (line_number, column_start, column_end) where line_number is 1-based
    fn calculate_line_column_info(content: &str, match_start: usize, match_end: usize) -> Result<(usize, usize, usize)> {
        let mut line_number = 1;
        let mut current_pos = 0;
        let mut line_start_pos = 0;

        // Find the line containing the match start
        for line in content.lines() {
            let line_end_pos = current_pos + line.len();
            
            if match_start >= current_pos && match_start <= line_end_pos {
                let column_start = match_start - line_start_pos + 1; // 1-based
                let column_end = match_end - line_start_pos + 1;     // 1-based
                return Ok((line_number, column_start, column_end));
            }

            current_pos = line_end_pos + 1; // +1 for newline character
            line_start_pos = current_pos;
            line_number += 1;
        }

        // Default fallback if line detection fails
        Ok((1, match_start + 1, match_end + 1))
    }

    /// Get pattern statistics for performance monitoring
    pub fn get_stats(&self) -> PatternStats {
        PatternStats {
            id: self.pattern.id.clone(),
            description: self.pattern.description.clone(),
            keyword_count: self.keywords.len(),
            has_entropy_validation: self.pattern.entropy.is_some(),
        }
    }
}

/// Pattern statistics for performance monitoring and analysis
#[derive(Debug, Clone)]
pub struct PatternStats {
    pub id: String,
    pub description: String,
    pub keyword_count: usize,
    pub has_entropy_validation: bool,
}

/// Pattern loading cache for performance optimization
pub struct PatternCache {
    compiled_patterns: HashMap<String, Vec<Pattern>>,
}

impl PatternCache {
    /// Create a new empty pattern cache
    pub fn new() -> Self {
        Self {
            compiled_patterns: HashMap::new(),
        }
    }

    /// Load patterns with caching based on configuration hash
    pub fn load_patterns_cached(&mut self, config: &ScannerConfig) -> Result<&Vec<Pattern>> {
        // Create a simple cache key based on relevant config
        let cache_key = format!("{:?}", config.patterns_file);

        if !self.compiled_patterns.contains_key(&cache_key) {
            let patterns = Pattern::load_patterns(config)?;
            self.compiled_patterns.insert(cache_key.clone(), patterns);
        }

        Ok(self.compiled_patterns.get(&cache_key).unwrap())
    }

    /// Clear the pattern cache (useful for configuration changes)
    pub fn clear(&mut self) {
        self.compiled_patterns.clear();
    }
}

impl Default for PatternCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords_from_regex() {
        // Test simple literal prefix
        let keywords = Pattern::extract_keywords_from_regex("sk-[a-zA-Z0-9]{24}");
        assert!(keywords.contains(&"sk-".to_string()));

        // Test GitHub token pattern
        let keywords = Pattern::extract_keywords_from_regex("ghp_[a-zA-Z0-9]{36}");
        assert!(keywords.contains(&"ghp_".to_string()));

        // Test quoted literals
        let keywords = Pattern::extract_keywords_from_regex(r#""Bearer "[a-zA-Z0-9]+"#);
        assert!(keywords.len() > 0);
    }

    #[test]
    fn test_pattern_creation() {
        let secret_pattern = SecretPattern {
            id: "test-pattern".to_string(),
            description: "Test Pattern".to_string(),
            regex: r"test-[0-9]+".to_string(),
            entropy: None,
            keywords: None,
            path: None,
        };

        let pattern = Pattern::new(secret_pattern).unwrap();
        assert_eq!(pattern.description(), "Test Pattern");
        assert_eq!(pattern.id(), "test-pattern");
        assert_eq!(pattern.regex(), r"test-[0-9]+");
    }

    #[test]
    fn test_find_all_matches() {
        let secret_pattern = SecretPattern {
            id: "test-pattern".to_string(),
            description: "Test Pattern".to_string(),
            regex: r"test-\d+".to_string(),
            entropy: None,
            keywords: None,
            path: None,
        };

        let pattern = Pattern::new(secret_pattern).unwrap();
        let content = "This is test-123 and test-456 in content.";
        
        let matches = pattern.find_all_matches(content).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].value, "test-123");
        assert_eq!(matches[1].value, "test-456");
    }

    #[test]
    fn test_filter_by_keywords() {
        let pattern1 = Pattern::new(SecretPattern {
            id: "github".to_string(),
            description: "GitHub Token".to_string(),
            regex: "ghp_[a-zA-Z0-9]{36}".to_string(),
            entropy: None,
            keywords: None,
            path: None,
        }).unwrap();

        let pattern2 = Pattern::new(SecretPattern {
            id: "stripe".to_string(),
            description: "Stripe Key".to_string(),
            regex: "sk-[a-zA-Z0-9]{24}".to_string(),
            entropy: None,
            keywords: None,
            path: None,
        }).unwrap();

        let patterns = vec![pattern1, pattern2];
        let found_keywords = vec!["ghp_".to_string()];

        let filtered = Pattern::filter_by_keywords(&patterns, &found_keywords);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id(), "github");
    }

    #[test]
    fn test_calculate_line_column_info() {
        let content = "Line 1\nLine 2 with match here\nLine 3";
        let match_start = 7; // Start of "Line 2"
        let match_end = 12;   // "Line "

        let (line_num, col_start, col_end) = Pattern::calculate_line_column_info(content, match_start, match_end).unwrap();
        assert_eq!(line_num, 2);
        assert_eq!(col_start, 1);
        assert_eq!(col_end, 6);
    }
}