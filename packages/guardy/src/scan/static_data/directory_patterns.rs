//! Static directory names for efficient path filtering
//!
//! This module provides optimized directory name filtering using LazyLock and HashSet
//! for O(1) lookup performance, similar to binary_extensions.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

/// Static set of directory names for O(1) lookups
/// Uses LazyLock for one-time initialization and Arc for zero-copy sharing
pub static SKIP_DIRECTORIES_SET: LazyLock<Arc<HashSet<&'static str>>> = LazyLock::new(|| {
    Arc::new([
        // Rust
        "target",
        // Node.js/JavaScript
        "node_modules",
        "dist",
        "build",
        ".next",
        ".nuxt",
        // Python
        "__pycache__",
        ".pytest_cache",
        "venv",
        ".venv",
        "env",
        ".env",
        // Go
        "vendor",
        // Java
        "out",
        // Generic
        "cache",
        ".cache",
        "tmp",
        ".tmp",
        "temp",
        ".temp",
        // Version control
        ".git",
        ".svn",
        ".hg",
        // IDE
        ".vscode",
        ".idea",
        ".vs",
        // Coverage
        "coverage",
        ".nyc_output",
    ].into_iter().collect())
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_directories_loaded() {
        let dirs = &*SKIP_DIRECTORIES_SET;
        assert!(!dirs.is_empty());
        assert!(dirs.contains("target"));
        assert!(dirs.contains("node_modules"));
        assert!(dirs.contains(".git"));
    }

    #[test]
    fn test_directories_are_cached() {
        let dirs1 = &*SKIP_DIRECTORIES_SET;
        let dirs2 = &*SKIP_DIRECTORIES_SET;
        // Should be the same Arc reference
        assert!(Arc::ptr_eq(dirs1, dirs2));
    }
}