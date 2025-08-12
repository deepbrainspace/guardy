use guardy::scan::filters::directory::PathFilter;
use guardy::scan::types::ScannerConfig;
use std::path::{Path, PathBuf};

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
    
    let filtered_paths: Vec<&str> = filtered.iter()
        .map(|p| p.to_str().unwrap())
        .collect();
    
    assert!(filtered_paths.contains(&"src/main.rs"));
    assert!(filtered_paths.contains(&"README.md"));
    assert!(filtered_paths.contains(&"Cargo.toml"));
    assert!(!filtered_paths.contains(&"tests/test.rs"));
    assert!(!filtered_paths.contains(&".git/objects/abc"));
}

#[test]
fn test_gitignore_integration() {
    let config = ScannerConfig::default();
    let filter = PathFilter::new(&config).unwrap();
    
    // Test various gitignore-style patterns
    let test_cases = [
        ("target/debug/main", true),
        ("build/output.txt", true),
        ("*.tmp", false), // Path needs actual filename
        ("temp.tmp", true),
        ("logs/error.log", true),
        ("src/temp.tmp", true),
    ];
    
    for (path, should_ignore) in &test_cases {
        assert_eq!(
            filter.should_ignore(Path::new(path)), 
            *should_ignore,
            "Path '{}' ignore check failed", path
        );
    }
}

#[test]
fn test_custom_ignore_patterns() {
    let mut config = ScannerConfig::default();
    config.ignore_paths = vec!["custom/*.ignore".to_string()];
    let filter = PathFilter::new(&config).unwrap();
    
    // Test that custom patterns work alongside defaults
    assert!(filter.should_ignore(Path::new("custom/test.ignore")));
    assert!(filter.should_ignore(Path::new(".git/config"))); // Default still works
    assert!(!filter.should_ignore(Path::new("custom/test.keep")));
}

#[test]
fn test_performance_stats() {
    let config = ScannerConfig::default();
    let filter = PathFilter::new(&config).unwrap();
    
    let stats = filter.get_stats();
    assert!(stats.total_pattern_count > 0);
    assert!(stats.is_using_shared_cache);
}