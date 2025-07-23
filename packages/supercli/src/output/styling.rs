//! Advanced styling functionality for SuperCLI
//! 
//! This module provides fine-grained control over text styling with the `styled!` macro,
//! allowing different parts of a single line to have different styling treatments.

use starbase_styles::color::{
    success, failure, caution, label, property, symbol as style_symbol, hash, url, file, shell, id
};

#[cfg(feature = "clap")]
use starbase_styles::color::owo::OwoColorize;

/// Apply a style to text based on style name and output mode
pub fn apply_style<T: AsRef<str>>(text: T, style: &str) -> String {
    let text = text.as_ref();
    #[cfg(feature = "clap")]
    {
        match crate::clap::get_output_style() {
            "none" => text.to_string(),
            "monochrome" => {
                match style {
                    "success" | "success_symbol" => text.bold().to_string(),
                    "warning" | "warning_symbol" => text.bold().to_string(),
                    "info" | "info_symbol" => text.bold().to_string(),
                    "error" | "error_symbol" => text.bold().to_string(),
                    "file_path" => text.bold().to_string(),
                    "number" | "accent" => text.bold().to_string(),
                    "primary" => text.bold().to_string(),
                    "muted" | "dim" | "secondary" => text.to_string(), // No bold for subdued text
                    _ => text.to_string(),
                }
            },
            _ => { // Color mode
                match style {
                    "success" | "success_symbol" => success(text),
                    "warning" | "warning_symbol" => caution(text),
                    "info" | "info_symbol" => label(text),
                    "error" | "error_symbol" => failure(text),
                    "file_path" => file(text),
                    "command" => shell(text),
                    "property" => property(text),
                    "hash" => hash(text),
                    "url" => url(text),
                    "symbol" => style_symbol(text),
                    "id" => id(text),
                    "number" => hash(text), // Use hash (green) for numbers - more vibrant than label (blue)
                    "accent" => property(text), // Use property (lavender) for accents - more vibrant
                    "primary" => text.to_string(), // Primary text stays unstyled in color mode
                    "muted" | "dim" | "secondary" => style_symbol(text), // Subdued styling (lime)
                    "branch" => id(text), // Git branches in purple
                    "time" => hash(text), // Time values in green like numbers
                    "debug" => style_symbol(text), // Debug messages in muted lime
                    _ => text.to_string(),
                }
            }
        }
    }
    #[cfg(not(feature = "clap"))]
    {
        // Default to color mode when clap feature is not enabled
        match style {
            "success" | "success_symbol" => success(text),
            "warning" | "warning_symbol" => caution(text),
            "info" | "info_symbol" => label(text),
            "error" | "error_symbol" => failure(text),
            "file_path" => file(text),
            "command" => shell(text),
            "property" => property(text),
            "hash" => hash(text),
            "url" => url(text),
            "symbol" => style_symbol(text),
            "id" => id(text),
            "number" => hash(text), // Use hash (green) for numbers - more vibrant than label (blue)
            "accent" => property(text), // Use property (lavender) for accents - more vibrant
            "primary" => text.to_string(),
            "muted" | "dim" | "secondary" => style_symbol(text), // Subdued styling (lime)
            "branch" => id(text), // Git branches in purple
            "time" => hash(text), // Time values in green like numbers
            "debug" => style_symbol(text), // Debug messages in muted lime
            _ => text.to_string(),
        }
    }
}

/// Print styled text with fine-grained control over each part
///
/// This macro allows mixing different styles within a single line for professional CLI output.
/// Each placeholder in the format string gets styled according to its corresponding style parameter.
///
/// # Examples
/// ```rust
/// use supercli::styled;
///
/// // Professional mixed styling - colored symbols with normal text
/// styled!("Processing {} files in {}", 
///     ("150", "number"),
///     ("/home/user", "file_path")
/// );
/// 
/// // Success message with mixed styling
/// styled!("   {} {} ({})",
///     ("✔", "success_symbol"),
///     ("target/", "file_path"),
///     ("Rust build directory", "muted")
/// );
///
/// // Error with accent colors for numbers and paths
/// styled!("Failed to process {} files in {} (took {}ms)",
///     ("42", "number"),
///     ("/invalid/path", "file_path"),
///     ("1250", "number")
/// );
/// ```
#[macro_export]
macro_rules! styled {
    ($format:expr, $(($text:expr, $style:expr)),+ $(,)?) => {
        {
            // Use a manual string replacement approach that works reliably
            let mut result = String::from($format);
            
            $(
                let styled_text = $crate::output::styling::apply_style($text, $style);
                result = result.replacen("{}", &styled_text, 1);
            )+
            
            println!("{}", result);
        }
    };
}