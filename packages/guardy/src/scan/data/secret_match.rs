//! Secret match data structure with memory optimization

use super::Coordinate;
use std::sync::Arc;


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
}

impl SecretMatch {
    /// Create a new SecretMatch
    pub fn new(
        file_path: Arc<str>,
        coordinate: Coordinate,
        matched_text: String,
        secret_type: Arc<str>,
        pattern_description: Arc<str>,
    ) -> Self {
        Self {
            location: super::FileSpan::new(file_path, coordinate),
            matched_text,
            secret_type,
            pattern_description,
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
    
    /// Get a fully redacted version for high-security contexts (logs, console output)
    pub fn redacted_match_secure(&self) -> String {
        use crate::scan::reports::utils::redact_secret_with_style;
        use crate::scan::reports::RedactionStyle;
        
        redact_secret_with_style(&self.matched_text, RedactionStyle::Full)
    }
}