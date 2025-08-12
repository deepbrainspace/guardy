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

/// Configuration for the optimized scanner (scan2)
///
/// Updated with modern defaults:
/// - 50MB max file size (was 10MB) for modern development with large bundle files
/// - 20MB streaming threshold for better memory management
/// - Performance-first parallel processing
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct ScannerConfig {
    // Core scanning options
    pub enable_entropy_analysis: bool,
    pub min_entropy_threshold: f64,
    pub follow_symlinks: bool,
    pub max_file_size_mb: usize,        // Updated: 50MB default (was 10MB)
    pub streaming_threshold_mb: usize,  // New: 20MB default (was hardcoded 5MB)
    pub include_binary: bool,

    // Ignore system configuration
    pub ignore_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub binary_extensions: Vec<String>,
    pub ignore_comments: Vec<String>,
    pub ignore_test_code: bool,
    pub test_attributes: Vec<String>,
    pub test_modules: Vec<String>,

    // Parallel processing configuration
    pub mode: ScanMode,
    pub max_threads: usize,             // 0 = no hard limit, use percentage calculation
    pub thread_percentage: u8,          // Default: 75% of available CPU cores
    pub min_files_for_parallel: usize,  // Default: 5 files (lower threshold for I/O-bound scanning)

    // New optimization options for scan2
    pub enable_keyword_prefilter: bool,    // Default: true
    pub pattern_classification: bool,      // Default: true
    pub prefilter_threshold: f32,          // Default: 0.1
    pub max_multiline_size: usize,         // Default: 1MB
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            // Core scanning options
            enable_entropy_analysis: true,
            min_entropy_threshold: 1.0 / 1e5,
            follow_symlinks: false,
            max_file_size_mb: 50,           // Modern default (was 10MB)
            streaming_threshold_mb: 20,     // Modern default (was hardcoded 5MB)
            include_binary: false,

            // Ignore system configuration
            ignore_paths: vec![
                "tests/*".to_string(),
                "testdata/*".to_string(),
                "*_test.rs".to_string(),
                "test_*.rs".to_string(),
                // Git objects and internal files (binary data)
                ".git/objects/**".to_string(),
                ".git_disabled/**".to_string(), // All of git_disabled is safe to skip
                ".git/refs/**".to_string(),
                ".git/logs/**".to_string(),
                ".git/index".to_string(),          // Git index file (binary)
                "**/.git/objects/**".to_string(),  // Match .git/objects anywhere in path
                "**/.git_disabled/**".to_string(), // Match .git_disabled anywhere in path
            ],
            ignore_patterns: vec![
                "# TEST_SECRET:".to_string(),
                "DEMO_KEY_".to_string(),
                "FAKE_".to_string(),
            ],
            binary_extensions: vec![
                // Images
                "png".to_string(), "jpg".to_string(), "jpeg".to_string(), "gif".to_string(),
                "bmp".to_string(), "ico".to_string(), "webp".to_string(), "tiff".to_string(),
                "tif".to_string(), "avif".to_string(), "heic".to_string(), "heif".to_string(),
                "dng".to_string(), "raw".to_string(), "nef".to_string(), "cr2".to_string(),
                "arw".to_string(), "orf".to_string(), "rw2".to_string(),
                // Documents
                "pdf".to_string(), "doc".to_string(), "docx".to_string(), "xls".to_string(),
                "xlsx".to_string(), "ppt".to_string(), "pptx".to_string(), "odt".to_string(),
                "ods".to_string(), "odp".to_string(), "indd".to_string(),
                // Archives
                "zip".to_string(), "tar".to_string(), "gz".to_string(), "bz2".to_string(),
                "xz".to_string(), "7z".to_string(), "rar".to_string(), "dmg".to_string(),
                "iso".to_string(), "ace".to_string(), "cab".to_string(), "lzh".to_string(),
                "arj".to_string(), "br".to_string(), "zst".to_string(), "lz4".to_string(),
                "lzo".to_string(), "lzma".to_string(),
                // Executables & Object Files
                "exe".to_string(), "dll".to_string(), "so".to_string(), "dylib".to_string(),
                "bin".to_string(), "app".to_string(), "deb".to_string(), "rpm".to_string(),
                "o".to_string(), "obj".to_string(), "lib".to_string(), "a".to_string(),
                "pdb".to_string(), "exp".to_string(), "ilk".to_string(),
                // Audio/Video
                "mp3".to_string(), "wav".to_string(), "ogg".to_string(), "flac".to_string(),
                "aac".to_string(), "mp4".to_string(), "avi".to_string(), "mkv".to_string(),
                "mov".to_string(), "wmv".to_string(), "webm".to_string(), "mp2".to_string(),
                "m4a".to_string(), "wma".to_string(), "amr".to_string(),
                // Fonts
                "ttf".to_string(), "otf".to_string(), "woff".to_string(), "woff2".to_string(),
                "eot".to_string(),
                // Security/Crypto (keeping PEM for secret detection)
                "gpg".to_string(), "pgp".to_string(), "p12".to_string(), "pfx".to_string(),
                "der".to_string(), "crt".to_string(), "keystore".to_string(),
                // Database & Data Files
                "db".to_string(), "sqlite".to_string(), "sqlite3".to_string(), "mdb".to_string(),
                "sst".to_string(), "ldb".to_string(), "wal".to_string(), "snap".to_string(),
                "dat".to_string(), "sas7bdat".to_string(), "sas7bcat".to_string(),
                // CAD & Design Files
                "dwg".to_string(), "dxf".to_string(), "skp".to_string(), "3ds".to_string(),
                "max".to_string(), "blend".to_string(), "fbx".to_string(),
                // Compiler & Build Artifacts
                "gcno".to_string(), "gcda".to_string(), "gcov".to_string(), "wasm".to_string(),
                "webc".to_string(),
                // Binary Data & Image Files
                "img".to_string(), "vhd".to_string(), "vmdk".to_string(), "qcow2".to_string(),
                // Other binary formats
                "pyc".to_string(), "pyo".to_string(), "class".to_string(), "jar".to_string(),
                "war".to_string(), "ear".to_string(), "swf".to_string(), "fla".to_string(),
                // NX cache files
                "nxt".to_string(),
                // Common DOS/Legacy executables
                "com".to_string(), "bat".to_string(), "cmd".to_string(),
                // Specialized formats that are definitely binary
                "bas".to_string(), "pic".to_string(), "b".to_string(), "mcw".to_string(),
                "ind".to_string(), "dsk".to_string(), "z".to_string(),
                // Test data and specialized formats that often cause UTF-8 issues
                "gdiff".to_string(), "srt".to_string(), "zeno".to_string(), "cba".to_string(),
                "parquet".to_string(), "avro".to_string(), "orc".to_string(),
                // Additional problematic formats discovered in scans
                "pak".to_string(), "rpak".to_string(), "toast".to_string(), "data".to_string(),
            ],
            ignore_comments: vec![
                "guardy:ignore".to_string(),
                "guardy:ignore-line".to_string(),
                "guardy:ignore-next".to_string(),
            ],
            ignore_test_code: true,
            test_attributes: vec![],
            test_modules: vec![],

            // Parallel processing defaults (performance-first approach)
            mode: ScanMode::Auto,
            max_threads: 0,                 // No hard limit - use percentage calculation
            thread_percentage: 75,          // Use 75% of available CPU cores
            min_files_for_parallel: 5,      // Lower threshold for I/O-bound scanning (was 50)

            // New optimization defaults for scan2
            enable_keyword_prefilter: true,
            pattern_classification: true,
            prefilter_threshold: 0.1,
            max_multiline_size: 1024 * 1024,  // 1MB
        }
    }
}

/// A secret detection pattern with regex and metadata
///
/// Each pattern represents a specific type of secret that can be detected,
/// including API keys, private keys, database credentials, and more.
#[derive(Debug, Clone)]
pub struct SecretPattern {
    /// Human-readable name for the pattern (e.g., "GitHub Token")
    pub name: String,
    /// Compiled regex for pattern matching
    pub regex: regex::Regex,
    /// Detailed description of what this pattern detects
    pub description: String,
}

/// Collection of secret detection patterns
///
/// Contains 40+ built-in patterns for comprehensive secret detection including:
/// - **Private keys**: SSH, PGP, RSA, EC, OpenSSH, PuTTY, Age encryption keys
/// - **API keys**: OpenAI, GitHub, AWS, Azure, Google Cloud, and 20+ more services
/// - **Database credentials**: MongoDB, PostgreSQL, MySQL connection strings
/// - **Generic detection**: Context-based patterns for unknown secrets
#[derive(Debug, Clone)]
pub struct SecretPatterns {
    /// Vector of all loaded patterns (built-in + custom)
    pub patterns: Vec<SecretPattern>,
}