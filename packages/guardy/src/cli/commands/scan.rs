use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::cli::output;
use crate::config::GuardyConfig;
use crate::scan::Scanner;
use crate::scan::types::{ScanMode, ScanResult, ScanStats, SecretMatch, Warning};

/// Format scan time intelligently - use ms for short times, mm:ss for longer times
fn format_scan_time(duration: Duration) -> String {
    let total_ms = duration.as_millis();

    // Use milliseconds for times under 10 seconds
    if total_ms < 10_000 {
        format!("{total_ms}ms")
    } else {
        // For longer times, show minutes, seconds, and milliseconds
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let remaining_ms = total_ms % 1000;

        if minutes > 0 {
            format!("{minutes}m {seconds}.{remaining_ms:03}s")
        } else {
            format!("{seconds}.{remaining_ms:03}s")
        }
    }
}

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
    use crate::scan::patterns::SecretPatterns;
    use regex::Regex;

    // Load configuration (CLI overrides handled separately due to SuperConfig limitations)
    // TODO: Fix SuperConfig bug where nested JSON objects and arrays in CLI overrides
    // cause "invalid type: sequence, expected a map" errors and prevent proper merging
    let config = GuardyConfig::load(config_path, None::<serde_json::Value>, verbose_level)?;

    // Load patterns and add custom ones
    let mut patterns = SecretPatterns::new(&config)?;

    // Handle --list-patterns flag
    if args.list_patterns {
        output::styled!(
            "{} Available Secret Detection Patterns ({} total):",
            ("📋", "info_symbol"),
            (patterns.pattern_count().to_string(), "property")
        );
        println!();

        for pattern in &patterns.patterns {
            if verbose_level > 0 {
                output::styled!(
                    "📋 {} - {}",
                    (pattern.name.clone(), "property"),
                    (pattern.description.clone(), "symbol")
                );
            } else {
                output::styled!("  - {}", (pattern.name.clone(), "property"));
            }
        }
        return Ok(());
    }

    for custom_pattern in &args.custom_patterns {
        match Regex::new(custom_pattern) {
            Ok(regex) => {
                patterns
                    .patterns
                    .push(crate::scan::patterns::SecretPattern {
                        name: "Custom Pattern".to_string(),
                        regex,
                        description: "User-defined pattern".to_string(),
                    });
            }
            Err(e) => {
                output::styled!(
                    "{} Invalid custom pattern '{}': {}",
                    ("⚠️", "warning_symbol"),
                    (custom_pattern, "property"),
                    (e.to_string(), "error")
                );
            }
        }
    }

    // Extract scanner config using the proper parsing method, passing CLI args directly
    let scanner_config = Scanner::parse_scanner_config_with_cli_overrides(&config, &args)?;

    // Create scanner with loaded config
    let scanner = Scanner::with_config(patterns, scanner_config)?;

    output::styled!("{} Starting security scan...", ("ℹ", "info_symbol"));
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
            // Note: If scan_file() returns, the file was processed (binary filtering is internal)
            // For individual file scanning, we assume all files that don't error were processed
            // This is a limitation - ideally scan_file() would return processing status

            all_scan_results.push(ScanResult {
                matches,
                stats: ScanStats {
                    files_scanned: 1, // Single file was processed
                    files_skipped: 0,  // No skipping at this level
                    total_matches: 0, // Will be updated below
                    scan_duration_ms: 0,
                },
                warnings: Vec::new(),
            });
        } else if path.is_dir() {
            let scan_result = scanner.scan_directory(path, None)?;
            all_scan_results.push(scan_result);
        } else {
            output::styled!(
                "{} Path not found: {}",
                ("⚠️", "warning_symbol"),
                (path.display().to_string(), "file_path")
            );
        }
    }

    let elapsed = start_time.elapsed();

    // Aggregate results
    let all_matches: Vec<_> = all_scan_results
        .iter()
        .flat_map(|r| r.matches.iter())
        .collect();
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

    // Collect all warnings for all output formats
    let all_warnings: Vec<_> = all_scan_results
        .iter()
        .flat_map(|r| r.warnings.iter())
        .collect();

    // Handle different output formats
    match args.format {
        OutputFormat::Json => {
            print_json_results(
                &all_matches,
                total_files,
                total_skipped,
                elapsed,
                &all_warnings,
            )?;
        }
        OutputFormat::Csv => {
            print_csv_results(&all_matches)?;
        }
        OutputFormat::Files => {
            print_files_only(&all_matches);
        }
        OutputFormat::Text => {
            print_text_results(
                &all_matches,
                total_files,
                total_skipped,
                elapsed,
                &args,
                verbose_level,
                &all_warnings,
            )?;
        }
    }

    // Exit with error code if secrets found
    if !all_matches.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

fn print_text_results(
    matches: &[&SecretMatch],
    total_files: usize,
    total_skipped: usize,
    elapsed: std::time::Duration,
    args: &ScanArgs,
    verbose_level: u8,
    warnings: &[&Warning],
) -> Result<()> {
    if matches.is_empty() {
        output::styled!("{} No secrets detected!", ("✔", "success_symbol"));

        // Print statistics if requested
        if args.stats {
            println!();
            output::styled!(
                "{} {}",
                ("📊", "info_symbol"),
                ("Scan Statistics", "property")
            );
            output::styled!("  Files scanned: {}", (total_files.to_string(), "symbol"));
            if total_skipped > 0 {
                output::styled!("  Files skipped: {}", (total_skipped.to_string(), "symbol"));
            }
            output::styled!("  Secrets found: {}", ("0", "symbol"));
            output::styled!("  Scan time: {}", (format_scan_time(elapsed), "symbol"));
            if !warnings.is_empty() {
                output::styled!("  Warnings: {}", (warnings.len().to_string(), "symbol"));
            }
        }

        return Ok(());
    }

    // Check if we should generate a report file instead of terminal output
    if matches.len() > 20 || warnings.len() > 20 {
        use crate::reports::{ReportFormat, ReportGenerator};

        let current_dir = std::env::current_dir()?;
        let report_path = ReportGenerator::generate_report(
            matches,
            warnings,
            total_files,
            total_skipped,
            elapsed,
            &current_dir,
            ReportFormat::Html,
        )?;

        println!();
        output::styled!(
            "{} Found {} secrets and {} warnings (too many to display)",
            ("📊", "info_symbol"),
            (matches.len().to_string(), "caution"),
            (warnings.len().to_string(), "warning")
        );
        println!();
        output::styled!(
            "{} Full report saved to: {}",
            ("📄", "info_symbol"),
            (report_path.display().to_string(), "file_path")
        );

        // Also save JSON for machine processing
        let json_path = ReportGenerator::generate_report(
            matches,
            warnings,
            total_files,
            total_skipped,
            elapsed,
            &current_dir,
            ReportFormat::Json,
        )?;
        output::styled!(
            "{} Machine-readable: {}",
            ("🤖", "info_symbol"),
            (
                json_path.file_name().unwrap().to_string_lossy(),
                "file_path"
            )
        );

        return Ok(());
    }

    // Group matches by file for more concise display
    let grouped_matches = group_matches_by_file(matches);

    println!();
    for (file_path, file_matches) in &grouped_matches {
        // Show file with count of secrets
        if file_matches.len() == 1 {
            output::styled!(
                "{} {} {}",
                ("📄", "info_symbol"),
                (
                    format!("{}:{}", file_path, file_matches[0].line_number),
                    "file_path"
                ),
                (format!("[{}]", file_matches[0].secret_type), "id_value")
            );
        } else {
            output::styled!(
                "{} {} {}",
                ("📄", "info_symbol"),
                (file_path.clone(), "file_path"),
                (
                    format!("[{} secrets found]", file_matches.len()),
                    "id_value"
                )
            );

            // Show individual lines for this file
            for secret_match in file_matches {
                output::styled!(
                    "   Line {}: {}",
                    (secret_match.line_number.to_string(), "number"),
                    (format!("[{}]", secret_match.secret_type), "id_value")
                );
            }
        }

        if verbose_level > 0 && !file_matches.is_empty() {
            output::styled!(
                "  📋 {}",
                (file_matches[0].pattern_description.clone(), "symbol")
            );
        }

        if args.show_content || verbose_level > 0 {
            if file_matches.len() == 1 {
                output::styled!(
                    "  Content: {}",
                    (file_matches[0].line_content.trim(), "symbol")
                );
                if !file_matches[0].matched_text.is_empty() {
                    output::styled!(
                        "  Matched: {}",
                        (file_matches[0].matched_text.clone(), "hash_value")
                    );
                }
            } else {
                output::styled!(
                    "  {}",
                    (
                        format!(
                            "{} lines with secrets - use report for details",
                            file_matches.len()
                        ),
                        "symbol"
                    )
                );
            }
        } else {
            // Hide the actual secret content for security - just show file location
            output::styled!(
                "  {}",
                (
                    "[Content hidden - use -v or --show-content to reveal]",
                    "symbol"
                )
            );
        }
    }

    println!();
    output::styled!(
        "{} Found {} potential secrets!",
        ("⚠", "warning_symbol"),
        (matches.len().to_string(), "caution")
    );

    // Print compact warnings summary
    if !warnings.is_empty() {
        println!();
        output::styled!(
            "{} {}",
            ("⚠️", "warning_symbol"),
            (
                format!("Scan completed with {} warnings", warnings.len()),
                "warning"
            )
        );

        if warnings.len() <= 5 {
            // Show all warnings if 5 or fewer
            for warning in warnings {
                output::styled!("   • {}", (warning.message.as_str(), "dim"));
            }
        } else {
            // Show first 2 warnings and a summary
            for warning in warnings.iter().take(2) {
                output::styled!("   • {}", (warning.message.as_str(), "dim"));
            }

            println!();
            output::styled!(
                "   {} {}",
                ("📝", "info_symbol"),
                (
                    format!(
                        "Run with --output json > warnings.json to save all {} warnings",
                        warnings.len()
                    ),
                    "info"
                )
            );
            output::styled!(
                "   {} {}",
                ("🔍", "info_symbol"),
                ("Or add --verbose for detailed warning display", "info")
            );
        }
    }

    // Print statistics if requested
    if args.stats {
        println!();
        output::styled!(
            "{} {}",
            ("📊", "info_symbol"),
            ("Scan Statistics", "property")
        );
        output::styled!("  Files scanned: {}", (total_files.to_string(), "symbol"));
        if total_skipped > 0 {
            output::styled!("  Files skipped: {}", (total_skipped.to_string(), "symbol"));
        }
        output::styled!("  Secrets found: {}", (matches.len().to_string(), "symbol"));
        output::styled!("  Scan time: {}", (format_scan_time(elapsed), "symbol"));
        if !warnings.is_empty() {
            output::styled!("  Warnings: {}", (warnings.len().to_string(), "symbol"));
        }
    }

    Ok(())
}

fn print_json_results(
    matches: &[&SecretMatch],
    total_files: usize,
    total_skipped: usize,
    elapsed: std::time::Duration,
    warnings: &[&Warning],
) -> Result<()> {
    use serde_json::json;

    let results = json!({
        "results": matches.iter().map(|m| json!({
            "file": m.file_path,
            "line": m.line_number,
            "type": m.secret_type,
            "matched_text": m.matched_text,
            "start_pos": m.start_pos,
            "end_pos": m.end_pos
        })).collect::<Vec<_>>(),
        "warnings": warnings.iter().map(|w| json!({
            "message": w.message
        })).collect::<Vec<_>>(),
        "statistics": {
            "files_scanned": total_files,
            "files_skipped": total_skipped,
            "secrets_found": matches.len(),
            "warnings_count": warnings.len(),
            "scan_duration_ms": elapsed.as_millis()
        }
    });

    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

fn print_csv_results(matches: &[&SecretMatch]) -> Result<()> {
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

fn print_files_only(matches: &[&SecretMatch]) {
    let mut files: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for secret_match in matches {
        files.insert(&secret_match.file_path);
    }

    for file in files {
        println!("{file}");
    }
}

/// Group secret matches by file path for more concise display
fn group_matches_by_file<'a>(
    matches: &'a [&'a SecretMatch],
) -> Vec<(String, Vec<&'a SecretMatch>)> {
    use std::collections::HashMap;

    let mut grouped: HashMap<String, Vec<&'a SecretMatch>> = HashMap::new();
    for secret_match in matches {
        grouped
            .entry(secret_match.file_path.clone())
            .or_default()
            .push(*secret_match);
    }

    let mut result: Vec<_> = grouped.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by file path
    result
}
