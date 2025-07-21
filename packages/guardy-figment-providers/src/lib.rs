//! Enhanced Figment providers for common configuration management challenges.
//!
//! This crate provides three specialized Figment providers to solve common pain points:
//! 
//! - [`SmartFormat`]: Automatically detects configuration file formats (JSON, TOML, YAML)
//! - [`SkipEmpty`]: Filters out empty values to prevent CLI overrides from masking config
//! - [`NestedEnv`]: Enhanced environment variable provider with better prefix/separator handling
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use figment::Figment;
//! use guardy_figment_providers::{SmartFormat, SkipEmpty, NestedEnv};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct Config {
//!     name: String,
//!     database: DatabaseConfig,
//! }
//!
//! #[derive(Deserialize)]
//! struct DatabaseConfig {
//!     host: String,
//!     port: u16,
//! }
//!
//! #[derive(Serialize)]
//! struct CliArgs {
//!     name: Option<String>,
//!     debug: Vec<String>, // Often empty from CLI
//! }
//!
//! let cli_args = CliArgs { 
//!     name: Some("my-app".to_string()), 
//!     debug: vec![] // Empty - will be filtered out
//! };
//!
//! let figment = Figment::new()
//!     .merge(SmartFormat::file("config.toml"))    // Auto-detects format!
//!     .merge(NestedEnv::prefixed("APP_"))         // APP_DATABASE_HOST -> database.host
//!     .merge(SkipEmpty::new(cli_args));           // Filters empty CLI values
//!
//! let config: Config = figment.extract()?;
//! # Ok::<(), figment::Error>(())
//! ```

pub mod smart_format;
pub mod skip_empty;
pub mod nested_env;

pub use smart_format::SmartFormat;
pub use skip_empty::SkipEmpty;
pub use nested_env::NestedEnv;