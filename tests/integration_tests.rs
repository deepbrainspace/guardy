//! Integration tests for Guardy CLI

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test CLI binary exists and responds to --help
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git workflow protection tool"));
}

/// Test CLI responds to --version
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("guardy"));
}

/// Test invalid subcommand shows error
#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

/// Test configuration functionality
#[test]
fn test_config_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.yml");

    // Test config validation (should pass with default template)
    fs::copy("templates/guardy.yml.template", &config_path).unwrap();

    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("config")
        .arg("validate")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .success();
}

/// Test secret detection with temporary files
#[test]
fn test_secret_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files with secrets
    let secret_file = temp_dir.path().join("secrets.rs");
    fs::write(
        &secret_file,
        r#"
fn main() {
    let api_key = "sk_test_1234567890abcdef1234567890";
    let github_token = "ghp_1234567890abcdef1234567890abcdef12";
}
"#,
    )
    .unwrap();

    // Create config file
    let config_file = temp_dir.path().join(".guardy.yml");
    fs::write(
        &config_file,
        r#"
security:
  secret_detection: true
  use_gitignore: false
  patterns:
    - name: "OpenAI API Key"
      regex: "sk_test_[a-zA-Z0-9]{26}"
    - name: "GitHub PAT"
      regex: "ghp_[a-zA-Z0-9]{36}"

hooks:
  pre_commit: true
  commit_msg: true
  pre_push: true
  timeout: 300

mcp:
  enabled: false

tools:
  auto_detect: true
  formatters: []
  linters: []
"#,
    )
    .unwrap();

    // Test secret scanning
    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("scan")
        .arg("--file")
        .arg(&secret_file)
        .assert()
        .failure() // Should fail because secrets were found
        .stdout(
            predicate::str::contains("OpenAI API Key").or(predicate::str::contains("GitHub PAT")),
        );
}

/// Test gitignore integration
#[test]
fn test_gitignore_integration() {
    let temp_dir = TempDir::new().unwrap();

    // Create .gitignore file
    let gitignore = temp_dir.path().join(".gitignore");
    fs::write(&gitignore, "*.log\ntarget/\n.env\n").unwrap();

    // Create config with gitignore enabled
    let config_file = temp_dir.path().join(".guardy.yml");
    fs::write(
        &config_file,
        r#"
security:
  secret_detection: true
  use_gitignore: true
  exclude_patterns:
    - "*.custom"

hooks:
  pre_commit: false
  commit_msg: false 
  pre_push: false
  timeout: 300

mcp:
  enabled: false

tools:
  auto_detect: false
  formatters: []
  linters: []
"#,
    )
    .unwrap();

    // Create files that should be excluded
    let log_file = temp_dir.path().join("debug.log");
    fs::write(&log_file, "secret=hidden").unwrap();

    let env_file = temp_dir.path().join(".env");
    fs::write(&env_file, "API_KEY=secret123").unwrap();

    // Create file that should be scanned
    let rs_file = temp_dir.path().join("main.rs");
    fs::write(&rs_file, "fn main() {}").unwrap();

    // Test that excluded files are not scanned
    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("scan")
        .arg("--directory")
        .arg(".")
        .assert()
        .success(); // Should succeed because excluded files aren't scanned
}

/// Test template generation
#[test]
fn test_template_generation() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("--template")
        .assert()
        .success();

    // Check that config file was created
    let config_path = temp_dir.path().join("guardy.yml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("security:"));
    assert!(content.contains("hooks:"));
    assert!(content.contains("mcp:"));
    assert!(content.contains("tools:"));
}

/// Test hook installation (dry-run)
#[test]
fn test_hook_installation_dry_run() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize a git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("install")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Would install"));
}

/// Test MCP server configuration validation
#[test]
fn test_mcp_config_validation() {
    let temp_dir = TempDir::new().unwrap();

    // Create invalid MCP config
    let config_file = temp_dir.path().join(".guardy.yml");
    fs::write(
        &config_file,
        r#"
security:
  secret_detection: true
  patterns: []

hooks:
  pre_commit: true
  commit_msg: true
  pre_push: true
  timeout: 300

mcp:
  enabled: true
  port: 0  # Invalid port
  host: ""  # Invalid host

tools:
  auto_detect: true
  formatters: []
  linters: []
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("config")
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("port").or(predicate::str::contains("host")));
}

/// Test tool auto-detection
#[test]
fn test_tool_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create Rust project files
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("tools")
        .arg("detect")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust").or(predicate::str::contains("cargo")));
}

/// Test batch file operations
#[test]
fn test_batch_scanning() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    for i in 1..=5 {
        let file_path = temp_dir.path().join(format!("test{}.rs", i));
        fs::write(&file_path, format!("fn test_{}() {{}}", i)).unwrap();
    }

    // Create config
    let config_file = temp_dir.path().join(".guardy.yml");
    fs::copy("templates/guardy.yml.template", &config_file).unwrap();

    let mut cmd = Command::cargo_bin("guardy").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("scan")
        .arg("--directory")
        .arg(".")
        .assert()
        .success();
}
