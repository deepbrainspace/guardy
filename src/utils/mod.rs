//! Utility functions for Guardy
//!
//! This module provides common utility functions used throughout the application.

use anyhow::Result;
use std::path::Path;

/// Project type detection
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    NodeJs,
    Python,
    Go,
    NxMonorepo,
    Generic,
}

/// Detect project type based on files in the directory
pub fn detect_project_type<P: AsRef<Path>>(path: P) -> ProjectType {
    let path = path.as_ref();

    // Check for NX monorepo first (most specific)
    if path.join("nx.json").exists() {
        return ProjectType::NxMonorepo;
    }

    // Check for Rust project
    if path.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }

    // Check for Node.js project
    if path.join("package.json").exists() {
        return ProjectType::NodeJs;
    }

    // Check for Python project
    if path.join("pyproject.toml").exists()
        || path.join("requirements.txt").exists()
        || path.join("setup.py").exists()
    {
        return ProjectType::Python;
    }

    // Check for Go project
    if path.join("go.mod").exists() {
        return ProjectType::Go;
    }

    ProjectType::Generic
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Get the default shell
pub fn get_default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "cmd".to_string()
        } else {
            "/bin/sh".to_string()
        }
    })
}

/// Create a temporary directory
pub fn create_temp_dir() -> Result<tempfile::TempDir> {
    tempfile::tempdir().map_err(Into::into)
}

/// Ensure a directory exists
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Get the user's home directory
pub fn get_home_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir()
}

/// Get the user's config directory
pub fn get_config_dir() -> Option<std::path::PathBuf> {
    dirs::config_dir()
}

/// Check if running in a CI environment
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("TRAVIS").is_ok()
        || std::env::var("CIRCLECI").is_ok()
}

/// Get the current working directory
pub fn get_current_dir() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(Into::into)
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Check if a file has a specific extension
pub fn has_extension<P: AsRef<Path>>(path: P, extension: &str) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

/// Get file modification time
pub fn get_file_mtime<P: AsRef<Path>>(path: P) -> Result<std::time::SystemTime> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.modified()?)
}

/// Check if a path is a git repository
pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().join(".git").exists()
}

/// Truncate string to specified length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
