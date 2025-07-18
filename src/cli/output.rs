//! Professional output system for Guardy
//!
//! Provides consistent, beautiful output formatting similar to lint-staged and other
//! modern CLI tools. Includes progress bars, styled messages, and professional symbols.

use console::style;
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
            println!("{} {}", style("●").green().bold(), message);
        }
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        // Errors are always shown, even in quiet mode
        eprintln!("{} {}", style("●").red().bold(), message);
    }

    /// Print a warning message
    pub fn warning(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", style("●").yellow().bold(), message);
        }
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", style("●").blue().bold(), message);
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
                style(format!("{} ", emoji)).cyan(),
                style(step_name).bold(),
                if current == total { style("🎉").green() } else { style("").white() }
            );
        }
    }

    /// Print a workflow step with timing
    pub fn workflow_step_timed(&self, current: usize, total: usize, step_name: &str, emoji: &str, duration: std::time::Duration) {
        if !self.quiet {
            let progress = format!("[{}/{}]", current, total);
            let duration_str = if duration.as_secs() > 0 {
                format!("{}s", duration.as_secs())
            } else {
                format!("{}ms", duration.as_millis())
            };
            let timing = format!("[{}]", duration_str);
            println!("{} {} {} {} {}", 
                style(progress).cyan().bold(),
                style(format!("{} ", emoji)).cyan(),
                style(step_name).bold(),
                style(timing).dim(),
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







    /// Print a beautiful language header with language-specific icon
    pub fn language_header(&self, name: &str, description: &str) {
        if !self.quiet {
            let icon = match name {
                // Well-established community icons
                "Rust" => "🦀",                    // Rust community's beloved crab
                "Python" => "🐍",                  // Python snake
                "Go" => "🐹",                      // Go gopher (hamster as closest emoji)
                "Java" => "☕",                    // Java coffee
                "PHP" => "🐘",                     // PHP elephant
                "Ruby" => "🔻",                    // Ruby red diamond
                "Swift" => "🐦",                   // Swift bird
                
                // JavaScript/TypeScript - colors based on their branding
                "JavaScript/TypeScript" => "🟨",  // Yellow square for JS
                
                // Other languages with thematic icons
                "C#" => "🔷",                      // Blue diamond for Microsoft
                "C++" => "⚡",                     // Lightning for speed/performance
                "Kotlin" => "🎯",                  // Target for precision
                "Scala" => "🌟",                   // Star for functional programming
                "Elixir" => "🧪",                  // Elixir/potion
                
                // Default for unknown languages
                _ => "🚀",
            };
            println!("{} {} {}", 
                style(icon).green().bold(),
                style(name).green().bold(),
                style(format!("• {}", description)).dim()
            );
        }
    }

    /// Print a beautiful tool section with icon
    pub fn tool_section(&self, icon: &str, title: &str, items: &[&str]) {
        if !self.quiet && !items.is_empty() {
            println!("    {} {} {}", 
                style(icon).cyan().bold(),
                style(title).cyan().bold(),
                style(items.join(", ")).white()
            );
        }
    }

    /// Print a package manager with special formatting
    pub fn package_manager(&self, pm: &str) {
        if !self.quiet {
            println!("    {} {} {}", 
                style("📦").magenta().bold(),
                style("Package Manager").magenta().bold(),
                style(pm).white().bold()
            );
        }
    }

    /// Print a beautiful section header with box drawing
    #[allow(dead_code)]
    pub fn section_header_box(&self, title: &str) {
        if !self.quiet {
            let len = title.len();
            let border = "─".repeat(len + 4);
            println!("{}", style(format!("╭{}╮", border)).cyan().dim());
            println!("{}", style(format!("│  {}  │", title)).cyan().bold());
            println!("{}", style(format!("╰{}╯", border)).cyan().dim());
        }
    }


    /// Print a modern CLI banner
    pub fn banner(&self, title: &str, subtitle: &str) {
        if !self.quiet {
            println!();
            println!("{}", style("━".repeat(60)).blue().dim());
            println!("{} {}", style("🎯").blue().bold(), style(title).blue().bold());
            println!("{}", style(subtitle).dim());
            println!("{}", style("━".repeat(60)).blue().dim());
            println!();
        }
    }

    /// Print a beautiful tree structure
    pub fn tree_item(&self, is_last: bool, name: &str, value: &str) {
        if !self.quiet {
            let connector = if is_last { "└─" } else { "├─" };
            println!("{} {} {}", 
                style(connector).dim(),
                style(name).bold(),
                style(value).white()
            );
        }
    }

    /// Print a beautiful badge
    pub fn badge(&self, text: &str, color: &str) {
        if !self.quiet {
            let styled_text = match color {
                "green" => style(format!(" {} ", text)).green().bold(),
                "blue" => style(format!(" {} ", text)).blue().bold(),
                "yellow" => style(format!(" {} ", text)).yellow().bold(),
                "red" => style(format!(" {} ", text)).red().bold(),
                "magenta" => style(format!(" {} ", text)).magenta().bold(),
                "cyan" => style(format!(" {} ", text)).cyan().bold(),
                _ => style(format!(" {} ", text)).white().bold(),
            };
            print!("{}", styled_text);
        }
    }

}
