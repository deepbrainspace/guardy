//! Security patterns for secret detection
//!
//! This module provides functions to load security patterns from configuration
//! and convert them to the internal SecurityPattern format.

use super::{SecurityPattern, Severity};
use crate::config::SecurityPatternConfig;
use anyhow::Result;

/// Convert severity string to enum
pub fn parse_severity(severity: &str) -> Severity {
    match severity.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "info" => Severity::Info,
        _ => Severity::Critical, // Default to critical - secrets are serious
    }
}

/// Convert configuration patterns to security patterns
pub fn patterns_from_config(
    config_patterns: &[SecurityPatternConfig],
) -> Result<Vec<SecurityPattern>> {
    let mut patterns = Vec::new();

    for config_pattern in config_patterns {
        if !config_pattern.enabled {
            continue;
        }

        let pattern = SecurityPattern::new(
            config_pattern.name.clone(),
            &config_pattern.regex,
            parse_severity(&config_pattern.severity),
            config_pattern.description.clone(),
        )?;

        patterns.push(pattern);
    }

    Ok(patterns)
}
