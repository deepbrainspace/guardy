//! Comment-based filtering for guardy:ignore directives
//!
//! This module handles filtering of secret matches based on ignore comments:
//! - `guardy:ignore` - ignore the current line
//! - `guardy:ignore-next` - ignore the next line
//! - `guardy:ignore-line` - ignore the current line (alternative syntax)

use crate::scan::{
    data::SecretMatch,
    filters::{ContentFilter, Filter}
};
use anyhow::Result;
use std::collections::HashSet;

/// Input for comment filtering - matches and the original file content for comment parsing
pub struct CommentFilterInput {
    pub matches: Vec<SecretMatch>,
    pub file_content: String,
}

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
pub struct CommentFilter;

impl CommentFilter {
    /// Create a new comment filter
    pub fn new() -> Self {
        Self
    }
    
    /// Parse ignore directives from file content
    /// Returns a set of line numbers (1-indexed) that should be ignored
    fn parse_ignore_lines(&self, content: &str) -> HashSet<u32> {
        let mut ignored_lines = HashSet::new();
        
        for (line_index, line) in content.lines().enumerate() {
            let line_number = (line_index + 1) as u32; // Convert to 1-indexed and u32
            let line_lower = line.to_lowercase();
            
            // Check for guardy:ignore directives in various comment formats
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
        
        ignored_lines
    }
    
    /// Check if a line contains any comment markers that might contain ignore directives
    fn has_comment_markers(&self, line: &str) -> bool {
        line.contains("//") ||     // JavaScript, Rust, C++, etc.
        line.contains('#') ||      // Python, Bash, YAML, etc.  
        line.contains("<!--") ||   // HTML, XML
        line.contains("/*") ||     // CSS, C, etc.
        line.contains("*")         // Catch multi-line comment continuations
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
                let line_number = secret_match.location.coordinate.line;
                
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

impl ContentFilter for CommentFilter {}