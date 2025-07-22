//! Integration tests for SuperFigment functionality
//!
//! These tests validate the complete SuperFigment feature set including:
//! - SuperFigment builder patterns
//! - Extension trait functionality  
//! - Provider integration
//! - Array merging capabilities
//! - Format detection and caching

use figment::Figment;
use superfigment::{SuperFigment, Universal};
use superfigment::{ExtendExt, FluentExt, AccessExt};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use std::fs;
use std::env;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestConfig {
    #[serde(default)]
    host: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DatabaseConfig {
    #[serde(default)]
    url: String,
    #[serde(default)]
    timeout: u32,
    #[serde(default)]
    allowed_origins: Vec<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            features: vec![],
            database: DatabaseConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost".to_string(),
            timeout: 30,
            allowed_origins: vec![],
        }
    }
}

#[test]
fn test_superfigment_builder_basic() {
    let config = SuperFigment::new()
        .with_defaults(TestConfig::default());
        
    let result: TestConfig = config.extract().expect("Failed to extract config");
    assert_eq!(result.host, "localhost");
    assert_eq!(result.port, 8080);
}

#[test]
fn test_superfigment_deref_compatibility() {
    let super_config = SuperFigment::new()
        .with_defaults(TestConfig::default());
    
    // Test that SuperFigment can be used as a Figment via Deref
    let _figment_ref: &Figment = &*super_config;
    
    // Test method calls that should work via Deref
    let profiles: Vec<_> = super_config.profiles().collect();
    assert!(!profiles.is_empty());
}

#[test]
fn test_extension_traits_individual() -> Result<(), Box<dyn std::error::Error>> {
    // Create temp files for testing
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.json");
    
    fs::write(&config_path, r#"{"host": "example.com", "features": ["auth"]}"#)?;
    
    // Test ExtendExt
    let figment_with_extend = Figment::new()
        .merge_extend(Universal::file(&config_path));
        
    let config: TestConfig = figment_with_extend.extract()?;
    assert_eq!(config.host, "example.com");
    
    // Test FluentExt (which includes ExtendExt automatically)
    let figment_with_fluent = Figment::new()
        .with_file(&config_path);
        
    let config2: TestConfig = figment_with_fluent.extract()?;
    assert_eq!(config2.host, "example.com");
    
    // Test AccessExt
    let json_output = figment_with_fluent.as_json()?;
    assert!(json_output.contains("example.com"));
    
    Ok(())
}

#[test]
fn test_extension_traits_all_ext() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.toml");
    
    fs::write(&config_path, r#"
host = "toml.example.com"
port = 9090

[database]
url = "mysql://localhost"
timeout = 60
"#)?;
    
    // Test combined trait functionality
    let figment = Figment::new()
        .merge_extend(figment::providers::Serialized::defaults(TestConfig::default())) // ExtendExt - defaults first
        .with_file(&config_path); // FluentExt - file overrides
        
    let config: TestConfig = figment.extract()?;
    assert_eq!(config.host, "toml.example.com");
    assert_eq!(config.port, 9090);
    assert_eq!(config.database.url, "mysql://localhost");
    
    // Test AccessExt methods
    let host = figment.get_string("host")?;
    assert_eq!(host, "toml.example.com");
    
    let has_database = figment.has_key("database")?;
    assert!(has_database);
    
    let keys = figment.keys()?;
    assert!(keys.contains(&"host".to_string()));
    assert!(keys.contains(&"database".to_string()));
    
    Ok(())
}

#[test]
fn test_array_merging_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    
    // Create base config
    let base_config = temp_dir.path().join("base.json");
    fs::write(&base_config, r#"
{
    "features": ["auth", "logging"],
    "database": {
        "allowed_origins": ["https://app.com", "https://admin.com"]
    }
}
"#)?;
    
    // Create override config with array operations
    let override_config = temp_dir.path().join("override.json");
    fs::write(&override_config, r#"
{
    "features_add": ["metrics", "tracing"],
    "features_remove": ["logging"],
    "database": {
        "allowed_origins_add": ["https://api.com"],
        "allowed_origins_remove": ["https://admin.com"]
    }
}
"#)?;
    
    let config = SuperFigment::new()
        .with_file(&base_config)
        .with_file(&override_config);
        
    let result: TestConfig = config.extract()?;
    
    // Verify array merging
    assert!(result.features.contains(&"auth".to_string()));
    assert!(result.features.contains(&"metrics".to_string()));
    assert!(result.features.contains(&"tracing".to_string()));
    assert!(!result.features.contains(&"logging".to_string())); // Should be removed
    
    assert!(result.database.allowed_origins.contains(&"https://app.com".to_string()));
    assert!(result.database.allowed_origins.contains(&"https://api.com".to_string()));
    assert!(!result.database.allowed_origins.contains(&"https://admin.com".to_string())); // Should be removed
    
    Ok(())
}

#[test]
fn test_nested_environment_variables() -> Result<(), Box<dyn std::error::Error>> {
    // Set up test environment variables
    env::set_var("TEST_HOST", "env.example.com");
    env::set_var("TEST_PORT", "3000");
    env::set_var("TEST_FEATURES", r#"["cache", "redis"]"#);
    env::set_var("TEST_DATABASE_URL", "redis://localhost");
    env::set_var("TEST_DATABASE_TIMEOUT", "120");
    
    let config = SuperFigment::new()
        .with_defaults(TestConfig::default())
        .with_env("TEST_");
        
    let result: TestConfig = config.extract()?;
    
    assert_eq!(result.host, "env.example.com");
    assert_eq!(result.port, 3000);
    assert_eq!(result.features, vec!["cache", "redis"]);
    assert_eq!(result.database.url, "redis://localhost");
    assert_eq!(result.database.timeout, 120);
    
    // Clean up
    env::remove_var("TEST_HOST");
    env::remove_var("TEST_PORT");
    env::remove_var("TEST_FEATURES");
    env::remove_var("TEST_DATABASE_URL");
    env::remove_var("TEST_DATABASE_TIMEOUT");
    
    Ok(())
}

#[test]
fn test_format_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    
    // Test JSON detection
    let json_file = temp_dir.path().join("config.json");
    fs::write(&json_file, r#"{"host": "json.example.com"}"#)?;
    
    // Test TOML detection  
    let toml_file = temp_dir.path().join("config.toml");
    fs::write(&toml_file, r#"host = "toml.example.com""#)?;
    
    // Test YAML detection
    let yaml_file = temp_dir.path().join("config.yaml");
    fs::write(&yaml_file, r#"host: yaml.example.com"#)?;
    
    // Test unknown extension (should try all formats)
    let unknown_file = temp_dir.path().join("config.cfg");
    fs::write(&unknown_file, r#"host = "cfg.example.com""#)?; // TOML content
    
    let json_config: TestConfig = SuperFigment::new()
        .with_file(&json_file)
        .extract()?;
    assert_eq!(json_config.host, "json.example.com");
    
    let toml_config: TestConfig = SuperFigment::new()
        .with_file(&toml_file)
        .extract()?;
    assert_eq!(toml_config.host, "toml.example.com");
    
    let yaml_config: TestConfig = SuperFigment::new()
        .with_file(&yaml_file)
        .extract()?;
    assert_eq!(yaml_config.host, "yaml.example.com");
    
    let unknown_config: TestConfig = SuperFigment::new()
        .with_file(&unknown_file)
        .extract()?;
    assert_eq!(unknown_config.host, "cfg.example.com");
    
    Ok(())
}

#[test] 
fn test_empty_value_filtering() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct FilterConfig {
        #[serde(default)]
        enabled: bool,
        #[serde(default)]
        name: String,
        #[serde(default)]
        count: u32,
        #[serde(default)]
        items: Vec<String>,
    }
    
    let temp_dir = TempDir::new()?;
    let base_config = temp_dir.path().join("base.json");
    fs::write(&base_config, r#"
{
    "enabled": true,
    "name": "production",
    "count": 42,
    "items": ["item1", "item2"]
}
"#)?;
    
    // Create CLI-like provider with empty values that should be filtered
    let cli_values = FilterConfig {
        enabled: false, // meaningful falsy value - should be preserved
        name: "".to_string(), // empty string - should be filtered
        count: 0, // meaningful zero - should be preserved  
        items: Vec::new(), // empty array - should be filtered
    };
    
    let config = SuperFigment::new()
        .with_file(&base_config)
        .with_cli_opt(Some(cli_values));
        
    let result: FilterConfig = config.extract()?;
    
    // enabled should be overridden to false (meaningful falsy value preserved)
    assert_eq!(result.enabled, false);
    
    // name should keep base config value (empty string filtered out)
    assert_eq!(result.name, "production");
    
    // count should be overridden to 0 (meaningful zero preserved)
    assert_eq!(result.count, 0);
    
    // items should keep base config value (empty array filtered out)
    assert_eq!(result.items, vec!["item1", "item2"]);
    
    Ok(())
}

#[test]
fn test_hierarchical_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    
    // Create system-level config
    let system_dir = temp_dir.path().join(".config").join("testapp");
    fs::create_dir_all(&system_dir)?;
    let system_config = system_dir.join("testapp.toml");
    fs::write(&system_config, r#"
host = "system.example.com"
port = 8080
[database]
timeout = 30
allowed_origins = ["https://system.com"]
"#)?;
    
    // Create user-level config  
    let user_dir = temp_dir.path().join(".testapp");
    fs::create_dir_all(&user_dir)?;
    let user_config = user_dir.join("testapp.toml");
    fs::write(&user_config, r#"
port = 9090
[database] 
allowed_origins_add = ["https://user.com"]
"#)?;
    
    // Create project-level config
    let project_config = temp_dir.path().join("testapp.toml");
    fs::write(&project_config, r#"
host = "project.example.com"
[database]
allowed_origins_remove = ["https://system.com"]
allowed_origins_add = ["https://project.com"]
"#)?;
    
    // Update HOME to point to our temp directory for testing
    let original_home = env::var("HOME").unwrap_or_default();
    env::set_var("HOME", temp_dir.path());
    
    // Debug: List what files we actually created
    println!("Files created:");
    println!("System: {:?} - exists: {}", system_config, system_config.exists());
    println!("User: {:?} - exists: {}", user_config, user_config.exists()); 
    println!("Project: {:?} - exists: {}", project_config, project_config.exists());
    
    // Change to temp directory to simulate project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;
    
    println!("Current dir: {:?}", env::current_dir()?);
    println!("HOME set to: {:?}", env::var("HOME"));
    
    let config = SuperFigment::new()
        .with_defaults(TestConfig::default()) // Add defaults first
        .with_hierarchical_config("testapp");
        
    // Note: Hierarchical configuration currently has profile handling issues
    // The provider correctly loads files but data structure needs architectural fixes
    // For now, test basic functionality without complex array merging
    
    let result: TestConfig = config.extract()?;
    
    // Test that hierarchical merging works correctly (system -> user -> project priority)
    assert_eq!(result.host, "project.example.com"); // Project config overrides system
    assert_eq!(result.port, 9090); // User config overrides system (no project override)
    assert_eq!(result.database.timeout, 30); // From system config (no overrides)
    
    // Test complex array merging across hierarchy (system -> user -> project)
    let origins = &result.database.allowed_origins;
    assert!(origins.contains(&"https://user.com".to_string()));   // Added by user config
    assert!(origins.contains(&"https://project.com".to_string())); // Added by project config  
    assert!(!origins.contains(&"https://system.com".to_string())); // Removed by project config
    
    // Restore environment
    if original_home.is_empty() {
        env::remove_var("HOME");
    } else {
        env::set_var("HOME", original_home);
    }
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_conversion_methods() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.json");
    
    fs::write(&config_path, r#"
{
    "host": "conversion.example.com",
    "port": 8080,
    "database": {
        "url": "postgres://localhost",
        "timeout": 60
    }
}
"#)?;
    
    let figment = SuperFigment::new()
        .with_file(&config_path);
        
    // Test format conversions
    let json_str = figment.as_json()?;
    assert!(json_str.contains("conversion.example.com"));
    assert!(json_str.contains("8080"));
    
    let yaml_str = figment.as_yaml()?;
    assert!(yaml_str.contains("conversion.example.com"));
    
    let toml_str = figment.as_toml()?;
    assert!(toml_str.contains("conversion.example.com"));
    
    // Test debug output
    let debug_str = figment.debug_config()?;
    assert!(debug_str.contains("Figment Configuration Debug"));
    assert!(debug_str.contains("conversion.example.com"));
    
    let sources = figment.debug_sources();
    assert!(!sources.is_empty());
    
    Ok(())
}