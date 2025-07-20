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

/// Result of a scanning operation
#[derive(Debug)]
pub struct ScanResult {
    pub matches: Vec<SecretMatch>,
    pub stats: ScanStats,
    pub warnings: Vec<Warning>,
}

/// Configuration for the scanner
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub enable_entropy_analysis: bool,
    pub min_entropy_threshold: f64,
    pub skip_binary_files: bool,
    pub follow_symlinks: bool,
    pub max_file_size_mb: usize,
    pub ignore_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub ignore_comments: Vec<String>,
    pub ignore_test_code: bool,
    pub test_attributes: Vec<String>,
    pub test_modules: Vec<String>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enable_entropy_analysis: true,
            min_entropy_threshold: 1.0 / 1e5,
            skip_binary_files: true,
            follow_symlinks: false,
            max_file_size_mb: 10,
            ignore_paths: vec![
                "tests/*".to_string(),
                "testdata/*".to_string(),
                "*_test.rs".to_string(),
                "test_*.rs".to_string(),
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
            ignore_test_code: true,
            test_attributes: vec![],
            test_modules: vec![],
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
}

