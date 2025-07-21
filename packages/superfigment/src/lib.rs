//! # SuperFigment
//!
//! SuperFigment gives "superpowers" to the already powerful Figment library.
//! It provides enhanced configuration management with 100% Figment compatibility
//! plus advanced features like array merging and smart format detection.
//!
//! ## Dual API Design
//!
//! SuperFigment provides two ways to use it:
//!
//! ### 1. Enhanced Providers (Drop-in for existing Figment users)
//! ```rust
//! use figment::Figment;
//! use superfigment::{Universal, NestedEnv, SkipEmpty, FigmentExt};
//!
//! let config = Figment::new()
//!     .merge(Universal::file("config"))           // Auto-detects format
//!     .merge_extend(NestedEnv::prefixed("APP_"))  // Array merging
//!     .merge(SkipEmpty::new(cli_args));          // Filters empty values
//! ```
//!
//! ### 2. SuperFigment Builder (Convenient fluent API)
//! ```rust
//! use superfigment::SuperFigment;
//!
//! let config = SuperFigment::new()
//!     .with_file("config")        // Auto-detects .toml/.json/.yaml
//!     .with_env("APP_")          // Enhanced env parsing with JSON arrays
//!     .with_cli_opt(args)        // Filtered CLI args (if Some)
//!     .extract()?;               // Direct extraction with auto array merging
//! ```

use std::ops::Deref;
use std::path::Path;
use figment::{Figment, Provider};

pub mod providers;
pub mod ext;

// Re-export enhanced providers for existing Figment users
pub use providers::{Universal, Nested, Empty, Hierarchical};

// Re-export extension traits
pub use ext::{ExtendExt, FluentExt, AccessExt, AllExt};

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
    /// use serde::Deserialize;
    /// 
    /// #[derive(Deserialize)]
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
        Self {
            figment: self.figment.merge_extend(Universal::file(path)),
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

    /// Add CLI arguments with empty value filtering and array merging (if provided)
    ///
    /// Uses the Empty provider internally to filter out empty values.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// 
    /// let cli_args = Some(/* CLI provider */);
    /// let config = SuperFigment::new()
    ///     .with_cli_opt(cli_args);     // Only merged if Some(), empty values filtered
    /// ```
    pub fn with_cli_opt<P: Provider>(self, provider: Option<P>) -> Self {
        match provider {
            Some(p) => Self {
                figment: self.figment.merge_extend(Empty::new(p)),
            },
            None => self,
        }
    }

    /// Add any provider with automatic array merging
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use figment::providers::Json;
    /// 
    /// let config = SuperFigment::new()
    ///     .with_provider(Json::file("config.json"));
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
    /// ```rust
    /// use superfigment::SuperFigment;
    /// use serde::Deserialize;
    /// 
    /// #[derive(Deserialize)]
    /// struct Config {
    ///     host: String,
    ///     port: u16,
    /// }
    /// 
    /// let config: Config = SuperFigment::new()
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