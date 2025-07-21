use std::path::Path;
use console;

/// Directory handling for the scanner - combines filtering and analysis logic
#[derive(Debug)]
pub struct DirectoryHandler {
    /// Rust-specific directories
    pub rust: &'static [&'static str],
    /// Node.js/JavaScript directories
    pub nodejs: &'static [&'static str],
    /// Python directories
    pub python: &'static [&'static str],
    /// Go directories
    pub go: &'static [&'static str],
    /// Java directories
    pub java: &'static [&'static str],
    /// Generic build/cache directories
    pub generic: &'static [&'static str],
    /// Version control directories
    pub vcs: &'static [&'static str],
    /// IDE directories
    pub ide: &'static [&'static str],
    /// Test coverage directories
    pub coverage: &'static [&'static str],
}

impl DirectoryHandler {
    /// Get the default directory handler
    pub fn default() -> Self {
        Self {
            rust: &["target"],
            nodejs: &["node_modules", "dist", "build", ".next", ".nuxt"],
            python: &["__pycache__", ".pytest_cache", "venv", ".venv", "env", ".env"],
            go: &["vendor"],
            java: &["out"],
            generic: &["cache", ".cache", "tmp", ".tmp", "temp", ".temp"],
            vcs: &[".git", ".svn", ".hg"],
            ide: &[".vscode", ".idea", ".vs"],
            coverage: &["coverage", ".nyc_output"],
        }
    }

    /// Get all directory names that should be filtered during scanning
    pub fn all_filtered_directories(&self) -> Vec<&'static str> {
        let mut dirs = Vec::new();
        dirs.extend_from_slice(self.rust);
        dirs.extend_from_slice(self.nodejs);
        dirs.extend_from_slice(self.python);
        dirs.extend_from_slice(self.go);
        dirs.extend_from_slice(self.java);
        dirs.extend_from_slice(self.generic);
        dirs.extend_from_slice(self.vcs);
        dirs.extend_from_slice(self.ide);
        dirs.extend_from_slice(self.coverage);
        dirs
    }

    /// Check if a directory name should be filtered (skipped) during scanning
    pub fn should_filter_directory(&self, dir_name: &str) -> bool {
        self.all_filtered_directories().contains(&dir_name)
    }

    /// Get directories that should be analyzed for gitignore patterns
    /// These are typically build/cache directories that projects generate
    pub fn analyzable_directories(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            // Rust
            ("target", "Rust build directory"),
            // Node.js
            ("node_modules", "Node.js dependencies"),
            ("dist", "Build output directory"),
            ("build", "Build output directory"),
            // Python
            ("__pycache__", "Python cache directory"),
            ("venv", "Python virtual environment"),
            (".venv", "Python virtual environment"),
            // Go
            ("vendor", "Go dependencies"),
        ]
    }

    /// Analyze directories in the given path and return analysis results
    pub fn analyze_directories(&self, path: &Path) -> DirectoryAnalysis {
        let mut properly_ignored = Vec::new();
        let mut needs_gitignore = Vec::new();
        
        // Helper function to check if a pattern exists in gitignore
        let check_gitignore_pattern = |pattern: &str| -> bool {
            if let Ok(gitignore_content) = std::fs::read_to_string(path.join(".gitignore")) {
                gitignore_content.lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .any(|line| line == pattern || line == pattern.trim_end_matches('/'))
            } else {
                false
            }
        };
        
        // Check all analyzable directories
        for (dir_name, description) in self.analyzable_directories() {
            if path.join(dir_name).exists() {
                let dir_with_slash = format!("{}/", dir_name);
                if check_gitignore_pattern(&dir_with_slash) || check_gitignore_pattern(dir_name) {
                    properly_ignored.push((dir_with_slash, description.to_string()));
                } else {
                    needs_gitignore.push((dir_with_slash, description.to_string()));
                }
            }
        }
        
        DirectoryAnalysis {
            properly_ignored,
            needs_gitignore,
        }
    }
}

/// Directory analysis results
#[derive(Debug)]
pub struct DirectoryAnalysis {
    pub properly_ignored: Vec<(String, String)>,
    pub needs_gitignore: Vec<(String, String)>,
}

impl DirectoryAnalysis {
    /// Display the analysis results to the user
    pub fn display(&self) {
        if self.properly_ignored.is_empty() && self.needs_gitignore.is_empty() {
            return;
        }
        
        let total_dirs = self.properly_ignored.len() + self.needs_gitignore.len();
        println!("📁 Discovered {} director{}:", 
                 total_dirs, 
                 if total_dirs == 1 { "y" } else { "ies" });
        
        // Show properly ignored directories
        for (dir, description) in &self.properly_ignored {
            println!("   ✅ {} ({})", 
                console::style(dir).green(),
                console::style(description).dim()
            );
        }
        
        // Show directories that need gitignore rules
        for (dir, description) in &self.needs_gitignore {
            println!("   ⚠️  {} ({})", 
                console::style(dir).yellow(),
                console::style(description).dim()
            );
        }
        
        // Only show gitignore recommendations for directories that need them
        if !self.needs_gitignore.is_empty() {
            let patterns: Vec<&str> = self.needs_gitignore.iter().map(|(dir, _)| dir.as_str()).collect();
            println!("💡 Consider adding to .gitignore: {}", 
                     console::style(patterns.join(", ")).cyan());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_directory_handler() {
        let handler = DirectoryHandler::default();
        
        // Test some known directories
        assert!(handler.should_filter_directory("target"));
        assert!(handler.should_filter_directory("node_modules"));
        assert!(handler.should_filter_directory("__pycache__"));
        assert!(handler.should_filter_directory(".git"));
        
        // Test non-filtered directory
        assert!(!handler.should_filter_directory("src"));
        assert!(!handler.should_filter_directory("lib"));
    }

    #[test]
    fn test_analyzable_directories() {
        let handler = DirectoryHandler::default();
        let analyzable = handler.analyzable_directories();
        
        // Should include common build directories
        assert!(analyzable.iter().any(|(name, _)| *name == "target"));
        assert!(analyzable.iter().any(|(name, _)| *name == "node_modules"));
        assert!(analyzable.iter().any(|(name, _)| *name == "__pycache__"));
    }

    #[test]
    fn test_directory_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let handler = DirectoryHandler::default();
        
        // Create a target directory (not in gitignore)
        fs::create_dir(temp_dir.path().join("target")).unwrap();
        
        let analysis = handler.analyze_directories(temp_dir.path());
        
        // Should find target directory that needs gitignore
        assert_eq!(analysis.properly_ignored.len(), 0);
        assert_eq!(analysis.needs_gitignore.len(), 1);
        assert_eq!(analysis.needs_gitignore[0].0, "target/");
        assert_eq!(analysis.needs_gitignore[0].1, "Rust build directory");
    }

    #[test]
    fn test_directory_analysis_with_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let handler = DirectoryHandler::default();
        
        // Create target directory and gitignore file
        fs::create_dir(temp_dir.path().join("target")).unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "target/\n").unwrap();
        
        let analysis = handler.analyze_directories(temp_dir.path());
        
        // Should find properly ignored target directory
        assert_eq!(analysis.properly_ignored.len(), 1);
        assert_eq!(analysis.needs_gitignore.len(), 0);
        assert_eq!(analysis.properly_ignored[0].0, "target/");
        assert_eq!(analysis.properly_ignored[0].1, "Rust build directory");
    }
}