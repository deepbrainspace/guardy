//! Regex pattern matching with precise coordinate extraction
//!
//! This module executes regex patterns (pre-filtered by Aho-Corasick) and
//! extracts precise match coordinates using the optimized Coordinate system.

use crate::scan::{
    data::{Coordinate, SecretMatch},
    filters::{Filter},
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
        // Count the number of characters up to the byte offset
        text[..byte_offset.min(text.len())]
            .chars()
            .count()
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


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordinate_calculation() {
        let content = "line 1\nline 2 with secret\nline 3";
        let start = content.find("secret").unwrap();
        let end = start + "secret".len();
        
        let coord = RegexExecutor::calculate_coordinate(content, start, end);
        
        assert_eq!(coord.line, 2);
        assert_eq!(coord.column_start, 12); // "line 2 with " is 12 characters
        assert_eq!(coord.column_end(), 18);  // 12 + "secret".len() = 18
        assert_eq!(coord.byte_start as usize, start);
        assert_eq!(coord.byte_end as usize, end);
    }
    
}