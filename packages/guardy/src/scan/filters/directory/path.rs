use crate::scan::types::ScannerConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use std::sync::{Arc, LazyLock};

/// Path Filter - Directory and path-based exclusion filtering
///
/// Responsibilities:
/// - Apply ignore_paths patterns for directory/file exclusions
/// - Provide O(1) path filtering using precompiled GlobSet
/// - Support gitignore-style patterns with wildcards
/// - Zero-copy sharing of compiled patterns across all threads
///
/// This filter is applied at the directory traversal stage BEFORE file content
/// is loaded, providing fast filtering to reduce I/O operations.
///
/// Performance Optimizations:
/// - Uses shared GlobSet compilation for O(1) pattern matching
/// - Avoids recompiling patterns on every filter call
/// - Thread-safe sharing via Arc<GlobSet>

/// Global shared ignore paths cache - compiled once, shared across all threads
/// 
/// This provides significant performance benefits for path filtering:
/// - GlobSet compiled only once per program execution 
/// - All threads share the same compiled patterns via Arc (zero-copy sharing)
/// - LazyLock ensures thread-safe initialization on first access
/// - Subsequent path checks are near-instant O(1) operations
/// - Loads both default patterns and custom patterns from configuration
static STATIC_IGNORE_PATHS: LazyLock<Arc<GlobSet>> = LazyLock::new(|| {
    tracing::debug!("Initializing shared ignore paths GlobSet - loading default and custom patterns");
    let start_time = std::time::Instant::now();
    
    // Step 1: Load default patterns (always available)
    let default_patterns = [
        // Test files and directories
        "tests/*",
        "testdata/*", 
        "*_test.rs",
        "test_*.rs",
        "test/**/*",
        "**/test/**/*",
        "**/tests/**/*",
        "**/*_test.*",
        "**/test_*.*",
        
        // Git objects and internal files (binary data)
        ".git/objects/**",
        ".git_disabled/**", // All of git_disabled is safe to skip
        ".git/refs/**",
        ".git/logs/**", 
        ".git/index",          // Git index file (binary)
        "**/.git/objects/**",  // Match .git/objects anywhere in path
        "**/.git_disabled/**", // Match .git_disabled anywhere in path
        
        // Common build and cache directories
        "node_modules/**/*",
        "target/**/*",
        "dist/**/*", 
        "build/**/*",
        ".cache/**/*",
        "**/.next/**/*",
        "**/node_modules/**/*",
        "**/target/**/*",
        
        // IDE and editor files
        ".vscode/**/*",
        ".idea/**/*",
        "*.swp",
        "*.swo",
        "*~",
        
        // Package manager locks and caches
        "package-lock.json",
        "yarn.lock",
        "Cargo.lock",
        ".yarn/**/*",
        ".pnpm-store/**/*",
    ];
    
    // Step 2: Try to load custom ignore patterns (optional, may fail)
    let custom_patterns = match load_custom_ignore_patterns() {
        Ok(patterns) => {
            if !patterns.is_empty() {
                tracing::info!("Loaded {} custom ignore patterns", patterns.len());
                patterns
            } else {
                Vec::new()
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load custom ignore patterns (default patterns still available): {}", e);
            Vec::new()
        }
    };
    
    // Step 3: Combine default and custom patterns
    let all_patterns: Vec<&str> = default_patterns
        .iter()
        .copied()
        .chain(custom_patterns.iter().map(|s| s.as_str()))
        .collect();
    
    match compile_glob_patterns(&all_patterns) {
        Ok(globset) => {
            let duration = start_time.elapsed();
            tracing::info!("Compiled {} total ignore patterns ({} default + {} custom) in {:?} - now cached for all threads", 
                          all_patterns.len(), default_patterns.len(), custom_patterns.len(), duration);
            Arc::new(globset)
        }
        Err(e) => {
            tracing::error!("Failed to compile ignore patterns: {}", e);
            // Try to fallback to just default patterns
            match compile_glob_patterns(&default_patterns) {
                Ok(globset) => {
                    tracing::warn!("Fallback: using only default patterns after compilation error");
                    Arc::new(globset)
                }
                Err(fallback_e) => {
                    tracing::error!("Failed to compile even default patterns: {}", fallback_e);
                    Arc::new(GlobSet::empty())
                }
            }
        }
    }
});

/// Load custom ignore patterns at runtime (used by LazyLock initialization)
fn load_custom_ignore_patterns() -> Result<Vec<String>> {
    // TODO: Implement custom ignore pattern loading from runtime config
    // This would check for:
    // - ~/.config/guardy/ignore_paths.txt
    // - --ignore-file CLI argument (if available in global config)
    // - Environment variables for custom ignore pattern paths
    // - guardy.yaml ignore_paths section
    
    let patterns = Vec::new();
    tracing::debug!("Custom ignore patterns not yet implemented");
    Ok(patterns)
}

/// Path filter for directory-level exclusions
pub struct PathFilter {
    /// Configuration reference for runtime options (patterns loaded globally now)
    config: ScannerConfig,
}

impl PathFilter {
    /// Create a new path filter with configuration
    /// 
    /// Since all patterns (default + custom) are now loaded globally in STATIC_IGNORE_PATHS,
    /// this just stores the config for any runtime options.
    pub fn new(config: &ScannerConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
    
    /// Get shared ignore patterns (includes both default + custom)
    /// 
    /// Returns the globally shared GlobSet containing all ignore patterns.
    /// This is zero-copy - just increments the Arc reference count.
    pub fn get_patterns() -> Arc<GlobSet> {
        STATIC_IGNORE_PATHS.clone()
    }
    
    /// Check if a path should be ignored
    /// 
    /// Tests the path against all patterns (default + custom) loaded in the shared GlobSet.
    /// Returns true if the path matches any ignore pattern.
    /// 
    /// # Performance
    /// - All patterns: O(1) lookup via shared GlobSet
    /// - Zero-copy access to compiled patterns
    /// - Near-instant for most common paths
    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        if STATIC_IGNORE_PATHS.is_match(&path_str) {
            tracing::trace!("Path ignored by pattern: {}", path_str);
            return true;
        }
        
        false
    }
    
    /// Apply path filtering to a list of paths
    /// 
    /// Efficiently filters a collection of paths, removing those that match
    /// ignore patterns. Returns only the paths that should be processed.
    pub fn filter_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<&P> {
        paths
            .iter()
            .filter(|path| !self.should_ignore(path.as_ref()))
            .collect()
    }
    
    /// Get statistics about pattern matching
    /// 
    /// Returns information about the compiled patterns for debugging
    /// and performance analysis.
    pub fn get_stats(&self) -> PathFilterStats {
        PathFilterStats {
            total_pattern_count: STATIC_IGNORE_PATHS.len(),
            is_using_shared_cache: true,
        }
    }
}

/// Statistics about path filter patterns
#[derive(Debug, Clone)]
pub struct PathFilterStats {
    pub total_pattern_count: usize,
    pub is_using_shared_cache: bool,
}

/// Compile a list of glob patterns into a GlobSet
/// 
/// Helper function to compile glob patterns with proper error handling
/// and performance optimization.
fn compile_glob_patterns(patterns: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|e| anyhow::anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
        builder.add(glob);
    }
    
    let globset = builder.build()
        .map_err(|e| anyhow::anyhow!("Failed to build GlobSet: {}", e))?;
    
    tracing::debug!("Compiled {} glob patterns into GlobSet", patterns.len());
    Ok(globset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_shared_patterns() {
        let patterns = PathFilter::get_patterns();
        assert!(patterns.len() > 0);
        
        // Test common ignore patterns
        assert!(patterns.is_match("tests/test_file.rs"));
        assert!(patterns.is_match("testdata/sample.txt"));
        assert!(patterns.is_match(".git/objects/abc123"));
        assert!(patterns.is_match("node_modules/package/index.js"));
        assert!(patterns.is_match("target/debug/main"));
    }
    
    #[test]
    fn test_path_filtering() {
        let config = ScannerConfig::default();
        let filter = PathFilter::new(&config).unwrap();
        
        // Test default patterns work
        assert!(filter.should_ignore(Path::new("tests/test.rs")));
        assert!(filter.should_ignore(Path::new(".git/objects/abc")));
        assert!(filter.should_ignore(Path::new("node_modules/package/index.js")));
        
        // Test that non-matching paths are allowed
        assert!(!filter.should_ignore(Path::new("src/main.rs")));
        assert!(!filter.should_ignore(Path::new("README.md")));
        assert!(!filter.should_ignore(Path::new("Cargo.toml")));
    }
    
    #[test]
    fn test_filter_paths() {
        let config = ScannerConfig::default();
        let filter = PathFilter::new(&config).unwrap();
        
        let paths = vec![
            PathBuf::from("src/main.rs"),         // Should be kept
            PathBuf::from("tests/test.rs"),       // Should be filtered
            PathBuf::from("README.md"),           // Should be kept  
            PathBuf::from(".git/objects/abc"),    // Should be filtered
            PathBuf::from("Cargo.toml"),          // Should be kept
        ];
        
        let filtered = filter.filter_paths(&paths);
        assert_eq!(filtered.len(), 3); // Only src/main.rs, README.md, Cargo.toml
        
        let filtered_paths: Vec<&PathBuf> = filtered;
        assert!(filtered_paths.contains(&&PathBuf::from("src/main.rs")));
        assert!(filtered_paths.contains(&&PathBuf::from("README.md")));
        assert!(filtered_paths.contains(&&PathBuf::from("Cargo.toml")));
    }
    
    #[test]
    fn test_stats() {
        let config = ScannerConfig::default();
        let filter = PathFilter::new(&config).unwrap();
        let stats = filter.get_stats();
        
        assert!(stats.total_pattern_count > 0);
        assert!(stats.is_using_shared_cache);
    }
    
    #[test]
    fn test_shared_patterns_performance() {
        use std::time::Instant;
        
        // Test that shared patterns provide performance benefit
        let test_paths = [
            "tests/test.rs",
            ".git/objects/abc123", 
            "node_modules/package/index.js",
            "src/main.rs",
            "README.md",
        ];
        
        // Multiple calls should be fast due to shared GlobSet
        let start = Instant::now();
        for _ in 0..1000 {
            let patterns = PathFilter::get_patterns();
            for path in &test_paths {
                let _ = patterns.is_match(path);
            }
        }
        let duration = start.elapsed();
        
        // Should complete 5000 pattern checks in reasonable time
        assert!(duration.as_millis() < 50, "Path filtering too slow: {:?}", duration);
    }
    
    #[test]
    fn test_pattern_compilation() {
        // Test that pattern compilation works with various glob patterns
        let patterns = [
            "*.txt",
            "dir/**/*",
            "**/test_*.rs",
            ".git/objects/**",
            "build/**/cache/*",
        ];
        
        let globset = compile_glob_patterns(&patterns).unwrap();
        assert_eq!(globset.len(), patterns.len());
        
        // Test that compiled patterns work
        assert!(globset.is_match("file.txt"));
        assert!(globset.is_match("dir/sub/file.rs"));
        assert!(globset.is_match("src/test_utils.rs"));
        assert!(globset.is_match(".git/objects/ab/cd123"));
        assert!(globset.is_match("build/debug/cache/file.dat"));
    }
}