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
    pub max_cpu_percentage: u8,      // Percentage of CPUs to use (1-100)
    pub max_threads: Option<usize>,  // Override computed threads if set
    
    // Progress display
    pub show_progress: bool,          // Show progress bars (auto-detects TTY)
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
            max_cpu_percentage: 80,  // Use 80% of CPUs by default
            max_threads: None,
            show_progress: atty::is(atty::Stream::Stdout),  // Auto-detect TTY
        }
    }
}

impl ScannerConfig {
    /// Create scanner config from CLI args + GuardyConfig
    pub fn from_cli_args(
        args: &crate::cli::commands::scan::ScanArgs,
        config: &crate::config::GuardyConfig,
    ) -> anyhow::Result<Self> {
        // Start with default config
        let mut scanner_config = Self::default();
        
        // Apply configuration file settings (if any)
        if let Ok(scanner_section) = config.get_section("scanner") {
            // Parse any scanner config from file
            if let Ok(file_config) = serde_json::from_value::<Self>(scanner_section) {
                scanner_config = file_config;
            }
        }
        
        // Apply CLI overrides
        scanner_config.max_file_size_mb = args.max_file_size;
        scanner_config.max_cpu_percentage = args.max_cpu;
        scanner_config.show_progress = args.progress;
        scanner_config.enable_entropy_analysis = !args.no_entropy;
        
        if let Some(threshold) = args.entropy_threshold {
            scanner_config.min_entropy_threshold = threshold;
        }
        
        scanner_config.skip_binary_files = !args.include_binary;
        scanner_config.follow_symlinks = args.follow_symlinks;
        
        // Extend ignore paths with CLI values
        scanner_config.ignore_paths.extend(args.ignore_paths.clone());
        
        Ok(scanner_config)
    }
}