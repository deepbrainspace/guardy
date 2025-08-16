//! # SuperConfig Prelude
//! 
//! The prelude module provides convenient glob imports for SuperConfig.
//! Import everything you need with a single `use superconfig::prelude::*;`
//! 
//! ## Usage
//! 
//! ```rust,ignore
//! use superconfig::prelude::*;
//! use serde::{Deserialize, Serialize};
//! 
//! #[derive(Debug, Clone, Serialize, Deserialize, Default)]
//! struct MyAppConfig {
//!     port: u16,
//!     debug: bool,
//! }
//! 
//! // All core functionality is now available:
//! config!("myapp" => MyAppConfig);
//! 
//! let config = Config::<MyAppConfig>::load("myapp");
//! ```
//! 
//! This brings in the most commonly used types and macros:
//! - [`Config`] - Main configuration loader
//! - [`config!`] - Procedural macro for auto-generating structs
//! - [`ConfigBuilder`] - Builder pattern for layered configuration
//! - [`Error`] and [`Result`] - Error handling types
//! - [`ConfigFormat`] - Format detection and parsing

/// Re-export core types for convenient access
pub use crate::{
    Config,          // Main configuration struct
    Error,           // Error type
    Result,          // Result type alias
    ConfigFormat,    // Format handling
    ConfigPaths,     // Path resolution
};

/// Re-export macros for zero-boilerplate configuration
pub use crate::{
    config,          // Procedural macro for auto-generation
};

/// Re-export builder and partial config support
pub use crate::{ConfigBuilder, PartialConfig, PartialConfigurable};

/// Re-export concurrent utilities
pub use crate::concurrent::{HashMap, HashSet, PATTERN_CACHE, FILE_CACHE};