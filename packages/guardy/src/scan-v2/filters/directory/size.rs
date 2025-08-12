use crate::scan::types::ScannerConfig;
use anyhow::Result;
use std::path::Path;

/// Size Filter - File size validation for directory-level filtering
///
/// Responsibilities:
/// - Apply file size limits to prevent processing huge files
/// - Provide configurable size thresholds (max_file_size_mb)
/// - Fast file size checks before content loading
/// - Integration with streaming thresholds for large file handling
///
/// This filter is applied at the directory traversal stage BEFORE file content
/// is loaded, providing fast filtering to reduce memory usage and I/O operations.
///
/// Performance Benefits:
/// - Prevents loading large files into memory unnecessarily
/// - Uses metadata check (no file I/O) for instant validation
/// - Configurable limits for different scanning scenarios

/// File size statistics for debugging and analysis
#[derive(Debug, Clone)]
pub struct SizeFilterStats {
    pub files_checked: usize,
    pub files_oversized: usize,
    pub total_bytes_saved: u64,
    pub max_file_size_bytes: u64,
    pub streaming_threshold_bytes: u64,
}

impl Default for SizeFilterStats {
    fn default() -> Self {
        Self {
            files_checked: 0,
            files_oversized: 0,
            total_bytes_saved: 0,
            max_file_size_bytes: 0,
            streaming_threshold_bytes: 0,
        }
    }
}

/// Size filter for directory-level file size validation
pub struct SizeFilter {
    /// Maximum file size in bytes (converted from MB config)
    max_file_size_bytes: u64,
    /// Streaming threshold in bytes (files above this size use streaming)
    streaming_threshold_bytes: u64,
    /// Statistics collection for debugging and performance analysis
    stats: std::sync::Mutex<SizeFilterStats>,
}

impl SizeFilter {
    /// Create a new size filter with configuration
    ///
    /// # Arguments
    /// * `config` - Scanner configuration with size limits in MB
    ///
    /// # Returns
    /// A configured size filter ready for use
    pub fn new(config: &ScannerConfig) -> Result<Self> {
        let max_file_size_bytes = (config.max_file_size_mb as u64) * 1024 * 1024;
        let streaming_threshold_bytes = (config.streaming_threshold_mb as u64) * 1024 * 1024;

        let mut stats = SizeFilterStats::default();
        stats.max_file_size_bytes = max_file_size_bytes;
        stats.streaming_threshold_bytes = streaming_threshold_bytes;

        tracing::debug!(
            "Size filter initialized: max_file_size={}MB ({}bytes), streaming_threshold={}MB ({}bytes)",
            config.max_file_size_mb,
            max_file_size_bytes,
            config.streaming_threshold_mb,
            streaming_threshold_bytes
        );

        Ok(Self {
            max_file_size_bytes,
            streaming_threshold_bytes,
            stats: std::sync::Mutex::new(stats),
        })
    }

    /// Check if a file should be filtered out due to size limits
    ///
    /// # Arguments
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    /// * `Ok(true)` - File should be filtered out (too large)
    /// * `Ok(false)` - File passes size check
    /// * `Err(_)` - Error accessing file metadata
    ///
    /// # Performance
    /// Uses filesystem metadata only - no file I/O required
    pub fn should_filter(&self, path: &Path) -> Result<bool> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for file: {}", path.display()))?;

        let file_size = metadata.len();
        let should_filter = file_size > self.max_file_size_bytes;

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.files_checked += 1;
            if should_filter {
                stats.files_oversized += 1;
                stats.total_bytes_saved += file_size;
            }
        }

        if should_filter {
            tracing::debug!(
                "File filtered due to size: {} ({} bytes > {} bytes limit)",
                path.display(),
                file_size,
                self.max_file_size_bytes
            );
        }

        Ok(should_filter)
    }

    /// Check if a file should use streaming processing
    ///
    /// Files larger than the streaming threshold should be processed using
    /// streaming to avoid loading the entire file into memory at once.
    ///
    /// # Arguments
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    /// * `Ok(true)` - File should use streaming processing
    /// * `Ok(false)` - File can be loaded fully into memory
    /// * `Err(_)` - Error accessing file metadata
    pub fn should_use_streaming(&self, path: &Path) -> Result<bool> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for file: {}", path.display()))?;

        let file_size = metadata.len();
        let use_streaming = file_size > self.streaming_threshold_bytes;

        if use_streaming {
            tracing::trace!(
                "File will use streaming: {} ({} bytes > {} bytes threshold)",
                path.display(),
                file_size,
                self.streaming_threshold_bytes
            );
        }

        Ok(use_streaming)
    }

    /// Get file size in bytes for a given path
    ///
    /// # Arguments
    /// * `path` - Path to the file
    ///
    /// # Returns
    /// File size in bytes, or error if file cannot be accessed
    pub fn get_file_size(&self, path: &Path) -> Result<u64> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for file: {}", path.display()))?;

        Ok(metadata.len())
    }

    /// Get file size in MB for human-readable display
    ///
    /// # Arguments
    /// * `path` - Path to the file
    ///
    /// # Returns
    /// File size in megabytes (MB), rounded to 2 decimal places
    pub fn get_file_size_mb(&self, path: &Path) -> Result<f64> {
        let size_bytes = self.get_file_size(path)?;
        Ok(size_bytes as f64 / (1024.0 * 1024.0))
    }

    /// Filter a list of paths, removing those that exceed size limits
    ///
    /// # Arguments
    /// * `paths` - List of paths to filter
    ///
    /// # Returns
    /// Vector of paths that pass size validation
    pub fn filter_paths<'a, P: AsRef<Path>>(&self, paths: &'a [P]) -> Vec<&'a P> {
        paths
            .iter()
            .filter(|path| {
                match self.should_filter(path.as_ref()) {
                    Ok(should_filter) => !should_filter,
                    Err(e) => {
                        tracing::warn!("Error checking file size for {}: {}",
                                     path.as_ref().display(), e);
                        false // Filter out files we can't check
                    }
                }
            })
            .collect()
    }

    /// Get current filter statistics
    ///
    /// # Returns
    /// Statistics about files processed by this filter
    pub fn get_stats(&self) -> SizeFilterStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.files_checked = 0;
            stats.files_oversized = 0;
            stats.total_bytes_saved = 0;
            // Keep configuration values (max_file_size_bytes, streaming_threshold_bytes)
        }
    }

    /// Get human-readable size limit information
    ///
    /// # Returns
    /// Tuple of (max_size_mb, streaming_threshold_mb) for display purposes
    pub fn get_limits_mb(&self) -> (f64, f64) {
        let max_size_mb = self.max_file_size_bytes as f64 / (1024.0 * 1024.0);
        let streaming_mb = self.streaming_threshold_bytes as f64 / (1024.0 * 1024.0);
        (max_size_mb, streaming_mb)
    }
}

use anyhow::Context;

