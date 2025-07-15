//! Security module tests

use super::patterns::patterns_from_config;
use super::*;
use crate::config::{GuardyConfig, SecurityPatternConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_severity_parsing() {
    use super::patterns::parse_severity;

    assert_eq!(parse_severity("Critical"), Severity::Critical);
    assert_eq!(parse_severity("critical"), Severity::Critical);
    assert_eq!(parse_severity("CRITICAL"), Severity::Critical);
    assert_eq!(parse_severity("Info"), Severity::Info);
    assert_eq!(parse_severity("info"), Severity::Info);
    assert_eq!(parse_severity("INFO"), Severity::Info);
    assert_eq!(parse_severity("Unknown"), Severity::Critical); // Default to Critical
}

#[test]
fn test_security_pattern_creation() {
    let pattern_config = SecurityPatternConfig {
        name: "Test API Key".to_string(),
        regex: r"test_[a-zA-Z0-9]{10}".to_string(),
        severity: "Info".to_string(),
        description: "Test API key pattern".to_string(),
        enabled: true,
    };

    let pattern = SecurityPattern::new(
        pattern_config.name.clone(),
        &pattern_config.regex,
        super::patterns::parse_severity(&pattern_config.severity),
        pattern_config.description.clone(),
    )
    .expect("Failed to create pattern");

    assert_eq!(pattern.name, "Test API Key");
    assert_eq!(pattern.severity, Severity::Info);
    assert_eq!(pattern.description, "Test API key pattern");

    // Test regex functionality
    assert!(pattern.regex.is_match("test_abcdef1234"));
    assert!(!pattern.regex.is_match("invalid_pattern"));
}

#[test]
fn test_patterns_from_config() {
    let pattern_configs = vec![
        SecurityPatternConfig {
            name: "API Key".to_string(),
            regex: r"api_[a-zA-Z0-9]+".to_string(),
            severity: "Critical".to_string(),
            description: "API key".to_string(),
            enabled: true,
        },
        SecurityPatternConfig {
            name: "Disabled Pattern".to_string(),
            regex: r"disabled_[a-zA-Z0-9]+".to_string(),
            severity: "Info".to_string(),
            description: "Disabled pattern".to_string(),
            enabled: false,
        },
    ];

    let patterns = patterns_from_config(&pattern_configs).expect("Failed to create patterns");

    // Should only include enabled patterns
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].name, "API Key");
    assert_eq!(patterns[0].severity, Severity::Critical);
}

#[test]
fn test_secret_scanner_from_config() {
    let config = GuardyConfig::default();
    let scanner = SecretScanner::from_config(&config).expect("Failed to create scanner");

    // Scanner should be created successfully
    // Note: Default config has no patterns, so scanner will have empty patterns list
    // We can't test pattern count directly since it's private, but creation success indicates it works
}

#[test]
fn test_secret_scanner_scan_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.rs");

    // Create test file with secrets
    fs::write(
        &test_file,
        r#"
fn main() {
    let api_key = "sk_test_1234567890abcdef1234567890";
    let github_token = "ghp_123456789012345678901234567890123456";
    let normal_var = "just_a_normal_string";
}
"#,
    )
    .expect("Failed to write test file");

    // Create config with test patterns
    let mut config = GuardyConfig::default();
    config.security.patterns = vec![
        SecurityPatternConfig {
            name: "OpenAI API Key".to_string(),
            regex: r"sk_test_[a-zA-Z0-9]{26}".to_string(),
            severity: "Critical".to_string(),
            description: "OpenAI test API key".to_string(),
            enabled: true,
        },
        SecurityPatternConfig {
            name: "GitHub PAT".to_string(),
            regex: r"ghp_[a-zA-Z0-9]{36}".to_string(),
            severity: "Critical".to_string(),
            description: "GitHub Personal Access Token".to_string(),
            enabled: true,
        },
    ];
    config.security.use_gitignore = false; // Disable for test

    let scanner = SecretScanner::from_config(&config).expect("Failed to create scanner");

    let matches = scanner.scan_file(&test_file).expect("Failed to scan file");

    // Should find 2 secrets
    assert_eq!(matches.len(), 2);

    // Check first match (OpenAI key)
    assert_eq!(matches[0].pattern_name, "OpenAI API Key");
    assert_eq!(matches[0].line_number, 3);
    assert_eq!(matches[0].severity, Severity::Critical);
    assert!(matches[0].content.contains("sk_test_"));

    // Check second match (GitHub token)
    assert_eq!(matches[1].pattern_name, "GitHub PAT");
    assert_eq!(matches[1].line_number, 4);
    assert_eq!(matches[1].severity, Severity::Critical);
    assert!(
        matches[1]
            .content
            .contains("ghp_123456789012345678901234567890123456")
    );
}

#[test]
fn test_secret_scanner_exclude_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("secrets.log");

    fs::write(&test_file, "api_key=secret123").expect("Failed to write test file");

    let mut config = GuardyConfig::default();
    config.security.patterns = vec![SecurityPatternConfig {
        name: "API Key".to_string(),
        regex: r"api_key=[a-zA-Z0-9]+".to_string(),
        severity: "Critical".to_string(),
        description: "API key".to_string(),
        enabled: true,
    }];
    config.security.exclude_patterns = vec!["*.log".to_string()];
    config.security.use_gitignore = false;

    let scanner = SecretScanner::from_config(&config).expect("Failed to create scanner");

    // File should be excluded due to .log extension - test via scan_files

    let matches = scanner
        .scan_files(&[&test_file])
        .expect("Failed to scan files");
    assert_eq!(matches.len(), 0); // Should be excluded
}

#[test]
fn test_secret_scanner_file_filtering() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create test files
    let log_file = temp_dir.path().join("debug.log");
    let rs_file = temp_dir.path().join("main.rs");

    fs::write(&log_file, "some content").expect("Failed to write log file");
    fs::write(&rs_file, "some content").expect("Failed to write rs file");

    let mut config = GuardyConfig::default();
    config.security.exclude_patterns = vec!["*.log".to_string()];
    config.security.use_gitignore = false;

    let scanner = SecretScanner::from_config(&config).expect("Failed to create scanner");

    // Test scanning files - excluded files should not produce matches
    let excluded_matches = scanner
        .scan_files(&[&log_file])
        .expect("Failed to scan excluded files");
    let included_matches = scanner
        .scan_files(&[&rs_file])
        .expect("Failed to scan included files");

    // Log files should be excluded (no matches), RS files should be included
    assert_eq!(excluded_matches.len(), 0);
    assert_eq!(included_matches.len(), 0); // No patterns match empty files
}

#[test]
fn test_security_match_display() {
    let security_match = SecurityMatch {
        file_path: "src/main.rs".to_string(),
        line_number: 42,
        column: 15,
        content: "sk_test_1234567890".to_string(),
        pattern_name: "OpenAI API Key".to_string(),
        severity: Severity::Critical,
    };

    // Test that the match was created correctly
    assert_eq!(security_match.file_path, "src/main.rs");
    assert_eq!(security_match.line_number, 42);
    assert_eq!(security_match.column, 15);
    assert_eq!(security_match.pattern_name, "OpenAI API Key");
    assert_eq!(security_match.severity, Severity::Critical);
}

#[test]
fn test_invalid_regex_pattern() {
    let pattern_config = SecurityPatternConfig {
        name: "Invalid Pattern".to_string(),
        regex: "[invalid_regex".to_string(), // Missing closing bracket
        severity: "Critical".to_string(),
        description: "Invalid regex".to_string(),
        enabled: true,
    };

    let result = SecurityPattern::new(
        pattern_config.name.clone(),
        &pattern_config.regex,
        super::patterns::parse_severity(&pattern_config.severity),
        pattern_config.description.clone(),
    );
    assert!(result.is_err());
}

#[test]
fn test_scanner_directory_scan() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");

    let main_file = src_dir.join("main.rs");
    let lib_file = src_dir.join("lib.rs");
    let log_file = temp_dir.path().join("debug.log");

    fs::write(
        &main_file,
        "let token = \"ghp_123456789012345678901234567890123456\";",
    )
    .expect("Failed to write main.rs");
    fs::write(
        &lib_file,
        "let key = \"sk_test_1234567890abcdef1234567890\";",
    )
    .expect("Failed to write lib.rs");
    fs::write(&log_file, "let secret = \"should_be_excluded\";").expect("Failed to write log file");

    let mut config = GuardyConfig::default();
    config.security.patterns = vec![
        SecurityPatternConfig {
            name: "GitHub PAT".to_string(),
            regex: r"ghp_[a-zA-Z0-9]{36}".to_string(),
            severity: "Critical".to_string(),
            description: "GitHub token".to_string(),
            enabled: true,
        },
        SecurityPatternConfig {
            name: "OpenAI Key".to_string(),
            regex: r"sk_test_[a-zA-Z0-9]{26}".to_string(),
            severity: "Critical".to_string(),
            description: "OpenAI key".to_string(),
            enabled: true,
        },
    ];
    config.security.exclude_patterns = vec!["*.log".to_string()];
    config.security.use_gitignore = false;

    let scanner = SecretScanner::from_config(&config).expect("Failed to create scanner");

    let matches = scanner
        .scan_directory(temp_dir.path())
        .expect("Failed to scan directory");

    // Should find secrets in .rs files but not in .log files
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().any(|m| m.pattern_name == "GitHub PAT"));
    assert!(matches.iter().any(|m| m.pattern_name == "OpenAI Key"));
    assert!(matches.iter().all(|m| m.file_path.ends_with(".rs")));
}
