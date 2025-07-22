//! # SuperFigment
//!
//! **Supercharged configuration management for Rust** - 100% Figment compatible with powerful enhancements.
//!
//! ## ✨ What SuperFigment Provides
//!
//! ### 🔧 Enhanced Providers (Additional superpowers)
//! - **Universal** - Smart format detection (.toml/.yaml/.json)
//! - **Nested** - Advanced environment variable parsing with JSON arrays
//! - **Empty** - Automatic empty value filtering
//! - **Hierarchical** - Cascading config files across directory hierarchy
//!
//! ### 🚀 Extension Traits (Add methods to regular Figment)
//! - **ExtendExt** - Array merging with `_add`/`_remove` patterns
//! - **FluentExt** - Builder methods (`.with_file()`, `.with_env()`, etc.)
//! - **AccessExt** - Convenience methods (`.as_json()`, `.get_string()`, etc.)
//!
//! ### 💫 SuperFigment Builder (All-in-one solution)
//! - Built-in methods combining all enhancements
//! - Zero import complexity for new projects
//! - Use existing Figment functionalities from within SuperFigment
//!
//! ## 🎯 Quick Start
//! ```rust
//! use superfigment::SuperFigment;  // Recommended: clean all-in-one API
//! // or
//! use superfigment::prelude::*;    // For existing Figment users: add superpowers to current setup
//! ```
//!
//! ## 🔗 100% Figment Compatibility
//!
//! SuperFigment is fully compatible with existing Figment code:
//! - All Figment methods work seamlessly 
//! - Existing Figment configurations can be enhanced without changes
//! - SuperFigment can be converted to/from regular Figment instances
//! - No breaking changes to your existing Figment workflow
//!
//! ## Two Ways to Use SuperFigment
//!
//! Choose the approach that best fits your project:
//!
//! ### Approach A: Enhance Existing Figment Setup (Extension Pattern)
//!
//! **For teams with existing Figment code** - Add SuperFigment powers without changing your setup:
//!
//! ```rust
//! use figment::Figment;
//! use superfigment::prelude::*;  // Everything: traits + providers
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Config { name: String }
//! 
//! let cli_args = Config { name: "test".to_string() };
//! 
//! let config = Figment::new()                     // Keep existing Figment code
//!     .merge(Universal::file("config"))           // Enhanced provider
//!     .merge_extend(Nested::prefixed("APP_"))     // Extension trait method
//!     .merge(Empty::new(figment::providers::Serialized::defaults(cli_args))); // Enhanced provider
//! ```
//!
//! ### Approach B: Pure SuperFigment (All-in-One Pattern)
//!
//! **For new projects or clean rewrites** - Use SuperFigment's fluent builder directly:
//!
//! ```rust,no_run
//! use superfigment::SuperFigment;  // Only import you need
//! use serde::{Deserialize, Serialize};
//! // No prelude needed - SuperFigment has built-in methods
//!
//! #[derive(Debug, Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//! 
//! let cli_args = AppConfig {
//!     name: "myapp".to_string(),
//!     port: 3000,
//! };
//!
//! let config: AppConfig = SuperFigment::new()
//!     .with_file("config")        // Auto-detects .toml/.json/.yaml
//!     .with_env("APP_")          // Enhanced env parsing with JSON arrays
//!     .with_cli_opt(Some(cli_args))  // Filtered CLI args (if Some)
//!     .extract()?;               // Direct extraction with auto array merging
//! 
//! # Ok::<(), figment::Error>(())
//! ```

use std::ops::Deref;
use std::path::Path;
use figment::{Figment, Provider};
// ExtendExt trait is imported in individual methods where needed

// Re-export figment for compatibility
pub use figment;


pub mod providers;
pub mod ext;

// Re-export enhanced providers for existing Figment users
pub use providers::{Universal, Nested, Empty, Hierarchical};

// Re-export extension traits  
pub use ext::{ExtendExt, FluentExt, AccessExt};

// Re-export prelude module for convenient imports
pub use ext::prelude;

/// SuperFigment provides a fluent builder API with automatic enhancements
/// while maintaining 100% compatibility with Figment through Deref.
#[derive(Debug, Clone)]
pub struct SuperFigment {
    figment: Figment,
}

impl SuperFigment {
    /// Create a new SuperFigment instance
    pub fn new() -> Self {
        Self {
            figment: Figment::new(),
        }
    }

    /// Create SuperFigment from an existing Figment
    pub fn from_figment(figment: Figment) -> Self {
        Self { figment }
    }

    /// Add default configuration values with automatic array merging
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Deserialize, Serialize)]
    /// struct Config {
    ///     host: String,
    ///     port: u16,
    /// }
    /// 
    /// let defaults = Config {
    ///     host: "localhost".to_string(),
    ///     port: 8080,
    /// };
    /// 
    /// let config = SuperFigment::new()
    ///     .with_defaults(defaults);
    /// ```
    pub fn with_defaults<T: serde::Serialize>(self, defaults: T) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self.figment.merge_extend(figment::providers::Serialized::defaults(defaults)),
        }
    }

    /// Add file-based configuration with automatic format detection and array merging
    ///
    /// Uses the Universal provider internally for smart format detection.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// 
    /// let config = SuperFigment::new()
    ///     .with_file("config");        // Auto-detects .toml/.yaml/.json
    /// ```
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self.figment.merge_extend(Universal::file(path)),
        }
    }

    /// Add optional file-based configuration with automatic format detection and array merging
    ///
    /// Only adds the file if the Option is Some. Useful for conditional configuration loading.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// 
    /// let custom_config: Option<&str> = Some("custom.toml");
    /// let config = SuperFigment::new()
    ///     .with_file_opt(custom_config);  // Only adds if Some
    /// ```
    pub fn with_file_opt<P: AsRef<Path>>(self, path: Option<P>) -> Self {
        match path {
            Some(p) => self.with_file(p),
            None => self,
        }
    }

    /// Add environment variable configuration with automatic nesting and array merging
    ///
    /// Uses the Nested provider internally for advanced environment variable processing.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// 
    /// // Environment: APP_DATABASE_HOST=localhost, APP_FEATURES=["auth","cache"]
    /// let config = SuperFigment::new()
    ///     .with_env("APP_");           // Creates nested structure with JSON parsing
    /// ```
    pub fn with_env<S: AsRef<str>>(self, prefix: S) -> Self {
        Self {
            figment: self.figment.merge_extend(Nested::prefixed(prefix)),
        }
    }

    /// Add environment variable configuration with empty value filtering and array merging
    ///
    /// Similar to `with_env` but filters out empty values (empty strings, arrays, objects)
    /// to prevent meaningless overrides from masking meaningful configuration values.
    /// 
    /// Uses both the Nested provider for advanced environment variable parsing and the 
    /// Empty provider for filtering, combined with ExtendExt for array merging support.
    ///
    /// **Filtered Values:**
    /// - Empty strings: `""`
    /// - Empty arrays: `[]`
    /// - Empty objects: `{}`
    ///
    /// **Preserved Values:**
    /// - Meaningful falsy values: `false`, `0`
    /// - Non-empty strings, arrays, objects
    /// - JSON arrays with array merging: `MYAPP_FEATURES_ADD=["new_item"]`
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Debug, Deserialize, Serialize)]
    /// struct Config {
    ///     debug: bool,
    ///     host: String,
    ///     features: Vec<String>,
    /// }
    /// 
    /// // Environment variables:
    /// // APP_DEBUG=""              <- filtered out (empty string)
    /// // APP_HOST="localhost"       <- preserved (non-empty)  
    /// // APP_FEATURES="[]"          <- filtered out (empty array)
    /// // APP_FEATURES_ADD=["auth"]  <- merged with existing features
    /// 
    /// let config: Config = SuperFigment::new()
    ///     .with_defaults(Config {
    ///         debug: true,
    ///         host: "0.0.0.0".to_string(),
    ///         features: vec!["core".to_string()],
    ///     })
    ///     .with_env_ignore_empty("APP_")  // Empty values filtered, meaningful ones applied
    ///     .extract()?;
    ///     
    /// // Result: debug=true (default preserved), host="localhost" (env applied), 
    /// //         features=["core", "auth"] (array merged, not replaced)
    /// # Ok::<(), figment::Error>(())
    /// ```
    /// 
    /// # When to Use
    /// - Use `with_env_ignore_empty()` when you want clean config overrides without empty noise
    /// - Use `with_env()` when you need maximum flexibility and explicit empty values matter
    pub fn with_env_ignore_empty<S: AsRef<str>>(self, prefix: S) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self.figment.merge_extend(Empty::new(Nested::prefixed(prefix))),
        }
    }

    /// Add CLI arguments with empty value filtering and array merging
    ///
    /// Uses the Empty provider internally to filter out empty values.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use serde::Serialize;
    /// 
    /// #[derive(Serialize)]
    /// struct CliArgs { verbose: bool }
    /// 
    /// let config = SuperFigment::new()
    ///     .with_cli(CliArgs { verbose: true });
    /// ```
    pub fn with_cli<T: serde::Serialize>(self, cli: T) -> Self {
        let provider = figment::providers::Serialized::defaults(cli);
        Self {
            figment: self.figment.merge_extend(Empty::new(provider)),
        }
    }

    /// Add optional CLI arguments with empty value filtering and array merging
    ///
    /// Only adds CLI arguments if the Option is Some. Uses the Empty provider internally
    /// to filter out empty values.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use serde::Serialize;
    /// 
    /// #[derive(Serialize)]
    /// struct CliArgs { verbose: bool }
    /// 
    /// let cli_args: Option<CliArgs> = Some(CliArgs { verbose: true });
    /// let config = SuperFigment::new()
    ///     .with_cli_opt(cli_args);     // Only merged if Some(), empty values filtered
    /// ```
    pub fn with_cli_opt<T: serde::Serialize>(self, cli: Option<T>) -> Self {
        match cli {
            Some(c) => self.with_cli(c),
            None => self,
        }
    }


    /// Add any provider with automatic array merging
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use figment::providers::{Json, Format};
    /// 
    /// let config = SuperFigment::new()
    ///     .with_provider(Json::string(r#"{"key": "value"}"#));
    /// ```
    pub fn with_provider<P: Provider>(self, provider: P) -> Self {
        Self {
            figment: self.figment.merge_extend(provider),
        }
    }

    /// Add hierarchical configuration files with automatic cascade merging
    ///
    /// Searches for configuration files across directory hierarchy and merges them
    /// from system-wide to project-local with array merging support.
    ///
    /// Uses the Hierarchical provider internally for directory traversal and Universal
    /// provider for format detection.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// 
    /// // Searches for config.* files in hierarchy:
    /// // ~/.config/myapp/config.*
    /// // ~/.myapp/config.*
    /// // ../../config.*, ../config.*, ./config.*
    /// let config = SuperFigment::new()
    ///     .with_hierarchical_config("config");
    /// ```
    pub fn with_hierarchical_config<S: AsRef<str>>(self, base_name: S) -> Self {
        Self {
            figment: self.figment.merge_extend(Hierarchical::new(base_name)),
        }
    }

    /// Extract configuration directly (equivalent to calling .extract() on the inner Figment)
    ///
    /// This is a convenience method that makes the SuperFigment API more fluent by avoiding
    /// the need to dereference before extraction.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superfigment::SuperFigment;
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Deserialize, Serialize, Default)]
    /// struct Config {
    ///     #[serde(default)]
    ///     host: String,
    ///     #[serde(default)]
    ///     port: u16,
    /// }
    /// 
    /// let config: Config = SuperFigment::new()
    ///     .with_defaults(Config::default())
    ///     .with_file("config.toml")
    ///     .with_env("APP_")
    ///     .extract()?;                 // Direct extraction with all enhancements
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn extract<'de, T: serde::Deserialize<'de>>(&self) -> Result<T, figment::Error> {
        self.figment.extract()
    }
}

impl Default for SuperFigment {
    fn default() -> Self {
        Self::new()
    }
}

/// Deref to Figment provides 100% compatibility - all Figment methods work seamlessly
impl Deref for SuperFigment {
    type Target = Figment;

    fn deref(&self) -> &Self::Target {
        &self.figment
    }
}

impl From<Figment> for SuperFigment {
    fn from(figment: Figment) -> Self {
        Self::from_figment(figment)
    }
}

impl From<SuperFigment> for Figment {
    fn from(super_figment: SuperFigment) -> Self {
        super_figment.figment
    }
}