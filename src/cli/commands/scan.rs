use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::scanner::{Scanner, types::ScanMode};

#[derive(Args, Serialize)]
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
    
    /// Follow symbolic links
    #[arg(long)]
    pub follow_symlinks: bool,
    
    /// Disable entropy analysis (faster but less accurate)
    #[arg(long)]
    pub no_entropy: bool,
    
    /// Set entropy threshold (default: 0.00001)
    #[arg(long)]
    pub entropy_threshold: Option<f64>,
    
    /// Disable intelligent test code detection
    #[arg(long)]
    pub no_ignore_tests: bool,
    
    /// Additional patterns to ignore (regex)
    #[arg(long, value_delimiter = ',')]
    pub ignore_patterns: Vec<String>,
    
    /// Additional paths to ignore (glob patterns)
    #[arg(long, value_delimiter = ',')]
    pub ignore_paths: Vec<String>,
    
    /// Additional comment patterns to ignore
    #[arg(long, value_delimiter = ',')]
    pub ignore_comments: Vec<String>,
    
    /// Custom secret patterns to add (regex)
    #[arg(long, value_delimiter = ',')]
    pub custom_patterns: Vec<String>,
    
    /// Output format
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,
    
    /// Only count matches, don't show details
    #[arg(long)]
    pub count_only: bool,
    
    /// Show matched text content (potentially sensitive)
    #[arg(long)]
    pub show_content: bool,
    
    /// List all available secret detection patterns and exit
    #[arg(long)]
    pub list_patterns: bool,
    
    /// Processing mode: auto (smart default), parallel, or sequential
    #[arg(long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ScanMode>,
    
}

#[derive(Clone, Debug, clap::ValueEnum, serde::Serialize)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Simple list of files with secrets
    Files,
}

pub async fn execute(args: ScanArgs, verbose_level: u8, config_path: Option<&str>) -> Result<()> {
    use crate::scanner::patterns::SecretPatterns;
    use regex::Regex;
    
    // Create scanner-specific config overrides
    let scanner_overrides = serde_json::json!({
        "scanner": {
            "mode": args.mode,
            "max_file_size_mb": args.max_file_size,
            "follow_symlinks": args.follow_symlinks,
            "enable_entropy_analysis": !args.no_entropy,
            "min_entropy_threshold": args.entropy_threshold,
            "ignore_test_code": !args.no_ignore_tests,
            "ignore_patterns": args.ignore_patterns,
            "ignore_paths": args.ignore_paths,
            "ignore_comments": args.ignore_comments
        }
    });
    
    // Load configuration with custom overrides
    println!("DEBUG: Scan execute received config_path: {:?}", config_path);
    let config = GuardyConfig::load(config_path, Some(scanner_overrides))?;
    
    // Create custom scanner config based on CLI args
    let mut scanner_config = Scanner::parse_scanner_config(&config)?;
    
    // Apply CLI overrides
    if args.include_binary {
        scanner_config.skip_binary_files = false;
    }
    
    scanner_config.max_file_size_mb = args.max_file_size;
    scanner_config.follow_symlinks = args.follow_symlinks;
    
    if args.no_entropy {
        scanner_config.enable_entropy_analysis = false;
    }
    
    if let Some(threshold) = args.entropy_threshold {
        scanner_config.min_entropy_threshold = threshold;
    }
    
    if args.no_ignore_tests {
        scanner_config.ignore_test_code = false;
    }
    
    // Add additional ignore patterns
    scanner_config.ignore_patterns.extend(args.ignore_patterns.clone());
    scanner_config.ignore_paths.extend(args.ignore_paths.clone());
    scanner_config.ignore_comments.extend(args.ignore_comments.clone());
    
    // Load patterns and add custom ones
    let mut patterns = SecretPatterns::new(&config)?;
    
    // Handle --list-patterns flag
    if args.list_patterns {
        println!("Available Secret Detection Patterns ({} total):", patterns.pattern_count());
        println!();
        
        for pattern in &patterns.patterns {
            if verbose_level > 0 {
                println!("📋 {} - {}", 
                    console::style(&pattern.name).cyan().bold(),
                    console::style(&pattern.description).dim()
                );
            } else {
                println!("  - {}", pattern.name);
            }
        }
        return Ok(());
    }
    
    for custom_pattern in &args.custom_patterns {
        match Regex::new(&custom_pattern) {
            Ok(regex) => {
                patterns.patterns.push(crate::scanner::patterns::SecretPattern {
                    name: "Custom Pattern".to_string(),
                    regex,
                    description: "User-defined pattern".to_string(),
                });
            }
            Err(e) => {
                output::warning(&format!("Invalid custom pattern '{}': {}", custom_pattern, e));
            }
        }
    }
    
    // Create scanner with custom config
    let scanner = Scanner::with_config(patterns, scanner_config)?;
    
    output::info("Starting security scan...");
    let start_time = Instant::now();
    
    // Determine paths to scan
    let scan_paths = if args.paths.is_empty() {
        // Default to current directory
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };
    
    // Scan all paths and collect detailed results
    let mut all_scan_results = Vec::new();
    for path in &scan_paths {
        if path.is_file() {
            let matches = scanner.scan_file(path)?;
            if !matches.is_empty() {
                // Create a mini scan result for this file
                all_scan_results.push(crate::scanner::types::ScanResult {
                    matches,
                    stats: crate::scanner::types::ScanStats {
                        files_scanned: 1,
                        files_skipped: 0,
                        total_matches: 0, // Will be updated below
                        scan_duration_ms: 0,
                    },
                    warnings: Vec::new(),
                });
            }
        } else if path.is_dir() {
            let scan_result = scanner.scan_directory(path, None)?;
            all_scan_results.push(scan_result);
        } else {
            output::warning(&format!("Path not found: {}", path.display()));
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // Aggregate results
    let all_matches: Vec<_> = all_scan_results.iter().flat_map(|r| r.matches.iter()).collect();
    let total_files: usize = all_scan_results.iter().map(|r| r.stats.files_scanned).sum();
    let total_skipped: usize = all_scan_results.iter().map(|r| r.stats.files_skipped).sum();
    
    // Handle count-only mode
    if args.count_only {
        println!("{}", all_matches.len());
        if !all_matches.is_empty() {
            std::process::exit(1);
        }
        return Ok(());
    }
    
    // Handle different output formats
    match args.format {
        OutputFormat::Json => {
            print_json_results(&all_matches, total_files, total_skipped, elapsed)?;
        }
        OutputFormat::Csv => {
            print_csv_results(&all_matches)?;
        }
        OutputFormat::Files => {
            print_files_only(&all_matches);
        }
        OutputFormat::Text => {
            let all_warnings: Vec<_> = all_scan_results.iter().flat_map(|r| r.warnings.iter()).collect();
            print_text_results(&all_matches, total_files, total_skipped, elapsed, &args, verbose_level, &all_warnings)?;
        }
    }
    
    // Exit with error code if secrets found
    if !all_matches.is_empty() {
        std::process::exit(1);
    }
    
    Ok(())
}

fn print_text_results(
    matches: &[&crate::scanner::types::SecretMatch], 
    total_files: usize, 
    total_skipped: usize, 
    elapsed: std::time::Duration,
    args: &ScanArgs,
    verbose_level: u8,
    warnings: &[&crate::scanner::types::Warning]
) -> Result<()> {
    if matches.is_empty() {
        output::success("No secrets detected!");
        
        // Print statistics if requested
        if args.stats {
            println!();
            println!("{} {}", 
                    console::style("📊").green().bold(), 
                    console::style("Scan Statistics").green().bold());
            println!("  Files scanned: {}", console::style(total_files).cyan());
            if total_skipped > 0 {
                println!("  Files skipped: {}", console::style(total_skipped).cyan());
            }
            println!("  Secrets found: {}", console::style(0).cyan());
            println!("  Scan time: {}ms", console::style(elapsed.as_millis()).cyan());
            if !warnings.is_empty() {
                println!("  Warnings: {}", console::style(warnings.len()).yellow());
            }
        }
        
        return Ok(());
    }

    println!();
    for secret_match in matches {
        println!(
            "{} {} {}",
            console::style("📄").blue(),
            console::style(format!("{}:{}", secret_match.file_path, secret_match.line_number)).cyan().bold(),
            console::style(format!("[{}]", secret_match.secret_type)).red().bold()
        );
        
        if verbose_level > 0 {
            println!("  📋 {}", console::style(&secret_match.pattern_description).dim());
        }
        
        if args.show_content || verbose_level > 0 {
            println!("  Content: {}", console::style(secret_match.line_content.trim()).dim());
            if !secret_match.matched_text.is_empty() {
                println!("  Matched: {}", console::style(&secret_match.matched_text).red().bold());
            }
        } else {
            // Hide the actual secret content for security - just show file location
            println!("  {}", console::style("[Content hidden - use -v or --show-content to reveal]").dim());
        }
    }
    
    println!();
    output::warning(&format!("Found {} potential secrets!", matches.len()));
    
    // Print warnings from scan results
    if !warnings.is_empty() {
        println!();
        for warning in warnings {
            output::warning(&warning.message);
        }
    }
    
    // Print statistics if requested
    if args.stats {
        println!();
        println!("{} {}", 
                console::style("📊").green().bold(), 
                console::style("Scan Statistics").green().bold());
        println!("  Files scanned: {}", console::style(total_files).cyan());
        if total_skipped > 0 {
            println!("  Files skipped: {}", console::style(total_skipped).cyan());
        }
        println!("  Secrets found: {}", console::style(matches.len()).cyan());
        println!("  Scan time: {}ms", console::style(elapsed.as_millis()).cyan());
        if !warnings.is_empty() {
            println!("  Warnings: {}", console::style(warnings.len()).yellow());
        }
    }
    
    Ok(())
}

fn print_json_results(
    matches: &[&crate::scanner::types::SecretMatch], 
    total_files: usize, 
    total_skipped: usize, 
    elapsed: std::time::Duration
) -> Result<()> {
    use serde_json::json;
    
    let results = json!({
        "results": matches.iter().map(|m| json!({
            "file": m.file_path,
            "line": m.line_number,
            "type": m.secret_type,
            "content": m.line_content.trim(),
            "matched_text": m.matched_text,
            "start_pos": m.start_pos,
            "end_pos": m.end_pos
        })).collect::<Vec<_>>(),
        "statistics": {
            "files_scanned": total_files,
            "files_skipped": total_skipped,
            "secrets_found": matches.len(),
            "scan_duration_ms": elapsed.as_millis()
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

fn print_csv_results(matches: &[&crate::scanner::types::SecretMatch]) -> Result<()> {
    println!("file,line,type,content");
    for secret_match in matches {
        println!(
            "{},{},{},\"{}\"",
            secret_match.file_path,
            secret_match.line_number,
            secret_match.secret_type,
            secret_match.line_content.trim().replace('"', "\"\"")
        );
    }
    Ok(())
}

fn print_files_only(matches: &[&crate::scanner::types::SecretMatch]) {
    let mut files: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for secret_match in matches {
        files.insert(&secret_match.file_path);
    }
    
    for file in files {
        println!("{}", file);
    }
}

