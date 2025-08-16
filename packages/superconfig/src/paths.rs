//! Configuration file path resolution and search logic

use std::path::PathBuf;

/// Manages configuration file search paths
pub struct ConfigPaths {
    config_name: String,
    search_paths: Vec<PathBuf>,
}

impl ConfigPaths {
    /// Create new path resolver for the given config name
    pub fn new(config_name: &str) -> Self {
        let mut resolver = Self {
            config_name: config_name.to_string(),
            search_paths: Vec::new(),
        };
        resolver.build_search_paths();
        resolver
    }

    /// Get iterator over search paths in priority order
    pub fn search_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.search_paths.iter()
    }

    fn build_search_paths(&mut self) {
        // Current directory (highest priority)
        self.add_current_dir_paths();

        // User config directory (~/.config/{name}.{json,yaml,yml})
        self.add_user_config_paths();
    }

    fn add_current_dir_paths(&mut self) {
        // JSON first (fastest parsing)
        self.search_paths
            .push(PathBuf::from(format!("{}.json", self.config_name)));

        // YAML alternatives
        self.search_paths
            .push(PathBuf::from(format!("{}.yaml", self.config_name)));
        self.search_paths
            .push(PathBuf::from(format!("{}.yml", self.config_name)));
    }

    fn add_user_config_paths(&mut self) {
        // Always check user config directory, not just with cache feature
        if let Some(config_dir) = dirs::config_dir() {
            // Check both ~/.config/{name}.{json,yaml,yml} and ~/.config/{name}/config.{json,yaml,yml}
            
            // Direct files in ~/.config/
            self.search_paths.push(config_dir.join(format!("{}.json", self.config_name)));
            self.search_paths.push(config_dir.join(format!("{}.yaml", self.config_name)));
            self.search_paths.push(config_dir.join(format!("{}.yml", self.config_name)));
            
            // Files in app-specific directory
            let app_config_dir = config_dir.join(&self.config_name);
            self.search_paths.push(app_config_dir.join("config.json"));
            self.search_paths.push(app_config_dir.join("config.yaml"));
            self.search_paths.push(app_config_dir.join("config.yml"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_generation() {
        let paths = ConfigPaths::new("myapp");
        let search_paths: Vec<_> = paths.search_paths().collect();

        // Should include current directory paths
        assert!(search_paths.iter().any(|p| p.ends_with("myapp.json")));
        assert!(search_paths.iter().any(|p| p.ends_with("myapp.yaml")));
        assert!(search_paths.iter().any(|p| p.ends_with("myapp.yml")));
    }
}
