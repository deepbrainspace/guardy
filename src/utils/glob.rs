//! Glob pattern utilities
//!
//! This module provides unified glob pattern matching functionality for file discovery

use anyhow::Result;
use globset::Glob;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Expand a list of file patterns (supporting both literal paths and glob patterns)
/// into a list of actual file paths
pub fn expand_file_patterns<P: AsRef<Path>>(
    patterns: &[String], 
    base_dir: P
) -> Result<Vec<PathBuf>> {
    let mut valid_paths = Vec::new();
    let base_dir = base_dir.as_ref();
    
    for pattern in patterns {
        if is_glob_pattern(pattern) {
            // Expand glob pattern
            let glob_paths = expand_glob_pattern(pattern, base_dir)?;
            valid_paths.extend(glob_paths);
        } else {
            // Regular file path
            let path = if Path::new(pattern).is_absolute() {
                PathBuf::from(pattern)
            } else {
                base_dir.join(pattern)
            };
            
            if path.exists() && path.is_file() {
                valid_paths.push(path);
            }
        }
    }
    
    Ok(valid_paths)
}

/// Check if a string contains glob pattern characters
pub fn is_glob_pattern(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}

/// Expand a single glob pattern to matching file paths
pub fn expand_glob_pattern<P: AsRef<Path>>(
    pattern: &str, 
    base_dir: P
) -> Result<Vec<PathBuf>> {
    let mut matching_paths = Vec::new();
    let base_dir = base_dir.as_ref();
    
    let glob = Glob::new(pattern)?;
    let matcher = glob.compile_matcher();
    
    // Walk the base directory and match files
    for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            // Check both absolute and relative paths
            let matches = matcher.is_match(path) ||
                (path.strip_prefix(base_dir).ok()
                    .map(|rel_path| matcher.is_match(rel_path))
                    .unwrap_or(false));
            
            if matches {
                matching_paths.push(path.to_path_buf());
            }
        }
    }
    
    Ok(matching_paths)
}

/// Create a GlobSet from a list of patterns for efficient batch matching
pub fn build_globset(patterns: &[String]) -> Result<globset::GlobSet> {
    build_globset_with_options(patterns, false)
}

/// Create a GlobSet with options for transforming ignore-style patterns
pub fn build_globset_with_options(patterns: &[String], transform_directory_patterns: bool) -> Result<globset::GlobSet> {
    use globset::GlobSetBuilder;
    
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let processed_pattern = if transform_directory_patterns && pattern.ends_with('/') {
            // For ignore patterns like "target/", add "**" to match all files under it
            format!("{}**", pattern)
        } else {
            pattern.clone()
        };
        
        let glob = Glob::new(&processed_pattern)?;
        builder.add(glob);
    }
    
    Ok(builder.build()?)
}

/// Process ignore patterns from file content (handles comments, empty lines, directory patterns)
pub fn process_ignore_patterns(content: &str, transform_directory_patterns: bool) -> Vec<String> {
    let mut patterns = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Convert directory patterns to glob patterns if requested
        let pattern = if transform_directory_patterns && line.ends_with('/') {
            // For patterns like "target/", add "**" to match all files under it
            format!("{}**", line)
        } else {
            line.to_string()
        };
        
        patterns.push(pattern);
    }
    
    patterns
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_is_glob_pattern() {
        assert!(is_glob_pattern("*.rs"));
        assert!(is_glob_pattern("src/**/*.js"));
        assert!(is_glob_pattern("test?.txt"));
        assert!(is_glob_pattern("file[123].txt"));
        assert!(!is_glob_pattern("simple.txt"));
        assert!(!is_glob_pattern("path/to/file.rs"));
    }

    #[test]
    fn test_expand_file_patterns() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();
        
        // Create test files
        fs::write(base_path.join("test1.rs"), "// test")?;
        fs::write(base_path.join("test2.js"), "// test")?;
        fs::write(base_path.join("readme.md"), "# readme")?;
        
        // Test literal file patterns
        let patterns = vec!["test1.rs".to_string(), "readme.md".to_string()];
        let results = expand_file_patterns(&patterns, base_path)?;
        assert_eq!(results.len(), 2);
        
        // Test glob patterns
        let patterns = vec!["*.rs".to_string()];
        let results = expand_file_patterns(&patterns, base_path)?;
        assert_eq!(results.len(), 1);
        assert!(results[0].file_name().unwrap().to_str().unwrap().contains("test1.rs"));
        
        Ok(())
    }
}