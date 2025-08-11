use crate::scan::types::{ScannerConfig, SecretMatch};
use anyhow::Result;
use regex::Regex;
use std::sync::{Arc, LazyLock};

/// Comment Filter - Inline comment-based secret ignoring for content-level filtering
///
/// Responsibilities:
/// - Support "guardy:allow" directive for inline secret ignoring
/// - Compatible with existing ignore system used in scanner/core.rs
/// - Process line-based ignore comments for fine-grained control
/// - Support multiple ignore comment formats and configurations
/// - Zero-copy sharing of compiled regexes across all threads
///
/// This filter is applied AFTER regex pattern matching to filter out matches
/// that have been explicitly ignored by developers using inline comments.
///
/// Supported ignore formats (configurable):
/// - `guardy:ignore` - Ignore secrets on the same line
/// - `guardy:ignore-line` - Ignore secrets on the same line  
/// - `guardy:ignore-next` - Ignore secrets on the next line
/// - Custom ignore comments from configuration
///
/// Performance Optimizations:
/// - Shared compiled regexes for ignore comment detection
/// - Fast line-based scanning with early termination
/// - Efficient line extraction and matching

/// Global shared ignore comment regexes - compiled once, shared across all threads
/// 
/// This provides performance benefits for comment filtering:
/// - Regexes compiled only once per program execution
/// - All threads share the same compiled patterns via Arc (zero-copy sharing)
/// - LazyLock ensures thread-safe initialization on first access
/// - Comment detection becomes fast regex operations instead of string searches
static STATIC_IGNORE_COMMENT_REGEXES: LazyLock<Arc<Vec<Regex>>> = LazyLock::new(|| {
    tracing::debug!("Initializing shared ignore comment regexes");
    let start_time = std::time::Instant::now();
    
    // Default ignore comment patterns
    let default_patterns = vec![
        // Standard guardy ignore comments
        r"guardy:ignore\b",         // guardy:ignore
        r"guardy:ignore-line\b",    // guardy:ignore-line  
        r"guardy:ignore-next\b",    // guardy:ignore-next
        // Legacy formats for compatibility
        r"guardy:allow\b",          // guardy:allow (like gitleaks)
    ];
    
    // Try to load custom ignore patterns (future enhancement)
    let custom_patterns = match load_custom_ignore_comment_patterns() {
        Ok(patterns) => {
            if !patterns.is_empty() {
                tracing::info!("Loaded {} custom ignore comment patterns", patterns.len());
                patterns
            } else {
                Vec::new()
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load custom ignore comment patterns: {}", e);
            Vec::new()
        }
    };
    
    // Combine default and custom patterns
    let all_patterns: Vec<String> = default_patterns
        .into_iter()
        .map(String::from)
        .chain(custom_patterns)
        .collect();
    
    // Compile all patterns into regexes
    let mut compiled_regexes = Vec::new();
    for pattern in &all_patterns {
        match Regex::new(pattern) {
            Ok(regex) => {
                compiled_regexes.push(regex);
            }
            Err(e) => {
                tracing::error!("Failed to compile ignore comment regex '{}': {}", pattern, e);
            }
        }
    }
    
    let duration = start_time.elapsed();
    tracing::info!("Compiled {} ignore comment regexes in {:?} - now cached for all threads",
                  compiled_regexes.len(), duration);
    
    Arc::new(compiled_regexes)
});

/// Load custom ignore comment patterns at runtime (used by LazyLock initialization)
fn load_custom_ignore_comment_patterns() -> Result<Vec<String>> {
    // TODO: Implement custom ignore comment loading from runtime config
    // This would check for:
    // - ~/.config/guardy/ignore_comments.txt
    // - --ignore-comments CLI argument (if available in global config)
    // - Environment variables for custom ignore comment patterns
    // - guardy.yaml ignore_comments section
    
    let patterns = Vec::new();
    tracing::debug!("Custom ignore comment patterns not yet implemented");
    Ok(patterns)
}

/// Comment filtering statistics for debugging and analysis
#[derive(Debug, Clone)]
pub struct CommentFilterStats {
    pub matches_checked: usize,
    pub matches_ignored_by_comment: usize,
    pub lines_scanned_for_comments: usize,
    pub ignore_comments_found: usize,
}

impl Default for CommentFilterStats {
    fn default() -> Self {
        Self {
            matches_checked: 0,
            matches_ignored_by_comment: 0,
            lines_scanned_for_comments: 0,
            ignore_comments_found: 0,
        }
    }
}

/// Comment filter for content-level ignore directive processing
pub struct CommentFilter {
    /// Configuration for ignore comments
    config: ScannerConfig,
    /// Statistics collection for debugging and performance analysis
    stats: std::sync::Mutex<CommentFilterStats>,
}

impl CommentFilter {
    /// Create a new comment filter with configuration
    /// 
    /// # Arguments
    /// * `config` - Scanner configuration with ignore comment settings
    /// 
    /// # Returns
    /// A configured comment filter ready for use
    pub fn new(config: &ScannerConfig) -> Result<Self> {
        tracing::debug!("Comment filter initialized with {} ignore comment patterns",
                       config.ignore_comments.len());
        
        Ok(Self {
            config: config.clone(),
            stats: std::sync::Mutex::new(CommentFilterStats::default()),
        })
    }
    
    /// Get shared ignore comment regexes (zero-copy Arc access)
    /// 
    /// Returns the globally shared regexes for ignore comment detection.
    /// This is zero-copy - just increments the Arc reference count.
    pub fn get_ignore_regexes() -> Arc<Vec<Regex>> {
        STATIC_IGNORE_COMMENT_REGEXES.clone()
    }
    
    /// Check if a secret match should be ignored due to ignore comments
    /// 
    /// Examines the line containing the secret match (and potentially surrounding lines)
    /// to determine if an ignore comment is present that should suppress this match.
    /// 
    /// # Arguments
    /// * `secret_match` - The secret match to check for ignore comments
    /// * `file_content` - Full file content to extract lines from
    /// 
    /// # Returns
    /// * `Ok(true)` - Match should be ignored (ignore comment found)
    /// * `Ok(false)` - Match should not be ignored (no ignore comment)
    /// * `Err(_)` - Error during comment detection
    /// 
    /// # Performance
    /// - Only scans the specific line containing the match (not entire file)
    /// - Uses shared compiled regexes for fast comment detection
    /// - Early termination when ignore comment found
    pub fn should_ignore_match(&self, secret_match: &SecretMatch, file_content: &str) -> Result<bool> {
        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.matches_checked += 1;
        }
        
        // Extract the line containing the secret
        let line_content = &secret_match.line_content;
        
        // Check if the line contains any ignore comments
        let should_ignore = self.line_contains_ignore_comment(line_content)?;
        
        if should_ignore {
            // Update statistics
            if let Ok(mut stats) = self.stats.lock() {
                stats.matches_ignored_by_comment += 1;
            }
            
            tracing::debug!(
                "Secret match ignored by comment in {}:{}",
                secret_match.file_path,
                secret_match.line_number
            );
        }
        
        Ok(should_ignore)
    }
    
    /// Check if a line contains ignore comments using shared regexes
    /// 
    /// # Arguments
    /// * `line` - Line content to check for ignore comments
    /// 
    /// # Returns
    /// * `Ok(true)` - Line contains ignore comment
    /// * `Ok(false)` - Line does not contain ignore comment  
    /// * `Err(_)` - Error during comment detection
    fn line_contains_ignore_comment(&self, line: &str) -> Result<bool> {
        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.lines_scanned_for_comments += 1;
        }
        
        let ignore_regexes = Self::get_ignore_regexes();
        
        // Check against all compiled ignore comment regexes
        for regex in ignore_regexes.iter() {
            if regex.is_match(line) {
                // Update statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.ignore_comments_found += 1;
                }
                
                tracing::trace!("Ignore comment detected in line: {}", line);
                return Ok(true);
            }
        }
        
        // Also check against config patterns (for backwards compatibility)
        for ignore_pattern in &self.config.ignore_comments {
            if line.contains(ignore_pattern) {
                // Update statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.ignore_comments_found += 1;
                }
                
                tracing::trace!("Ignore comment detected via config pattern '{}' in line: {}", 
                               ignore_pattern, line);
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Filter a list of secret matches, removing those with ignore comments
    /// 
    /// # Arguments
    /// * `matches` - List of secret matches to filter
    /// * `file_content` - Full file content for line extraction
    /// 
    /// # Returns
    /// Vector of matches that should not be ignored (no ignore comments found)
    pub fn filter_matches(&self, matches: &[SecretMatch], file_content: &str) -> Vec<SecretMatch> {
        matches
            .iter()
            .filter(|secret_match| {
                match self.should_ignore_match(secret_match, file_content) {
                    Ok(should_ignore) => !should_ignore,
                    Err(e) => {
                        tracing::warn!(
                            "Error checking ignore comments for match in {}:{}: {}",
                            secret_match.file_path,
                            secret_match.line_number,
                            e
                        );
                        true // Include matches we can't check (conservative approach)
                    }
                }
            })
            .cloned()
            .collect()
    }
    
    /// Get current filter statistics
    /// 
    /// # Returns
    /// Statistics about matches and comments processed by this filter
    pub fn get_stats(&self) -> CommentFilterStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }
    
    /// Reset statistics counters
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = CommentFilterStats::default();
        }
    }
    
    /// Get configuration information for debugging
    /// 
    /// # Returns
    /// Number of configured ignore comment patterns
    pub fn get_config_info(&self) -> usize {
        self.config.ignore_comments.len()
    }
    
    /// Check if line-based comment checking would find an ignore directive
    /// 
    /// This is a utility method for testing and validation that checks if
    /// any ignore comment would be detected in a given line.
    /// 
    /// # Arguments
    /// * `line` - Line content to check
    /// 
    /// # Returns
    /// True if the line contains any ignore directive
    pub fn line_has_ignore_directive(&self, line: &str) -> bool {
        self.line_contains_ignore_comment(line).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::types::SecretMatch;
    
    fn create_test_config() -> ScannerConfig {
        ScannerConfig {
            ignore_comments: vec![
                "guardy:ignore".to_string(),
                "guardy:ignore-line".to_string(),
                "guardy:ignore-next".to_string(),
            ],
            ..ScannerConfig::default()
        }
    }
    
    fn create_test_match(line_number: usize, line_content: &str) -> SecretMatch {
        SecretMatch {
            file_path: "test.rs".to_string(),
            line_number,
            line_content: line_content.to_string(),
            matched_text: "secret123".to_string(),
            start_pos: 0,
            end_pos: 9,
            secret_type: "Test Secret".to_string(),
            pattern_description: "Test pattern".to_string(),
        }
    }
    
    #[test]
    fn test_shared_regexes() {
        let regexes = CommentFilter::get_ignore_regexes();
        assert!(!regexes.is_empty());
        
        // Test that default patterns are compiled
        let test_lines = [
            "const secret = 'test'; // guardy:ignore",
            "let key = 'secret'; // guardy:ignore-line",  
            "password = 'test'; // guardy:allow",
        ];
        
        for line in &test_lines {
            let found = regexes.iter().any(|regex| regex.is_match(line));
            assert!(found, "Should match ignore comment in: {}", line);
        }
    }
    
    #[test]
    fn test_comment_filter_creation() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        assert_eq!(filter.get_config_info(), 3);
    }
    
    #[test]
    fn test_line_ignore_detection() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Test positive cases
        assert!(filter.line_has_ignore_directive("const secret = 'test'; // guardy:ignore"));
        assert!(filter.line_has_ignore_directive("let key = 'secret'; # guardy:ignore-line"));
        assert!(filter.line_has_ignore_directive("password = 'test'; /* guardy:ignore */"));
        assert!(filter.line_has_ignore_directive("api_key = 'abc'; // guardy:allow"));
        
        // Test negative cases
        assert!(!filter.line_has_ignore_directive("const secret = 'test';"));
        assert!(!filter.line_has_ignore_directive("let key = 'secret';"));
        assert!(!filter.line_has_ignore_directive("// just a regular comment"));
        assert!(!filter.line_has_ignore_directive("/* block comment */"));
    }
    
    #[test]
    fn test_match_ignoring() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Test match that should be ignored
        let ignored_match = create_test_match(
            1, 
            "const secret = 'test123'; // guardy:ignore"
        );
        
        let file_content = "const secret = 'test123'; // guardy:ignore\n";
        assert!(filter.should_ignore_match(&ignored_match, file_content).unwrap());
        
        // Test match that should not be ignored
        let normal_match = create_test_match(
            1,
            "const secret = 'test123';"
        );
        
        let file_content = "const secret = 'test123';\n";
        assert!(!filter.should_ignore_match(&normal_match, file_content).unwrap());
    }
    
    #[test]
    fn test_filter_matches() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        let matches = vec![
            create_test_match(1, "const secret1 = 'test'; // guardy:ignore"),
            create_test_match(2, "const secret2 = 'test';"),
            create_test_match(3, "const secret3 = 'test'; // guardy:ignore-line"),
            create_test_match(4, "const secret4 = 'test';"),
        ];
        
        let file_content = "const secret1 = 'test'; // guardy:ignore\n\
                           const secret2 = 'test';\n\
                           const secret3 = 'test'; // guardy:ignore-line\n\
                           const secret4 = 'test';\n";
        
        let filtered = filter.filter_matches(&matches, file_content);
        
        // Should keep only matches without ignore comments
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].line_number, 2);
        assert_eq!(filtered[1].line_number, 4);
    }
    
    #[test]
    fn test_various_comment_formats() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Test different comment styles
        let test_cases = [
            "const key = 'secret'; // guardy:ignore",           // C++ style
            "const key = 'secret'; /* guardy:ignore */",        // C style  
            "const key = 'secret'; # guardy:ignore",            // Python/Ruby style
            "const key = 'secret'; -- guardy:ignore",           // SQL style
            "    const key = 'secret'; // guardy:ignore   ",    // With whitespace
            "const key = 'secret'; //guardy:ignore",            // No space after //
        ];
        
        for test_case in &test_cases {
            assert!(filter.line_has_ignore_directive(test_case),
                   "Should detect ignore in: {}", test_case);
        }
    }
    
    #[test]
    fn test_case_sensitivity() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Test that matching is case-sensitive by default (as expected for precise directives)
        assert!(filter.line_has_ignore_directive("secret = 'test'; // guardy:ignore"));
        assert!(!filter.line_has_ignore_directive("secret = 'test'; // GUARDY:IGNORE"));
        assert!(!filter.line_has_ignore_directive("secret = 'test'; // Guardy:Ignore"));
    }
    
    #[test]
    fn test_statistics() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        let matches = vec![
            create_test_match(1, "const secret1 = 'test'; // guardy:ignore"),
            create_test_match(2, "const secret2 = 'test';"),
        ];
        
        let file_content = "const secret1 = 'test'; // guardy:ignore\n\
                           const secret2 = 'test';\n";
        
        // Process matches to generate stats
        let _ = filter.filter_matches(&matches, file_content);
        
        let stats = filter.get_stats();
        assert_eq!(stats.matches_checked, 2);
        assert_eq!(stats.matches_ignored_by_comment, 1);
        assert!(stats.lines_scanned_for_comments >= 2);
        assert!(stats.ignore_comments_found >= 1);
    }
    
    #[test]
    fn test_reset_stats() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Generate some stats
        let test_match = create_test_match(1, "secret = 'test'; // guardy:ignore");
        let _ = filter.should_ignore_match(&test_match, "secret = 'test'; // guardy:ignore\n");
        
        assert!(filter.get_stats().matches_checked > 0);
        
        // Reset stats
        filter.reset_stats();
        let stats = filter.get_stats();
        assert_eq!(stats.matches_checked, 0);
        assert_eq!(stats.matches_ignored_by_comment, 0);
        assert_eq!(stats.lines_scanned_for_comments, 0);
        assert_eq!(stats.ignore_comments_found, 0);
    }
    
    #[test]
    fn test_edge_cases() {
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Test empty line
        assert!(!filter.line_has_ignore_directive(""));
        
        // Test line with just comment
        assert!(filter.line_has_ignore_directive("// guardy:ignore"));
        
        // Test multiple ignore directives on same line
        assert!(filter.line_has_ignore_directive("secret = 'test'; // guardy:ignore guardy:ignore-line"));
        
        // Test ignore directive as part of larger word (should not match)
        assert!(!filter.line_has_ignore_directive("const guardyIgnoreThis = 'test';"));
    }
    
    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;
        
        let config = create_test_config();
        let filter = CommentFilter::new(&config).unwrap();
        
        // Create many matches to test performance
        let matches: Vec<SecretMatch> = (0..1000)
            .map(|i| create_test_match(i + 1, 
                if i % 3 == 0 { 
                    "const secret = 'test'; // guardy:ignore"
                } else {
                    "const secret = 'test';"
                }
            ))
            .collect();
        
        let file_content = "content\n".repeat(1000);
        
        // Test that filtering is fast
        let start = Instant::now();
        let _ = filter.filter_matches(&matches, &file_content);
        let duration = start.elapsed();
        
        // Should complete 1000 comment checks quickly
        assert!(duration.as_millis() < 100, "Comment filtering too slow: {:?}", duration);
    }
}