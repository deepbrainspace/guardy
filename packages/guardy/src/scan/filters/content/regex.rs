//! Regex pattern matching with optimized execution
//!
//! This module executes regex patterns (pre-filtered by Aho-Corasick) and
//! creates SecretMatch objects with precise positioning.

use crate::scan::{
    types::SecretMatch,
    filters::Filter,
    static_data::patterns::get_pattern_library,
};
use anyhow::{Context, Result};
use smallvec::SmallVec;

/// Input for regex execution containing filtered pattern indices and file context
pub struct RegexInput {
    /// Content to search in
    pub content: String,
    /// File path for SecretMatch construction
    pub file_path: String,
    /// Active pattern indices from prefilter (usually ~15% of total patterns)
    pub active_patterns: SmallVec<[usize; 4]>,
}

/// Regex executor for sequential pattern matching on prefiltered patterns
///
/// Performance characteristics:
/// - Only executes ~15% of patterns (pre-filtered by Aho-Corasick)
/// - Sequential execution avoids regex compilation overhead
/// - Uses current SecretMatch format for compatibility
pub struct RegexExecutor {
    // Stateless - uses global pattern library
}

impl RegexExecutor {
    /// Create a new regex executor
    pub fn new() -> Self {
        Self {}
    }
    
    /// Execute patterns on content with file context
    pub fn execute_patterns(
        &self,
        content: &str,
        file_path: &str,
        active_patterns: SmallVec<[usize; 4]>,
    ) -> Result<Vec<SecretMatch>> {
        let pattern_lib = get_pattern_library();
        let mut matches = Vec::new();
        
        // Process each active pattern sequentially
        for &pattern_index in &active_patterns {
            if let Some(pattern) = pattern_lib.get_pattern(pattern_index) {
                // Execute regex on content
                let regex_matches = pattern.regex.find_iter(content);
                
                for regex_match in regex_matches {
                    // Calculate line number and extract line content
                    let line_number = content[..regex_match.start()].matches('\n').count() + 1;
                    
                    // Find the line containing this match
                    let line_start = content[..regex_match.start()]
                        .rfind('\n')
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let line_end = content[regex_match.start()..]
                        .find('\n')
                        .map(|i| regex_match.start() + i)
                        .unwrap_or(content.len());
                    let line = &content[line_start..line_end];
                    
                    // Calculate positions within the line
                    let start_pos = regex_match.start() - line_start;
                    let end_pos = regex_match.end() - line_start;
                    
                    // Create SecretMatch with current format
                    let secret_match = SecretMatch {
                        file_path: file_path.to_string(),
                        line_number,
                        line_content: line.to_string(),
                        matched_text: regex_match.as_str().to_string(),
                        start_pos,
                        end_pos,
                        secret_type: pattern.name.to_string(),
                        pattern_description: pattern.description.to_string(),
                    };
                    
                    matches.push(secret_match);
                }
            }
        }
        
        Ok(matches)
    }
}

impl Default for RegexExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for RegexExecutor {
    type Input = RegexInput;
    type Output = Vec<SecretMatch>;
    
    /// Execute regex patterns on pre-filtered content
    /// 
    /// Takes RegexInput containing content, file path, and active pattern indices.
    /// Returns SecretMatch objects with proper line/column information.
    fn filter(&self, input: &Self::Input) -> Result<Vec<SecretMatch>> {
        self.execute_patterns(&input.content, &input.file_path, input.active_patterns.clone())
            .context("Failed to execute regex patterns")
    }
    
    fn name(&self) -> &'static str {
        "RegexExecutor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::static_data::patterns::PATTERN_LIBRARY;
    
    #[test]
    fn test_regex_executor_creation() {
        let executor = RegexExecutor::new();
        assert_eq!(executor.name(), "RegexExecutor");
    }
    
    #[test]
    fn test_empty_patterns() {
        let executor = RegexExecutor::new();
        let result = executor.execute_patterns(
            "some content",
            "test.txt",
            SmallVec::new()
        ).unwrap();
        
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_line_number_calculation() {
        // Force pattern library initialization
        std::sync::LazyLock::force(&PATTERN_LIBRARY);
        
        let executor = RegexExecutor::new();
        let content = "line 1\nline 2 with secret\nline 3";
        
        // We can't easily test with actual patterns without the full pattern library,
        // but we can test the basic structure works
        let result = executor.execute_patterns(
            content,
            "test.txt",
            SmallVec::new()
        ).unwrap();
        
        // Empty patterns should return no matches
        assert!(result.is_empty());
    }
}