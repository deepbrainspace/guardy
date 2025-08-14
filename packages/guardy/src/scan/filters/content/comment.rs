//! Comment-based filtering for guardy:ignore directives
//!
//! This module handles filtering of secret matches based on ignore comments:
//! - `guardy:ignore` - ignore the current line
//! - `guardy:ignore-next` - ignore the next line
//! - `guardy:ignore-line` - ignore the current line (alternative syntax)

use crate::scan::{
    types::SecretMatch,
    filters::Filter,
};
use anyhow::Result;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

/// Input for comment filtering - matches and the original file content for comment parsing
pub struct CommentFilterInput {
    pub matches: Vec<SecretMatch>,
    pub file_content: String,
}

/// Static set of comment markers for fast lookup
/// Pre-compiled at startup for optimal performance
static COMMENT_MARKERS: LazyLock<Arc<HashSet<&'static str>>> = LazyLock::new(|| {
    let markers = HashSet::from([
        "//",     // JavaScript, Rust, C++, etc.
        "#",      // Python, Bash, YAML, etc.
        "<!--",   // HTML, XML
        "/*",     // CSS, C, etc.
        "*",      // Multi-line comment continuations
    ]);
    Arc::new(markers)
});

/// Filter matches based on guardy:ignore directives in comments
/// 
/// Supported ignore patterns:
/// - `guardy:ignore` - ignore secrets on the same line
/// - `guardy:ignore-next` - ignore secrets on the next line  
/// - `guardy:ignore-line` - ignore secrets on the same line (alternative)
/// 
/// Works with various comment styles:
/// - `// guardy:ignore` (JavaScript, Rust, C++, etc.)
/// - `# guardy:ignore` (Python, Bash, YAML, etc.)
/// - `<!-- guardy:ignore -->` (HTML, XML)
/// - `/* guardy:ignore */` (CSS, C, etc.)
pub struct CommentFilter {
    // Empty - uses static data for maximum performance
}

impl CommentFilter {
    /// Create a new comment filter
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse ignore directives from file content
    /// Returns a set of line numbers (1-indexed) that should be ignored
    fn parse_ignore_lines(&self, content: &str) -> HashSet<usize> {
        let mut ignored_lines = HashSet::new();
        
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index + 1; // Convert to 1-indexed
            let line_lower = line.to_lowercase();
            
            // Only process lines that contain comment markers - use static lookup for speed
            if self.has_comment_markers(line) {
                // Check for guardy:ignore directives in comment lines
                if line_lower.contains("guardy:ignore-next") {
                    // Ignore the next line
                    ignored_lines.insert(line_number + 1);
                    tracing::debug!("Found guardy:ignore-next on line {}, ignoring line {}", line_number, line_number + 1);
                } else if line_lower.contains("guardy:ignore-line") || line_lower.contains("guardy:ignore") {
                    // Ignore the current line
                    ignored_lines.insert(line_number);
                    tracing::debug!("Found guardy:ignore on line {}, ignoring line {}", line_number, line_number);
                }
            }
        }
        
        ignored_lines
    }
    
    /// Check if a line contains any comment markers that might contain ignore directives
    /// Uses static HashSet for optimal performance
    fn has_comment_markers(&self, line: &str) -> bool {
        let markers = &*COMMENT_MARKERS;
        markers.iter().any(|marker| line.contains(marker))
    }
}

impl Filter for CommentFilter {
    type Input = CommentFilterInput;
    type Output = Vec<SecretMatch>;
    
    fn filter(&self, input: &CommentFilterInput) -> Result<Vec<SecretMatch>> {
        if input.matches.is_empty() {
            return Ok(Vec::new());
        }
        
        // Parse ignore directives from the file content
        let ignored_lines = self.parse_ignore_lines(&input.file_content);
        
        if ignored_lines.is_empty() {
            // No ignore directives found, return all matches
            tracing::trace!("No guardy:ignore directives found, keeping all {} matches", input.matches.len());
            return Ok(input.matches.clone());
        }
        
        // Filter matches based on ignored lines
        let filtered_matches: Vec<SecretMatch> = input.matches
            .iter()
            .filter(|secret_match| {
                let line_number = secret_match.line_number;
                
                if ignored_lines.contains(&line_number) {
                    tracing::debug!(
                        "Filtering out match '{}' on line {} due to guardy:ignore directive",
                        secret_match.matched_text,
                        line_number
                    );
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        let filtered_count = input.matches.len() - filtered_matches.len();
        if filtered_count > 0 {
            tracing::info!(
                "CommentFilter removed {} matches due to guardy:ignore directives", 
                filtered_count
            );
        }
        
        Ok(filtered_matches)
    }
    
    fn name(&self) -> &'static str {
        "CommentFilter"
    }
}

impl Default for CommentFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comment_markers_detection() {
        let filter = CommentFilter::new();
        
        assert!(filter.has_comment_markers("// This is a comment"));
        assert!(filter.has_comment_markers("# Python comment"));  
        assert!(filter.has_comment_markers("<!-- HTML comment -->"));
        assert!(filter.has_comment_markers("/* CSS comment */"));
        assert!(filter.has_comment_markers("* Continuation"));
        
        assert!(!filter.has_comment_markers("No comments here"));
        assert!(!filter.has_comment_markers("Just regular text"));
    }
    
    #[test]
    fn test_ignore_lines_parsing() {
        let filter = CommentFilter::new();
        let content = r#"line 1
// guardy:ignore
line 3  
# guardy:ignore-next
line 5
line 6
<!-- guardy:ignore-line -->
line 8"#;
        
        let ignored = filter.parse_ignore_lines(content);
        
        // Should ignore: line 2 (guardy:ignore), line 5 (guardy:ignore-next), line 7 (guardy:ignore-line)
        assert!(ignored.contains(&2));
        assert!(ignored.contains(&5)); 
        assert!(ignored.contains(&7));
        assert!(!ignored.contains(&1));
        assert!(!ignored.contains(&3));
        assert!(!ignored.contains(&4));
        assert!(!ignored.contains(&6));
        assert!(!ignored.contains(&8));
    }
    
    #[test]
    fn test_filter_empty_matches() {
        let filter = CommentFilter::new();
        let input = CommentFilterInput {
            matches: Vec::new(),
            file_content: "// guardy:ignore".to_string(),
        };
        
        let result = filter.filter(&input).unwrap();
        assert!(result.is_empty());
    }
    
    #[test] 
    fn test_filter_no_ignore_directives() {
        let filter = CommentFilter::new();
        let matches = vec![
            SecretMatch {
                file_path: "test.rs".to_string(),
                line_number: 1,
                line_content: "secret here".to_string(),
                matched_text: "secret".to_string(),
                start_pos: 0,
                end_pos: 6,
                secret_type: "test".to_string(),
                pattern_description: "test pattern".to_string(),
            }
        ];
        let input = CommentFilterInput {
            matches: matches.clone(),
            file_content: "No ignore directives".to_string(),
        };
        
        let result = filter.filter(&input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].matched_text, "secret");
    }
    
    #[test]
    fn test_filter_with_ignore_directives() {
        let filter = CommentFilter::new();
        let matches = vec![
            SecretMatch {
                file_path: "test.rs".to_string(),
                line_number: 2,
                line_content: "secret here".to_string(),
                matched_text: "secret".to_string(),
                start_pos: 0,
                end_pos: 6,
                secret_type: "test".to_string(),
                pattern_description: "test pattern".to_string(),
            },
            SecretMatch {
                file_path: "test.rs".to_string(),
                line_number: 4,
                line_content: "another secret".to_string(),
                matched_text: "another".to_string(),
                start_pos: 0,
                end_pos: 7,
                secret_type: "test".to_string(),
                pattern_description: "test pattern".to_string(),
            }
        ];
        let input = CommentFilterInput {
            matches,
            file_content: r#"line 1
// guardy:ignore
line 3
another secret"#.to_string(),
        };
        
        let result = filter.filter(&input).unwrap();
        // First match (line 2) should be filtered out, second match (line 4) should remain
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].matched_text, "another");
        assert_eq!(result[0].line_number, 4);
    }
}