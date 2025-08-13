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
    
    // Test 2: Scan src directory if it exists
    let src_dir = Path::new("src");
    if src_dir.exists() {
        println!("\nTest 2: Scanning src directory...");
        let src_result = scanner.scan(src_dir)?;
        println!("  {}", src_result.summary());
        println!("  Files scanned: {}", src_result.stats.files_scanned);
        println!("  Matches found: {}", src_result.matches.len());
    }
    
    // Test 3: Scan tests directory if it exists
    let tests_dir = Path::new("tests");
    if tests_dir.exists() {
        println!("\nTest 3: Scanning tests directory...");
        let tests_result = scanner.scan(tests_dir)?;
        println!("  {}", tests_result.summary());
        println!("  Files scanned: {}", tests_result.stats.files_scanned);
        println!("  Matches found: {}", tests_result.matches.len());
    }
    
    // Test 4: Check performance metrics from the initial scan
    println!("\nPerformance Summary (from initial scan):");
    println!("  Throughput: {:.2} MB/s", result.stats.throughput_mb_per_sec());
    println!("  Files per second: {:.0}", 
        result.stats.files_scanned as f64 / (result.stats.scan_duration_ms as f64 / 1000.0)
    );
    
    Ok(())
}