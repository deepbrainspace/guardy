use guardy::scan::filters::directory::SizeFilter;
use guardy::scan::types::ScannerConfig;
use std::fs;
use tempfile::TempDir;

fn create_test_config(max_mb: usize, streaming_mb: usize) -> ScannerConfig {
    ScannerConfig {
        max_file_size_mb: max_mb,
        streaming_threshold_mb: streaming_mb,
        ..ScannerConfig::default()
    }
}

#[test]
fn test_size_filter_creation() {
    let config = create_test_config(50, 20);
    let filter = SizeFilter::new(&config).unwrap();
    
    let (max_mb, streaming_mb) = filter.get_limits_mb();
    assert_eq!(max_mb, 50.0);
    assert_eq!(streaming_mb, 20.0);
}

#[test]
fn test_should_filter_small_file() {
    let temp_dir = TempDir::new().unwrap();
    let small_file = temp_dir.path().join("small.txt");
    
    // Create a small file (1KB)
    let content = "x".repeat(1024);
    fs::write(&small_file, content).unwrap();
    
    let config = create_test_config(1, 1); // 1MB limit
    let filter = SizeFilter::new(&config).unwrap();
    
    assert!(!filter.should_filter(&small_file).unwrap());
}

#[test]  
fn test_should_filter_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let large_file = temp_dir.path().join("large.txt");
    
    // Create a file larger than 1MB
    let content = "x".repeat(2 * 1024 * 1024); // 2MB
    fs::write(&large_file, content).unwrap();
    
    let config = create_test_config(1, 1); // 1MB limit  
    let filter = SizeFilter::new(&config).unwrap();
    
    assert!(filter.should_filter(&large_file).unwrap());
}

#[test]
fn test_streaming_threshold() {
    let temp_dir = TempDir::new().unwrap();
    let medium_file = temp_dir.path().join("medium.txt");
    
    // Create a 15MB file  
    let content = "x".repeat(15 * 1024 * 1024);
    fs::write(&medium_file, content).unwrap();
    
    let config = create_test_config(50, 10); // 50MB max, 10MB streaming threshold
    let filter = SizeFilter::new(&config).unwrap();
    
    // Should not be filtered (under 50MB limit)
    assert!(!filter.should_filter(&medium_file).unwrap());
    
    // Should use streaming (over 10MB threshold)
    assert!(filter.should_use_streaming(&medium_file).unwrap());
}

#[test]
fn test_get_file_size() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    let content = "x".repeat(5000); // 5KB
    fs::write(&test_file, content).unwrap();
    
    let config = create_test_config(10, 5);
    let filter = SizeFilter::new(&config).unwrap();
    
    let size_bytes = filter.get_file_size(&test_file).unwrap();
    assert_eq!(size_bytes, 5000);
    
    let size_mb = filter.get_file_size_mb(&test_file).unwrap();
    assert!((size_mb - 0.00476).abs() < 0.001); // ~0.00476 MB
}

#[test]
fn test_filter_paths() {
    let temp_dir = TempDir::new().unwrap();
    let small_file = temp_dir.path().join("small.txt");
    let large_file = temp_dir.path().join("large.txt");
    
    fs::write(&small_file, "small content").unwrap();
    fs::write(&large_file, "x".repeat(2 * 1024 * 1024)).unwrap(); // 2MB
    
    let config = create_test_config(1, 1); // 1MB limit
    let filter = SizeFilter::new(&config).unwrap();
    
    let paths = vec![&small_file, &large_file];
    let filtered = filter.filter_paths(&paths);
    
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0], &small_file);
}

#[test]
fn test_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    
    fs::write(&file1, "small").unwrap();
    fs::write(&file2, "x".repeat(2 * 1024 * 1024)).unwrap(); // 2MB
    
    let config = create_test_config(1, 1); // 1MB limit
    let filter = SizeFilter::new(&config).unwrap();
    
    // Check both files
    let _ = filter.should_filter(&file1).unwrap();
    let _ = filter.should_filter(&file2).unwrap();
    
    // Statistics should be tracked
    let stats = filter.get_stats();
    assert_eq!(stats.files_checked, 2);
    assert_eq!(stats.files_filtered, 1); // file2 is too large
    assert_eq!(stats.bytes_filtered, 2 * 1024 * 1024); // 2MB
}

#[test]
fn test_zero_limits() {
    let config = create_test_config(0, 0); // Zero limits
    let filter = SizeFilter::new(&config).unwrap();
    
    let (max_mb, streaming_mb) = filter.get_limits_mb();
    assert_eq!(max_mb, 0.0);
    assert_eq!(streaming_mb, 0.0);
}

#[test]
fn test_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    let empty_file = temp_dir.path().join("empty.txt");
    let exact_limit_file = temp_dir.path().join("exact.txt");
    
    // Create empty file
    fs::write(&empty_file, "").unwrap();
    
    // Create file exactly at limit (1MB)
    let content = "x".repeat(1024 * 1024); // Exactly 1MB
    fs::write(&exact_limit_file, content).unwrap();
    
    let config = create_test_config(1, 1); // 1MB limit
    let filter = SizeFilter::new(&config).unwrap();
    
    // Empty file should not be filtered
    assert!(!filter.should_filter(&empty_file).unwrap());
    
    // File at exact limit should not be filtered
    assert!(!filter.should_filter(&exact_limit_file).unwrap());
}