//! Test the optimized Scanner implementation

use guardy::config::GuardyConfig;
use guardy::scan::{Scanner, directory::DirectoryHandler};
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("guardy=debug")
        .init();
    
    println!("Testing optimized Scanner with filter pipeline...\n");
    
    // Create scanner from config
    let config = GuardyConfig::load(None, None::<&()>, 0)?;
    let scanner = Arc::new(Scanner::new(&config)?);
    let directory_handler = DirectoryHandler::new();
    
    // Test 1: Scan current directory
    println!("Test 1: Scanning current directory...");
    let result = directory_handler.scan(
        scanner.clone(),
        Path::new("."),
        None  // Use config-based strategy
    )?;
    println!("  Files scanned: {}", result.stats.files_scanned);
    println!("  Matches found: {}", result.matches.len());
    println!("  Duration: {} ms", result.stats.scan_duration_ms);
    
    // Test 2: Scan src directory if it exists
    let src_dir = Path::new("src");
    if src_dir.exists() {
        println!("\nTest 2: Scanning src directory...");
        let src_result = directory_handler.scan(
            scanner.clone(),
            src_dir,
            None
        )?;
        println!("  Files scanned: {}", src_result.stats.files_scanned);
        println!("  Matches found: {}", src_result.matches.len());
        println!("  Duration: {} ms", src_result.stats.scan_duration_ms);
    }
    
    // Test 3: Scan tests directory if it exists
    let tests_dir = Path::new("tests");
    if tests_dir.exists() {
        println!("\nTest 3: Scanning tests directory...");
        let tests_result = directory_handler.scan(
            scanner.clone(),
            tests_dir,
            None
        )?;
        println!("  Files scanned: {}", tests_result.stats.files_scanned);
        println!("  Matches found: {}", tests_result.matches.len());
        println!("  Duration: {} ms", tests_result.stats.scan_duration_ms);
    }
    
    // Test 4: Check performance metrics from the initial scan
    if result.stats.scan_duration_ms > 0 {
        println!("\nPerformance Summary (from initial scan):");
        println!("  Files per second: {:.0}", 
            result.stats.files_scanned as f64 * 1000.0 / result.stats.scan_duration_ms as f64
        );
    }
    
    Ok(())
}