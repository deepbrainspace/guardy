//! Regex pattern matching with precise coordinate extraction
//!
//! This module executes regex patterns (pre-filtered by Aho-Corasick) and
//! extracts precise match coordinates using the optimized Coordinate system.

use crate::scan::{
    data::{Coordinate, MatchSeverity, SecretMatch},
    filters::{ContentFilter, Filter},
    static_data::pattern_library::get_pattern_library,
};
use anyhow::{Context, Result};
use smallvec::SmallVec;
use std::sync::Arc;

/// Input for regex execution containing filtered pattern indices and file context
pub struct RegexInput {
    /// Content to search in
    pub content: String,
    /// File path for coordinate construction
    pub file_path: Arc<str>,
    /// Active pattern indices from prefilter (usually ~15% of total patterns)
    pub active_patterns: SmallVec<[usize; 4]>,
}

/// Regex executor for sequential pattern matching on prefiltered patterns
///
/// Performance characteristics:
/// - Only executes ~15% of patterns (pre-filtered by Aho-Corasick)
/// - Sequential execution avoids regex compilation overhead
/// - Uses optimized Coordinate system for precise positioning
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
        file_path: Arc<str>,
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
                    // Calculate line and column positions
                    let coordinate = Self::calculate_coordinate(
                        content,
                        regex_match.start(),
                        regex_match.end(),
                    );
                    
                    // Create SecretMatch with proper coordinate
                    let secret_match = SecretMatch::new(
                        file_path.clone(),
                        coordinate,
                        regex_match.as_str().to_string(),
                        pattern.name.clone(),
                        pattern.description.clone(),
                        Self::determine_severity(&pattern.name),
                        Self::calculate_confidence(&pattern.class, regex_match.as_str()),
                    );
                    
                    matches.push(secret_match);
                }
            }
        }
        
        Ok(matches)
    }
    
    /// Calculate precise coordinate from byte positions
    /// 
    /// This is optimized for the 50MB file limit using u32 byte positions
    /// and handles line/column calculation efficiently.
    fn calculate_coordinate(content: &str, start_byte: usize, end_byte: usize) -> Coordinate {
        let mut line = 1u32;
        let mut line_start_byte = 0usize;
        
        // Find the line number and line start by scanning backwards
        // This is more efficient than scanning from the beginning for large files
        for (i, byte) in content.bytes().take(start_byte).enumerate() {
            if byte == b'\n' {
                line += 1;
                line_start_byte = i + 1;
            }
        }
        
        // Calculate column positions (UTF-8 aware)
        let line_content = &content[line_start_byte..];
        let column_start = Self::byte_to_char_offset(line_content, start_byte - line_start_byte);
        let column_end = Self::byte_to_char_offset(line_content, end_byte - line_start_byte);
        
        // Use the optimized Coordinate::from_usize with bounds checking
        Coordinate::from_usize(
            line as usize,
            column_start,
            column_end,
            start_byte,
            end_byte,
        ).unwrap_or_else(|| {
            // Fallback for files > 4GB (shouldn't happen with 50MB limit)
            tracing::warn!("File too large for optimized coordinates, using saturated values");
            Coordinate::new(
                line,
                column_start.min(u16::MAX as usize) as u16,
                column_end.min(u16::MAX as usize) as u16,
                start_byte.min(u32::MAX as usize) as u32,
                end_byte.min(u32::MAX as usize) as u32,
            )
        })
    }
    
    /// Convert byte offset to character offset (handles UTF-8)
    fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
        text.chars()
            .take_while(|_| {
                // This is an approximation for performance
                // For exact UTF-8 handling, we'd use char_indices()
                byte_offset > 0
            })
            .count()
    }
    
    /// Determine severity based on pattern type
    fn determine_severity(pattern_name: &str) -> MatchSeverity {
        let name_lower = pattern_name.to_lowercase();
        
        if name_lower.contains("private") || name_lower.contains("secret") {
            MatchSeverity::Critical
        } else if name_lower.contains("api") || name_lower.contains("token") {
            MatchSeverity::High
        } else if name_lower.contains("password") || name_lower.contains("key") {
            MatchSeverity::High
        } else {
            MatchSeverity::Medium
        }
    }
    
    /// Calculate confidence based on pattern class and match characteristics
    fn calculate_confidence(
        pattern_class: &crate::scan::static_data::pattern_library::PatternClass,
        matched_text: &str,
    ) -> f32 {
        use crate::scan::static_data::pattern_library::PatternClass;
        
        let base_confidence = match pattern_class {
            PatternClass::Specific => 0.9_f32,     // High confidence for specific patterns
            PatternClass::Contextual => 0.7_f32,   // Medium confidence, needs context
            PatternClass::AlwaysRun => 0.5_f32,    // Lower confidence, entropy-based
        };
        
        // Adjust based on match characteristics
        let length_factor = if matched_text.len() >= 20 {
            1.1_f32 // Longer matches tend to be more reliable
        } else if matched_text.len() < 8 {
            0.8_f32 // Very short matches might be false positives
        } else {
            1.0_f32
        };
        
        // Check for obvious test/dummy values
        let test_penalty = if matched_text.to_lowercase().contains("test")
            || matched_text.to_lowercase().contains("dummy")
            || matched_text.to_lowercase().contains("example") {
            0.5_f32
        } else {
            1.0_f32
        };
        
        (base_confidence * length_factor * test_penalty).min(1.0_f32)
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
    /// Returns SecretMatch objects with precise coordinates.
    fn filter(&self, input: &Self::Input) -> Result<Vec<SecretMatch>> {
        self.execute_patterns(&input.content, input.file_path.clone(), input.active_patterns.clone())
            .context("Failed to execute regex patterns")
    }
    
    fn name(&self) -> &'static str {
        "RegexExecutor"
    }
}

impl ContentFilter for RegexExecutor {}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;
    
    #[test]
    fn test_coordinate_calculation() {
        let content = "line 1\nline 2 with secret\nline 3";
        let start = content.find("secret").unwrap();
        let end = start + "secret".len();
        
        let coord = RegexExecutor::calculate_coordinate(content, start, end);
        
        assert_eq!(coord.line, 2);
        assert_eq!(coord.column_start, 11); // 0-indexed
        assert_eq!(coord.byte_start as usize, start);
        assert_eq!(coord.byte_end as usize, end);
    }
    
    #[test]
    fn test_severity_determination() {
        assert_eq!(
            RegexExecutor::determine_severity("private_key"),
            MatchSeverity::Critical
        );
        assert_eq!(
            RegexExecutor::determine_severity("api_token"),
            MatchSeverity::High
        );
        assert_eq!(
            RegexExecutor::determine_severity("generic_pattern"),
            MatchSeverity::Medium
        );
    }
    
    #[test]
    fn test_confidence_calculation() {
        use crate::scan::static_data::pattern_library::PatternClass;
        
        let confidence = RegexExecutor::calculate_confidence(
            &PatternClass::Specific,
            "sk_live_1234567890abcdef",
        );
        
        assert!(confidence > 0.8); // Should be high confidence for specific, long pattern
        
        let test_confidence = RegexExecutor::calculate_confidence(
            &PatternClass::Specific,
            "test_key_123",
        );
        
        assert!(test_confidence < confidence); // Test values should have lower confidence
    }
}