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
    pub category: WarningCategory,
}

/// Categories of warnings that can occur during scanning
#[derive(Debug)]
pub enum WarningCategory {
    GitignoreMismatch,
    BinaryFileSkipped,
    PermissionDenied,
    UnknownFileType,
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
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enable_entropy_analysis: true,
            min_entropy_threshold: 1.0 / 1e5,
            skip_binary_files: true,
            follow_symlinks: false,
            max_file_size_mb: 10,
        }
    }
}