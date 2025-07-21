//! Custom Figment providers for enhanced configuration management.
//!
//! This crate provides specialized providers that extend Figment's capabilities with:
//!
//! - **Smart Format Detection**: Auto-detects JSON, TOML, or YAML format from file content
//! - **Empty Value Filtering**: Skips empty CLI arguments to prevent config override
//! - **Enhanced Environment Variables**: Creates nested structures from prefixed env vars
//! - **Array Merging**: Add/remove array items instead of replacing entire arrays
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use figment::Figment;
//! use figment::providers::Serialized;
//! use guardy_figment_providers::providers::{Universal, SkipEmpty, NestedEnv};
//! use guardy_figment_providers::ext::FigmentExt;
//! 
//! // Create a figment with all enhanced providers
//! let figment = Figment::new()
//!     .merge(Serialized::defaults(MyConfig::default()))
//!     .merge_extend(Universal::file("config.xyz"))  // Any extension works!
//!     .merge(NestedEnv::prefixed("MYAPP_"))           // MYAPP_DB__HOST -> db.host
//!     .merge(SkipEmpty::new(cli_args));               // Skip empty CLI values
//! ```
//!
//! # Examples
//!
//! ## Smart Format Detection
//!
//! Works with any file extension:
//!
//! ```rust,no_run
//! use guardy_figment_providers::providers::Universal;
//!
//! let provider = Universal::file("config.xyz");  // Detects format from content
//! ```
//!
//! ## Array Merging
//!
//! ```toml
//! # config.toml
//! ignore_paths = ["base/*", "default/*"]
//! ignore_paths_add = ["custom/*", "temp/*"]     # Add these items
//! ignore_paths_remove = ["default/*"]           # Remove these items
//! # Result: ignore_paths = ["base/*", "custom/*", "temp/*"]
//! ```
//!
//! ## Environment Variables
//!
//! ```bash
//! export MYAPP_DATABASE__HOST=localhost
//! export MYAPP_DATABASE__PORT=5432
//! export MYAPP_FEATURES__ENABLED='["auth", "metrics"]'
//! ```
//!
//! Creates:
//! ```rust,ignore
//! MyConfig {
//!     database: DatabaseConfig {
//!         host: "localhost",
//!         port: 5432,
//!     },
//!     features: FeaturesConfig {
//!         enabled: ["auth", "metrics"],
//!     },
//! }
//! ```

pub mod providers;
pub mod ext;

// Re-export for convenience
pub use providers::{Universal, SkipEmpty, NestedEnv, detect_format_from_content, DetectedFormat};
pub use ext::FigmentExt;