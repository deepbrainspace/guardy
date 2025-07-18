//! Configuration tests

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = GuardyConfig::default();

    assert!(config.security.secret_detection);
    assert!(config.security.use_gitignore);
    assert_eq!(config.security.exclude_patterns.len(), 2);
    assert!(
        config
            .security
            .exclude_patterns
            .contains(&"*.tmp".to_string())
    );
    assert!(
        config
            .security
            .exclude_patterns
            .contains(&"*.temp".to_string())
    );
    assert!(
        config
            .security
            .protected_branches
            .contains(&"main".to_string())
    );
    assert!(!config.security.git_crypt);

    assert!(config.hooks.pre_commit);
    assert!(config.hooks.commit_msg);
    assert!(config.hooks.pre_push);
    assert_eq!(config.hooks.timeout, 300);

    assert!(!config.mcp.enabled);
    assert_eq!(config.mcp.port, 8080);
    assert_eq!(config.mcp.host, "localhost");

    assert!(config.tools.auto_detect);
}

#[test]
fn test_config_serialization() {
    let config = GuardyConfig::default();
    let yaml = serde_yaml::to_string(&config).expect("Failed to serialize config");

    // Should contain expected sections
    assert!(yaml.contains("security:"));
    assert!(yaml.contains("hooks:"));
    assert!(yaml.contains("mcp:"));
    assert!(yaml.contains("tools:"));
}

#[test]
fn test_config_deserialization() {
    let yaml = r#"
security:
  secret_detection: true
  use_gitignore: false
  exclude_patterns:
    - "*.test"
  protected_branches:
    - "main"
  git_crypt: false
  patterns:
    - name: "Test Pattern"
      regex: "test.*"

hooks:
  pre_commit: true
  commit_msg: false
  pre_push: true
  timeout: 600

mcp:
  enabled: true
  port: 9090
  host: "0.0.0.0"
  daemon: true

tools:
  auto_detect: false
  formatters: []
  linters: []
"#;

    let config: GuardyConfig = serde_yaml::from_str(yaml).expect("Failed to deserialize config");

    assert!(config.security.secret_detection);
    assert!(!config.security.use_gitignore);
    assert_eq!(config.security.exclude_patterns, vec!["*.test"]);
    assert_eq!(config.security.protected_branches, vec!["main"]);
    assert!(!config.security.git_crypt);
    assert_eq!(config.security.patterns.len(), 1);
    assert_eq!(config.security.patterns[0].name, "Test Pattern");

    assert!(config.hooks.pre_commit);
    assert!(!config.hooks.commit_msg);
    assert!(config.hooks.pre_push);
    assert_eq!(config.hooks.timeout, 600);

    assert!(config.mcp.enabled);
    assert_eq!(config.mcp.port, 9090);
    assert_eq!(config.mcp.host, "0.0.0.0");
    assert!(config.mcp.daemon);

    assert!(!config.tools.auto_detect);
}

#[test]
fn test_pattern_defaults() {
    let yaml = r#"
security:
  secret_detection: true
  exclude_patterns: []
  protected_branches: ["main"]
  git_crypt: false
  patterns:
    - name: "Test Pattern"
      regex: "test.*"
    - name: "Another Pattern"
      regex: "another.*"
      severity: "Info"
      enabled: false

hooks:
  pre_commit: true
  commit_msg: true
  pre_push: true
  timeout: 300

mcp:
  enabled: false
  port: 8080
  host: "localhost"
  daemon: false

tools:
  auto_detect: true
  formatters: []
  linters: []
"#;

    let config: GuardyConfig = serde_yaml::from_str(yaml).expect("Failed to deserialize");

    // First pattern should use defaults
    assert_eq!(config.security.patterns[0].severity, "Critical");
    assert!(config.security.patterns[0].enabled);
    assert_eq!(config.security.patterns[0].description, "");

    // Second pattern has explicit values
    assert_eq!(config.security.patterns[1].severity, "Info");
    assert!(!config.security.patterns[1].enabled);
}

#[test]
fn test_load_save_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_guardy.yml");

    let original_config = GuardyConfig::default();

    // Save config
    original_config
        .save_to_file(&config_path)
        .expect("Failed to save config");

    // Load config
    let loaded_config = GuardyConfig::load_from_file(&config_path).expect("Failed to load config");

    // Compare configs
    assert_eq!(
        original_config.security.secret_detection,
        loaded_config.security.secret_detection
    );
    assert_eq!(original_config.hooks.timeout, loaded_config.hooks.timeout);
    assert_eq!(original_config.mcp.port, loaded_config.mcp.port);
}

#[test]
fn test_config_validation() {
    let mut config = GuardyConfig::default();

    // Valid config should pass
    assert!(config.validate().is_ok());

    // Invalid MCP port
    config.mcp.enabled = true;
    config.mcp.port = 0;
    assert!(config.validate().is_err());

    // Fix port, invalid host
    config.mcp.port = 8080;
    config.mcp.host = "".to_string();
    assert!(config.validate().is_err());

    // Fix host, invalid timeout
    config.mcp.host = "localhost".to_string();
    config.hooks.timeout = 0;
    assert!(config.validate().is_err());

    // Fix timeout, empty protected branches
    config.hooks.timeout = 300;
    config.security.protected_branches.clear();
    assert!(config.validate().is_err());
}

#[test]
#[serial_test::serial]
fn test_gitignore_patterns_loading() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gitignore_path = temp_dir.path().join(".gitignore");

    // Create a test .gitignore file
    fs::write(
        &gitignore_path,
        r#"
# Comment line
*.log
target/
node_modules/
*.tmp

# Another comment
.env
"#,
    )
    .expect("Failed to write gitignore");

    // Change to temp directory
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Test loading patterns
    let patterns = GuardyConfig::load_gitignore_patterns().expect("Failed to load patterns");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    // Verify patterns (should exclude comments and empty lines)
    assert!(patterns.contains(&"*.log".to_string()));
    assert!(patterns.contains(&"target/".to_string()));
    assert!(patterns.contains(&"node_modules/".to_string()));
    assert!(patterns.contains(&"*.tmp".to_string()));
    assert!(patterns.contains(&".env".to_string()));
    assert!(!patterns.iter().any(|p| p.starts_with('#')));
    assert!(!patterns.iter().any(|p| p.trim().is_empty()));
}

#[test]
#[serial_test::serial]
fn test_effective_exclude_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gitignore_path = temp_dir.path().join(".gitignore");

    // Create test .gitignore
    fs::write(&gitignore_path, "*.log\ntarget/\n").expect("Failed to write gitignore");

    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let mut config = GuardyConfig::default();
    config
        .security
        .exclude_patterns
        .push("*.custom".to_string()); // Add to existing defaults
    config.security.use_gitignore = true;

    let effective_patterns = config.get_effective_exclude_patterns();

    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    // Should contain both config patterns and gitignore patterns
    assert!(effective_patterns.contains(&"*.custom".to_string()));
    assert!(effective_patterns.contains(&"*.tmp".to_string())); // from default
    assert!(effective_patterns.contains(&"*.temp".to_string())); // from default
    assert!(effective_patterns.contains(&"*.log".to_string())); // from gitignore
    assert!(effective_patterns.contains(&"target/".to_string())); // from gitignore
}

#[test]
#[serial_test::serial]
fn test_effective_exclude_patterns_disabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let guardyignore_path = temp_dir.path().join(".guardyignore");

    // Create test .guardyignore
    fs::write(&guardyignore_path, "TESTING.md\n*.test.rs\n").expect("Failed to write guardyignore");

    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let mut config = GuardyConfig::default();
    config
        .security
        .exclude_patterns
        .push("*.custom".to_string()); // Add to existing defaults
    config.security.use_gitignore = false;

    let effective_patterns = config.get_effective_exclude_patterns();

    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    // Should contain config patterns and guardyignore patterns (but not gitignore)
    assert!(effective_patterns.contains(&"*.custom".to_string()));
    assert!(effective_patterns.contains(&"*.tmp".to_string())); // from default
    assert!(effective_patterns.contains(&"*.temp".to_string())); // from default
    // guardyignore patterns are always loaded
    assert!(effective_patterns.contains(&"TESTING.md".to_string())); // from guardyignore
    assert!(effective_patterns.contains(&"*.test.rs".to_string())); // from guardyignore
    // Should not contain patterns from gitignore (since it's disabled)
    assert!(effective_patterns.len() >= 5); // *.custom + *.tmp + *.temp + TESTING.md + *.test.rs
}

#[test]
#[serial_test::serial]
fn test_guardyignore_patterns_loading() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let guardyignore_path = temp_dir.path().join(".guardyignore");

    // Create a test .guardyignore file
    fs::write(
        &guardyignore_path,
        r#"
# Guardy ignore file
TESTING.md
*.test.rs
test_*

# Another comment
docs/
"#,
    )
    .expect("Failed to write guardyignore");

    // Change to temp directory
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Test loading patterns
    let patterns = GuardyConfig::load_guardyignore_patterns().expect("Failed to load patterns");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    // Verify patterns (should exclude comments and empty lines)
    assert!(patterns.contains(&"TESTING.md".to_string()));
    assert!(patterns.contains(&"*.test.rs".to_string()));
    assert!(patterns.contains(&"test_*".to_string()));
    assert!(patterns.contains(&"docs/**".to_string()));
    assert!(!patterns.iter().any(|p| p.starts_with('#')));
    assert!(!patterns.iter().any(|p| p.trim().is_empty()));
}

#[test]
#[serial_test::serial]
fn test_effective_exclude_patterns_with_guardyignore() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let guardyignore_path = temp_dir.path().join(".guardyignore");

    // Create test .guardyignore
    fs::write(&guardyignore_path, "TESTING.md\n*.test.rs\n").expect("Failed to write guardyignore");

    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let mut config = GuardyConfig::default();
    config
        .security
        .exclude_patterns
        .push("*.custom".to_string()); // Add to existing defaults
    config.security.use_gitignore = false; // Disable gitignore for this test

    let effective_patterns = config.get_effective_exclude_patterns();

    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    // Should contain both config patterns and guardyignore patterns
    assert!(effective_patterns.contains(&"*.custom".to_string()));
    assert!(effective_patterns.contains(&"*.tmp".to_string())); // from default
    assert!(effective_patterns.contains(&"*.temp".to_string())); // from default
    assert!(effective_patterns.contains(&"TESTING.md".to_string())); // from guardyignore
    assert!(effective_patterns.contains(&"*.test.rs".to_string())); // from guardyignore
}
