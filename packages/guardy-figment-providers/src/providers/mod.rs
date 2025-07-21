//! Custom Figment providers for enhanced configuration management.
//!
//! This module provides specialized providers that extend Figment's capabilities:
//!
//! - [`Universal`]: Auto-detects configuration file formats (JSON/TOML/YAML)
//! - [`SkipEmpty`]: Filters empty CLI values to prevent config override
//! - [`NestedEnv`]: Creates nested config structures from environment variables

mod universal;
mod skip_empty;
mod nested_env;

pub use universal::{Universal, detect_format_from_content, DetectedFormat};
pub use skip_empty::SkipEmpty;
pub use nested_env::NestedEnv;