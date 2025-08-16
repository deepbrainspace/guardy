//! # SuperConfig Prelude
//! 
//! The prelude module provides convenient glob imports for SuperConfig.
//! Import everything you need with a single `use superconfig::prelude::*;`
//! 
//! ## Usage
//! 
//! ```rust,no_run
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
//! static_config!(CONFIG, MyAppConfig, "myapp");
//! 
//! let config = FastConfig::<MyAppConfig>::load("myapp");
//! ```
//! 
//! This brings in the most commonly used types and macros:
//! - [`FastConfig`] - Main configuration loader
//! - [`config!`] - Procedural macro for auto-generating structs
//! - [`static_config!`] - Macro for creating static instances
//! - [`Error`] and [`Result`] - Error handling types
//! - [`ConfigFormat`] - Format detection and parsing
//! - [`CacheManager`] - When cache feature is enabled

/// Re-export core types for convenient access
pub use crate::{
    FastConfig,      // Main configuration struct
    Error,           // Error type
    Result,          // Result type alias
    ConfigFormat,    // Format handling
    ConfigPaths,     // Path resolution
};

/// Re-export macros for zero-boilerplate configuration
pub use crate::{
    config,          // Procedural macro for auto-generation
    static_config,   // Macro for static instances
};

/// Re-export cache management when available
#[cfg(feature = "cache")]
pub use crate::CacheManager;

/// Re-export concurrent utilities
pub use crate::concurrent::{HashMap, HashSet, PATTERN_CACHE, FILE_CACHE};