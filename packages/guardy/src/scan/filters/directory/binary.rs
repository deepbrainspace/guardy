use crate::scan::types::ScannerConfig;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, LazyLock};

/// Binary Filter - Binary file detection for directory-level filtering
///
/// Responsibilities:
/// - Two-stage binary file detection (extension + content inspection)
/// - Apply include_binary configuration for binary file handling
/// - Fast extension-based filtering using HashSet O(1) lookup
/// - Content inspection only for unknown extensions
/// - Zero-copy sharing of binary extensions across all threads
///
/// This filter is applied at the directory traversal stage BEFORE file content
/// is loaded, providing fast filtering to reduce unnecessary regex processing
/// on binary files.
///
/// Performance Optimizations:
/// - Stage 1: O(1) HashSet lookup for known binary extensions (95% of cases)
/// - Stage 2: content_inspector crate for unknown extensions only
/// - Shared binary extensions set via Arc<LazyLock> for zero-copy access
/// - Prevents regex processing on binary content

/// Global shared binary extensions cache - compiled once, shared across all threads
///
/// This provides significant performance benefits for binary file detection:
/// - HashSet compiled only once per program execution
/// - All threads share the same extension set via Arc (zero-copy sharing)
/// - LazyLock ensures thread-safe initialization on first access
/// - Extension checks are O(1) lookups instead of linear searches
/// - Loads both default extensions and custom extensions from configuration
static STATIC_BINARY_EXTENSIONS: LazyLock<Arc<HashSet<String>>> = LazyLock::new(|| {
    tracing::debug!("Initializing shared binary extensions HashSet - loading default and custom extensions");
    let start_time = std::time::Instant::now();

    // Step 1: Load default binary extensions (always available)
    let default_extensions = vec![
        // Images
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "tiff",
        "tif", "avif", "heic", "heif", "dng", "raw", "nef", "cr2",
        "arw", "orf", "rw2",
        // Documents
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt",
        "ods", "odp", "indd",
        // Archives
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "dmg",
        "iso", "ace", "cab", "lzh", "arj", "br", "zst", "lz4",
        "lzo", "lzma",
        // Executables & Object Files
        "exe", "dll", "so", "dylib", "bin", "app", "deb", "rpm",
        "o", "obj", "lib", "a", "pdb", "exp", "ilk",
        // Audio/Video
        "mp3", "wav", "ogg", "flac", "aac", "mp4", "avi", "mkv",
        "mov", "wmv", "webm", "mp2", "m4a", "wma", "amr",
        // Fonts
        "ttf", "otf", "woff", "woff2", "eot",
        // Security/Crypto (keeping PEM for secret detection)
        "gpg", "pgp", "p12", "pfx", "der", "crt", "keystore",
        // Database & Data Files
        "db", "sqlite", "sqlite3", "mdb", "sst", "ldb", "wal", "snap",
        "dat", "sas7bdat", "sas7bcat",
        // CAD & Design Files
        "dwg", "dxf", "skp", "3ds", "max", "blend", "fbx",
        // Compiler & Build Artifacts
        "gcno", "gcda", "gcov", "wasm", "webc",
        // Binary Data & Image Files
        "img", "vhd", "vmdk", "qcow2",
        // Other binary formats
        "pyc", "pyo", "class", "jar", "war", "ear", "swf", "fla",
        // NX cache files
        "nxt",
        // Common DOS/Legacy executables
        "com", "bat", "cmd",
        // Specialized formats that are definitely binary
        "bas", "pic", "b", "mcw", "ind", "dsk", "z",
        // Test data and specialized formats that often cause UTF-8 issues
        "gdiff", "srt", "zeno", "cba", "parquet", "avro", "orc",
        // Additional problematic formats discovered in scans
        "pak", "rpak", "toast", "data",
    ];

    // Step 2: Try to load custom binary extensions (optional, may fail)
    let custom_extensions = match load_custom_binary_extensions() {
        Ok(extensions) => {
            if !extensions.is_empty() {
                tracing::info!("Loaded {} custom binary extensions", extensions.len());
                extensions
            } else {
                Vec::new()
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load custom binary extensions (default extensions still available): {}", e);
            Vec::new()
        }
    };

    // Step 3: Combine default and custom extensions into HashSet
    let mut all_extensions = HashSet::new();

    // Add default extensions
    for ext in default_extensions {
        all_extensions.insert(ext.to_string());
    }

    // Add custom extensions
    for ext in custom_extensions {
        all_extensions.insert(ext);
    }

    let total_count = all_extensions.len();
    let default_count = default_extensions.len();
    let custom_count = total_count - default_count;
    let duration = start_time.elapsed();

    tracing::info!("Compiled {} total binary extensions ({} default + {} custom) in {:?} - now cached for all threads",
                  total_count, default_count, custom_count, duration);

    Arc::new(all_extensions)
});

/// Load custom binary extensions at runtime (used by LazyLock initialization)
fn load_custom_binary_extensions() -> Result<Vec<String>> {
    // TODO: Implement custom binary extension loading from runtime config
    // This would check for:
    // - ~/.config/guardy/binary_extensions.txt
    // - --binary-extensions CLI argument (if available in global config)
    // - Environment variables for custom binary extension lists
    // - guardy.yaml binary_extensions section

    let extensions = Vec::new();
    tracing::debug!("Custom binary extensions not yet implemented");
    Ok(extensions)
}

/// Binary file detection statistics for debugging and analysis
#[derive(Debug, Clone)]
pub struct BinaryFilterStats {
    pub files_checked: usize,
    pub files_binary_by_extension: usize,
    pub files_binary_by_content: usize,
    pub files_text_confirmed: usize,
    pub content_inspections_performed: usize,
    pub extension_cache_hits: usize,
}

impl Default for BinaryFilterStats {
    fn default() -> Self {
        Self {
            files_checked: 0,
            files_binary_by_extension: 0,
            files_binary_by_content: 0,
            files_text_confirmed: 0,
            content_inspections_performed: 0,
            extension_cache_hits: 0,
        }
    }
}

/// Binary filter for directory-level binary file detection
pub struct BinaryFilter {
    /// Whether to include binary files in scanning or filter them out
    include_binary: bool,
    /// Statistics collection for debugging and performance analysis
    stats: std::sync::Mutex<BinaryFilterStats>,
}

impl BinaryFilter {
    /// Create a new binary filter with configuration
    ///
    /// # Arguments
    /// * `config` - Scanner configuration with binary handling options
    ///
    /// # Returns
    /// A configured binary filter ready for use
    pub fn new(config: &ScannerConfig) -> Result<Self> {
        tracing::debug!("Binary filter initialized: include_binary={}", config.include_binary);

        Ok(Self {
            include_binary: config.include_binary,
            stats: std::sync::Mutex::new(BinaryFilterStats::default()),
        })
    }

    /// Get shared binary extensions (includes both default + custom)
    ///
    /// Returns the globally shared HashSet containing all binary extensions.
    /// This is zero-copy - just increments the Arc reference count.
    pub fn get_extensions() -> Arc<HashSet<String>> {
        STATIC_BINARY_EXTENSIONS.clone()
    }

    /// Check if a file should be filtered out due to being binary
    ///
    /// Uses two-stage detection for optimal performance:
    /// 1. Extension check (O(1) HashSet lookup) - handles ~95% of cases
    /// 2. Content inspection (reads first 512 bytes) - only for unknown extensions
    ///
    /// # Arguments
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    /// * `Ok(true)` - File should be filtered out (is binary and include_binary=false)
    /// * `Ok(false)` - File should be processed (is text or include_binary=true)
    /// * `Err(_)` - Error during binary detection
    ///
    /// # Performance
    /// - Stage 1: Extension check only (no I/O) - ~95% of binary files caught here
    /// - Stage 2: Content inspection only when needed - reads first 512 bytes only
    pub fn should_filter(&self, path: &Path) -> Result<bool> {
        let is_binary = self.is_binary_file(path)?;

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.files_checked += 1;
        }

        // If file is binary and we're not including binary files, filter it out
        let should_filter = is_binary && !self.include_binary;

        if should_filter {
            tracing::debug!("File filtered as binary: {}", path.display());
        } else if is_binary && self.include_binary {
            tracing::debug!("Binary file included in scan: {}", path.display());
        }

        Ok(should_filter)
    }

    /// Determine if a file is binary using two-stage detection
    ///
    /// # Stage 1: Extension Check (O(1) HashSet lookup)
    /// - Check if file extension is in the known binary extensions set
    /// - Handles ~95% of binary files instantly with no I/O
    /// - Uses shared STATIC_BINARY_EXTENSIONS for zero-copy access
    ///
    /// # Stage 2: Content Inspection (only for unknown extensions)
    /// - Uses content_inspector crate to read first 512 bytes
    /// - Detects binary content even without proper extension
    /// - Only performed when extension check is inconclusive
    ///
    /// # Arguments
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    /// * `Ok(true)` - File is binary
    /// * `Ok(false)` - File is text/should be scanned
    /// * `Err(_)` - Error during detection
    pub fn is_binary_file(&self, path: &Path) -> Result<bool> {
        // Stage 1: Fast extension check (O(1) lookup)
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();

                if STATIC_BINARY_EXTENSIONS.contains(&ext_lower) {
                    // Update statistics
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.extension_cache_hits += 1;
                        stats.files_binary_by_extension += 1;
                    }

                    tracing::trace!("File detected as binary by extension '{}': {}", ext_lower, path.display());
                    return Ok(true);
                }
            }
        }

        // Stage 2: Content inspection for unknown extensions
        // Only performed when extension check doesn't give a definitive binary result
        tracing::trace!("Performing content inspection for unknown extension: {}", path.display());

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.content_inspections_performed += 1;
        }

        let is_binary = match content_inspector::inspect(path) {
            Ok(content_type) => match content_type {
                content_inspector::ContentType::BINARY => {
                    tracing::trace!("File detected as binary by content inspection: {}", path.display());

                    // Update statistics
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.files_binary_by_content += 1;
                    }

                    true
                }
                content_inspector::ContentType::UTF_8 | content_inspector::ContentType::UTF_8_BOM => {
                    tracing::trace!("File confirmed as text by content inspection: {}", path.display());

                    // Update statistics
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.files_text_confirmed += 1;
                    }

                    false
                }
            }
            Err(e) => {
                tracing::warn!("Failed to inspect file content for {}: {}", path.display(), e);
                // When in doubt, assume it's text and let the scanner handle errors
                false
            }
        };

        Ok(is_binary)
    }

    /// Filter a list of paths, removing binary files (if include_binary=false)
    ///
    /// # Arguments
    /// * `paths` - List of paths to filter
    ///
    /// # Returns
    /// Vector of paths that should be processed (non-binary or include_binary=true)
    pub fn filter_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<&P> {
        paths
            .iter()
            .filter(|path| {
                match self.should_filter(path.as_ref()) {
                    Ok(should_filter) => !should_filter,
                    Err(e) => {
                        tracing::warn!("Error checking binary status for {}: {}",
                                     path.as_ref().display(), e);
                        true // Include files we can't check (let scanner handle errors)
                    }
                }
            })
            .collect()
    }

    /// Get current filter statistics
    ///
    /// # Returns
    /// Statistics about files processed by this filter
    pub fn get_stats(&self) -> BinaryFilterStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = BinaryFilterStats::default();
        }
    }

    /// Get configuration information for debugging
    ///
    /// # Returns
    /// Tuple of (include_binary, total_extensions_count)
    pub fn get_config_info(&self) -> (bool, usize) {
        (self.include_binary, STATIC_BINARY_EXTENSIONS.len())
    }
}

use anyhow::Context;

