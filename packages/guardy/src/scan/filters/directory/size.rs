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
    pub fn filter_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<&P> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    fn create_test_config(max_mb: usize, streaming_mb: usize) -> ScannerConfig {
        ScannerConfig {
            max_file_size_mb: max_mb,
            streaming_threshold_mb: streaming_mb,
            ..ScannerConfig::default()
        }
    }
    
    #[test]
    fn test_size_filter_creation() {
        let config = create_test_config(50, 20);
        let filter = SizeFilter::new(&config).unwrap();
        
        let (max_mb, streaming_mb) = filter.get_limits_mb();
        assert_eq!(max_mb, 50.0);
        assert_eq!(streaming_mb, 20.0);
    }
    
    #[test]
    fn test_should_filter_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let small_file = temp_dir.path().join("small.txt");
        
        // Create a small file (1KB)
        let content = "x".repeat(1024);
        fs::write(&small_file, content).unwrap();
        
        let config = create_test_config(1, 1); // 1MB limit
        let filter = SizeFilter::new(&config).unwrap();
        
        assert!(!filter.should_filter(&small_file).unwrap());
    }
    
    #[test]  
    fn test_should_filter_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.txt");
        
        // Create a file larger than 1MB
        let content = "x".repeat(2 * 1024 * 1024); // 2MB
        fs::write(&large_file, content).unwrap();
        
        let config = create_test_config(1, 1); // 1MB limit  
        let filter = SizeFilter::new(&config).unwrap();
        
        assert!(filter.should_filter(&large_file).unwrap());
    }
    
    #[test]
    fn test_streaming_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let medium_file = temp_dir.path().join("medium.txt");
        
        // Create a 15MB file  
        let content = "x".repeat(15 * 1024 * 1024);
        fs::write(&medium_file, content).unwrap();
        
        let config = create_test_config(50, 10); // 50MB max, 10MB streaming threshold
        let filter = SizeFilter::new(&config).unwrap();
        
        // Should not be filtered (under 50MB limit)
        assert!(!filter.should_filter(&medium_file).unwrap());
        
        // Should use streaming (over 10MB threshold)
        assert!(filter.should_use_streaming(&medium_file).unwrap());
    }
    
    #[test]
    fn test_get_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let content = "x".repeat(5000); // 5KB
        fs::write(&test_file, content).unwrap();
        
        let config = create_test_config(10, 5);
        let filter = SizeFilter::new(&config).unwrap();
        
        let size_bytes = filter.get_file_size(&test_file).unwrap();
        assert_eq!(size_bytes, 5000);
        
        let size_mb = filter.get_file_size_mb(&test_file).unwrap();
        assert!((size_mb - 0.00476).abs() < 0.001); // ~0.00476 MB
    }
    
    #[test]
    fn test_filter_paths() {
        let temp_dir = TempDir::new().unwrap();
        let small_file = temp_dir.path().join("small.txt");
        let large_file = temp_dir.path().join("large.txt");
        
        fs::write(&small_file, "small content").unwrap();
        fs::write(&large_file, "x".repeat(2 * 1024 * 1024)).unwrap(); // 2MB
        
        let config = create_test_config(1, 1); // 1MB limit
        let filter = SizeFilter::new(&config).unwrap();
        
        let paths = vec![&small_file, &large_file];
        let filtered = filter.filter_paths(&paths);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], &small_file);
    }
    
    #[test]
    fn test_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        
        fs::write(&file1, "small").unwrap();
        fs::write(&file2, "x".repeat(2 * 1024 * 1024)).unwrap(); // 2MB
        
        let config = create_test_config(1, 1); // 1MB limit
        let filter = SizeFilter::new(&config).unwrap();
        
        // Check both files
        let _ = filter.should_filter(&file1);
        let _ = filter.should_filter(&file2);
        
        let stats = filter.get_stats();
        assert_eq!(stats.files_checked, 2);
        assert_eq!(stats.files_oversized, 1);
        assert!(stats.total_bytes_saved > 0);
    }
    
    #[test]
    fn test_reset_stats() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        let config = create_test_config(10, 5);
        let filter = SizeFilter::new(&config).unwrap();
        
        // Generate some stats
        let _ = filter.should_filter(&test_file);
        assert!(filter.get_stats().files_checked > 0);
        
        // Reset stats
        filter.reset_stats();
        let stats = filter.get_stats();
        assert_eq!(stats.files_checked, 0);
        assert_eq!(stats.files_oversized, 0);
        assert_eq!(stats.total_bytes_saved, 0);
        
        // Configuration values should be preserved
        assert_eq!(stats.max_file_size_bytes, 10 * 1024 * 1024);
        assert_eq!(stats.streaming_threshold_bytes, 5 * 1024 * 1024);
    }
    
    #[test]
    fn test_nonexistent_file() {
        let config = create_test_config(10, 5);
        let filter = SizeFilter::new(&config).unwrap();
        
        let nonexistent = std::path::Path::new("/nonexistent/file.txt");
        let result = filter.should_filter(nonexistent);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to get metadata"));
    }
    
    #[test]
    fn test_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.txt");
        let exact_limit_file = temp_dir.path().join("exact.txt");
        
        // Empty file
        fs::write(&empty_file, "").unwrap();
        
        // File exactly at limit (1MB = 1,048,576 bytes)
        let content = "x".repeat(1024 * 1024);
        fs::write(&exact_limit_file, content).unwrap();
        
        let config = create_test_config(1, 1);
        let filter = SizeFilter::new(&config).unwrap();
        
        // Empty file should pass
        assert!(!filter.should_filter(&empty_file).unwrap());
        
        // Exact limit file should pass (not greater than limit)
        assert!(!filter.should_filter(&exact_limit_file).unwrap());
    }
    
    #[test]
    fn test_configuration_validation() {
        // Test various configuration combinations
        let configs = [
            (0, 0),   // Minimum values
            (1, 1),   // Small values
            (50, 20), // Default-like values
            (1000, 500), // Large values
        ];
        
        for (max_mb, streaming_mb) in configs {
            let config = create_test_config(max_mb, streaming_mb);
            let filter = SizeFilter::new(&config);
            assert!(filter.is_ok(), "Configuration ({}, {}) should be valid", max_mb, streaming_mb);
            
            let filter = filter.unwrap();
            let (actual_max, actual_streaming) = filter.get_limits_mb();
            assert_eq!(actual_max, max_mb as f64);
            assert_eq!(actual_streaming, streaming_mb as f64);
        }
    }
}