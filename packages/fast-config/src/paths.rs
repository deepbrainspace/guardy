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

        // Git repository root
        self.add_git_repo_paths();

        // User config directory (lowest priority)
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

    fn add_git_repo_paths(&mut self) {
        if let Ok(repo_root) = self.find_git_root() {
            // Repository root
            self.search_paths
                .push(repo_root.join(format!("{}.json", self.config_name)));
            self.search_paths
                .push(repo_root.join(format!("{}.yaml", self.config_name)));
            self.search_paths
                .push(repo_root.join(format!("{}.yml", self.config_name)));

            // .config subdirectory
            let config_dir = repo_root.join(".config").join(&self.config_name);
            self.search_paths.push(config_dir.join("config.json"));
            self.search_paths.push(config_dir.join("config.yaml"));
            self.search_paths.push(config_dir.join("config.yml"));
        }
    }

    fn add_user_config_paths(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join(&self.config_name);
            self.search_paths.push(app_config_dir.join("config.json"));
            self.search_paths.push(app_config_dir.join("config.yaml"));
            self.search_paths.push(app_config_dir.join("config.yml"));
        }
    }

    fn find_git_root(&self) -> Result<PathBuf, std::io::Error> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(PathBuf::from(stdout.trim()))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not in a git repository",
            ))
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
