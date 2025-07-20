use console::{style, Emoji};
use crate::scanner::core::ScanResult;

// Professional symbols matching Claude Code style  
const SUCCESS: Emoji = Emoji("✔", "✓");
const WARNING: Emoji = Emoji("⚠", "!");
const INFO: Emoji = Emoji("ℹ", "i");
const ERROR: Emoji = Emoji("✗", "X");
const FILE: Emoji = Emoji("📄", "F");
const STATS: Emoji = Emoji("📊", "S");

// Simple output functions for basic messages
pub fn success(message: &str) {
    println!("{} {}", style(SUCCESS).green().bold(), style(message).green());
}

pub fn warning(message: &str) {
    println!("{} {}", style(WARNING).yellow().bold(), style(message).yellow());
}

pub fn info(message: &str) {
    println!("{} {}", style(INFO).blue().bold(), style(message).blue());
}

pub fn error(message: &str) {
    println!("{} {}", style(ERROR).red().bold(), style(message).red());
}

/// Print scan results in a clean format
pub fn print_scan_results(results: &ScanResult) {
    // Print matches
    for secret_match in &results.matches {
        println!(
            "{} {} {}",
            style(FILE).blue(),
            style(format!("{}:{}", secret_match.file_path, secret_match.line_number)).cyan().bold(),
            style(format!("[{}]", secret_match.secret_type)).red().bold()
        );
        println!("  {}", style(secret_match.line_content.trim()).dim());
    }

    // Print statistics
    if results.matches.is_empty() {
        println!();
        success("No secrets detected!");
    } else {
        println!();
        println!("{} {}", style(STATS).yellow().bold(), style("Scan Summary").yellow().bold());
        println!("  Files scanned: {}", style(results.stats.files_scanned).cyan());
        if results.stats.files_skipped > 0 {
            println!("  Files skipped: {}", style(results.stats.files_skipped).cyan());
        }
        println!("  Secrets found: {}", style(results.stats.total_matches).red().bold());
        
        if results.stats.scan_duration_ms > 0 {
            println!("  Scan time: {}ms", style(results.stats.scan_duration_ms).cyan());
        }
    }

    // Print warnings
    for warning_msg in &results.warnings {
        warning(&warning_msg.message);
    }
}