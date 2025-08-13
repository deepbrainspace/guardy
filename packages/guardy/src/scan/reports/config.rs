//! Reporting configuration and metadata

use std::time::SystemTime;

/// Report generation configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Include per-file timing details
    pub include_file_timing: bool,
    /// Include line content in reports (may contain sensitive data)
    pub include_line_content: bool,
    /// Display actual secrets (DANGEROUS - false by default)
    pub display_secrets: bool,
    /// How to redact secrets when display_secrets=false
    pub redaction_style: RedactionStyle,
    /// Maximum number of matches to include (0 = unlimited)
    pub max_matches: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_file_timing: true,
            include_line_content: false, // Safe by default
            display_secrets: false,      // Safe by default
            redaction_style: RedactionStyle::Partial,
            max_matches: 0,              // Unlimited by default
        }
    }
}

/// Secret redaction styles
#[derive(Debug, Clone, Copy)]
pub enum RedactionStyle {
    /// "**REDACTED**"
    Full,
    /// "sk-1234...cdef" (show first/last few chars)
    Partial,
    /// "*".repeat(actual_length)
    Length,
}

/// Report format types
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Json,
    Html,
}

impl ReportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::Html => "html",
        }
    }
}

/// Metadata about the report generation
#[derive(Debug)]
pub struct ReportMetadata {
    pub generated_at: SystemTime,
    pub guardy_version: &'static str,
    pub scan_duration_ms: u64,
}

impl ReportMetadata {
    pub fn new(scan_duration_ms: u64) -> Self {
        Self {
            generated_at: SystemTime::now(),
            guardy_version: env!("CARGO_PKG_VERSION"),
            scan_duration_ms,
        }
    }
}