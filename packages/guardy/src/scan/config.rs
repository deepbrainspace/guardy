//! Scanner configuration

use serde::{Deserialize, Serialize};

/// Configuration for the scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScannerConfig {
    // Core options
    pub max_file_size_mb: usize,
    pub follow_symlinks: bool,
    
    // Entropy analysis
    pub enable_entropy_analysis: bool,
    pub min_entropy_threshold: f64,
    
    // Path filtering
    pub ignore_paths: Vec<String>,
    
    // Binary filtering
    pub binary_extensions: Vec<String>,
    pub skip_binary_files: bool,
    
    // Comment filtering
    pub respect_ignore_comments: bool,
    
    // Parallel processing (rayon)
    pub max_threads: Option<usize>, // None = use rayon defaults
    pub min_files_for_parallel: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: 50,
            follow_symlinks: false,
            enable_entropy_analysis: true,
            min_entropy_threshold: 4.5,
            ignore_paths: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            binary_extensions: vec![
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "bin".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
            ],
            skip_binary_files: true,
            respect_ignore_comments: true,
            max_threads: None,
            min_files_for_parallel: 5,
        }
    }
}