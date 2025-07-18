//! Professional output system for Guardy
//!
//! Provides consistent, beautiful output formatting similar to lint-staged and other
//! modern CLI tools. Includes progress bars, styled messages, and professional symbols.

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};

/// Output handler for consistent CLI formatting
pub struct Output {
    verbose: bool,
    quiet: bool,
}

impl Output {
    /// Create a new output handler
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self { verbose, quiet }
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", style("✔").green(), message);
        }
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        // Errors are always shown, even in quiet mode
        eprintln!("{} {}", style("✖").red(), message);
    }

    /// Print a warning message
    pub fn warning(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", style("⚠").yellow(), message);
        }
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", style("ℹ️ ").blue(), message);
        }
    }

    /// Print a verbose message (only if verbose mode is enabled)
    pub fn verbose(&self, message: &str) {
        if self.verbose {
            println!("{} {}", style("ℹ️ ").dim(), style(message).dim());
        }
    }

    /// Print a verbose step with emoji and styling
    pub fn verbose_step(&self, emoji: &str, message: &str) {
        if self.verbose {
            println!("{} {}", style(emoji).cyan(), style(message).dim());
        }
    }

    /// Print a verbose summary with styling
    pub fn verbose_summary(&self, icon: &str, message: &str, count: usize) {
        if self.verbose {
            println!("{} {} {}", 
                style(icon).cyan(), 
                style(message).dim(), 
                style(format!("({})", count)).yellow().bold()
            );
        }
    }

    /// Print a verbose breakdown item
    pub fn verbose_breakdown(&self, label: &str, count: usize) {
        if self.verbose {
            println!("  {} {} {}", 
                style("•").cyan(), 
                style(count.to_string()).yellow().bold(), 
                style(label).dim()
            );
        }
    }

    /// Get verbose mode status
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Get quiet mode status
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Print a header/title
    pub fn header(&self, title: &str) {
        if !self.quiet {
            println!("\n{}", style(title).bold().underlined());
        }
    }

    /// Print a section header with enhanced styling
    pub fn section_header(&self, title: &str) {
        if !self.quiet {
            println!("\n{}", style(title).bold().cyan());
        }
    }

    /// Print a step in a process
    pub fn step(&self, step: &str) {
        if !self.quiet {
            println!("{} {}", style("❯").cyan(), step);
        }
    }

    /// Create a progress bar
    pub fn progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create a spinner for indefinite progress
    pub fn spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Print a table row
    pub fn table_row(&self, key: &str, value: &str) {
        println!("  {:<20} {}", style(key).dim(), value);
    }

    /// Print a list item
    pub fn list_item(&self, item: &str) {
        println!("  • {}", item);
    }

    /// Print an indented message
    pub fn indent(&self, message: &str) {
        println!("    {}", message);
    }

    /// Print a section separator
    pub fn separator(&self) {
        println!("{}", style("─".repeat(50)).dim());
    }

    /// Print a task completion summary like Claude Code
    pub fn task_summary(&self, icon: &str, message: &str, success: bool) {
        let styled_icon = if success {
            style(icon).green().bold()
        } else {
            style(icon).red().bold()
        };
        let styled_message = if success {
            style(message).green()
        } else {
            style(message).red()
        };
        println!("{} {}", styled_icon, styled_message);
    }

    /// Ask for user confirmation
    pub fn confirm(&self, message: &str) -> bool {
        print!("{} {} (y/N): ", style("❯").cyan(), message);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    }

    /// Print blank line
    pub fn blank_line(&self) {
        println!();
    }

    /// Print a critical error with enhanced styling
    pub fn critical(&self, message: &str) {
        eprintln!("{} {}", style("✖").red().bold(), style(message).red().bold());
    }

    /// Print a count/summary with enhanced styling
    pub fn count(&self, icon: &str, message: &str, count: usize) {
        if !self.quiet {
            println!("{} {} {}", 
                style(icon).cyan().bold(), 
                style(message).bold(), 
                style(format!("({})", count)).dim()
            );
        }
    }

    /// Print a file location with enhanced styling
    pub fn file_location(&self, file: &str, line: usize) {
        println!("    {} {}:{}", 
            style("•").cyan(), 
            style(file).underlined(), 
            style(line.to_string()).yellow()
        );
    }

    /// Print summary statistics with enhanced styling
    pub fn summary_stats(&self, label: &str, value: usize) {
        if !self.quiet {
            println!("  {} {}", 
                style(label).dim(), 
                style(value.to_string()).bold()
            );
        }
    }

    /// Print a key-value pair with consistent styling
    pub fn key_value(&self, key: &str, value: &str, highlight: bool) {
        if !self.quiet {
            let styled_value = if highlight {
                style(value).green().bold()
            } else {
                style(value).white()
            };
            println!("  {} {}", style(key).dim(), styled_value);
        }
    }

    /// Print a status indicator with consistent styling
    pub fn status_indicator(&self, status: &str, message: &str, is_success: bool) {
        if !self.quiet {
            let (icon, color) = if is_success {
                ("✓", style(status).green())
            } else {
                ("✗", style(status).red())
            };
            println!("{} {} {}", 
                style(icon).bold(),
                color.bold(),
                message
            );
        }
    }

    /// Print a category header with consistent styling
    pub fn category(&self, category: &str) {
        if !self.quiet {
            println!("\n{}", style(category).bold().cyan());
        }
    }

    /// Print a progress indicator with consistent styling
    pub fn progress_indicator(&self, current: usize, total: usize, message: &str) {
        if !self.quiet {
            let percentage = if total > 0 { (current * 100) / total } else { 0 };
            println!("{} {} {}% ({}/{})", 
                style("►").cyan(),
                message,
                style(percentage.to_string()).bold(),
                current,
                total
            );
        }
    }

    /// Print an action result with consistent styling
    pub fn action_result(&self, action: &str, result: &str, success: bool) {
        if !self.quiet {
            let icon = if success { "✓" } else { "✗" };
            let styled_icon = if success {
                style(icon).green().bold()
            } else {
                style(icon).red().bold()
            };
            println!("{} {} {}", 
                styled_icon,
                style(action).bold(),
                style(result).dim()
            );
        }
    }

    /// Print a multi-step workflow indicator
    pub fn workflow_step(&self, current: usize, total: usize, step_name: &str, emoji: &str) {
        if !self.quiet {
            let progress = format!("[{}/{}]", current, total);
            println!("{} {} {} {}", 
                style(progress).cyan().bold(),
                style(emoji).cyan(),
                style(step_name).bold(),
                if current == total { style("🎉").green() } else { style("").white() }
            );
        }
    }

    /// Print a scanning progress update
    pub fn scanning_progress(&self, current: usize, total: usize, file_name: &str) {
        if !self.quiet {
            let percentage = if total > 0 { (current * 100) / total } else { 0 };
            print!("\r{} Scanning... {}% ({}/{}) {}", 
                style("🔍").cyan(),
                style(percentage.to_string()).bold(),
                current,
                total,
                style(file_name).dim()
            );
            io::stdout().flush().unwrap();
        }
    }

    /// Clear the current line (for progress updates)
    pub fn clear_line(&self) {
        if !self.quiet {
            print!("\r{}\r", " ".repeat(80));
            io::stdout().flush().unwrap();
        }
    }

    /// Print a completion message with timing
    pub fn completion_summary(&self, task: &str, duration: std::time::Duration, success: bool) {
        if !self.quiet {
            let icon = if success { "✨" } else { "💥" };
            let status = if success { "completed" } else { "failed" };
            let duration_str = if duration.as_secs() > 0 {
                format!("{}s", duration.as_secs())
            } else {
                format!("{}ms", duration.as_millis())
            };
            
            println!("{} {} {} in {}", 
                style(icon).bold(),
                style(task).bold(),
                style(status).bold(),
                style(duration_str).dim()
            );
        }
    }

    /// Print an interactive menu option
    pub fn menu_option(&self, key: &str, description: &str, selected: bool) {
        if !self.quiet {
            let indicator = if selected { "►" } else { " " };
            let key_style = if selected { style(key).cyan().bold() } else { style(key).dim() };
            let desc_style = if selected { style(description).white().bold() } else { style(description).dim() };
            
            println!("{} {} {}", 
                style(indicator).cyan().bold(),
                key_style,
                desc_style
            );
        }
    }
}
