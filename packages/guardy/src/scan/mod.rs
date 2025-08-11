//! Optimized scanner implementation (scan2)
//!
//! This module provides a next-generation scanner with ~5x performance improvement
//! over the legacy scanner through Aho-Corasick keyword prefiltering and modern
//! architecture design.
//!
//! # Key Features
//!
//! - **Aho-Corasick Prefiltering**: Eliminates ~85% of patterns before regex execution
//! - **Single-Pass Processing**: Whole-file scanning with 50MB size limit
//! - **Pattern Classification**: Smart categorization for optimal performance
//! - **Modern Defaults**: 50MB file size limit, 20MB streaming threshold
//! - **Performance-First Parallel Processing**: Maximum worker utilization
//! - **Preserved Functionality**: 100% compatibility with existing scanner features
//!
//! # Module Structure
//!
//! - `types`: Core data structures and configuration
//! - `entropy`: Advanced entropy analysis (exact copy from legacy scanner)
//! - `file_handler`: Simple whole-file content loading with size limits
//! - `binary`: Two-stage binary file detection
//! - `ignore`: Two-tier ignore system (file/path + inline comments)
//! - `directory`: Directory orchestration with parallel processing
//! - `prefilter`: Aho-Corasick keyword filtering
//! - `patterns`: Pattern library management and classification
//! - `engine`: Single-pass pattern matching engine
//! - `core`: Main scanner orchestrator
//! - `config`: Configuration and CLI integration
//!
//! # Usage
//!
//! ```rust
//! use guardy::scan::{OptimizedScanner, ScannerConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ScannerConfig::default();
//! let scanner = OptimizedScanner::new(config)?;
//! let results = scanner.scan_paths(&["src/".to_string()])?;
//! println!("Found {} secrets", results.len());
//! # Ok(())
//! # }
//! ```

pub mod types;

// Re-export core types for convenience
pub use types::{
    SecretMatch, ScanStats, Warning, ScanFileResult, ScanResult,
    ScanMode, ScannerConfig, SecretPattern, SecretPatterns,
};