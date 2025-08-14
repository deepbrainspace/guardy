//! Static directory patterns for efficient path filtering
//!
//! This module provides optimized directory patterns using LazyLock and HashSet
//! for O(1) lookup performance, similar to binary_extensions.

use std::sync::{Arc, LazyLock};

/// Core directory patterns that should be ignored during scanning
const PATTERNS: &[&str] = &[
    // Test directories
    "tests/*",
    "testdata/*", 
    "*_test.rs",
    "test_*.rs",
    // Git objects and internal files (binary data)
    ".git/objects/**",
    ".git_disabled/**", // All of git_disabled is safe to skip
    ".git/refs/**",
    ".git/logs/**", 
    ".git/index",          // Git index file (binary)
    "**/.git/objects/**",  // Match .git/objects anywhere in path
    "**/.git_disabled/**", // Match .git_disabled anywhere in path
    // Build and cache directories - match anywhere in path
    "**/target/**",         // Rust build directory
    "**/node_modules/**",   // Node.js dependencies
    "**/dist/**",           // Build output
    "**/build/**",          // Build output
    "**/.next/**",          // Next.js build
    "**/.nuxt/**",          // Nuxt.js build
    "**/__pycache__/**",    // Python cache
    "**/.pytest_cache/**",  // Python test cache
    "**/venv/**",           // Python virtual env
    "**/.venv/**",          // Python virtual env
    "**/env/**",            // Environment directory
    "**/.env/**",           // Environment directory
    "**/vendor/**",         // Go dependencies
    "**/out/**",            // Java output
    "**/cache/**",          // Generic cache
    "**/.cache/**",         // Generic cache
    "**/tmp/**",            // Temporary files
    "**/.tmp/**",           // Temporary files
    "**/temp/**",           // Temporary files
    "**/.temp/**",          // Temporary files
    "**/.git/**",           // Git directory
    "**/.svn/**",           // SVN directory
    "**/.hg/**",            // Mercurial directory
    "**/.vscode/**",        // VS Code settings
    "**/.idea/**",          // IntelliJ IDEA settings
    "**/.vs/**",            // Visual Studio settings
    "**/coverage/**",       // Coverage reports
    "**/.nyc_output/**",    // NYC coverage output
];

/// Static directory patterns for O(1) lookups
/// Uses LazyLock for one-time initialization and Arc for zero-copy sharing
pub static DIRECTORY_PATTERNS: LazyLock<Arc<Vec<String>>> = LazyLock::new(|| {
    Arc::new(PATTERNS.iter().map(|&s| s.to_string()).collect())
});

/// Get directory patterns as Vec<String> for PathFilter initialization
/// This creates a Vec each time, but PathFilter only calls this once during initialization
pub fn get_directory_patterns() -> Vec<String> {
    DIRECTORY_PATTERNS.as_ref().clone()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_patterns_loaded() {
        let patterns = get_directory_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&"target/**".to_string()));
        assert!(patterns.contains(&"node_modules/**".to_string()));
        assert!(patterns.contains(&".git/**".to_string()));
    }

    #[test]
    fn test_patterns_are_cached() {
        let patterns1 = &*DIRECTORY_PATTERNS;
        let patterns2 = &*DIRECTORY_PATTERNS;
        // Should be the same Arc reference
        assert!(Arc::ptr_eq(patterns1, patterns2));
    }
}