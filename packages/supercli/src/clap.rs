//! Clap integration for SuperCLI
//!
//! Provides consistent styling for clap-based CLI help output using starbase-styles colors.
//! This module allows all CLI tools to have the same professional, colored help appearance.

#[cfg(feature = "clap")]
use clap::builder::styling::{AnsiColor, Color, Style, Styles};
use starbase_styles::color::Style as StarbaseStyle;

/// Create consistent clap help styles using starbase-styles color scheme
///
/// This provides professional, themed help output that matches your CLI tool's
/// overall aesthetic. All SuperCLI-powered tools will have consistent help styling.
///
/// # Usage
///
/// Add this to your clap Parser derive:
///
/// ```rust
/// use clap::Parser;
/// use supercli::clap::create_help_styles;
///
/// #[derive(Parser)]
/// #[command(
///     name = "my-tool",
///     about = "My awesome CLI tool",
///     styles = create_help_styles()
/// )]
/// pub struct Cli {
///     // ... your fields
/// }
/// ```
/// Create help styles with custom app prefix for environment variables
#[cfg(feature = "clap")]
pub fn create_help_styles_with_prefix(app_prefix: &str) -> Styles {
    // Set theme from environment first
    set_theme_from_env_with_prefix(app_prefix);

    match get_output_style_with_prefix(app_prefix) {
        "color" => {
            Styles::default()
                // Headers (e.g., "Commands:", "Options:") - use theme-aware label color
                .header(
                    Style::new()
                        .bold()
                        .fg_color(Some(Color::from(StarbaseStyle::Label.ansi_color()))),
                )
                // Usage line at the top - use theme-aware property color
                .usage(
                    Style::new()
                        .bold()
                        .fg_color(Some(Color::from(StarbaseStyle::Property.ansi_color()))),
                )
                // Commands, options, and argument names - use theme-aware shell color
                .literal(
                    Style::new().fg_color(Some(Color::from(StarbaseStyle::Shell.ansi_color()))),
                )
                // Placeholders like <VALUE>, [OPTIONS] - use theme-aware muted color
                .placeholder(
                    Style::new().fg_color(Some(Color::from(StarbaseStyle::Muted.ansi_color()))),
                )
                // Error messages - use theme-aware failure color
                .error(
                    Style::new()
                        .bold()
                        .fg_color(Some(Color::from(StarbaseStyle::Failure.ansi_color()))),
                )
                // Valid values in help - use theme-aware success color
                .valid(
                    Style::new().fg_color(Some(Color::from(StarbaseStyle::Success.ansi_color()))),
                )
                // Invalid values or warnings - use theme-aware caution color
                .invalid(
                    Style::new().fg_color(Some(Color::from(StarbaseStyle::Caution.ansi_color()))),
                )
        }
        "monochrome" => {
            Styles::default()
                .header(Style::new().bold()) // Bold headers
                .usage(Style::new().bold()) // Bold usage line
                .error(Style::new().bold()) // Bold errors
            // literal, placeholder, valid, invalid remain default (no special styling)
        }
        "none" => {
            // Completely plain text
            Styles::default()
        }
        _ => {
            // Default fallback - completely plain text
            Styles::default()
        }
    }
}

/// Create help styles using default "GUARDY" prefix
#[cfg(feature = "clap")]
pub fn create_help_styles() -> Styles {
    create_help_styles_with_prefix("GUARDY")
}

/// Create minimal clap help styles (less colorful, more conservative)
///
/// For environments where you want subtle coloring without being too flashy.
#[cfg(feature = "clap")]
pub fn create_minimal_help_styles() -> Styles {
    Styles::default()
        .header(Style::new().bold())
        .usage(Style::new().bold())
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .error(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))))
}

/// Check output style preference from environment variables
///
/// Supports APP_OUTPUT_STYLE with values:
/// - color: Full colored output (default)
/// - monochrome: Bold/styling but no colors
/// - none: Completely plain text
///
/// Also respects standard NO_COLOR and FORCE_COLOR variables.
#[cfg(feature = "clap")]
pub fn get_output_style_with_prefix(app_prefix: &str) -> &'static str {
    let app_var = format!("{}_OUTPUT_STYLE", app_prefix.to_uppercase());

    // Check app-specific style first
    if let Ok(style) = std::env::var(&app_var) {
        match style.to_lowercase().as_str() {
            "color" | "colour" => "color",
            "monochrome" | "mono" => "monochrome",
            "none" | "plain" => "none",
            _ => "color", // default for invalid values
        }
    }
    // Check NO_COLOR (universal standard)
    else if let Ok(no_color) = std::env::var("NO_COLOR") {
        if !no_color.is_empty() {
            "none"
        } else {
            "color"
        }
    }
    // Check FORCE_COLOR or if we're in a terminal
    else if std::env::var("FORCE_COLOR").is_ok() || atty::is(atty::Stream::Stdout) {
        "color"
    } else {
        "none"
    }
}

/// Check output style preference using default "GUARDY" prefix
#[cfg(feature = "clap")]
pub fn get_output_style() -> &'static str {
    get_output_style_with_prefix("GUARDY")
}

/// Set theme override from environment variable
///
/// Supports APP_OUTPUT_THEME with values:
/// - light: Force light theme
/// - dark: Force dark theme
/// - auto: Use starbase default detection (default)
#[cfg(feature = "clap")]
pub fn set_theme_from_env_with_prefix(app_prefix: &str) {
    let theme_var = format!("{}_OUTPUT_THEME", app_prefix.to_uppercase());

    if let Ok(theme) = std::env::var(&theme_var) {
        match theme.to_lowercase().as_str() {
            "light" => unsafe { std::env::set_var("STARBASE_THEME", "light") },
            "dark" => unsafe { std::env::set_var("STARBASE_THEME", "dark") },
            "auto" => unsafe { std::env::remove_var("STARBASE_THEME") },
            _ => {} // ignore invalid values
        }
    }
}

/// Set theme override using default "GUARDY" prefix
#[cfg(feature = "clap")]
pub fn set_theme_from_env() {
    set_theme_from_env_with_prefix("GUARDY")
}

/// Get appropriate clap styles based on environment
///
/// Automatically chooses between colored and plain styles based on
/// environment variables and terminal detection.
#[cfg(feature = "clap")]
pub fn get_adaptive_help_styles() -> Styles {
    create_help_styles() // Now handles all modes internally
}

/// Get optional clap styles - returns None when style is "none" to let clap use defaults
///
/// This allows clap to use its built-in styling behavior when colors are disabled,
/// which may include some formatting like bold headers.
#[cfg(feature = "clap")]
pub fn get_optional_help_styles() -> Option<Styles> {
    match get_output_style() {
        "none" => None, // Let clap use its default styling
        _ => Some(create_help_styles()),
    }
}
