//! Tool management tests

use super::*;
use crate::config::{FormatterConfig, InstallConfig, LinterConfig, ToolsConfig};

#[test]
fn test_tool_manager_creation() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    // Should create successfully
    assert!(!manager.auto_install);
}

#[test]
fn test_tool_manager_with_auto_install() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, true);
    
    assert!(manager.auto_install);
}

#[test]
fn test_has_cargo() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    // This test depends on environment, but cargo should be available in Rust projects
    let has_cargo = manager.has_cargo();
    // We can't assert specific value since it depends on environment
    // but the method should not panic
    println!("Has cargo: {}", has_cargo);
}

#[test]
fn test_is_tool_available_with_valid_command() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    // Test with a command that should always be available
    assert!(manager.is_tool_available("echo 'test'"));
}

#[test]
fn test_is_tool_available_with_invalid_command() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    // Test with a command that should not exist
    assert!(!manager.is_tool_available("nonexistent_command_12345 --version"));
}

#[test]
fn test_ensure_formatter_available_no_check_command() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    let formatter = FormatterConfig {
        name: "test_formatter".to_string(),
        command: "echo 'format'".to_string(),
        patterns: vec!["*.test".to_string()],
        check_command: None, // No check command
        install: None,
    };
    
    // Should succeed when no check command is provided
    assert!(manager.ensure_formatter_available(&formatter).is_ok());
}

#[test]
fn test_ensure_linter_available_no_check_command() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    let linter = LinterConfig {
        name: "test_linter".to_string(),
        command: "echo 'lint'".to_string(),
        patterns: vec!["*.test".to_string()],
        check_command: None, // No check command
        install: None,
    };
    
    // Should succeed when no check command is provided
    assert!(manager.ensure_linter_available(&linter).is_ok());
}

#[test]
fn test_ensure_tool_available_with_existing_tool() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false);
    
    let formatter = FormatterConfig {
        name: "echo".to_string(),
        command: "echo 'format'".to_string(),
        patterns: vec!["*.test".to_string()],
        check_command: Some("echo 'test' > /dev/null".to_string()), // Should succeed
        install: None,
    };
    
    // Should succeed since echo command exists
    assert!(manager.ensure_formatter_available(&formatter).is_ok());
}

#[test]
fn test_ensure_tool_available_missing_tool_no_auto_install() {
    let config = ToolsConfig::default();
    let manager = ToolManager::new(config, false); // auto_install = false
    
    let formatter = FormatterConfig {
        name: "nonexistent_tool".to_string(),
        command: "nonexistent_tool format".to_string(),
        patterns: vec!["*.test".to_string()],
        check_command: Some("nonexistent_tool --version".to_string()), // Should fail
        install: Some(InstallConfig {
            cargo: Some("cargo install nonexistent_tool".to_string()),
            npm: None,
            brew: None,
            apt: None,
            manual: "Install manually".to_string(),
        }),
    };
    
    // Should fail with installation instructions
    let result = manager.ensure_formatter_available(&formatter);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Required tool not available"));
}

#[test]
fn test_create_example_tools_config() {
    let config = create_example_tools_config();
    
    assert!(config.auto_detect);
    assert!(!config.auto_install); // Should default to false for security
    
    // Should have example formatters
    assert!(!config.formatters.is_empty());
    assert!(config.formatters.iter().any(|f| f.name == "rustfmt"));
    assert!(config.formatters.iter().any(|f| f.name == "prettier"));
    
    // Should have example linters
    assert!(!config.linters.is_empty());
    assert!(config.linters.iter().any(|l| l.name == "clippy"));
    assert!(config.linters.iter().any(|l| l.name == "eslint"));
    
    // All tools should have installation instructions
    for formatter in &config.formatters {
        assert!(formatter.install.is_some());
        let install = formatter.install.as_ref().unwrap();
        assert!(!install.manual.is_empty());
    }
    
    for linter in &config.linters {
        assert!(linter.install.is_some());
        let install = linter.install.as_ref().unwrap();
        assert!(!install.manual.is_empty());
    }
}

#[test]
fn test_install_config_structure() {
    let install_config = InstallConfig {
        cargo: Some("cargo install tool".to_string()),
        npm: Some("npm install -g tool".to_string()),
        brew: Some("brew install tool".to_string()),
        apt: Some("apt install tool".to_string()),
        manual: "Manual instructions".to_string(),
    };
    
    assert_eq!(install_config.cargo.unwrap(), "cargo install tool");
    assert_eq!(install_config.npm.unwrap(), "npm install -g tool");
    assert_eq!(install_config.brew.unwrap(), "brew install tool");
    assert_eq!(install_config.apt.unwrap(), "apt install tool");
    assert_eq!(install_config.manual, "Manual instructions");
}