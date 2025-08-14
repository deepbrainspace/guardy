//! Scan v3 - High-performance secret scanning engine
//! 
//! This module provides a clean, efficient implementation of secret scanning
//! with Aho-Corasick prefiltering and parallel file processing.

// Sub-modules
mod config;
mod core;
mod data;
pub mod filters;
pub mod pipeline;
pub mod reports;
mod static_data;

// Public API exports
pub use config::ScannerConfig;
pub use core::Scanner;
pub use data::{ScanResult, SecretMatch, ScanStats};

// Re-export commonly used types for convenience
