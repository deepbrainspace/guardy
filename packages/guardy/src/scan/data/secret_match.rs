//! Secret match data structure with memory optimization

use super::Coordinate;
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
/// - Use Coordinate struct (16 bytes) instead of storing content
/// - Use Arc<str> for strings shared across matches
/// - Store only the matched value, not the entire line
#[derive(Debug, Clone)]
pub struct SecretMatch {
    /// Position in the file (16 bytes, includes file path via Arc)
    pub location: super::FileSpan,
    
    /// The matched secret text (just the secret, not the whole line)
    pub matched_text: String,
    
    /// Type of secret (e.g., "AWS Access Key", "GitHub Token")
    /// Shared from PatternLibrary via Arc
    pub secret_type: Arc<str>,
    
    /// Pattern description for user display
    /// Shared from PatternLibrary via Arc
    pub pattern_description: Arc<str>,
    
    /// Severity of the match
    pub severity: MatchSeverity,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

impl SecretMatch {
    /// Create a new SecretMatch
    pub fn new(
        file_path: Arc<str>,
        coordinate: Coordinate,
        matched_text: String,
        secret_type: Arc<str>,
        pattern_description: Arc<str>,
        severity: MatchSeverity,
        confidence: f32,
    ) -> Self {
        Self {
            location: super::FileSpan::new(file_path, coordinate),
            matched_text,
            secret_type,
            pattern_description,
            severity,
            confidence,
        }
    }
    
    /// Get the file path
    pub fn file_path(&self) -> &str {
        &self.location.file_path
    }
    
    /// Get the line number
    pub fn line_number(&self) -> u32 {
        self.location.coordinate.line
    }
    
    /// Get the coordinate
    pub fn coordinate(&self) -> &Coordinate {
        &self.location.coordinate
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