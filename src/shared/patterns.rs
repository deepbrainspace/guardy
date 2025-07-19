//! Pattern matching utilities
//!
//! This module provides pattern matching functionality for file matching,
//! glob patterns, and other pattern-based operations.

/// Find files that match formatter patterns
pub fn find_matching_files(files: &[String], patterns: &[String]) -> Vec<String> {
    let mut matching_files = Vec::new();
    
    for file in files {
        for pattern in patterns {
            if glob_match(pattern, file) {
                matching_files.push(file.clone());
                break;
            }
        }
    }
    
    matching_files
}

/// Simple glob matching for file patterns
pub fn glob_match(pattern: &str, file: &str) -> bool {
    // Convert simple glob patterns to regex
    let mut regex_pattern = pattern.to_string();
    
    // Replace ** with .* (matches anything including path separators)
    regex_pattern = regex_pattern.replace("**", "DOUBLE_STAR");
    // Replace * with [^/]* (matches anything except path separators)
    regex_pattern = regex_pattern.replace("*", "[^/]*");
    // Replace the placeholder back with .*
    regex_pattern = regex_pattern.replace("DOUBLE_STAR", ".*");
    // Replace ? with [^/] (matches single character except path separator)
    regex_pattern = regex_pattern.replace("?", "[^/]");
    
    if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
        regex.is_match(file)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("**/*.rs", "src/main.rs"));
        assert!(glob_match("**/*.rs", "lib/utils/mod.rs"));
        assert!(glob_match("*.js", "index.js"));
        assert!(!glob_match("*.js", "src/main.rs"));
        assert!(!glob_match("**/*.rs", "Cargo.toml"));
    }

    #[test]
    fn test_find_matching_files() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "index.js".to_string(),
            "package.json".to_string(),
        ];
        
        let rust_patterns = vec!["**/*.rs".to_string()];
        let matching_rust = find_matching_files(&files, &rust_patterns);
        assert_eq!(matching_rust.len(), 2);
        assert!(matching_rust.contains(&"src/main.rs".to_string()));
        assert!(matching_rust.contains(&"src/lib.rs".to_string()));
    }
}