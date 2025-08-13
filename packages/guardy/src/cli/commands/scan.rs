//! New scan command using v3 scanner
//!
//! This provides a clean, optimized interface to the new v3 scanner
//! with simplified parameters and better defaults.

use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::scan::{Scanner, ScannerConfig};

/// Simplified scan arguments optimized for v3 scanner
#[derive(Args, Serialize)]
pub struct ScanArgs {
    /// Files or directories to scan
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Maximum file size to scan in MB
    #[arg(long, default_value = "50")]
    pub max_file_size: usize,
    
    /// CPU usage percentage (1-100)
    #[arg(long, default_value = "80")]
    pub max_cpu: u8,
    
    /// Show progress bars (auto-detects TTY if not specified)
    #[arg(long)]
    pub progress: bool,
    
    /// Disable entropy analysis for faster scanning
    #[arg(long)]
    pub no_entropy: bool,
    
    /// Entropy threshold (default: 4.5)
    #[arg(long)]
    pub entropy_threshold: Option<f64>,
    
    /// Include binary files in scan
    #[arg(long)]
    pub include_binary: bool,
    
    /// Follow symbolic links
    #[arg(long)]
    pub follow_symlinks: bool,
    
    /// Additional paths to ignore (glob patterns)
    #[arg(long, value_delimiter = ',')]
    pub ignore_paths: Vec<String>,
    
    /// Output format
    #[arg(long, value_enum, default_value = "summary")]
    pub format: OutputFormat,
    
    /// Only count matches, don't show details
    #[arg(long)]
    pub count_only: bool,

    /// Show matched text content (potentially sensitive)
    #[arg(long)]
    pub show_content: bool,
}

/// Output format options optimized for v3
#[derive(Clone, Debug, clap::ValueEnum, Serialize)]
pub enum OutputFormat {
    /// Concise summary with progress bars and key stats
    Summary,
    /// Detailed text output with full match information
    Detailed,
    /// JSON format for machine processing
    Json,
    /// Only show files containing secrets
    Files,
}

/// Main execution function for the new scan command
pub async fn execute(args: ScanArgs, verbose_level: u8, config_path: Option<&str>) -> Result<()> {
    let start_time = Instant::now();
    
    // Load configuration
    let guardy_config = GuardyConfig::load(config_path, None::<serde_json::Value>, verbose_level)?;
    
    // Create v3 scanner config from CLI args + config file
    let scanner_config = ScannerConfig::from_cli_args(&args, &guardy_config)?;
    
    // Initialize v3 scanner
    let scanner = Scanner::new(scanner_config)?;
    
    // Determine paths to scan  
    let scan_paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };
    
    output::styled!("{} Starting security scan with v3 engine...", ("🔍", "info_symbol"));
    
    // Scan all paths
    let mut all_results = Vec::new();
    for path in &scan_paths {
        if !path.exists() {
            output::styled!(
                "{} Path not found: {}",
                ("⚠️", "warning_symbol"),
                (path.display().to_string(), "file_path")
            );
            continue;
        }
        
        let result = scanner.scan(path)?;
        all_results.push(result);
    }
    
    let elapsed = start_time.elapsed();
    
    // Handle count-only mode
    if args.count_only {
        let total_matches: usize = all_results.iter()
            .map(|r| r.matches.len())
            .sum();
        println!("{}", total_matches);
        if total_matches > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }
    
    // Handle different output formats
    match args.format {
        OutputFormat::Json => {
            print_json_results(&all_results, elapsed)?;
        }
        OutputFormat::Files => {
            print_files_only(&all_results);
        }
        OutputFormat::Summary => {
            print_summary_results(&all_results, elapsed, verbose_level)?;
        }
        OutputFormat::Detailed => {
            print_detailed_results(&all_results, elapsed, &args, verbose_level)?;
        }
    }
    
    // Exit with error code if secrets found
    let has_secrets = all_results.iter().any(|r| !r.matches.is_empty());
    if has_secrets {
        std::process::exit(1);
    }
    
    Ok(())
}

/// Print summary results (new default format)
fn print_summary_results(
    results: &[crate::scan::ScanResult], 
    elapsed: std::time::Duration,
    verbose_level: u8,
) -> Result<()> {
    let total_matches: usize = results.iter().map(|r| r.matches.len()).sum();
    let total_files: usize = results.iter().map(|r| r.stats.files_scanned).sum();
    let total_warnings: usize = results.iter().map(|r| r.warnings.len()).sum();
    
    if total_matches == 0 {
        output::styled!("{} No secrets detected!", ("✅", "success_symbol"));
    } else {
        output::styled!(
            "{} Found {} potential secrets in {} files!",
            ("⚠️", "warning_symbol"),
            (total_matches.to_string(), "caution"),
            (unique_files_with_secrets(results).len().to_string(), "caution")
        );
        
        // Show brief summary of findings
        println!();
        for result in results {
            if !result.matches.is_empty() {
                let file_groups = group_matches_by_file(&result.matches);
                for (file_path, matches) in file_groups.iter().take(5) { // Show first 5 files
                    output::styled!(
                        "  📄 {} ({})",
                        (file_path, "file_path"),
                        (format!("{} secrets", matches.len()), "caution")
                    );
                }
                if file_groups.len() > 5 {
                    output::styled!(
                        "  ... and {} more files",
                        ((file_groups.len() - 5).to_string(), "muted")
                    );
                }
            }
        }
    }
    
    // Performance summary
    println!();
    let total_bytes: u64 = results.iter().map(|r| r.stats.total_bytes_processed).sum();
    let throughput = if elapsed.as_secs() > 0 { 
        total_bytes as f64 / elapsed.as_secs() as f64 / 1_000_000.0 
    } else { 0.0 };
    
    output::styled!(
        "{} Scanned {} files ({:.1} MB) in {:.2}s • {:.1} MB/s",
        ("📊", "info_symbol"),
        (total_files.to_string(), "number"),
        (format!("{:.1}", total_bytes as f64 / 1_000_000.0), "number"),
        (format!("{:.2}", elapsed.as_secs_f64()), "number"),
        (format!("{:.1}", throughput), "number")
    );
    
    if total_warnings > 0 {
        output::styled!(
            "{} {} warnings (use --format detailed to view)",
            ("⚠️", "warning_symbol"),
            (total_warnings.to_string(), "warning")
        );
    }
    
    Ok(())
}

/// Print detailed results (equivalent to old format)
fn print_detailed_results(
    results: &[crate::scan::ScanResult],
    elapsed: std::time::Duration, 
    args: &ScanArgs,
    verbose_level: u8,
) -> Result<()> {
    for result in results {
        if !result.matches.is_empty() {
            let file_groups = group_matches_by_file(&result.matches);
            
            println!();
            for (file_path, matches) in &file_groups {
                if matches.len() == 1 {
                    output::styled!(
                        "{} {}:{}",
                        ("📄", "info_symbol"),
                        (file_path, "file_path"),
                        (matches[0].coordinate().line.to_string(), "number")
                    );
                } else {
                    output::styled!(
                        "{} {} ({} secrets)",
                        ("📄", "info_symbol"),
                        (file_path, "file_path"),
                        (matches.len().to_string(), "caution")
                    );
                }
                
                if args.show_content || verbose_level > 0 {
                    for secret_match in matches {
                        output::styled!(
                            "  Line {}: [{}]",
                            (secret_match.line_number().to_string(), "number"),
                            (secret_match.secret_type.to_string(), "id_value")
                        );
                        if args.show_content {
                            output::styled!(
                                "  Content: {}",
                                (secret_match.matched_text.clone(), "hash_value")
                            );
                        }
                    }
                }
            }
        }
        
        // Show warnings
        if !result.warnings.is_empty() {
            println!();
            output::styled!(
                "{} {} warnings:",
                ("⚠️", "warning_symbol"),
                (result.warnings.len().to_string(), "warning")
            );
            for warning in &result.warnings {
                output::styled!("  • {}", (warning, "muted"));
            }
        }
    }
    
    Ok(())
}

/// Print JSON results
fn print_json_results(
    results: &[crate::scan::ScanResult],
    elapsed: std::time::Duration,
) -> Result<()> {
    use serde_json::json;
    
    let total_files: usize = results.iter().map(|r| r.stats.files_scanned).sum();
    let total_matches: usize = results.iter().map(|r| r.matches.len()).sum();
    let all_matches: Vec<_> = results.iter().flat_map(|r| &r.matches).collect();
    let all_warnings: Vec<_> = results.iter().flat_map(|r| &r.warnings).collect();
    
    let json_result = json!({
        "results": all_matches.iter().map(|m| json!({
            "file": m.file_path(),
            "line": m.line_number(),
            "column": m.coordinate().column_start,
            "pattern": m.secret_type.to_string(),
            "matched_text": m.matched_text,
            "confidence": m.confidence
        })).collect::<Vec<_>>(),
        "warnings": all_warnings,
        "summary": {
            "files_scanned": total_files,
            "secrets_found": total_matches,
            "unique_files": unique_files_with_secrets(results).len(),
            "scan_duration_ms": elapsed.as_millis(),
            "warnings_count": all_warnings.len()
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&json_result)?);
    Ok(())
}

/// Print only files containing secrets
fn print_files_only(results: &[crate::scan::ScanResult]) {
    let unique_files = unique_files_with_secrets(results);
    for file in unique_files {
        println!("{}", file);
    }
}

/// Get unique files that contain secrets
fn unique_files_with_secrets(results: &[crate::scan::ScanResult]) -> Vec<&str> {
    use std::collections::HashSet;
    let mut files = HashSet::new();
    
    for result in results {
        for secret_match in &result.matches {
            files.insert(secret_match.file_path());
        }
    }
    
    let mut file_list: Vec<&str> = files.into_iter().collect();
    file_list.sort();
    file_list
}

/// Group matches by file path
fn group_matches_by_file(
    matches: &[crate::scan::SecretMatch],
) -> Vec<(String, Vec<&crate::scan::SecretMatch>)> {
    use std::collections::HashMap;
    
    let mut grouped: HashMap<String, Vec<&crate::scan::SecretMatch>> = HashMap::new();
    for secret_match in matches {
        grouped
            .entry(secret_match.file_path().to_string())
            .or_default()
            .push(secret_match);
    }
    
    let mut result: Vec<_> = grouped.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by file path
    result
}

