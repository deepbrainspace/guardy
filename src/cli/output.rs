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
            println!("{} {}", style("ℹ").blue(), message);
        }
    }

    /// Print a verbose message (only if verbose mode is enabled)
    pub fn verbose(&self, message: &str) {
        if self.verbose {
            println!("{} {}", style("ℹ").dim(), style(message).dim());
        }
    }

    /// Get verbose mode status
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Print a header/title
    pub fn header(&self, title: &str) {
        if !self.quiet {
            println!("\n{}", style(title).bold().underlined());
        }
    }

    /// Print a step in a process
    pub fn step(&self, step: &str) {
        if !self.quiet {
            println!("{} {}", style("❯").cyan(), step);
        }
    }

    /// Create a progress bar
    /// TODO: Remove #[allow(dead_code)] when progress bars are used
    #[allow(dead_code)]
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
    /// TODO: Remove #[allow(dead_code)] when spinners are used
    #[allow(dead_code)]
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
}
