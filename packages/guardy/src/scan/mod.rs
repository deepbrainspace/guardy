//! Scan v3 - High-performance secret scanning engine
//! 
//! This module provides a clean, efficient implementation of secret scanning
//! with Aho-Corasick prefiltering and parallel file processing.

// Sub-modules
mod config;
mod data;
mod filters;
mod pipeline;
mod scanner;
mod static_data;
mod tracking;

// Public API exports
pub use config::ScannerConfig;
pub use data::{FileResult, ScanResult, ScanStats, SecretMatch};
pub use scanner::Scanner;

// Re-export commonly used types for convenience
pub use data::MatchSeverity;
pub use tracking::ProgressTracker;