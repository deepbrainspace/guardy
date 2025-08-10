//! Semantic output macros that respect output styling modes
//!
//! These macros provide the core SuperCLI functionality - semantic output
//! that automatically adapts to different output styles (color, monochrome, none).

use starbase_styles::color::{caution, failure, label, success, symbol as style_symbol};

#[cfg(feature = "clap")]
use starbase_styles::color::owo::OwoColorize;

/// Internal implementation for success messages
pub fn success_impl(message: &str, symbol: &str) {
    #[cfg(feature = "clap")]
    {
        match crate::clap::get_output_style() {
            "none" => println!("{symbol} {message}"),
            "monochrome" => println!("{} {}", symbol.bold(), message.bold()),
            _ => println!("{} {}", style_symbol(symbol), success(message)), // Color
        }
    }
    #[cfg(not(feature = "clap"))]
    {
        println!("{} {}", style_symbol(symbol), success(message));
    }
}

/// Print a success message with optional custom symbol
///
/// # Examples
/// ```rust
/// use supercli::success;
///
/// success!("Operation completed successfully!"); // Uses default ✔
/// success!("Custom success message!", "✨"); // Uses custom symbol
/// ```
#[macro_export]
macro_rules! success {
    ($msg:expr) => {
        $crate::output::macros::success_impl($msg, $crate::output::symbols::CHECK_MARK)
    };
    ($msg:expr, $symbol:expr) => {
        $crate::output::macros::success_impl($msg, $symbol)
    };
}

/// Internal implementation for warning messages
pub fn warning_impl(message: &str, symbol: &str) {
    #[cfg(feature = "clap")]
    {
        match crate::clap::get_output_style() {
            "none" => println!("{symbol} {message}"),
            "monochrome" => println!("{} {}", symbol.bold(), message.bold()),
            _ => println!("{} {}", style_symbol(symbol), caution(message)), // Color
        }
    }
    #[cfg(not(feature = "clap"))]
    {
        println!("{} {}", style_symbol(symbol), caution(message));
    }
}

/// Print a warning message with optional custom symbol
///
/// # Examples
/// ```rust
/// use supercli::warning;
///
/// warning!("This action cannot be undone"); // Uses default ⚠
/// warning!("Critical warning!", "🚨"); // Uses custom symbol
/// ```
#[macro_export]
macro_rules! warning {
    ($msg:expr) => {
        $crate::output::macros::warning_impl($msg, $crate::output::symbols::WARNING_SIGN)
    };
    ($msg:expr, $symbol:expr) => {
        $crate::output::macros::warning_impl($msg, $symbol)
    };
}

/// Internal implementation for info messages
pub fn info_impl(message: &str, symbol: &str) {
    #[cfg(feature = "clap")]
    {
        match crate::clap::get_output_style() {
            "none" => println!("{symbol} {message}"),
            "monochrome" => println!("{} {}", symbol.bold(), message.bold()),
            _ => println!("{} {}", style_symbol(symbol), label(message)), // Color
        }
    }
    #[cfg(not(feature = "clap"))]
    {
        println!("{} {}", style_symbol(symbol), label(message));
    }
}

/// Print an info message with optional custom symbol
///
/// # Examples
/// ```rust
/// use supercli::info;
///
/// info!("Processing files..."); // Uses default ℹ
/// info!("Scanning files...", "🔍"); // Uses custom symbol
/// info!("Using parallel processing...", "⚡"); // Uses custom symbol
/// ```
#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        $crate::output::macros::info_impl($msg, $crate::output::symbols::INFORMATION)
    };
    ($msg:expr, $symbol:expr) => {
        $crate::output::macros::info_impl($msg, $symbol)
    };
}

/// Internal implementation for error messages
pub fn error_impl(message: &str, symbol: &str) {
    #[cfg(feature = "clap")]
    {
        match crate::clap::get_output_style() {
            "none" => println!("{symbol} {message}"),
            "monochrome" => println!("{} {}", symbol.bold(), message.bold()),
            _ => println!("{} {}", style_symbol(symbol), failure(message)), // Color
        }
    }
    #[cfg(not(feature = "clap"))]
    {
        println!("{} {}", style_symbol(symbol), failure(message));
    }
}

/// Print an error message with optional custom symbol
///
/// # Examples
/// ```rust
/// use supercli::error;
///
/// error!("Configuration file not found"); // Uses default ✗
/// error!("Critical error occurred!", "💥"); // Uses custom symbol
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        $crate::output::macros::error_impl($msg, $crate::output::symbols::CROSS_MARK)
    };
    ($msg:expr, $symbol:expr) => {
        $crate::output::macros::error_impl($msg, $symbol)
    };
}
