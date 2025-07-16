//! Security features for Guardy
//!
//! This module provides secret detection, branch protection, and other
//! security-related functionality.

use anyhow::{Context, Result};
use regex::Regex;

pub mod patterns;
pub mod scanner;

#[cfg(test)]
mod tests;

pub use scanner::SecretScanner;

/// Security check result
/// TODO: Remove #[allow(dead_code)] when security features are used in Phase 1.3
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SecurityMatch {
    /// File path where secret was found
    pub file_path: String,

    /// Line number (1-based)
    pub line_number: usize,

    /// Column number (1-based)
    pub column: usize,

    /// The matched content
    pub content: String,

    /// Pattern name that matched
    pub pattern_name: String,

    /// Severity level
    pub severity: Severity,
}

/// Severity levels for security matches
///
/// We keep this simple - secrets are either critical security issues or informational warnings.
/// There's no middle ground with secrets - they either pose immediate risk or they don't.
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Critical: Confirmed secrets that pose immediate security risk
    Critical,
    /// Informational: Patterns that might be false positives but worth checking
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Security pattern definition
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SecurityPattern {
    /// Pattern name
    pub name: String,

    /// Regular expression
    pub regex: Regex,

    /// Severity level
    pub severity: Severity,

    /// Description
    pub description: String,
}

impl SecurityPattern {
    /// Create a new security pattern
    pub fn new(
        name: String,
        pattern: &str,
        severity: Severity,
        description: String,
    ) -> Result<Self> {
        let regex = Regex::new(pattern)
            .with_context(|| format!("Invalid regex pattern for {}: {}", name, pattern))?;

        Ok(Self {
            name,
            regex,
            severity,
            description,
        })
    }
}
