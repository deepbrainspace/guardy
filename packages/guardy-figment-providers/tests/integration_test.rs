use figment::{Figment, providers::{Format, Json}};
use guardy_figment_providers::{SmartFormat, SkipEmpty, NestedEnv};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TestConfig {
    name: String,
    version: String,
    database: DatabaseConfig,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ExtendedTestConfig {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    count: u32,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[test]
fn test_smartformat_with_figment_chain() {
    // Test that SmartFormat integrates properly with Figment chains
    let json_content = r#"{"name": "test-app", "version": "1.0.0", "database": {"host": "localhost", "port": 5432}}"#;
    let toml_content = r#"name = "override-app"
version = "2.0.0"
[database]
host = "production"
port = 3306"#;
    
    let figment = Figment::new()
        .merge(SmartFormat::string(json_content))    // Should detect JSON
        .merge(SmartFormat::string(toml_content));   // Should detect TOML and override
    
    let config: TestConfig = figment.extract().expect("Failed to extract config");
    
    // TOML should override JSON values
    assert_eq!(config.name, "override-app");
    assert_eq!(config.version, "2.0.0");
    assert_eq!(config.database.host, "production");
    assert_eq!(config.database.port, 3306);
}

#[test] 
fn test_smartformat_yaml_detection() {
    let yaml_content = r#"
name: yaml-app
version: 3.0.0
database:
  host: yaml-host
  port: 8080
"#;
    
    let figment = Figment::new().merge(SmartFormat::string(yaml_content));
    let config: TestConfig = figment.extract().expect("Failed to extract YAML config");
    
    assert_eq!(config.name, "yaml-app");
    assert_eq!(config.version, "3.0.0");
    assert_eq!(config.database.host, "yaml-host");
    assert_eq!(config.database.port, 8080);
}

#[test]
fn test_smartformat_empty_content_fallback() {
    // Empty content should default to TOML and not crash
    let figment = Figment::new().merge(SmartFormat::string(""));
    
    // Should not panic - empty TOML is valid (results in empty config)
    let result = figment.extract::<serde_json::Value>();
    assert!(result.is_ok());
}

#[test]
fn test_skipempty_with_cli_args() {
    // Test that SkipEmpty filters out empty CLI values as intended
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct CliArgs {
        name: Option<String>,
        tags: Vec<String>,
        count: Option<u32>,
        verbose: bool,
    }
    
    let cli_args = CliArgs {
        name: Some("test-app".to_string()),  // Should be kept
        tags: Vec::new(),                    // Empty - should be filtered
        count: None,                         // None - should be filtered
        verbose: false,                      // false is a valid value - should be kept
    };
    
    let figment = Figment::new()
        .merge(SkipEmpty::new(cli_args));
    
    let result: serde_json::Value = figment.extract().expect("Failed to extract config");
    
    // Only name and verbose should be present in the result
    let expected = serde_json::json!({
        "name": "test-app",
        "verbose": false
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_skipempty_in_figment_chain() {
    // Test that SkipEmpty works properly in a Figment chain, 
    // allowing meaningful config values to be preserved
    
    // Base config with some meaningful values
    let base_config = serde_json::json!({
        "name": "base-app",
        "tags": ["prod", "api"],
        "count": 42,
        "verbose": true
    });
    
    // CLI args with some empty values that shouldn't override the base config
    #[derive(Debug, serde::Serialize)]
    struct CliOverrides {
        name: Option<String>,
        tags: Vec<String>,    // Empty - shouldn't override base tags
        count: Option<u32>,   // None - shouldn't override base count
    }
    
    let cli_overrides = CliOverrides {
        name: Some("overridden-app".to_string()),  // Should override
        tags: Vec::new(),                          // Empty - should be filtered out
        count: None,                               // None - should be filtered out
    };
    
    let figment = Figment::new()
        .merge(Json::string(&base_config.to_string()))
        .merge(SkipEmpty::new(cli_overrides));  // Empty values filtered, won't override base
    
    let result: ExtendedTestConfig = figment.extract().expect("Failed to extract config");
    
    // name should be overridden, but tags and count should come from base config
    assert_eq!(result.name, "overridden-app");  // Overridden by CLI
    assert_eq!(result.tags, vec!["prod", "api"]); // From base config (CLI empty was filtered)
    assert_eq!(result.count, 42);               // From base config (CLI None was filtered)
}

#[test]
fn test_nestedenv_basic_functionality() {
    // Test that NestedEnv creates proper nested structures from environment variables
    
    #[derive(Debug, PartialEq, Deserialize)]
    struct Config {
        database: DatabaseConfig,
        server: ServerConfig,
        debug: bool,
    }
    
    #[derive(Debug, PartialEq, Deserialize)]
    struct ServerConfig {
        port: u16,
        workers: u32,
    }
    
    // Set up test environment variables
    env::set_var("MYAPP_DATABASE_HOST", "localhost");
    env::set_var("MYAPP_DATABASE_PORT", "5432");
    env::set_var("MYAPP_SERVER_PORT", "8080");
    env::set_var("MYAPP_SERVER_WORKERS", "4");
    env::set_var("MYAPP_DEBUG", "true");
    
    let figment = Figment::new()
        .merge(NestedEnv::prefixed("MYAPP_"));
    
    let result: Config = figment.extract().expect("Failed to extract config from NestedEnv");
    
    // Verify nested structure was created correctly
    assert_eq!(result.database.host, "localhost");
    assert_eq!(result.database.port, 5432);
    assert_eq!(result.server.port, 8080);
    assert_eq!(result.server.workers, 4);
    assert_eq!(result.debug, true);
    
    // Clean up test environment variables
    env::remove_var("MYAPP_DATABASE_HOST");
    env::remove_var("MYAPP_DATABASE_PORT");
    env::remove_var("MYAPP_SERVER_PORT");
    env::remove_var("MYAPP_SERVER_WORKERS");
    env::remove_var("MYAPP_DEBUG");
}

#[test]
fn test_nestedenv_in_figment_chain() {
    // Test that NestedEnv works properly in a Figment chain with other providers
    
    #[derive(Debug, PartialEq, Deserialize)]
    struct Config {
        name: String,
        database: DatabaseConfig,
        debug: bool,
    }
    
    // Base config from file
    let base_config = serde_json::json!({
        "name": "base-app",
        "database": {
            "host": "base-host",
            "port": 3000
        },
        "debug": false
    });
    
    // Environment variables that should override base config
    env::set_var("OVERRIDE_DATABASE_HOST", "env-host");
    env::set_var("OVERRIDE_DEBUG", "true");
    // Note: DATABASE_PORT not set, should keep base value
    
    let figment = Figment::new()
        .merge(Json::string(&base_config.to_string()))  // Base config
        .merge(NestedEnv::prefixed("OVERRIDE_"));        // Environment overrides
    
    let result: Config = figment.extract().expect("Failed to extract config from chain");
    
    // name should be from base config (no env var)
    assert_eq!(result.name, "base-app");
    
    // database.host should be from env (overridden)  
    assert_eq!(result.database.host, "env-host");
    
    // database.port should be from base config (not overridden by env)
    assert_eq!(result.database.port, 3000);
    
    // debug should be from env (overridden)
    assert_eq!(result.debug, true);
    
    // Clean up test environment variables
    env::remove_var("OVERRIDE_DATABASE_HOST");
    env::remove_var("OVERRIDE_DEBUG");
}

#[test]
fn test_all_three_providers_together() {
    // Test that SmartFormat, SkipEmpty, and NestedEnv work together
    
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Config {
        name: String,
        database: DatabaseConfig,
        debug: bool,
        extra_tags: Vec<String>,
    }
    
    #[derive(Debug, Serialize)]
    struct CliArgs {
        name: Option<String>,
        extra_tags: Vec<String>, // Will be empty
        debug: Option<bool>,     // Will be None
    }
    
    // 1. Base config as TOML string (SmartFormat will detect)
    let toml_config = r#"
        name = "base-app"
        debug = false
        extra_tags = ["base", "config"]
        
        [database]
        host = "base-host"
        port = 3000
    "#;
    
    // 2. Environment variables (NestedEnv will process)
    env::set_var("APP_DATABASE_HOST", "env-host");
    env::set_var("APP_DATABASE_PORT", "5432");
    
    // 3. CLI args with some empty values (SkipEmpty will filter)
    let cli_args = CliArgs {
        name: Some("final-app".to_string()), // Should override
        extra_tags: Vec::new(),              // Empty - should be filtered, keeping base
        debug: None,                         // None - should be filtered, keeping env/base
    };
    
    let figment = Figment::new()
        .merge(SmartFormat::string(toml_config))  // 1. Base config (auto-detects TOML)
        .merge(NestedEnv::prefixed("APP_"))       // 2. Environment overrides  
        .merge(SkipEmpty::new(cli_args));         // 3. CLI overrides (empty filtered)
    
    let result: Config = figment.extract().expect("Failed to extract config with all 3 providers");
    
    // name: from CLI (highest priority)
    assert_eq!(result.name, "final-app");
    
    // database.host: from environment (middle priority)
    assert_eq!(result.database.host, "env-host");
    
    // database.port: from environment (middle priority) 
    assert_eq!(result.database.port, 5432);
    
    // debug: from base config (CLI None was filtered)
    assert_eq!(result.debug, false);
    
    // extra_tags: from base config (CLI empty was filtered)
    assert_eq!(result.extra_tags, vec!["base", "config"]);
    
    // Clean up test environment variables
    env::remove_var("APP_DATABASE_HOST");
    env::remove_var("APP_DATABASE_PORT");
}