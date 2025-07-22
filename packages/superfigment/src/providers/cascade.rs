//! Hierarchical configuration provider for cascading configuration files
//!
//! The Hierarchical provider searches for configuration files across a directory hierarchy
//! and merges them from least specific to most specific, similar to how Git handles .gitignore files.

use figment::{
    value::{Map, Value},
    Error, Metadata, Profile, Provider,
};
use std::{
    env,
    path::{Path, PathBuf},
};
use super::Universal;
use crate::ext::ExtendExt;

/// Hierarchical configuration provider that searches and merges config files across directory hierarchy
///
/// ## Search Strategy
///
/// The provider searches for configuration files in this order (least to most specific):
///
/// 1. **System Config**: `~/.config/APP_NAME/config.*` (XDG Base Directory)
/// 2. **User Config**: `~/APP_NAME/config.*` or `~/.APP_NAME/config.*`
/// 3. **Ancestor Directories**: `../../config.*`, `../config.*` (walking up the directory tree)
/// 4. **Current Directory**: `./config.*` (highest priority)
///
/// ## Merging Behavior
///
/// All found configuration files are merged with array merging support:
/// - Later files override earlier files for scalar values
/// - Arrays are merged using `_add` and `_remove` patterns across all hierarchy levels
/// - Each file participates in the complete array merging process
///
/// ## Examples
///
/// ### Basic Usage
/// ```rust
/// use superfigment::Hierarchical;
/// use figment::Figment;
/// use superfigment::ExtendExt;
///
/// // Search for "config.*" files across hierarchy
/// let provider = Hierarchical::new("config");
/// let config = Figment::new().merge_extend(provider);
/// ```
///
/// ### With Custom Provider
/// ```rust
/// use superfigment::{Hierarchical, Universal};
/// use figment::providers::{Toml, Format};
///
/// // Use only TOML files instead of auto-detection
/// let provider = Hierarchical::new("myapp")
///     .with_provider(|path| Box::new(Toml::file(path)));
/// ```
///
/// ### With Custom Search Paths
/// ```rust
/// use superfigment::Hierarchical;
/// use std::path::PathBuf;
///
/// let custom_paths = vec![
///     PathBuf::from("/etc/myapp"),     // System-wide
///     PathBuf::from("./config"),       // Local
/// ];
///
/// let provider = Hierarchical::new("config")
///     .with_search_paths(custom_paths);
/// ```
///
/// ## Real-World Scenario
///
/// ```text
/// ~/.config/myapp/config.toml:        # System defaults
/// [database]
/// host = "localhost"
/// port = 5432
/// allowed_origins = ["https://app.com"]
///
/// ~/myapp/config.toml:                # User preferences  
/// [database]
/// timeout = 30
/// allowed_origins_add = ["https://dev.com"]
///
/// ./config.toml:                      # Project-specific
/// [database]
/// host = "prod.db"
/// allowed_origins_remove = ["https://app.com"]
/// allowed_origins_add = ["https://prod.com"]
///
/// Final merged result:
/// [database]
/// host = "prod.db"                    # Overridden by local config
/// port = 5432                         # Inherited from system config
/// timeout = 30                        # Inherited from user config
/// allowed_origins = [                 # Merged from all levels
///     "https://dev.com",              # Added by user config
///     "https://prod.com"              # Added by local config
/// ]                                   # https://app.com removed by local config
/// ```
pub struct Hierarchical {
    base_name: String,
    search_paths: Vec<PathBuf>,
    provider_factory: Box<dyn Fn(&Path) -> Box<dyn Provider> + Send + Sync>,
}

impl Hierarchical {
    /// Create a new Hierarchical provider with default search paths
    ///
    /// # Default Search Paths
    /// 
    /// The provider automatically generates these search paths:
    /// - `~/.config/{base_name}/` (XDG Base Directory standard)
    /// - `~/.{base_name}/` (traditional dot directory)  
    /// - `~/` (home directory root)
    /// - All ancestor directories up to filesystem root
    /// - Current directory `.`
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::Hierarchical;
    /// 
    /// // Searches for myapp.* in hierarchy
    /// let provider = Hierarchical::new("myapp");
    /// 
    /// // Generated search paths (in order):
    /// // ~/.config/myapp/myapp.*
    /// // ~/.myapp/myapp.*  
    /// // ~/myapp.*
    /// // ../../myapp.*
    /// // ../myapp.*
    /// // ./myapp.*
    /// ```
    pub fn new<S: AsRef<str>>(base_name: S) -> Self {
        let base_name = base_name.as_ref().to_string();
        let search_paths = Self::generate_default_search_paths(&base_name);

        Self {
            base_name,
            search_paths,
            provider_factory: Box::new(|path| Box::new(Universal::file(path))),
        }
    }

    /// Use a custom provider for reading configuration files
    ///
    /// By default, the Hierarchical provider uses `Universal::file()` for automatic
    /// format detection. This method allows you to specify a different provider.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::Hierarchical;
    /// use figment::providers::{Json, Toml, Yaml, Format};
    /// 
    /// // Only read TOML files
    /// let provider = Hierarchical::new("config")
    ///     .with_provider(|path| Box::new(Toml::file_exact(path)));
    /// 
    /// // Only read JSON files  
    /// let provider = Hierarchical::new("config")
    ///     .with_provider(|path| Box::new(Json::file_exact(path)));
    /// 
    /// // Only read YAML files
    /// let provider = Hierarchical::new("config")
    ///     .with_provider(|path| Box::new(Yaml::file_exact(path)));
    /// ```
    pub fn with_provider<F>(mut self, factory: F) -> Self
    where
        F: Fn(&Path) -> Box<dyn Provider> + Send + Sync + 'static,
    {
        self.provider_factory = Box::new(factory);
        self
    }

    /// Override the default search paths with custom directories
    ///
    /// The search order follows the order of the provided paths vector.
    /// Earlier paths have lower priority than later paths.
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::Hierarchical;
    /// use std::path::PathBuf;
    ///
    /// let custom_paths = vec![
    ///     PathBuf::from("/etc/myapp"),           // System-wide (lowest priority)
    ///     PathBuf::from("/usr/local/etc/myapp"), // System-local
    ///     PathBuf::from("~/.config/myapp"),      // User config  
    ///     PathBuf::from("./config"),             // Project-local (highest priority)
    /// ];
    ///
    /// let provider = Hierarchical::new("myapp")
    ///     .with_search_paths(custom_paths);
    /// ```
    pub fn with_search_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.search_paths = paths;
        self
    }

    /// Generate default search paths for the given base name
    ///
    /// Creates a comprehensive search hierarchy covering system, user, and project levels.
    fn generate_default_search_paths(base_name: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Add user-level configuration directories  
        if let Some(home) = Self::get_home_directory() {
            // XDG Base Directory standard: ~/.config/app/
            paths.push(home.join(".config").join(base_name));
            
            // Traditional dot directory: ~/.app/
            paths.push(home.join(format!(".{}", base_name)));
            
            // Home directory root: ~/
            paths.push(home);
        }

        // Add ancestor directories (walking up the tree)
        let mut current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        // Walk up directory tree until root
        loop {
            paths.push(current.clone());
            
            if let Some(parent) = current.parent() {
                if parent == current {
                    break; // Reached filesystem root
                }
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        // Reverse to get correct priority order (system -> user -> project -> local)
        paths.reverse();
        paths
    }

    /// Get the user's home directory
    fn get_home_directory() -> Option<PathBuf> {
        env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
    }
    
    /// Efficiently merge multiple configuration maps for a single profile
    fn merge_profile_configs(configs: Vec<Map<String, Value>>) -> Map<String, Value> {
        if configs.is_empty() {
            return Map::new();
        }
        
        // Build up configuration by direct map merging (preserving _add/_remove patterns)
        let mut result = Map::new();
        
        for config in configs {
            Self::merge_maps_without_array_processing(&mut result, config);
        }
        
        // Apply array merging ONCE at the end to process all _add/_remove patterns together
        Self::apply_array_merging_to_map(result)
    }
    
    /// Merge two configuration maps without applying array processing (preserving _add/_remove patterns)
    fn merge_maps_without_array_processing(base: &mut Map<String, Value>, override_config: Map<String, Value>) {
        for (key, value) in override_config {
            match base.get_mut(&key) {
                Some(existing_value) => {
                    // If both are objects, merge recursively
                    if let Value::Dict(_, existing_dict) = existing_value {
                        if let Value::Dict(_, override_dict) = &value {
                            Self::merge_maps_without_array_processing(existing_dict, override_dict.clone());
                            continue;
                        }
                    }
                    
                    // Special handling for _add and _remove patterns - these should be combined, not overridden
                    if key.ends_with("_add") || key.ends_with("_remove") {
                        if let Value::Array(_, existing_arr) = existing_value {
                            if let Value::Array(_, new_arr) = &value {
                                // Combine arrays instead of overriding
                                existing_arr.extend(new_arr.clone());
                                continue;
                            }
                        }
                    }
                    
                    // Otherwise, override takes precedence
                    *existing_value = value;
                }
                None => {
                    // Key doesn't exist, insert it
                    base.insert(key, value);
                }
            }
        }
    }
    
    
    /// Apply array merging patterns (_add, _remove) to a configuration map
    fn apply_array_merging_to_map(config: Map<String, Value>) -> Map<String, Value> {
        // Use the existing ArrayMergeHelper functionality through a temporary figment
        // This is still efficient since it's only called once per profile
        let temp_figment = figment::Figment::new()
            .merge(figment::providers::Serialized::from(&config, figment::Profile::Default))
            .merge_arrays();
            
        // Extract the merged data
        if let Ok(merged_data) = temp_figment.data() {
            if let Some(default_data) = merged_data.get(&figment::Profile::Default) {
                return default_data.clone();
            }
        }
        
        // Fallback to original config if array merging fails
        config
    }


    /// Find all existing configuration files in the search hierarchy
    ///
    /// Returns paths in merge order (least specific to most specific).
    fn find_config_files(&self) -> Vec<PathBuf> {
        let mut found_files = Vec::new();

        for search_dir in &self.search_paths {
            let mut found_in_dir = false;
            
            // Try common extensions for the base name
            let extensions = ["toml", "yaml", "yml", "json", "cfg"];
            
            for ext in &extensions {
                let config_file = search_dir.join(format!("{}.{}", self.base_name, ext));
                if config_file.exists() && config_file.is_file() {
                    found_files.push(config_file);
                    found_in_dir = true;
                    break; // Only take the first matching extension per directory
                }
            }

            // Only try the base name without extension if no extension file was found
            if !found_in_dir {
                let config_file = search_dir.join(&self.base_name);
                if config_file.exists() && config_file.is_file() {
                    found_files.push(config_file);
                }
            }
        }

        // Reverse to get correct priority order: system -> user -> project
        found_files.reverse();
        
        // Remove duplicates while preserving order
        let mut unique_files = Vec::new();
        for file in found_files {
            if !unique_files.contains(&file) {
                unique_files.push(file);
            }
        }
        
        unique_files
    }
}

impl Provider for Hierarchical {
    fn metadata(&self) -> Metadata {
        Metadata::named(format!("Hierarchy::{}", self.base_name))
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        let config_files = self.find_config_files();
        
        if config_files.is_empty() {
            // No config files found - return empty data
            return Ok(Map::new());
        }

        // Collect all profile data across files for efficient merging
        let mut profile_data_by_profile: Map<Profile, Vec<Map<String, Value>>> = Map::new();
        
        // Process config files in correct order (least specific to most specific)
        for config_file in config_files {
            let provider = (self.provider_factory)(&config_file);
            
            // Extract provider data and group by profile
            match provider.data() {
                Ok(provider_data) => {
                    for (profile, profile_data) in provider_data {
                        profile_data_by_profile
                            .entry(profile)
                            .or_insert_with(Vec::new)
                            .push(profile_data);
                    }
                }
                Err(_) => {
                    // Skip files that fail to load
                    continue;
                }
            }
        }
        
        // Now merge each profile's data efficiently
        let mut final_data: Map<Profile, Map<String, Value>> = Map::new();
        
        for (profile, profile_configs) in profile_data_by_profile {
            if profile_configs.is_empty() {
                continue;
            }
            
            // Optimize for single config (common case)
            let merged_data = if profile_configs.len() == 1 {
                // Fast path: no merging needed, just apply array merging
                let single_config = profile_configs.into_iter().next().unwrap();
                Self::apply_array_merging_to_map(single_config)
            } else {
                // Merge multiple configs for this profile with array merging
                Self::merge_profile_configs(profile_configs)
            };
            
            final_data.insert(profile, merged_data);
        }
        
        Ok(final_data)
    }

    fn profile(&self) -> Option<Profile> {
        None
    }
}

impl std::fmt::Debug for Hierarchical {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hierarchical")
            .field("base_name", &self.base_name)
            .field("search_paths", &self.search_paths)
            .field("provider_factory", &"<function>")
            .finish()
    }
}