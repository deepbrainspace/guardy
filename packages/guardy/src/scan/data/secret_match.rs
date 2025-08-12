//! Secret match data structure with memory optimization

use std::sync::Arc;

/// Severity level for a detected secret
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Represents a detected secret match in a file
/// 
/// Memory optimizations:
/// - Use Arc<str> for strings that might be shared across matches
/// - Store only essential data
/// - Lazy computation of display strings
#[derive(Debug, Clone)]
pub struct SecretMatch {
    /// File path (shared across all matches in same file)
    pub file_path: Arc<str>,
    
    /// Line number (1-indexed)
    pub line_number: usize,
    
    /// The actual line content (might be truncated for very long lines)
    pub line_content: String,
    
    /// The matched secret text
    pub matched_text: String,
    
    /// Start position in the line (0-indexed)
    pub start_pos: usize,
    
    /// End position in the line (0-indexed)
    pub end_pos: usize,
    
    /// Type of secret (e.g., "AWS Access Key", "GitHub Token")
    pub secret_type: Arc<str>,
    
    /// Pattern description for user display
    pub pattern_description: Arc<str>,
    
    /// Severity of the match
    pub severity: MatchSeverity,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

impl SecretMatch {
    /// Create a new SecretMatch with all required fields
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_path: Arc<str>,
        line_number: usize,
        line_content: String,
        matched_text: String,
        start_pos: usize,
        end_pos: usize,
        secret_type: Arc<str>,
        pattern_description: Arc<str>,
        severity: MatchSeverity,
        confidence: f32,
    ) -> Self {
        Self {
            file_path,
            line_number,
            line_content,
            matched_text,
            start_pos,
            end_pos,
            secret_type,
            pattern_description,
            severity,
            confidence,
        }
    }
    
    /// Get a truncated version of the line for display (max 200 chars)
    pub fn display_line(&self) -> &str {
        if self.line_content.len() > 200 {
            &self.line_content[..200]
        } else {
            &self.line_content
        }
    }
    
    /// Get a redacted version of the matched text for safe display
    pub fn redacted_match(&self) -> String {
        let len = self.matched_text.len();
        if len <= 8 {
            "*".repeat(len)
        } else {
            format!("{}...{}", 
                &self.matched_text[..3],
                &self.matched_text[len-3..])
        }
    }
}