//! Secret scanner implementation
//!
//! This module provides the core secret scanning functionality.

use super::patterns::patterns_from_config;
use super::{SecurityMatch, SecurityPattern};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Secret scanner for detecting secrets in files
pub struct SecretScanner {
    patterns: Vec<SecurityPattern>,
    exclude_globset: GlobSet,
    verbose: bool,
}

impl SecretScanner {
    /// Create a new secret scanner with patterns from configuration
    pub fn from_config(config: &crate::config::GuardyConfig, output: &crate::cli::Output) -> Result<Self> {
        let patterns = patterns_from_config(&config.security.patterns)?;
        let exclude_patterns = config.get_effective_exclude_patterns();

        // Build GlobSet from exclude patterns
        let mut builder = GlobSetBuilder::new();
        if output.is_verbose() {
            output.verbose(&format!("Loading {} exclude patterns", exclude_patterns.len()));
        }
        for pattern in &exclude_patterns {
            let glob = Glob::new(pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            builder.add(glob);
        }
        let exclude_globset = builder.build()
            .with_context(|| "Failed to build exclude pattern globset")?;

        Ok(Self {
            patterns,
            exclude_globset,
            verbose: output.is_verbose(),
        })
    }

    /// Create a new secret scanner with default patterns (deprecated - use from_config instead)
    pub fn new() -> Result<Self> {
        let config = crate::config::GuardyConfig::default();
        let output = crate::cli::Output::new(false, false); // No verbose, no quiet for default
        Self::from_config(&config, &output)
    }

    /// Add a custom pattern
    #[allow(dead_code)]
    pub fn add_pattern(&mut self, pattern: SecurityPattern) {
        self.patterns.push(pattern);
    }


    /// Scan a single file for secrets
    pub fn scan_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<SecurityMatch>> {
        let path = file_path.as_ref();
        
        // Check if file should be scanned
        if !self.should_scan_file(path) {
            if self.verbose {
                println!("Skipping excluded file: {}", path.display());
            }
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let mut matches = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                for mat in pattern.regex.find_iter(line) {
                    matches.push(SecurityMatch {
                        file_path: path.display().to_string(),
                        line_number: line_num + 1,
                        column: mat.start() + 1,
                        content: mat.as_str().to_string(),
                        pattern_name: pattern.name.clone(),
                        severity: pattern.severity.clone(),
                    });
                }
            }
        }

        Ok(matches)
    }

    /// Scan multiple files for secrets
    pub fn scan_files<P: AsRef<Path>>(&self, files: &[P]) -> Result<Vec<SecurityMatch>> {
        let mut all_matches = Vec::new();

        for file_path in files {
            if self.should_scan_file(file_path.as_ref()) {
                let matches = self.scan_file(file_path)?;
                all_matches.extend(matches);
            }
        }

        Ok(all_matches)
    }

    /// Scan a directory recursively for secrets
    pub fn scan_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<SecurityMatch>> {
        let mut all_matches = Vec::new();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_file() && self.should_scan_file(path) {
                let matches = self.scan_file(path)?;
                all_matches.extend(matches);
            }
        }

        Ok(all_matches)
    }

    /// Check if a file should be scanned based on exclude patterns
    fn should_scan_file(&self, path: &Path) -> bool {
        if self.verbose {
            println!("Checking file: {}", path.display());
        }
        
        // Check absolute path
        if self.exclude_globset.is_match(path) {
            if self.verbose {
                println!("  -> Excluded by absolute path match");
            }
            return false;
        }

        // Check relative path for patterns like ".claude/**/*.md"
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative_path) = path.strip_prefix(current_dir) {
                if self.verbose {
                    println!("  -> Relative path: {}", relative_path.display());
                }
                if self.exclude_globset.is_match(relative_path) {
                    if self.verbose {
                        println!("  -> Excluded by relative path match");
                    }
                    return false;
                }
            }
        }

        // Check if any parent directory matches the patterns
        // This handles directory patterns like "target/" and "node_modules/"
        if let Some(parent) = path.parent() {
            if self.exclude_globset.is_match(parent) {
                if self.verbose {
                    println!("  -> Excluded by parent directory match");
                }
                return false;
            }
            
            // Also check relative parent paths
            if let Ok(current_dir) = std::env::current_dir() {
                if let Ok(relative_parent) = parent.strip_prefix(current_dir) {
                    if self.exclude_globset.is_match(relative_parent) {
                        if self.verbose {
                            println!("  -> Excluded by relative parent match");
                        }
                        return false;
                    }
                }
            }
        }

        if self.verbose {
            println!("  -> File will be scanned");
        }

        // Only scan text files
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(
                ext.as_str(),
                "rs" | "js"
                    | "ts"
                    | "py"
                    | "go"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "scala"
                    | "sh"
                    | "bash"
                    | "zsh"
                    | "fish"
                    | "ps1"
                    | "bat"
                    | "cmd"
                    | "yaml"
                    | "yml"
                    | "json"
                    | "xml"
                    | "toml"
                    | "ini"
                    | "cfg"
                    | "conf"
                    | "config"
                    | "env"
                    | "txt"
                    | "md"
                    | "rst"
                    | "sql"
                    | "dockerfile"
                    | "makefile"
                    | "gradle"
                    | "maven"
                    | "pom"
                    | "build"
                    | "cmake"
                    | "meson"
            )
        } else {
            // Check for common files without extensions
            if let Some(filename) = path.file_name() {
                let name = filename.to_string_lossy().to_lowercase();
                matches!(
                    name.as_str(),
                    "dockerfile"
                        | "makefile"
                        | "cmakelists.txt"
                        | "readme"
                        | "license"
                        | "changelog"
                        | "todo"
                        | "authors"
                        | "contributors"
                )
            } else {
                false
            }
        }
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new().expect("Failed to create default secret scanner")
    }
}
