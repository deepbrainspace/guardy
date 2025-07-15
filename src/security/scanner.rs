//! Secret scanner implementation
//!
//! This module provides the core secret scanning functionality.

use super::patterns::patterns_from_config;
use super::{SecurityMatch, SecurityPattern, Severity};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Secret scanner for detecting secrets in files
pub struct SecretScanner {
    patterns: Vec<SecurityPattern>,
    exclude_patterns: Vec<String>,
}

impl SecretScanner {
    /// Create a new secret scanner with patterns from configuration
    pub fn from_config(config: &crate::config::GuardyConfig) -> Result<Self> {
        let patterns = patterns_from_config(&config.security.patterns)?;
        let exclude_patterns = config.get_effective_exclude_patterns();

        Ok(Self {
            patterns,
            exclude_patterns,
        })
    }

    /// Create a new secret scanner with default patterns (deprecated - use from_config instead)
    pub fn new() -> Result<Self> {
        let config = crate::config::GuardyConfig::default();
        Self::from_config(&config)
    }

    /// Add a custom pattern
    pub fn add_pattern(&mut self, pattern: SecurityPattern) {
        self.patterns.push(pattern);
    }

    /// Add exclude pattern
    pub fn add_exclude_pattern(&mut self, pattern: String) {
        self.exclude_patterns.push(pattern);
    }

    /// Scan a single file for secrets
    pub fn scan_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<SecurityMatch>> {
        let path = file_path.as_ref();
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
        let path_str = path.to_string_lossy();

        // Check if file should be excluded
        for exclude_pattern in &self.exclude_patterns {
            if path_str.contains(exclude_pattern) {
                return false;
            }
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
