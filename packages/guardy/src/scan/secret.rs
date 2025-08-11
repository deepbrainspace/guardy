use crate::scan::pattern::{Pattern, RegexMatch};
use crate::scan::types::SecretMatch;
use anyhow::{Context, Result};
use std::path::Path;

/// Secret - SecretMatch creation & validation
///
/// Responsibilities:
/// - Create SecretMatch instances from pattern matches
/// - Extract line content and context information
/// - Validate and enrich match metadata
/// - Handle edge cases in match processing
///
/// This module implements the secret match creation following the plan's strategy:
/// 1. Port the SecretMatch structure for compatibility
/// 2. Add comprehensive validation and error handling
/// 3. Extract surrounding context for better match reporting
/// 4. Follow the existing data model from scanner/types.rs
pub struct Secret;

impl Secret {
    /// Create a SecretMatch from a pattern match with full context
    ///
    /// This method creates a comprehensive SecretMatch object that includes:
    /// - File path and location information
    /// - Pattern metadata (name, description)
    /// - Match details (text, position, line/column)
    /// - Surrounding line content for context
    ///
    /// # Parameters
    /// - `file_path`: Path to the file where the match was found
    /// - `pattern`: The pattern that generated this match
    /// - `regex_match`: The regex match details from pattern matching
    /// - `file_content`: Full file content for extracting line context
    ///
    /// # Returns
    /// A complete SecretMatch with all metadata populated
    ///
    /// # Errors
    /// - File path conversion errors
    /// - Line content extraction issues
    /// - Match validation failures
    pub fn create_match(
        file_path: &Path,
        pattern: &Pattern,
        regex_match: &RegexMatch,
        file_content: &str,
    ) -> Result<SecretMatch> {
        // Convert file path to string
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Extract the line content containing this match
        let line_content = Self::extract_line_content(file_content, regex_match.line_number)
            .with_context(|| format!(
                "Failed to extract line content for match at line {} in {}",
                regex_match.line_number, file_path_str
            ))?;
        
        // Validate the match data
        Self::validate_match_data(regex_match, &line_content)
            .with_context(|| format!(
                "Invalid match data for pattern '{}' in {}",
                pattern.name, file_path_str
            ))?;
        
        // Create the SecretMatch object
        Ok(SecretMatch {
            file_path: file_path_str,
            line_number: regex_match.line_number,
            line_content,
            matched_text: regex_match.value.clone(),
            start_pos: regex_match.start,
            end_pos: regex_match.end,
            secret_type: pattern.name.clone(),
            pattern_description: pattern.description.clone(),
        })
    }

    /// Extract the line content for a given line number
    ///
    /// This method safely extracts the line content from file content,
    /// handling edge cases like empty files, invalid line numbers, and
    /// different line ending formats.
    ///
    /// # Parameters
    /// - `file_content`: The complete file content
    /// - `line_number`: Target line number (1-based)
    ///
    /// # Returns
    /// The line content as a string, or empty string if line doesn't exist
    ///
    /// # Errors
    /// - Line number out of bounds
    /// - Content parsing issues
    fn extract_line_content(file_content: &str, line_number: usize) -> Result<String> {
        if line_number == 0 {
            return Err(anyhow::anyhow!("Line number must be >= 1, got {}", line_number));
        }
        
        let lines: Vec<&str> = file_content.lines().collect();
        
        if line_number > lines.len() {
            return Err(anyhow::anyhow!(
                "Line number {} exceeds file line count {}",
                line_number, lines.len()
            ));
        }
        
        // Convert to 0-based index and extract line
        let line_index = line_number - 1;
        let line_content = lines.get(line_index)
            .ok_or_else(|| anyhow::anyhow!("Failed to get line {} from content", line_number))?;
        
        Ok(line_content.to_string())
    }

    /// Validate match data for consistency and correctness
    ///
    /// This method performs sanity checks on the match data to ensure:
    /// - Match positions are valid
    /// - Match text is non-empty
    /// - Line numbers are reasonable
    /// - Data consistency between components
    ///
    /// # Parameters
    /// - `regex_match`: The regex match to validate
    /// - `line_content`: The line content where the match was found
    ///
    /// # Returns
    /// Ok(()) if validation passes
    ///
    /// # Errors
    /// - Invalid match positions
    /// - Empty or malformed match data
    /// - Inconsistent data between match components
    fn validate_match_data(regex_match: &RegexMatch, line_content: &str) -> Result<()> {
        // Check basic match data
        if regex_match.value.is_empty() {
            return Err(anyhow::anyhow!("Match text cannot be empty"));
        }
        
        if regex_match.start >= regex_match.end {
            return Err(anyhow::anyhow!(
                "Invalid match positions: start {} >= end {}",
                regex_match.start, regex_match.end
            ));
        }
        
        if regex_match.line_number == 0 {
            return Err(anyhow::anyhow!("Line number must be >= 1"));
        }
        
        // Check column positions are within line bounds
        if regex_match.column_start > line_content.len() {
            return Err(anyhow::anyhow!(
                "Column start {} exceeds line length {}",
                regex_match.column_start, line_content.len()
            ));
        }
        
        if regex_match.column_end > line_content.len() {
            return Err(anyhow::anyhow!(
                "Column end {} exceeds line length {}",
                regex_match.column_end, line_content.len()
            ));
        }
        
        // Verify the match text appears in the line content
        // Note: This is a sanity check - the exact position might differ due to line endings
        if !line_content.contains(&regex_match.value) {
            tracing::warn!(
                "Match text '{}' not found in line content '{}' - possible line ending mismatch",
                regex_match.value, line_content
            );
            // Don't fail here as this can happen with different line endings
        }
        
        Ok(())
    }

    /// Create multiple SecretMatch objects from a list of regex matches
    ///
    /// This is a convenience method for batch processing multiple matches
    /// from the same file and pattern, which is common in the scanning pipeline.
    ///
    /// # Parameters
    /// - `file_path`: Path to the file where matches were found
    /// - `pattern`: The pattern that generated these matches  
    /// - `regex_matches`: List of regex matches to process
    /// - `file_content`: Full file content for context extraction
    ///
    /// # Returns
    /// Vector of SecretMatch objects, filtering out any that failed validation
    ///
    /// # Note
    /// This method logs errors for failed matches but continues processing
    /// the remaining matches to maximize successful detection.
    pub fn create_matches(
        file_path: &Path,
        pattern: &Pattern,
        regex_matches: &[RegexMatch],
        file_content: &str,
    ) -> Vec<SecretMatch> {
        let mut secret_matches = Vec::new();
        let file_path_str = file_path.to_string_lossy();
        
        for (i, regex_match) in regex_matches.iter().enumerate() {
            match Self::create_match(file_path, pattern, regex_match, file_content) {
                Ok(secret_match) => {
                    secret_matches.push(secret_match);
                }
                Err(e) => {
                    // Log the error but continue processing other matches
                    tracing::debug!(
                        "Failed to create secret match {} for pattern '{}' in {}: {}",
                        i + 1, pattern.name, file_path_str, e
                    );
                }
            }
        }
        
        tracing::debug!(
            "Created {} secret matches from {} regex matches for pattern '{}' in {}",
            secret_matches.len(), regex_matches.len(), pattern.name, file_path_str
        );
        
        secret_matches
    }

    /// Extract context around a match for enhanced reporting
    ///
    /// This method extracts additional lines around a match to provide
    /// better context for security analysis and false positive reduction.
    ///
    /// # Parameters
    /// - `file_content`: The complete file content
    /// - `line_number`: Line number of the match (1-based)
    /// - `context_lines`: Number of lines to include before/after (default: 2)
    ///
    /// # Returns
    /// Context information with surrounding lines
    ///
    /// # Note
    /// This is prepared for future enhancement but not currently used
    /// in the basic SecretMatch structure. Could be added for advanced reporting.
    pub fn extract_context(
        file_content: &str,
        line_number: usize,
        context_lines: usize,
    ) -> Result<MatchContext> {
        let lines: Vec<&str> = file_content.lines().collect();
        
        if line_number == 0 || line_number > lines.len() {
            return Err(anyhow::anyhow!(
                "Line number {} out of bounds (1-{})",
                line_number, lines.len()
            ));
        }
        
        let line_index = line_number - 1; // Convert to 0-based
        
        // Calculate context range
        let start_line = line_index.saturating_sub(context_lines);
        let end_line = std::cmp::min(line_index + context_lines + 1, lines.len());
        
        // Extract context lines
        let context_lines_vec: Vec<String> = lines[start_line..end_line]
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        Ok(MatchContext {
            target_line: line_number,
            context_start_line: start_line + 1, // Convert back to 1-based
            context_lines: context_lines_vec,
        })
    }
}

/// Context information around a match (for future enhancement)
#[derive(Debug, Clone)]
pub struct MatchContext {
    /// The line number containing the actual match (1-based)
    pub target_line: usize,
    /// The starting line number of the context (1-based)
    pub context_start_line: usize,
    /// The lines of context including the target line
    pub context_lines: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::pattern::{Pattern, PatternClass, RegexMatch};
    use regex::Regex;
    use std::path::PathBuf;

    fn create_test_pattern() -> Pattern {
        Pattern {
            name: "Test Pattern".to_string(),
            regex: Regex::new(r"test-key-[a-z]{5}").unwrap(),
            description: "A test pattern for unit testing".to_string(),
            class: PatternClass::Specific,
            keywords: vec!["test-key-".to_string()],
            priority: 5,
        }
    }

    fn create_test_match() -> RegexMatch {
        RegexMatch {
            start: 20,
            end: 35,
            value: "test-key-abcde".to_string(),
            line_number: 2,
            column_start: 8,
            column_end: 23,
        }
    }

    #[test]
    fn test_create_match_success() {
        let pattern = create_test_pattern();
        let regex_match = create_test_match();
        let file_path = PathBuf::from("/test/file.txt");
        let file_content = "line 1 content\nSecure test-key-abcde here\nline 3 content";
        
        let secret_match = Secret::create_match(&file_path, &pattern, &regex_match, file_content).unwrap();
        
        assert_eq!(secret_match.file_path, "/test/file.txt");
        assert_eq!(secret_match.line_number, 2);
        assert_eq!(secret_match.line_content, "Secure test-key-abcde here");
        assert_eq!(secret_match.matched_text, "test-key-abcde");
        assert_eq!(secret_match.start_pos, 20);
        assert_eq!(secret_match.end_pos, 35);
        assert_eq!(secret_match.secret_type, "Test Pattern");
        assert_eq!(secret_match.pattern_description, "A test pattern for unit testing");
    }

    #[test]
    fn test_extract_line_content() {
        let content = "first line\nsecond line\nthird line";
        
        assert_eq!(Secret::extract_line_content(content, 1).unwrap(), "first line");
        assert_eq!(Secret::extract_line_content(content, 2).unwrap(), "second line");
        assert_eq!(Secret::extract_line_content(content, 3).unwrap(), "third line");
        
        // Test error cases
        assert!(Secret::extract_line_content(content, 0).is_err());
        assert!(Secret::extract_line_content(content, 4).is_err());
    }

    #[test]
    fn test_validate_match_data() {
        let valid_match = create_test_match();
        let line_content = "Secure test-key-abcde here";
        
        // Valid match should pass
        assert!(Secret::validate_match_data(&valid_match, line_content).is_ok());
        
        // Test empty match text
        let empty_match = RegexMatch {
            value: String::new(),
            ..valid_match.clone()
        };
        assert!(Secret::validate_match_data(&empty_match, line_content).is_err());
        
        // Test invalid positions
        let invalid_positions = RegexMatch {
            start: 25,
            end: 20, // end < start
            ..valid_match.clone()
        };
        assert!(Secret::validate_match_data(&invalid_positions, line_content).is_err());
        
        // Test zero line number
        let zero_line = RegexMatch {
            line_number: 0,
            ..valid_match
        };
        assert!(Secret::validate_match_data(&zero_line, line_content).is_err());
    }

    #[test]
    fn test_create_matches_batch() {
        let pattern = create_test_pattern();
        let file_path = PathBuf::from("/test/file.txt");
        let file_content = "line 1\ntest-key-alpha and test-key-bravo\nline 3";
        
        let regex_matches = vec![
            RegexMatch {
                start: 7,
                end: 21,
                value: "test-key-alpha".to_string(),
                line_number: 2,
                column_start: 0,
                column_end: 14,
            },
            RegexMatch {
                start: 26,
                end: 40,
                value: "test-key-bravo".to_string(),
                line_number: 2,
                column_start: 19,
                column_end: 33,
            },
        ];
        
        let secret_matches = Secret::create_matches(&file_path, &pattern, &regex_matches, file_content);
        
        assert_eq!(secret_matches.len(), 2);
        assert_eq!(secret_matches[0].matched_text, "test-key-alpha");
        assert_eq!(secret_matches[1].matched_text, "test-key-bravo");
        assert_eq!(secret_matches[0].line_number, 2);
        assert_eq!(secret_matches[1].line_number, 2);
    }

    #[test]
    fn test_extract_context() {
        let content = "line 1\nline 2\nline 3 TARGET\nline 4\nline 5";
        
        let context = Secret::extract_context(content, 3, 1).unwrap();
        
        assert_eq!(context.target_line, 3);
        assert_eq!(context.context_start_line, 2);
        assert_eq!(context.context_lines.len(), 3); // lines 2, 3, 4
        assert_eq!(context.context_lines[0], "line 2");
        assert_eq!(context.context_lines[1], "line 3 TARGET");
        assert_eq!(context.context_lines[2], "line 4");
    }

    #[test]
    fn test_extract_context_edge_cases() {
        let content = "only line";
        
        // Single line file
        let context = Secret::extract_context(content, 1, 2).unwrap();
        assert_eq!(context.context_lines.len(), 1);
        assert_eq!(context.context_lines[0], "only line");
        
        // Invalid line number
        assert!(Secret::extract_context(content, 0).is_err());
        assert!(Secret::extract_context(content, 2).is_err());
    }

    #[test]
    fn test_create_match_with_different_line_endings() {
        let pattern = create_test_pattern();
        let file_path = PathBuf::from("/test/file.txt");
        
        // Test with different line ending styles
        let windows_content = "line 1\r\nSecure test-key-abcde here\r\nline 3";
        let regex_match = RegexMatch {
            start: 23, // Adjusted for \r\n
            end: 38,
            value: "test-key-abcde".to_string(),
            line_number: 2,
            column_start: 7,
            column_end: 22,
        };
        
        let secret_match = Secret::create_match(&file_path, &pattern, &regex_match, windows_content).unwrap();
        assert_eq!(secret_match.line_content, "Secure test-key-abcde here");
    }
}