//! Static shared data structures

pub mod binary_extensions;
pub mod configuration;
pub mod pattern_library;

// Re-export key functions and types
pub use binary_extensions::{get_binary_extensions, is_binary_extension, BINARY_EXTENSIONS};
pub use configuration::{get_config, init_config, is_initialized};
pub use pattern_library::{get_pattern_library, CompiledPattern, PatternClass, PatternLibrary, PATTERN_LIBRARY};