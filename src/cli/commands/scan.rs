use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use std::time::Instant;

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::scanner::core::Scanner;

#[derive(Args)]
pub struct ScanArgs {
    /// Files or directories to scan
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
    
    /// Scan all files (including binary files)
    #[arg(long)]
    pub include_binary: bool,
    
    /// Maximum file size to scan in MB
    #[arg(long, default_value = "10")]
    pub max_file_size: usize,
    
    /// Show statistics after scanning
    #[arg(long)]
    pub stats: bool,
}

pub async fn execute(args: ScanArgs) -> Result<()> {
    // Load configuration
    let config = GuardyConfig::load()?;
    
    // Create scanner
    let scanner = Scanner::new(&config)?;
    
    output::info("Starting security scan...");
    let start_time = Instant::now();
    
    // Determine paths to scan
    let scan_paths = if args.paths.is_empty() {
        // Default to current directory
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };
    
    // Scan all paths
    let mut all_results = Vec::new();
    for path in &scan_paths {
        if path.is_file() {
            let matches = scanner.scan_file(path)?;
            for secret_match in matches {
                all_results.push(secret_match);
            }
        } else if path.is_dir() {
            let scan_result = scanner.scan_directory(path)?;
            all_results.extend(scan_result.matches);
        } else {
            output::warning(&format!("Path not found: {}", path.display()));
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // Print results
    if all_results.is_empty() {
        output::success("No secrets detected!");
    } else {
        println!();
        for secret_match in &all_results {
            println!(
                "{} {} {}",
                console::style("📄").blue(),
                console::style(format!("{}:{}", secret_match.file_path, secret_match.line_number)).cyan().bold(),
                console::style(format!("[{}]", secret_match.secret_type)).red().bold()
            );
            println!("  {}", console::style(secret_match.line_content.trim()).dim());
        }
        
        println!();
        output::warning(&format!("Found {} potential secrets!", all_results.len()));
    }
    
    // Print statistics if requested
    if args.stats {
        println!();
        println!("{} {}", 
                console::style("📊").green().bold(), 
                console::style("Scan Statistics").green().bold());
        println!("  Paths scanned: {}", console::style(scan_paths.len()).cyan());
        println!("  Secrets found: {}", console::style(all_results.len()).cyan());
        println!("  Scan time: {}ms", console::style(elapsed.as_millis()).cyan());
    }
    
    // Exit with error code if secrets found
    if !all_results.is_empty() {
        std::process::exit(1);
    }
    
    Ok(())
}