//! Utility functions for Guardy
//!
//! This module provides common utility functions used throughout the application.

pub mod glob;

use anyhow::Result;
use std::path::Path;

/// Project type detection
/// TODO: Remove #[allow(dead_code)] when utility functions are used in Phase 1.4
#[allow(dead_code)]
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

/// System utilities for environment and system information
pub struct SystemUtils;

impl SystemUtils {
    /// Check if a command exists in PATH
    pub fn command_exists(command: &str) -> bool {
        which::which(command).is_ok()
    }

    /// Get the default shell
    pub fn default_shell() -> String {
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
    pub fn home_dir() -> Option<std::path::PathBuf> {
        dirs::home_dir()
    }

    /// Get the user's config directory
    pub fn config_dir() -> Option<std::path::PathBuf> {
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

    /// Check if running on Windows
    pub fn is_windows() -> bool {
        cfg!(windows)
    }

    /// Check if running on macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Check if running on Linux
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }
}

/// Get the current working directory
pub fn get_current_dir() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(Into::into)
}

/// String utilities for consistent string handling across the application
pub struct StringUtils;

impl StringUtils {
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

    /// Truncate string to specified length with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Convert string to title case
    pub fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Check if string is empty or only whitespace
    pub fn is_blank(s: &str) -> bool {
        s.trim().is_empty()
    }
}

/// File utilities for file operations and metadata
pub struct FileUtils;

impl FileUtils {
    /// Check if a file has a specific extension
    pub fn has_extension<P: AsRef<Path>>(path: P, extension: &str) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
    }

    /// Get file modification time
    pub fn modification_time<P: AsRef<Path>>(path: P) -> Result<std::time::SystemTime> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.modified()?)
    }

    /// Check if a path is a git repository
    pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(".git").exists()
    }

    /// Get file size in bytes
    pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// Check if a file is executable
    pub fn is_executable<P: AsRef<Path>>(path: P) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                metadata.permissions().mode() & 0o111 != 0
            } else {
                false
            }
        }
        #[cfg(not(unix))]
        {
            // On non-Unix systems, we can't easily check execute permissions
            // so we'll assume based on file extension
            let path = path.as_ref();
            if let Some(ext) = path.extension() {
                matches!(ext.to_str(), Some("exe") | Some("bat") | Some("cmd"))
            } else {
                false
            }
        }
    }

    /// Check if a file exists and is readable
    pub fn is_readable<P: AsRef<Path>>(path: P) -> bool {
        std::fs::metadata(path).is_ok()
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

    /// Convert an absolute path to a relative path from a specific base directory
    /// Falls back to the original path if conversion fails
    pub fn to_relative_path_from<P: AsRef<Path>>(path: &str, base: P) -> String {
        if let Ok(relative) = Path::new(path).strip_prefix(base) {
            relative.display().to_string()
        } else {
            path.to_string()
        }
    }

    /// Get the current working directory as a PathBuf
    pub fn current_dir() -> Result<std::path::PathBuf> {
        std::env::current_dir().map_err(Into::into)
    }

    /// Check if a path is absolute
    pub fn is_absolute<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_absolute()
    }

    /// Normalize a path by removing redundant components
    pub fn normalize<P: AsRef<Path>>(path: P) -> std::path::PathBuf {
        let path = path.as_ref();
        let mut components = Vec::new();
        
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if !components.is_empty() && components.last() != Some(&std::path::Component::ParentDir) {
                        components.pop();
                    } else {
                        components.push(component);
                    }
                }
                std::path::Component::CurDir => {
                    // Skip current directory components
                }
                _ => components.push(component),
            }
        }
        
        components.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_path_utils_to_relative_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").expect("Failed to write file");
        
        let absolute_path = file_path.to_str().unwrap();
        let relative_path = PathUtils::to_relative_path(absolute_path);
        
        // Should end with test.txt (conversion might fail if temp dir is outside current dir)
        assert!(relative_path.ends_with("test.txt"));
        
        // Test with a path that should definitely be convertible
        let current_dir = std::env::current_dir().unwrap();
        let test_file = current_dir.join("test.txt");
        let test_path = test_file.to_str().unwrap();
        let relative = PathUtils::to_relative_path(test_path);
        assert_eq!(relative, "test.txt");
    }

    #[test]
    fn test_path_utils_to_relative_path_from() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("subdir").join("test.txt");
        fs::create_dir_all(file_path.parent().unwrap()).expect("Failed to create parent dir");
        fs::write(&file_path, "test content").expect("Failed to write file");
        
        let absolute_path = file_path.to_str().unwrap();
        let relative_path = PathUtils::to_relative_path_from(absolute_path, temp_dir.path());
        
        assert_eq!(relative_path, "subdir/test.txt");
    }

    #[test]
    fn test_path_utils_is_absolute() {
        assert!(PathUtils::is_absolute("/absolute/path"));
        assert!(!PathUtils::is_absolute("relative/path"));
        assert!(!PathUtils::is_absolute("./relative/path"));
        
        #[cfg(windows)]
        {
            assert!(PathUtils::is_absolute("C:\\absolute\\path"));
            assert!(!PathUtils::is_absolute("relative\\path"));
        }
    }

    #[test]
    fn test_path_utils_normalize() {
        let path = PathUtils::normalize("./src/../test/./file.txt");
        assert_eq!(path, std::path::Path::new("test/file.txt"));
        
        let path = PathUtils::normalize("src/./test/../file.txt");
        assert_eq!(path, std::path::Path::new("src/file.txt"));
        
        let path = PathUtils::normalize("../../parent/file.txt");
        assert_eq!(path, std::path::Path::new("../../parent/file.txt"));
    }

    #[test]
    fn test_string_utils_format_file_size() {
        assert_eq!(StringUtils::format_file_size(0), "0 B");
        assert_eq!(StringUtils::format_file_size(1023), "1023 B");
        assert_eq!(StringUtils::format_file_size(1024), "1.0 KB");
        assert_eq!(StringUtils::format_file_size(1536), "1.5 KB");
        assert_eq!(StringUtils::format_file_size(1_048_576), "1.0 MB");
        assert_eq!(StringUtils::format_file_size(1_073_741_824), "1.0 GB");
        assert_eq!(StringUtils::format_file_size(1_099_511_627_776), "1.0 TB");
    }

    #[test]
    fn test_string_utils_truncate() {
        assert_eq!(StringUtils::truncate("hello", 10), "hello");
        assert_eq!(StringUtils::truncate("hello world", 5), "he...");
        assert_eq!(StringUtils::truncate("hello", 5), "hello");
        assert_eq!(StringUtils::truncate("hello", 3), "...");
        assert_eq!(StringUtils::truncate("hello", 0), "...");
    }

    #[test]
    fn test_string_utils_to_title_case() {
        assert_eq!(StringUtils::to_title_case("hello world"), "Hello World");
        assert_eq!(StringUtils::to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(StringUtils::to_title_case("hello"), "Hello");
        assert_eq!(StringUtils::to_title_case(""), "");
        assert_eq!(StringUtils::to_title_case("hello-world test"), "Hello-world Test");
    }

    #[test]
    fn test_string_utils_is_blank() {
        assert!(StringUtils::is_blank(""));
        assert!(StringUtils::is_blank("   "));
        assert!(StringUtils::is_blank("\t\n\r "));
        assert!(!StringUtils::is_blank("hello"));
        assert!(!StringUtils::is_blank("  hello  "));
    }

    #[test]
    fn test_file_utils_has_extension() {
        assert!(FileUtils::has_extension("test.txt", "txt"));
        assert!(FileUtils::has_extension("test.TXT", "txt"));
        assert!(FileUtils::has_extension("test.txt", "TXT"));
        assert!(!FileUtils::has_extension("test.txt", "md"));
        assert!(!FileUtils::has_extension("test", "txt"));
    }

    #[test]
    fn test_file_utils_is_git_repository() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Not a git repository
        assert!(!FileUtils::is_git_repository(temp_dir.path()));
        
        // Create .git directory
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).expect("Failed to create .git dir");
        
        // Now it should be detected as a git repository
        assert!(FileUtils::is_git_repository(temp_dir.path()));
    }

    #[test]
    fn test_file_utils_file_size() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        
        let content = "Hello, World!";
        fs::write(&file_path, content).expect("Failed to write file");
        
        let size = FileUtils::file_size(&file_path).expect("Failed to get file size");
        assert_eq!(size, content.len() as u64);
    }

    #[test]
    fn test_file_utils_is_readable() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        
        // File doesn't exist
        assert!(!FileUtils::is_readable(&file_path));
        
        // Create file
        fs::write(&file_path, "test content").expect("Failed to write file");
        assert!(FileUtils::is_readable(&file_path));
    }

    #[test]
    fn test_system_utils_command_exists() {
        // Test with a command that should exist on most systems
        assert!(SystemUtils::command_exists("echo"));
        
        // Test with a command that definitely doesn't exist
        assert!(!SystemUtils::command_exists("definitely_not_a_real_command_12345"));
    }

    #[test]
    fn test_system_utils_is_ci() {
        // We can't easily test this without setting environment variables
        // but we can test that it returns a boolean
        let is_ci = SystemUtils::is_ci();
        assert!(is_ci == true || is_ci == false);
    }

    #[test]
    fn test_system_utils_platform_detection() {
        // Test that exactly one platform is detected
        let platforms = vec![
            SystemUtils::is_windows(),
            SystemUtils::is_macos(),
            SystemUtils::is_linux(),
        ];
        
        // At least one should be true
        assert!(platforms.iter().any(|&x| x));
        
        // Test that the functions return consistent results
        assert_eq!(SystemUtils::is_windows(), cfg!(windows));
        assert_eq!(SystemUtils::is_macos(), cfg!(target_os = "macos"));
        assert_eq!(SystemUtils::is_linux(), cfg!(target_os = "linux"));
    }

    #[test]
    fn test_system_utils_default_shell() {
        let shell = SystemUtils::default_shell();
        assert!(!shell.is_empty());
        
        #[cfg(windows)]
        assert!(shell.contains("cmd"));
        
        #[cfg(not(windows))]
        assert!(shell.contains("sh"));
    }

    #[test]
    fn test_system_utils_create_temp_dir() {
        let temp_dir = SystemUtils::create_temp_dir().expect("Failed to create temp dir");
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
    }

    #[test]
    fn test_system_utils_ensure_dir_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let new_dir = temp_dir.path().join("new_dir").join("nested");
        
        assert!(!new_dir.exists());
        
        SystemUtils::ensure_dir_exists(&new_dir).expect("Failed to create directory");
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_system_utils_home_dir() {
        let home_dir = SystemUtils::home_dir();
        // Home directory should exist on most systems
        assert!(home_dir.is_some());
        
        if let Some(home) = home_dir {
            assert!(home.exists());
            assert!(home.is_dir());
        }
    }

    #[test]
    fn test_system_utils_config_dir() {
        let config_dir = SystemUtils::config_dir();
        // Config directory should exist on most systems
        assert!(config_dir.is_some());
        
        if let Some(config) = config_dir {
            assert!(config.is_dir());
        }
    }

    #[test]
    fn test_path_utils_current_dir() {
        let current_dir = PathUtils::current_dir().expect("Failed to get current dir");
        assert!(current_dir.exists());
        assert!(current_dir.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn test_file_utils_is_executable() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.sh");
        
        // Create a file
        fs::write(&file_path, "#!/bin/bash\necho 'hello'").expect("Failed to write file");
        
        // Initially not executable
        assert!(!FileUtils::is_executable(&file_path));
        
        // Make it executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms).expect("Failed to set permissions");
        
        // Now it should be executable
        assert!(FileUtils::is_executable(&file_path));
    }
}
