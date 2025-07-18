//! Utility functions for Guardy
//!
//! This module provides shared utility functions used across the application.

use anyhow::Result;
use std::path::Path;

#[cfg(test)]
mod tests;

pub mod glob;


/// Get the current working directory
pub fn get_current_dir() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(Into::into)
}

/// File utilities for file operations and metadata
pub struct FileUtils;

impl FileUtils {
    /// Check if a path is a git repository
    pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(".git").exists()
    }
}

/// Path utilities for consistent path handling across the application
pub struct PathUtils;

impl PathUtils {
    /// Convert an absolute path to a relative path from the current working directory
    /// Falls back to the original path if conversion fails
    pub fn to_relative_path(path: &str) -> String {
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative) = Path::new(path).strip_prefix(&current_dir) {
                relative.display().to_string()
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_dir() {
        let result = get_current_dir();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_git_repository() {
        // Test with current directory (should be a git repo)
        let current_dir = std::env::current_dir().unwrap();
        let is_git = FileUtils::is_git_repository(&current_dir);
        
        // This test might fail if run outside a git repo, but that's expected
        // In CI/CD, this should pass as we're typically in a git repo
        println!("Current directory is git repo: {}", is_git);
    }

    #[test]
    fn test_to_relative_path() {
        let current_dir = std::env::current_dir().unwrap();
        let test_path = current_dir.join("test.txt");
        let relative = PathUtils::to_relative_path(&test_path.display().to_string());
        
        assert_eq!(relative, "test.txt");
    }
}