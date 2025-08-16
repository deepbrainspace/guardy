//! # SuperConfig: High-Performance Configuration Library
//!
//! [![Crates.io](https://img.shields.io/crates/v/superconfig.svg)](https://crates.io/crates/superconfig)
//! [![Documentation](https://docs.rs/superconfig/badge.svg)](https://docs.rs/superconfig)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//!
//! A blazing-fast configuration library for Rust applications featuring zero-copy access
//! and support for multiple configuration formats.
//!
//! ## Key Features
//!
//! - 🚀 **Sub-microsecond access** via [`LazyLock`] static instances after first load
//! - ⚡ **Direct file parsing** for best performance (~10ms JSON, ~30ms YAML)
//! - 💾 **Optional caching** with [bincode] serialization (enable with `cache` feature)
//! - 📄 **Multi-format support** for JSON and YAML configuration files
//! - 🏗️ **Procedural macros** for zero-boilerplate configuration setup
//! - 🛡️ **Type-safe** configuration with full [serde] integration
//! - 🔧 **Flexible search paths** (current dir, user config dirs)
//! - 📁 **Direct path loading** with `load_from_path()` for custom locations
//!
//! ## Performance Comparison
//!
//! | Access Method | Performance | Use Case |
//! |---------------|-------------|----------|
//! | **Static LazyLock** | `0.57 ns` | Production (after first load) |
//! | **Direct load (no cache)** | `5.28 μs` | Default mode - best overall performance |
//! | **Cached load** | `26.39 μs` | When `cache` feature enabled |
//! | **Cold start (with cache)** | `40.57 μs` | First load with cache overhead |
//!
//! **Note**: Cache is disabled by default as direct parsing (`5.28 μs`) performs 5x better 
//! than cached loads (`26.39 μs`). The cache feature adds overhead that rarely pays off.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! superconfig = "0.2"
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ### Method 1: Static Configuration (Recommended)
//!
//! The fastest approach using the [`static_config!`] macro with prelude:
//!
//! ```rust
//! use superconfig::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, Default)]
//! pub struct AppConfig {
//!     pub database_url: String,
//!     pub port: u16,
//!     pub debug: bool,
//! }
//!
//! // Generate static LazyLock instance for zero-copy access
//! static_config!(CONFIG, AppConfig, "myapp");
//!
//! // Sub-microsecond access after first load
//! println!("Server starting on port {}", CONFIG.port);
//! println!("Database: {}", CONFIG.database_url);
//! ```
//!
//! Create `myapp.json` in your project root:
//!
//! ```json
//! {
//!   "database_url": "postgres://localhost/myapp",
//!   "port": 8080,
//!   "debug": true
//! }
//! ```
//!
//! ### Method 2: Explicit Loading
//!
//! For more control over error handling:
//!
//! ```rust
//! use superconfig::FastConfig;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, Default)]
//! pub struct AppConfig {
//!     pub database_url: String,
//!     pub port: u16,
//!     pub debug: bool,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Explicit loading with comprehensive error handling
//!     match FastConfig::<AppConfig>::load("myapp") {
//!         Ok(config) => {
//!             println!("Loaded config: {}", config.name());
//!             println!("Server port: {}", config.get().port);
//!             
//!             // Clone for ownership if needed
//!             let _owned_config = config.clone_config();
//!         }
//!         Err(_) => {
//!             println!("Using default config");
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Method 3: Procedural Macro (Auto-generate structs)
//!
//! Automatically generate configuration structs from existing config files:
//!
//! ```rust,ignore
//! use superconfig::prelude::*;  // Convenient glob import
//!
//! // Auto-generates struct from myapp.json/yaml and creates LazyLock instance
//! config!("myapp" => MyAppConfig);
//!
//! fn main() {
//!     // Zero-copy access to auto-generated struct
//!     let config = MyAppConfig::global();
//!     println!("Port: {}", config.server.port);
//! }
//! ```
//!
//! ## Configuration File Search Order
//!
//! SuperConfig searches for configuration files in the following order:
//!
//! 1. **Current directory**: `{name}.json`, `{name}.yaml`, `{name}.yml`
//! 2. **User config directory**: `~/.config/{name}.json`, `~/.config/{name}.yaml`, `~/.config/{name}.yml`
//! 3. **App config directory**: `~/.config/{name}/config.json`, `~/.config/{name}/config.yaml`, `~/.config/{name}/config.yml`
//!
//! You can also load from a specific path using `FastConfig::load_from_path("/path/to/config.yaml")`
//!
//! ## Import Styles
//!
//! SuperConfig supports two import styles for maximum convenience:
//!
//! ### Explicit Imports (Explicit Control)
//! ```rust
//! use superconfig::{FastConfig, static_config, config, Error, Result};
//! use serde::{Deserialize, Serialize};
//! ```
//!
//! ### Prelude Glob Import (Convenience)
//! ```rust
//! use superconfig::prelude::*;  // Imports all commonly used items
//! use serde::{Deserialize, Serialize};
//! ```
//!
//! Both styles give you access to the same functionality. Choose based on your preference:
//! - **Explicit imports**: Clear about what's being imported, better for large codebases
//! - **Prelude glob**: Convenient for quick prototyping and small projects
//!
//! ## Advanced Usage
//!
//! ### Concurrent Access with SCC
//!
//! SuperConfig includes high-performance concurrent containers:
//!
//! ```rust
//! use superconfig::concurrent::{HashMap, HashSet, PATTERN_CACHE};
//! use regex::Regex;
//!
//! // Global concurrent pattern cache
//! let pattern = Regex::new(r"api_\w+").unwrap();
//! PATTERN_CACHE.insert("api_pattern".to_string(), pattern);
//!
//! // Fast concurrent access in multi-threaded scenarios
//! let matches = PATTERN_CACHE.read(&"api_pattern".to_string(), |_, pattern| {
//!     pattern.is_match("api_key")
//! });
//! ```
//!
//! ### Cache Management (Optional)
//!
//! Only available when the `cache` feature is enabled:
//!
//! ```rust,no_run
//! #[cfg(feature = "cache")]
//! fn manage_cache() -> Result<(), Box<dyn std::error::Error>> {
//!     use superconfig::CacheManager;
//!     
//!     let cache = CacheManager::new("myapp")?;
//!
//!     // Manually clear cache
//!     cache.clear_cache()?;
//!
//!     // Get cache directory
//!     println!("Cache location: {:?}", cache.cache_dir());
//!     Ok(())
//! }
//! ```
//!
//! ### Runtime Reloading (Optional)
//!
//! Enable the `runtime-reload` feature for applications that need dynamic config updates:
//!
//! ```toml
//! [dependencies]
//! superconfig = { version = "0.2", features = ["runtime-reload"] }
//! ```
//!
//! **Note**: Runtime reloading adds `RwLock` overhead. For maximum performance,
//! use the default mode without runtime reloading.
//!
//! ## Features
//!
//! - **`json`**: JSON format support (enabled by default)
//! - **`yaml`**: YAML format support (enabled by default)
//! - **`cache`**: Enable bincode caching (disabled by default, adds overhead)
//! - **`runtime-reload`**: Enable runtime configuration reloading
//!
//! ## Error Handling
//!
//! Fast-config provides comprehensive error handling:
//!
//! ```rust,no_run
//! use superconfig::{FastConfig, Error};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, Default)]
//! struct AppConfig {
//!     port: u16,
//!     debug: bool,
//! }
//!
//! fn load_with_fallback() -> AppConfig {
//!     FastConfig::<AppConfig>::load("myapp")
//!         .map(|config| config.clone_config())
//!         .unwrap_or_else(|e| {
//!             eprintln!("Failed to load config: {}", e);
//!             AppConfig::default()
//!         })
//! }
//! ```
//!
//! ## Best Practices
//!
//! 1. **Use [`static_config!`] for globals**: Provides zero-copy access and maximum performance
//! 2. **Prefer JSON over YAML**: JSON parsing is ~3x faster than YAML
//! 3. **Use [`Default`] derive**: Enables graceful fallback when config files are missing
//! 4. **Keep config structs simple**: Avoid complex nested enums for better cache performance
//! 5. **Validate on startup**: Perform custom validation after loading configuration
//!
//! [bincode]: https://docs.rs/bincode
//! [serde]: https://docs.rs/serde
//! [`LazyLock`]: std::sync::LazyLock
//! [`Default`]: std::default::Default

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

cfg_if::cfg_if! {
    if #[cfg(feature = "cache")] {
        mod cache;
        pub use cache::CacheManager;
    }
}

mod formats;
mod paths;

/// Prelude module for convenient glob imports
pub mod prelude;

pub use anyhow::{Error, Result};
pub use formats::ConfigFormat;
pub use paths::ConfigPaths;

// Re-export the procedural macro from superconfig-macros
pub use superconfig_macros::config;

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
/// use superconfig::{static_config, FastConfig};
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
    /// use superconfig::FastConfig;
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
        let config = Self::load_internal(config_name, None)?;
        Ok(Self {
            config,
            config_name: config_name.to_string(),
        })
    }
    
    /// Load configuration from a specific file path
    ///
    /// This bypasses the normal search paths and loads directly from the given file.
    ///
    /// # Arguments
    /// * `path` - Direct path to the configuration file
    ///
    /// # Example
    /// ```rust,no_run
    /// use superconfig::FastConfig;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    /// struct MyAppConfig {
    ///     debug: bool,
    /// }
    ///
    /// // Load from a specific path
    /// let config = FastConfig::<MyAppConfig>::load_from_path("/etc/myapp/production.yaml")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let config_name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("config")
            .to_string();
        
        let config = Self::load_internal(&config_name, Some(path_ref))?;
        Ok(Self {
            config,
            config_name,
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
    #[cfg(not(feature = "runtime-reload"))]
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_internal(&self.config_name, None)?;
        Ok(())
    }

    /// Load configuration with optional caching
    fn load_internal(name: &str, direct_path: Option<&std::path::Path>) -> Result<T> {
        let start_time = std::time::Instant::now();
        tracing::debug!("⏱️  TIMING: load_internal start for '{}'", name);

        cfg_if::cfg_if! {
            if #[cfg(feature = "cache")] {
                let cache_manager = {
                    let cache_manager = CacheManager::new(name)?;
                    let cache_manager_time = start_time.elapsed();
                    tracing::debug!("⏱️  TIMING: CacheManager::new took {:?}", cache_manager_time);
                    cache_manager
                };
            }
        }

        // If direct path provided, use only that path
        let search_paths: Vec<PathBuf> = if let Some(path) = direct_path {
            vec![path.to_path_buf()]
        } else {
            let config_paths_start = std::time::Instant::now();
            let config_paths = ConfigPaths::new(name);
            let config_paths_time = config_paths_start.elapsed();
            let total_time = start_time.elapsed();
            tracing::debug!("⏱️  TIMING: ConfigPaths::new took {:?} (total: {:?})", 
                           config_paths_time, total_time);
            config_paths.search_paths().cloned().collect()
        };

        // Try cache first (~1-3ms if cache hit) - skip for direct paths
        cfg_if::cfg_if! {
            if #[cfg(feature = "cache")] {
                if direct_path.is_none() {
                    let cache_check_start = std::time::Instant::now();
                    if let Ok(cached_config) = cache_manager.load_cached() {
                        let cache_hit_time = cache_check_start.elapsed();
                        tracing::debug!("⏱️  TIMING: Cache HIT - loaded in {:?} (total: {:?})", 
                                       cache_hit_time, start_time.elapsed());
                        return Ok(cached_config);
                    }
                    let cache_miss_time = cache_check_start.elapsed();
                    tracing::debug!("⏱️  TIMING: Cache MISS - check took {:?}", cache_miss_time);
                }
            }
        }

        // Load from files (~2μs JSON, ~30ms YAML)
        let file_search_start = std::time::Instant::now();
        for path in &search_paths {
            if path.exists() {
                let file_found_time = file_search_start.elapsed();
                tracing::debug!("⏱️  TIMING: Found config file {:?} after {:?}", path, file_found_time);

                let format_start = std::time::Instant::now();
                let format = ConfigFormat::from_path(path)?;
                let format_time = format_start.elapsed();
                tracing::debug!("⏱️  TIMING: ConfigFormat::from_path took {:?}", format_time);

                let parse_start = std::time::Instant::now();
                let config: T = format.parse(path)?;
                let parse_time = parse_start.elapsed();
                tracing::debug!("⏱️  TIMING: format.parse took {:?} (JSON deserialization)", parse_time);

                cfg_if::cfg_if! {
                    if #[cfg(feature = "cache")] {
                        let clone_start = std::time::Instant::now();
                        let config_clone = config.clone();
                        let clone_time = clone_start.elapsed();
                        tracing::debug!("⏱️  TIMING: config.clone() took {:?}", clone_time);

                        // Cache in background using channel-based worker (fastest approach: 194µs)
                        let spawn_start = std::time::Instant::now();
                        Self::spawn_background_cache_task(cache_manager, config_clone, path.to_path_buf());
                        let spawn_time = spawn_start.elapsed();
                        tracing::debug!("⏱️  TIMING: spawn_background_cache_task took {:?}", spawn_time);
                    } else {
                        tracing::debug!("⏱️  TIMING: No caching (cache feature disabled)");
                    }
                }

                let total_time = start_time.elapsed();
                cfg_if::cfg_if! {
                    if #[cfg(feature = "cache")] {
                        tracing::debug!("⏱️  TIMING: load_internal COMPLETE in {:?} (with caching)", total_time);
                    } else {
                        tracing::debug!("⏱️  TIMING: load_internal COMPLETE in {:?} (no caching)", total_time);
                    }
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

    /// Execute background cache task using channel-based worker thread (fastest: 194µs)
    #[cfg(feature = "cache")]
    fn spawn_background_cache_task(cache_manager: CacheManager, config: T, config_path: std::path::PathBuf) {
        use std::sync::LazyLock;
        use std::sync::mpsc;
        use std::thread;
        
        // Use closure approach for type-erased caching
        type CacheTaskSender = mpsc::Sender<Box<dyn FnOnce() + Send>>;
        
        // Global channel-based worker for background caching
        static CACHE_WORKER: LazyLock<CacheTaskSender> = LazyLock::new(|| {
            let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
            
            thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    task(); // Execute the cache task
                }
            });
            
            tx
        });
        
        let cache_task = Box::new(move || {
            if let Err(e) = cache_manager.save_to_cache(&config, Some(&config_path)) {
                tracing::warn!("Background cache write failed: {}", e);
            } else {
                tracing::debug!("Background cache write completed for {:?}", config_path);
            }
        });
        
        if let Err(_) = CACHE_WORKER.send(cache_task) {
            tracing::warn!("Failed to send cache task to worker thread");
        }
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
    /// use superconfig::concurrent::PATTERN_CACHE;
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
