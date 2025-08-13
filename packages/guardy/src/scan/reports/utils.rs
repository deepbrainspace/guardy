//! Common utilities for report generation

use super::{ReportConfig, RedactionStyle};
use crate::scan::data::SecretMatch;

/// Get display value for a secret in a specific report format (context-aware)
pub fn get_secret_display_value_for_format(
    secret: &SecretMatch, 
    config: &ReportConfig, 
    format: super::ReportFormat
) -> String {
    if config.display_secrets {
        secret.matched_text.clone()
    } else {
        let style = config.redaction_style_for_format(format);
        redact_secret_with_style(&secret.matched_text, style)
    }
}

/// Redact a secret according to the specified style
pub fn redact_secret_with_style(secret: &str, style: RedactionStyle) -> String {
    match style {
        RedactionStyle::Full => "**REDACTED**".to_string(),
        RedactionStyle::Partial => {
            let len = secret.len();
            if len <= 8 {
                "*".repeat(len)
            } else {
                format!("{}...{}", 
                    &secret[..3.min(len)],
                    &secret[len.saturating_sub(3)..])
            }
        },
        RedactionStyle::Length => "*".repeat(secret.len()),
    }
}

/// HTML escape a string for safe inclusion in HTML
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) as f64 / 1000.0;
        format!("{}m {:.1}s", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secret() {
        assert_eq!(redact_secret("short", RedactionStyle::Full), "**REDACTED**");
        assert_eq!(redact_secret("short", RedactionStyle::Partial), "*****");
        assert_eq!(redact_secret("short", RedactionStyle::Length), "*****");
        
        assert_eq!(redact_secret("sk-1234567890abcdef", RedactionStyle::Partial), "sk-...def");
        assert_eq!(redact_secret("sk-1234567890abcdef", RedactionStyle::Length), "******************");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>alert('xss')</script>"), 
                   "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(2_097_152), "2.0 MB");
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(65_000), "1m 5.0s");
    }
}