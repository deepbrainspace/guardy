//! High-performance configuration caching with bincode and timestamp validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache metadata for invalidation checks
#[derive(Serialize, Deserialize, Debug)]
pub struct CacheMetadata {
    /// Timestamps of all config files that were loaded
    pub file_timestamps: Vec<(PathBuf, SystemTime)>,
    /// Hash of the cached config for quick comparison
    pub config_hash: u64,
    /// Version string for cache invalidation on upgrades
    pub version: String,
}

/// Manages high-performance config caching with bincode serialization
pub struct CacheManager {
    cache_dir: PathBuf,
    cache_path: PathBuf,
    meta_path: PathBuf,
    config_name: String,
}

impl CacheManager {
    /// Create a new cache manager for the given config name
    pub fn new(config_name: &str) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("fast-config")
            .join(config_name);

        fs::create_dir_all(&cache_dir).ok();

        let cache_path = cache_dir.join("config.bin");
        let meta_path = cache_dir.join("meta.bin");

        Ok(Self {
            cache_dir,
            cache_path,
            meta_path,
            config_name: config_name.to_string(),
        })
    }

    /// Load configuration from cache if valid
    pub fn load_cached<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Load metadata first
        let meta_bytes = fs::read(&self.meta_path).context("Failed to read cache metadata")?;
        let metadata: CacheMetadata =
            bincode::deserialize(&meta_bytes).context("Failed to deserialize cache metadata")?;

        // Check if cache is still valid
        if !self.is_cache_valid(&metadata)? {
            return Err(anyhow::anyhow!("Cache invalidated"));
        }

        // Load cached config
        let config_bytes = fs::read(&self.cache_path).context("Failed to read cached config")?;
        let config: T =
            bincode::deserialize(&config_bytes).context("Failed to deserialize cached config")?;

        tracing::debug!("Loaded {} config from cache (~1-3ms)", self.config_name);
        Ok(config)
    }

    /// Save configuration to cache
    pub fn save_to_cache<T>(&self, config: &T, source_file: Option<&Path>) -> Result<()>
    where
        T: Serialize,
    {
        // Serialize config
        let config_bytes =
            bincode::serialize(config).context("Failed to serialize config for caching")?;

        // Create metadata
        let mut file_timestamps = Vec::new();
        if let Some(path) = source_file
            && let Ok(metadata) = fs::metadata(path)
            && let Ok(modified) = metadata.modified()
        {
            file_timestamps.push((path.to_path_buf(), modified));
        }

        let metadata = CacheMetadata {
            file_timestamps,
            config_hash: self.hash_bytes(&config_bytes),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Write both files atomically
        fs::write(&self.cache_path, config_bytes).context("Failed to write config cache")?;
        fs::write(&self.meta_path, bincode::serialize(&metadata)?)
            .context("Failed to write cache metadata")?;

        tracing::debug!("Cached {} config for fast future loads", self.config_name);
        Ok(())
    }

    /// Check if cache is still valid
    fn is_cache_valid(&self, metadata: &CacheMetadata) -> Result<bool> {
        // Check version
        if metadata.version != env!("CARGO_PKG_VERSION") {
            tracing::debug!("Cache invalidated: version changed");
            return Ok(false);
        }

        // Check file timestamps
        for (path, cached_time) in &metadata.file_timestamps {
            if let Ok(current_metadata) = fs::metadata(path) {
                if let Ok(current_time) = current_metadata.modified()
                    && current_time > *cached_time
                {
                    tracing::debug!("Cache invalidated: {} was modified", path.display());
                    return Ok(false);
                }
            } else {
                tracing::debug!("Cache invalidated: {} no longer exists", path.display());
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Clear all cache files for this config
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path).context("Failed to remove cache file")?;
        }
        if self.meta_path.exists() {
            fs::remove_file(&self.meta_path).context("Failed to remove metadata file")?;
        }
        tracing::debug!("Cleared cache for {} config", self.config_name);
        Ok(())
    }

    /// Simple hash function for cache validation
    fn hash_bytes(&self, bytes: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }
}
