/// Represents a detected secret match in a file
#[derive(Debug, Clone)]
pub struct SecretMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub matched_text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub secret_type: String,
    pub pattern_description: String,
}

/// Statistics from a scanning operation
#[derive(Debug, Default)]
pub struct ScanStats {
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub total_matches: usize,
    pub scan_duration_ms: u64,
}

/// Warning generated during scanning
#[derive(Debug)]
pub struct Warning {
    pub message: String,
}

/// Result from scanning a single file (used in parallel processing)
#[derive(Debug)]
pub struct ScanFileResult {
    pub matches: Vec<SecretMatch>,
    pub file_path: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of a scanning operation
#[derive(Debug)]
pub struct ScanResult {
    pub matches: Vec<SecretMatch>,
    pub stats: ScanStats,
    pub warnings: Vec<Warning>,
}

/// Scanning mode for determining parallelization strategy
#[derive(
    Debug, Clone, Copy, PartialEq, clap::ValueEnum, serde::Serialize, serde::Deserialize, Default,
)]
pub enum ScanMode {
    /// Always use sequential processing
    Sequential,
    /// Always use parallel processing
    Parallel,
    /// Automatically choose based on file count (smart default)
    #[default]
    Auto,
}

// ScannerConfig removed - use config::core::ScannerConfig directly from GUARDY_CONFIG
// This eliminates duplication and ensures single source of truth
use std::sync::Arc;

/// Main scanner struct - handles secret detection across files and directories
///
/// All configuration is accessed directly from GUARDY_CONFIG for simplicity and efficiency.
/// Filters are cached for performance (created once, reused everywhere).
#[derive(Clone)]
pub struct Scanner {
    /// Cached filters for performance (created once, reused everywhere)
    pub(crate) binary_filter: std::sync::Arc<super::filters::directory::BinaryFilter>,
    pub(crate) path_filter: std::sync::Arc<super::filters::directory::PathFilter>,
    pub(crate) size_filter: std::sync::Arc<super::filters::directory::SizeFilter>,
    /// Content filters for secret detection optimization
    pub(crate) prefilter: std::sync::Arc<super::filters::content::ContextPrefilter>,
    pub(crate) regex_executor: std::sync::Arc<super::filters::content::RegexExecutor>,
    pub(crate) comment_filter: std::sync::Arc<super::filters::content::CommentFilter>,
}
