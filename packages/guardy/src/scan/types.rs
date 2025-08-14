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
    Debug, Clone, PartialEq, clap::ValueEnum, serde::Serialize, serde::Deserialize, Default,
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

/// Configuration for the scanner
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ScannerConfig {
    pub enable_entropy_analysis: bool,
    pub min_entropy_threshold: f64,
    pub follow_symlinks: bool,
    pub max_file_size_mb: usize,
    pub include_binary: bool,
    pub ignore_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub ignore_comments: Vec<String>,
    // Processing mode settings
    pub mode: ScanMode,
    pub max_threads: usize,
    pub thread_percentage: u8,
    pub min_files_for_parallel: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enable_entropy_analysis: true,
            min_entropy_threshold: 1.0 / 1e5,
            follow_symlinks: false,
            max_file_size_mb: 50,
            include_binary: false, // Skip binary files by default
            ignore_paths: vec![
                // File-specific patterns only - directories are handled by DirectoryHandler
                // Users can add custom patterns via config
            ],
            ignore_patterns: vec![
                "# TEST_SECRET:".to_string(),
                "DEMO_KEY_".to_string(),
                "FAKE_".to_string(),
            ],
            ignore_comments: vec![
                "guardy:ignore".to_string(),
                "guardy:ignore-line".to_string(),
                "guardy:ignore-next".to_string(),
            ],
            // Processing mode defaults
            mode: ScanMode::Auto,
            max_threads: 0, // 0 = auto-detect
            thread_percentage: 75,
            min_files_for_parallel: 50,
        }
    }
}

/// Main scanner struct - handles secret detection across files and directories
///
/// NOTE: All scanner-related types should be defined in types.rs, not here.
/// This keeps the type definitions modular and the implementation focused.
#[derive(Clone)]
pub struct Scanner {
    pub(crate) patterns: super::patterns::SecretPatterns,
    pub(crate) config: ScannerConfig,
    /// Cached filters for performance (created once, reused everywhere)
    pub(crate) binary_filter: std::sync::Arc<super::filters::directory::BinaryFilter>,
    pub(crate) path_filter: std::sync::Arc<super::filters::directory::PathFilter>,
    pub(crate) size_filter: std::sync::Arc<super::filters::directory::SizeFilter>,
}
