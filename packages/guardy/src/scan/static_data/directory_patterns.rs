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

/// Static list of directories that should be analyzed for gitignore patterns
/// These are commonly generated build/cache directories that users should typically ignore
static ANALYZABLE_DIRECTORIES: LazyLock<Arc<Vec<(&'static str, &'static str)>>> = LazyLock::new(|| {
    Arc::new(vec![
        ("target", "Rust build directory"),
        ("node_modules", "Node.js dependencies"),
        ("dist", "Build output directory"), 
        ("build", "Build output directory"),
        ("__pycache__", "Python cache directory"),
        ("venv", "Python virtual environment"),
        (".venv", "Python virtual environment"),
        ("vendor", "Go dependencies"),
    ])
});

/// Get analyzable directories for gitignore analysis (zero-copy access)
pub fn get_analyzable_directories() -> &'static Arc<Vec<(&'static str, &'static str)>> {
    &ANALYZABLE_DIRECTORIES
}

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