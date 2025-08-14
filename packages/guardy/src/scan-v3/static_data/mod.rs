//! Static shared data structures

pub mod binary_extensions;
pub mod configuration;
pub mod pattern_library;

// Re-export key functions and types
pub use configuration::{get_config, init_config, is_initialized};
