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
use crate::scan_v3::{Scanner, ScannerConfig};
use crate::scan_v3::filters::content::ContextPrefilter;
use crate::scan_v3::reports::{ReportOrchestrator, ReportConfig, ReportFormat, RedactionStyle};

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
    pub progress: Option<bool>,
    
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

    /// Show detailed statistics after scanning
    #[arg(long)]
    pub stats: bool,

    /// Generate report files (HTML and JSON)
    #[arg(long)]
    pub report: bool,

    /// Directory to save reports (default: ./.guardy/reports/)
    #[arg(long)]
    pub report_dir: Option<PathBuf>,

    /// Specific output path for report (overrides --report-dir)
    #[arg(long)]
    pub report_output: Option<PathBuf>,

    /// Include real secrets in report (creates .sensitive. files)
    #[arg(long)]
    pub report_show_secrets: bool,
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
    
    // Generate reports if requested
    if args.report {
        generate_reports(&all_results, &args).await?;
    }
    
    // Handle count-only mode
    if args.count_only {
        let total_matches: usize = all_results.iter()
            .map(|r| r.matches.len())
            .sum();
        println!("{total_matches}");
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
            print_summary_results(&all_results, elapsed, verbose_level, &args, Some(&scanner))?;
            
            // Show detailed summary for verbose mode
            if verbose_level > 1 && !all_results.is_empty() {
                println!();
                for (i, result) in all_results.iter().enumerate() {
                    if all_results.len() > 1 {
                        output::styled!("Path {}: {}", ((i + 1).to_string(), "number"), (result.summary(), "info"));
                    } else {
                        output::styled!("Summary: {}", (result.summary(), "info"));
                    }
                }
            }
        }
        OutputFormat::Detailed => {
            print_detailed_results(&all_results, elapsed, &args, verbose_level)?;
        }
    }
    
    // Print final completion message
    let total_matches: usize = all_results.iter().map(|r| r.matches.len()).sum();
    let total_files: usize = all_results.iter().map(|r| r.stats.files_scanned).sum();
    
    println!();
    if total_matches == 0 {
        output::styled!(
            "{} Scan completed successfully - no secrets detected in {} files in {}s",
            ("✅", "success_symbol"),
            (total_files.to_string(), "number"),
            (format!("{:.2}", elapsed.as_secs_f64()), "number")
        );
    } else {
        output::styled!(
            "{} Scan completed - found {} potential secrets in {} files in {}s",
            ("⚠️", "warning_symbol"),
            (total_matches.to_string(), "caution"),
            (total_files.to_string(), "number"),
            (format!("{:.2}", elapsed.as_secs_f64()), "number")
        );
    }
    
    // Exit with error code if secrets found
    let has_secrets = all_results.iter().any(|r| r.has_secrets());
    if has_secrets {
        std::process::exit(1);
    }
    
    Ok(())
}

/// Print summary results (new default format)
fn print_summary_results(
    results: &[crate::scan_v3::ScanResult], 
    elapsed: std::time::Duration,
    _verbose_level: u8,
    args: &ScanArgs,
    scanner_ref: Option<&crate::scan_v3::Scanner>,
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
            if result.has_secrets() {
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
        "{} Scanned {} files ({} MB) in {}s • {} MB/s",
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
    
    // Show detailed statistics if --stats requested
    if args.stats && !results.is_empty() {
        println!();
        output::styled!(
            "{} {}",
            ("📊", "info_symbol"),
            ("Scan Statistics", "property")
        );
        
        // Aggregate detailed stats from all results
        let stats = &results[0].stats; // Use first result's stats (they should all be similar)
        
        output::styled!("  Directories traversed: {}", (stats.directories_traversed.to_string(), "symbol"));
        output::styled!("  Files discovered: {}", (stats.total_files_discovered.to_string(), "symbol"));
        output::styled!("  Files scanned: {}", (stats.files_scanned.to_string(), "symbol"));
        output::styled!("  Files skipped: {}", (stats.files_skipped.to_string(), "symbol"));
        if stats.files_failed > 0 {
            output::styled!("  Files failed: {}", (stats.files_failed.to_string(), "warning"));
        }
        
        println!();
        output::styled!("  {} Filtering Statistics", ("🔽", "info_symbol"));
        output::styled!("    By size: {}", (stats.files_filtered_by_size.to_string(), "symbol"));
        output::styled!("    By binary: {}", (stats.files_filtered_by_binary.to_string(), "symbol"));
        output::styled!("    By path: {}", (stats.files_filtered_by_path.to_string(), "symbol"));
        output::styled!("    Filter efficiency: {:.1}%", (format!("{:.1}", stats.filter_efficiency()), "symbol"));
        
        println!();
        output::styled!("  {} Match Statistics", ("🔍", "info_symbol"));
        output::styled!("    Total matches: {}", (stats.total_matches.to_string(), "symbol"));
        if stats.matches_filtered_by_comments > 0 {
            output::styled!("    Filtered by comments: {}", (stats.matches_filtered_by_comments.to_string(), "symbol"));
        }
        if stats.matches_filtered_by_entropy > 0 {
            output::styled!("    Filtered by entropy: {}", (stats.matches_filtered_by_entropy.to_string(), "symbol"));
        }
        
        println!();
        output::styled!("  {} Performance", ("⚡", "info_symbol"));
        output::styled!("    Scan time: {}", (format!("{:.2}s", elapsed.as_secs_f64()), "symbol"));
        output::styled!("    Throughput: {}", (format!("{:.1} MB/s", throughput), "symbol"));
        output::styled!("    Files per second: {}", (format!("{:.1} files/s", stats.files_per_sec()), "symbol"));
        output::styled!("    Data processed: {}", (format!("{:.1} MB", total_bytes as f64 / 1_000_000.0), "symbol"));
        output::styled!("    Lines processed: {}", (stats.total_lines_processed.to_string(), "symbol"));
        
        // Add prefilter performance stats
        let prefilter_stats = ContextPrefilter::stats();
        output::styled!("    Pattern library: {} patterns, {} keywords, {:.1} patterns/keyword", 
            (prefilter_stats.total_patterns.to_string(), "symbol"),
            (prefilter_stats.total_keywords.to_string(), "symbol"),
            (prefilter_stats.avg_patterns_per_keyword.to_string(), "symbol")
        );
        
        // Add filter performance stats
        if let Some(scanner) = scanner_ref {
            let filter_stats = scanner.get_filter_stats();
            print_filter_performance_stats(&filter_stats);
        }
        
        if total_warnings > 0 {
            println!();
            output::styled!("  {} Warnings: {}", ("⚠️", "warning_symbol"), (total_warnings.to_string(), "warning"));
        }
    }
    
    Ok(())
}

/// Print filter performance statistics
fn print_filter_performance_stats(filter_stats: &crate::scan_v3::pipeline::directory::FilterStats) {
    println!();
    output::styled!("    {} Filter Performance:", ("🔍", "info_symbol"));
    
    // Binary filter stats
    if !filter_stats.binary_filter_stats.is_empty() {
        output::styled!("      Binary Filter:");
        for (key, value) in &filter_stats.binary_filter_stats {
            output::styled!("        {}: {}", (key, "property"), (value, "symbol"));
        }
    }
    
    // Path filter stats
    if !filter_stats.path_filter_stats.is_empty() {
        output::styled!("      Path Filter:");
        for (key, value) in &filter_stats.path_filter_stats {
            output::styled!("        {}: {}", (key, "property"), (value, "symbol"));
        }
    }
    
    // Size filter stats
    if !filter_stats.size_filter_stats.is_empty() {
        output::styled!("      Size Filter:");
        for (key, value) in &filter_stats.size_filter_stats {
            output::styled!("        {}: {}", (key, "property"), (value, "symbol"));
        }
    }
}


/// Print detailed results (equivalent to old format)
fn print_detailed_results(
    results: &[crate::scan_v3::ScanResult],
    _elapsed: std::time::Duration, 
    args: &ScanArgs,
    verbose_level: u8,
) -> Result<()> {
    for result in results {
        if result.has_secrets() {
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
                            (&secret_match.secret_type, "id_value")
                        );
                        if args.show_content {
                            output::styled!(
                                "  Content: {}",
                                (secret_match.matched_text.clone(), "hash_value")
                            );
                        } else if verbose_level > 0 {
                            output::styled!(
                                "  Content: {}",
                                (secret_match.redacted_match_secure(), "muted")
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
    results: &[crate::scan_v3::ScanResult],
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
            "matched_text": m.matched_text
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
fn print_files_only(results: &[crate::scan_v3::ScanResult]) {
    let unique_files = unique_files_with_secrets(results);
    for file in unique_files {
        println!("{file}");
    }
}

/// Get unique files that contain secrets
fn unique_files_with_secrets(results: &[crate::scan_v3::ScanResult]) -> Vec<&str> {
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
    matches: &[crate::scan_v3::SecretMatch],
) -> Vec<(String, Vec<&crate::scan_v3::SecretMatch>)> {
    use std::collections::HashMap;
    
    let mut grouped: HashMap<String, Vec<&crate::scan_v3::SecretMatch>> = HashMap::new();
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

/// Generate reports for scan results
async fn generate_reports(
    results: &[crate::scan_v3::ScanResult], 
    args: &ScanArgs,
) -> Result<()> {
    if results.is_empty() {
        return Ok(());
    }
    
    // Combine all results into a single result for reporting
    let combined_result = combine_scan_results(results);
    
    // Create report configuration
    let report_config = ReportConfig {
        display_secrets: args.report_show_secrets,
        redaction_style: RedactionStyle::Partial,
        include_file_timing: true,
        max_matches: 0, // Include all matches
    };
    
    // Generate both HTML and JSON reports
    let formats = vec![ReportFormat::Html, ReportFormat::Json];
    
    for format in formats {
        let result = ReportOrchestrator::generate_report(
            &combined_result,
            format,
            args.report_output.clone(),
            args.report_dir.clone(),
            report_config.clone(),
        );
        
        match result {
            Ok(file_path) => {
                let format_name = match format {
                    ReportFormat::Html => "HTML",
                    ReportFormat::Json => "JSON",
                };
                output::styled!(
                    "{} {} report generated: {}",
                    ("📄", "info_symbol"),
                    (format_name, "info"),
                    (file_path.display().to_string(), "file_path")
                );
            }
            Err(e) => {
                output::styled!(
                    "{} Failed to generate {} report: {}",
                    ("❌", "error_symbol"),
                    (format!("{:?}", format), "error"),
                    (e.to_string(), "error")
                );
            }
        }
    }
    
    Ok(())
}

/// Combine multiple scan results into a single result for reporting
fn combine_scan_results(results: &[crate::scan_v3::ScanResult]) -> crate::scan_v3::ScanResult {
    if results.len() == 1 {
        let result = &results[0];
        return crate::scan_v3::ScanResult::new(
            result.matches.clone(),
            result.stats.clone(),
            result.file_results.clone(),
            result.warnings.clone(),
        );
    }
    
    use crate::scan_v3::{ScanResult, ScanStats};
    
    let mut all_matches = Vec::new();
    let mut all_warnings = Vec::new();
    let mut all_file_results = Vec::new();
    let mut combined_stats = ScanStats::default();
    
    for result in results {
        all_matches.extend_from_slice(&result.matches);
        all_warnings.extend_from_slice(&result.warnings);
        all_file_results.extend_from_slice(&result.file_results);
        
        // Combine stats
        combined_stats.files_scanned += result.stats.files_scanned;
        combined_stats.total_files_discovered += result.stats.total_files_discovered;
        combined_stats.files_skipped += result.stats.files_skipped;
        combined_stats.files_failed += result.stats.files_failed;
        combined_stats.total_bytes_processed += result.stats.total_bytes_processed;
        combined_stats.total_lines_processed += result.stats.total_lines_processed;
        combined_stats.total_matches += result.stats.total_matches;
        combined_stats.files_filtered_by_size += result.stats.files_filtered_by_size;
        combined_stats.files_filtered_by_binary += result.stats.files_filtered_by_binary;
        combined_stats.files_filtered_by_path += result.stats.files_filtered_by_path;
        combined_stats.matches_filtered_by_comments += result.stats.matches_filtered_by_comments;
        combined_stats.matches_filtered_by_entropy += result.stats.matches_filtered_by_entropy;
    }
    
    ScanResult::new(all_matches, combined_stats, all_file_results, all_warnings)
}

