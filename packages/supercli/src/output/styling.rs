//! Advanced styling functionality for SuperCLI
//!
//! This module provides fine-grained control over text styling with the `styled!` macro,
//! allowing different parts of a single line to have different styling treatments.

use starbase_styles::color::{
    caution, failure, file, hash, id, label, property, shell, success, symbol as style_symbol, url,
};

#[cfg(feature = "clap")]
use starbase_styles::color::owo::OwoColorize;

/// Replace symbol tags like `<info>`, `<success>` with styled symbols
pub fn replace_symbols(text: &str) -> String {
    let mut result = String::from(text);

    // Map of symbol names to (emoji, style)
    let symbol_map = [
        ("info", ("ℹ️", "info_symbol")),
        ("success", ("✅", "success_symbol")),
        ("warning", ("⚠️", "warning_symbol")),
        ("error", ("❌", "error_symbol")),
        ("check", ("✔", "success_symbol")),
        ("cross", ("✗", "error_symbol")),
        ("search", ("🔍", "info_symbol")),
        ("lightning", ("⚡", "info_symbol")),
        ("rocket", ("🚀", "success_symbol")),
        ("sparkles", ("✨", "success_symbol")),
        ("shield", ("🛡️", "info_symbol")),
        ("folder", ("📁", "info_symbol")),
        ("chart", ("📊", "info_symbol")),
    ];

    for (name, (emoji, style)) in &symbol_map {
        let tag = format!("<{name}>");
        if result.contains(&tag) {
            let styled_symbol = apply_style(emoji, style);
            // Just replace the tag with the styled symbol - space comes from original text
            result = result.replace(&tag, &styled_symbol);
        }
    }

    result
}

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
            }
            _ => {
                // Color mode
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
                    "branch" => id(text),       // Git branches in purple
                    "time" => hash(text),       // Time values in green like numbers
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
            "branch" => id(text),                                // Git branches in purple
            "time" => hash(text),          // Time values in green like numbers
            "debug" => style_symbol(text), // Debug messages in muted lime
            _ => text.to_string(),
        }
    }
}

/// Print styled text with symbol replacement and fine-grained control
///
/// This macro supports two modes:
/// 1. Symbol replacement: Use `<symbol>` tags that get replaced with styled symbols
/// 2. Fine-grained styling: Use {} placeholders with style tuples
///
/// # Symbol Replacement Examples
/// ```rust
/// use supercli::styled;
///
/// // Simple symbol messages
/// styled!("<info> No files were updated");
/// styled!("<success> Operation completed successfully");
/// styled!("<warning> This action cannot be undone");
/// styled!("<error> Configuration file not found");
///
/// // Multiple symbols in one message
/// styled!("Sync: <success> 42 updated, <info> 15 unchanged, <error> 2 failed");
/// styled!("Processing files... <info> Found 3 secrets <warning> 1 critical");
/// ```
///
/// # Fine-Grained Styling Examples
/// ```rust
/// use supercli::styled;
///
/// // Mixed styling with placeholders
/// styled!("Processing {} files in {}",
///     ("150", "number"),
///     ("/home/user", "file_path")
/// );
/// ```
#[macro_export]
macro_rules! styled {
    // Single string with symbol replacement
    ($text:expr) => {
        {
            let result = $crate::output::styling::replace_symbols($text);
            println!("{}", result);
        }
    };

    // Fine-grained styling with tuples (existing functionality)
    ($format:expr, $(($text:expr, $style:expr)),+ $(,)?) => {
        {
            // First replace any symbols in the format string
            let mut result = $crate::output::styling::replace_symbols($format);

            // Then do the normal placeholder replacements
            $(
                let styled_text = $crate::output::styling::apply_style($text, $style);
                result = result.replacen("{}", &styled_text, 1);
            )+

            println!("{}", result);
        }
    };
}
