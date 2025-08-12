use guardy::scan::filters::directory::BinaryFilter;
use guardy::scan::types::ScannerConfig;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_test_config(include_binary: bool) -> ScannerConfig {
    ScannerConfig {
        include_binary,
        ..ScannerConfig::default()
    }
}

#[test]
fn test_shared_extensions() {
    let extensions = BinaryFilter::get_extensions();
    assert!(!extensions.is_empty());
    
    // Test common binary extensions
    assert!(extensions.contains("exe"));
    assert!(extensions.contains("jpg"));
    assert!(extensions.contains("png"));
    assert!(extensions.contains("zip"));
    assert!(extensions.contains("dll"));
    assert!(extensions.contains("so"));
    assert!(extensions.contains("dylib"));
    
    // Test that text extensions are not included
    assert!(!extensions.contains("txt"));
    assert!(!extensions.contains("rs"));
    assert!(!extensions.contains("js"));
    assert!(!extensions.contains("py"));
}

#[test]
fn test_binary_filter_creation() {
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    let (include_binary, ext_count) = filter.get_config_info();
    assert!(!include_binary);
    assert!(ext_count > 0);
}

#[test]
fn test_extension_detection() {
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Test binary extensions
    assert!(filter.is_binary_file(Path::new("test.exe")).unwrap());
    assert!(filter.is_binary_file(Path::new("image.jpg")).unwrap());
    assert!(filter.is_binary_file(Path::new("archive.zip")).unwrap());
    assert!(filter.is_binary_file(Path::new("library.dll")).unwrap());
    
    // Test non-binary extensions (content inspection will be used)
    // Note: These will perform content inspection since they're not in binary_extensions
    // For pure unit testing without files, we can't easily test content inspection
}

#[test] 
fn test_should_filter_logic() {
    // Test with include_binary = false (filter binary files)
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Binary files should be filtered out
    assert!(filter.should_filter(Path::new("test.exe")).unwrap());
    assert!(filter.should_filter(Path::new("image.png")).unwrap());
    
    // Test with include_binary = true (include binary files)
    let config = create_test_config(true);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Binary files should not be filtered out
    assert!(!filter.should_filter(Path::new("test.exe")).unwrap());
    assert!(!filter.should_filter(Path::new("image.png")).unwrap());
}

#[test]
fn test_content_inspection() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a text file
    let text_file = temp_dir.path().join("test.unknown");
    fs::write(&text_file, "This is text content\nwith multiple lines").unwrap();
    
    // Create a binary file
    let binary_file = temp_dir.path().join("test.unknown_binary");
    let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    fs::write(&binary_file, binary_data).unwrap();
    
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Text file should not be detected as binary
    assert!(!filter.is_binary_file(&text_file).unwrap());
    
    // Binary file should be detected as binary
    assert!(filter.is_binary_file(&binary_file).unwrap());
}

#[test]
fn test_filter_paths() {
    let temp_dir = TempDir::new().unwrap();
    let text_file = temp_dir.path().join("text.txt");
    let binary_file = temp_dir.path().join("binary.exe");
    let unknown_text = temp_dir.path().join("unknown.dat");
    
    // Create files
    fs::write(&text_file, "text content").unwrap();
    fs::write(&binary_file, "binary content").unwrap(); // .exe will be detected by extension
    fs::write(&unknown_text, "text in unknown extension").unwrap(); // Will use content inspection
    
    let config = create_test_config(false); // Filter binary files
    let filter = BinaryFilter::new(&config).unwrap();
    
    let paths = vec![&text_file, &binary_file, &unknown_text];
    let filtered = filter.filter_paths(&paths);
    
    // Should include text files and exclude binary files
    assert!(filtered.len() >= 1); // At least the text file
    assert!(filtered.contains(&&text_file));
    assert!(!filtered.contains(&&binary_file)); // Binary should be filtered out
    // unknown_text may or may not be included depending on content inspection
}

#[test]
fn test_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let exe_file = temp_dir.path().join("test.exe");
    let txt_file = temp_dir.path().join("test.txt");
    
    fs::write(&exe_file, "fake exe").unwrap();
    fs::write(&txt_file, "text content").unwrap();
    
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Check both files
    let _ = filter.is_binary_file(&exe_file).unwrap();
    let _ = filter.is_binary_file(&txt_file).unwrap();
    
    let stats = filter.get_stats();
    assert_eq!(stats.files_checked, 2);
    assert_eq!(stats.binary_files_detected, 1); // exe file
    assert_eq!(stats.extension_hits, 1); // exe detected by extension
    assert_eq!(stats.content_inspections, 0); // No content inspections for known extensions
}

#[test]
fn test_case_insensitive_extensions() {
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Test different cases
    assert!(filter.is_binary_file(Path::new("test.EXE")).unwrap());
    assert!(filter.is_binary_file(Path::new("image.JPG")).unwrap());
    assert!(filter.is_binary_file(Path::new("archive.ZIP")).unwrap());
    assert!(filter.is_binary_file(Path::new("library.DLL")).unwrap());
}

#[test]
fn test_no_extension_files() {
    let temp_dir = TempDir::new().unwrap();
    let no_ext_file = temp_dir.path().join("no_extension");
    
    // Create file without extension
    fs::write(&no_ext_file, "This is a text file without extension").unwrap();
    
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Should use content inspection for files without extensions
    let is_binary = filter.is_binary_file(&no_ext_file).unwrap();
    assert!(!is_binary); // Text content should not be detected as binary
}

#[test]
fn test_performance_with_large_file() {
    use std::time::Instant;
    
    let temp_dir = TempDir::new().unwrap();
    let large_text_file = temp_dir.path().join("large.unknown");
    
    // Create a large text file
    let content = "text content\n".repeat(10000); // ~130KB
    fs::write(&large_text_file, content).unwrap();
    
    let config = create_test_config(false);
    let filter = BinaryFilter::new(&config).unwrap();
    
    // Test that content inspection is reasonably fast
    let start = Instant::now();
    let is_binary = filter.is_binary_file(&large_text_file).unwrap();
    let duration = start.elapsed();
    
    assert!(!is_binary);
    assert!(duration.as_millis() < 50, "Content inspection too slow: {:?}", duration);
}