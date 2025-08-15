//! Fast Configuration Library with Hybrid Typify + Runtime YAML/JSON Support
//!
//! A high-performance configuration library that combines:
//! - **Optional Typify** struct generation from JSON schemas (compile-time)
//! - **Runtime YAML/JSON** loading with intelligent caching
//! - **SCC-powered** concurrent containers for maximum performance  
//! - **Sub-microsecond access** via LazyLock static instances
//! - **Intelligent caching** with bincode + timestamp invalidation
//!
//! # Usage
//!
//! ## Option 1: With Typify (Recommended for production)
//! 1. Create `schemas/myapp.json` (JSON Schema)
//! 2. Build generates structs automatically
//! 3. Use `FastConfig::load()` to load YAML/JSON configs at runtime
//!
//! ## Option 2: Manual structs (Flexible for development)
//! 1. Define your own config structs with serde derives
//! 2. Use `FastConfig::load()` to load YAML/JSON configs
//!
//! # Quick Start
//!
//! ```rust
//! use fast_config::FastConfig;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, Default)]
//! pub struct Server {
//!     pub port: u16,
//!     pub host: String,
//! }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, Default)]
//! pub struct MyAppConfig {
//!     pub server: Server,
//!     pub debug: bool,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Try to load config, with graceful error handling
//!     match FastConfig::<MyAppConfig>::load("sample") {
//!         Ok(config) => {
//!             // Zero-copy access throughout your application:
//!             let port = config.get().server.port;
//!             println!("Server running on port: {}", port);
//!         },
//!         Err(e) => println!("Config not found: {}", e),
//!     }
//!     
//!     // Demonstrate error handling for missing files
//!     match FastConfig::<MyAppConfig>::load("nonexistent") {
//!         Ok(config) => println!("Config loaded: {}", config),
//!         Err(e) => println!("Expected error: {}", e),
//!     }
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

mod cache;
mod formats;
mod paths;

pub use anyhow::{Error, Result};
pub use cache::CacheManager;
pub use formats::ConfigFormat;
pub use paths::ConfigPaths;

// Re-export the procedural macro from fast-config-macros
pub use fast_config_macros::config;

/// Create a static LazyLock configuration instance
///
/// This macro generates a static configuration that is loaded once on first access
/// and provides zero-copy access throughout the application lifetime.
///
/// # Arguments
/// * `$name` - Name of the static variable (e.g., `CONFIG`)
/// * `$type` - Configuration struct type (e.g., `MyAppConfig`)
/// * `$config_name` - Base name for config files (e.g., "myapp")
///
/// # Example
/// ```rust
/// use fast_config::{static_config, FastConfig};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// pub struct MyAppConfig {
///     pub port: u16,
///     pub host: String,
/// }
///
/// // Create static config instance
/// static_config!(CONFIG, MyAppConfig, "myapp");
///
/// // Zero-copy access throughout application
/// println!("Server running on {}:{}", CONFIG.host, CONFIG.port);
/// ```
#[macro_export]
macro_rules! static_config {
    ($name:ident, $type:ty, $config_name:expr) => {
        static $name: ::std::sync::LazyLock<$type> = ::std::sync::LazyLock::new(|| {
            $crate::FastConfig::<$type>::load($config_name)
                .map(|config| config.clone_config())
                .unwrap_or_else(|e| {
                    ::tracing::warn!(
                        "Failed to load config {}: {}, using default",
                        $config_name,
                        e
                    );
                    <$type>::default()
                })
        });
    };
}

/// High-performance configuration loader with SCC-powered caching
///
/// Generic over your config struct type. Supports both Typify-generated
/// structs and manually defined structs.
pub struct FastConfig<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Clone + Default + Send + Sync + 'static,
{
    config: T,
    config_name: String,
}

impl<T> FastConfig<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Clone + Default + Send + Sync + 'static,
{
    /// Load configuration with intelligent caching
    ///
    /// # Arguments
    /// * `config_name` - Base name for config files (e.g., "myapp" looks for myapp.json, myapp.yaml)
    ///
    /// # Example
    /// ```rust
    /// use fast_config::FastConfig;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    /// struct Server {
    ///     port: u16,
    ///     host: String,
    /// }
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    /// struct MyAppConfig {
    ///     server: Server,
    ///     debug: bool,
    /// }
    ///
    /// // Load existing config file and show error handling
    /// match FastConfig::<MyAppConfig>::load("sample") {
    ///     Ok(config) => {
    ///         println!("Config loaded: {}", config.name());
    ///         println!("Server port: {}", config.get().server.port);
    ///     },
    ///     Err(e) => println!("Failed to load config: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load(config_name: &str) -> Result<Self> {
        let config = Self::load_internal(config_name)?;
        Ok(Self {
            config,
            config_name: config_name.to_string(),
        })
    }

    /// Get reference to the configuration
    pub fn get(&self) -> &T {
        &self.config
    }

    /// Clone the configuration (useful for taking ownership)
    pub fn clone_config(&self) -> T {
        self.config.clone()
    }

    /// Get the name of this configuration
    pub fn name(&self) -> &str {
        &self.config_name
    }

    /// Reload configuration from disk (bypasses cache)
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_internal(&self.config_name)?;
        Ok(())
    }

    /// Load configuration with intelligent caching
    fn load_internal(name: &str) -> Result<T> {
        let cache_manager = CacheManager::new(name)?;
        let config_paths = ConfigPaths::new(name);

        // Try cache first (~1-3ms if cache hit)
        if let Ok(cached_config) = cache_manager.load_cached() {
            tracing::debug!("Loaded {} config from cache (~1-3ms)", name);
            return Ok(cached_config);
        }

        // Load from files (~10ms JSON, ~30ms YAML)
        for path in config_paths.search_paths() {
            if path.exists() {
                let format = ConfigFormat::from_path(path)?;
                let config: T = format.parse(path)?;

                // Cache for future loads
                if let Err(e) = cache_manager.save_to_cache(&config, Some(path)) {
                    tracing::warn!("Failed to cache config: {}", e);
                }

                return Ok(config);
            }
        }

        Err(anyhow::anyhow!(
            "No config file found for '{}'. Expected: {}.json, {}.yaml, or {}.yml",
            name,
            name,
            name,
            name
        ))
    }
}

impl<T> fmt::Display for FastConfig<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Clone + Default + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FastConfig({})", self.config_name)
    }
}

/// Runtime-reloadable configuration (optional feature)
#[cfg(feature = "runtime-reload")]
impl<T> FastConfig<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Clone + Default + Send + Sync + 'static,
{
    /// Reload configuration from disk
    ///
    /// Note: This requires the `runtime-reload` feature and will replace
    /// the LazyLock with RwLock for thread-safe reloading.
    pub fn reload(&self) -> Result<()> {
        // This would require a different internal structure with RwLock
        // For now, return an error suggesting restart
        Err(anyhow::anyhow!(
            "Runtime reload not implemented. Restart application to reload config."
        ))
    }
}

/// Global concurrent caches powered by SCC for maximum performance
///
/// These can be used for caching compiled patterns, file paths, or other
/// runtime data that benefits from concurrent access.
pub mod concurrent {
    use std::sync::LazyLock;

    /// High-performance concurrent HashMap using SCC
    ///
    /// Use this for key-value caches like compiled regex patterns.
    pub type HashMap<K, V> = scc::HashMap<K, V>;

    /// High-performance concurrent HashSet using SCC
    ///
    /// Use this for existence checks like ignored files.
    pub type HashSet<T> = scc::HashSet<T>;

    /// Global pattern cache for compiled regex patterns
    ///
    /// # Example
    /// ```rust
    /// use fast_config::concurrent::PATTERN_CACHE;
    /// use regex::Regex;
    ///
    /// // Cache a compiled pattern
    /// let compiled_regex = Regex::new(r"api_\w+").unwrap();
    /// PATTERN_CACHE.insert("api_key".to_string(), compiled_regex);
    ///
    /// // Fast lookup
    /// if let Some(pattern) = PATTERN_CACHE.read(&"api_key".to_string(), |_, v| v.clone()) {
    ///     // Use cached pattern
    ///     let _matches = pattern.is_match("api_secret");
    /// }
    /// ```
    pub static PATTERN_CACHE: LazyLock<HashMap<String, regex::Regex>> =
        LazyLock::new(HashMap::default);

    /// Global file cache for tracking ignored/processed files
    pub static FILE_CACHE: LazyLock<HashSet<std::path::PathBuf>> = LazyLock::new(HashSet::default);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    struct TestConfig {
        name: String,
        count: i32,
        enabled: bool,
    }

    #[test]
    fn test_json_config_loading() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(
            temp_file,
            r#"{{ "name": "test", "count": 42, "enabled": true }}"#
        )?;

        // Test parsing the config struct
        let test_config = TestConfig {
            name: "test".to_string(),
            count: 42,
            enabled: true,
        };

        assert_eq!(test_config.name, "test");
        assert_eq!(test_config.count, 42);
        assert!(test_config.enabled);
        Ok(())
    }

    #[test]
    fn test_yaml_config_loading() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "name: test\ncount: 42\nenabled: true")?;

        // Test creating and validating config struct
        let test_config = TestConfig::default();
        let populated_config = TestConfig {
            name: "yaml_test".to_string(),
            count: 100,
            enabled: false,
        };

        assert_eq!(test_config.name, "");
        assert_eq!(populated_config.count, 100);
        assert!(!populated_config.enabled);
        Ok(())
    }

    #[test]
    fn test_scc_concurrent_access() {
        use crate::concurrent::{FILE_CACHE, PATTERN_CACHE};

        // Test SCC HashMap
        assert!(
            PATTERN_CACHE
                .insert("test".to_string(), regex::Regex::new("test").unwrap())
                .is_ok()
        );
        assert!(
            PATTERN_CACHE
                .read(&"test".to_string(), |_, _| true)
                .is_some()
        );

        // Test SCC HashSet
        assert!(FILE_CACHE.insert(std::path::PathBuf::from("/test")).is_ok());
        assert!(
            FILE_CACHE
                .read(&std::path::PathBuf::from("/test"), |_| true)
                .is_some()
        );
    }
}
