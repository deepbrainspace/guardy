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

