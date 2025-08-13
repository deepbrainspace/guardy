//! Test the enhanced Scanner v3 implementation

use guardy::scan::{Scanner, ScannerConfig};
use std::path::Path;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("guardy=debug")
        .init();
    
    println!("Testing Scanner v3 with optimized architecture...\n");
    
    // Create scanner with custom config
    let config = ScannerConfig {
        show_progress: true,
        max_cpu_percentage: 80,
        ..Default::default()
    };
    
    let scanner = Scanner::new(config)?;
    
    // Test 1: Scan current directory
    println!("Test 1: Scanning current directory...");
    let result = scanner.scan(Path::new("."))?;
    println!("  {}", result.summary());
    
    // Test 2: Scan specific file if it exists
    let test_file = Path::new("src/main.rs");
    if test_file.exists() {
        println!("\nTest 2: Scanning single file...");
        let file_result = scanner.scan_file(test_file)?;
        println!("  File: {}", file_result.file_path);
        println!("  Success: {}", file_result.success);
        println!("  Matches: {}", file_result.matches.len());
    }
    
    // Test 3: Scan multiple paths
    println!("\nTest 3: Scanning multiple paths in parallel...");
    let paths = vec![
        Path::new("src").to_path_buf(),
        Path::new("tests").to_path_buf(),
    ];
    
    let multi_results = scanner.scan_multiple(&paths)?;
    for (i, result) in multi_results.iter().enumerate() {
        println!("  Path {}: {}", i + 1, result.summary());
    }
    
    // Test 4: Check performance metrics
    println!("\nPerformance Summary:");
    if let Some(result) = multi_results.first() {
        println!("  Throughput: {:.2} MB/s", result.stats.throughput_mb_per_sec());
        println!("  Files per second: {:.0}", 
            result.stats.files_scanned as f64 / (result.stats.scan_duration_ms as f64 / 1000.0)
        );
    }
    
    Ok(())
}