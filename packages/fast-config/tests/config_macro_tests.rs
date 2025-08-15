//! Integration tests for the config! procedural macro
//! 
//! These tests verify that the macro correctly:
//! - Discovers config files in various locations
//! - Generates appropriate Rust structs from config content
//! - Creates LazyLock static instances for zero-copy access
//! - Handles various config formats (JSON, YAML)

use fast_config::config;

fn setup_config_environment() {
    // Change to tests/configs directory if we're not already there
    let configs_dir = std::path::Path::new("tests/configs");
    if configs_dir.exists() {
        std::env::set_current_dir(configs_dir).unwrap();
    }
}

#[test]
fn test_testapp_config_generation() {
    setup_config_environment();
    
    // This should generate a struct and static instance from test_testapp_config.json
    config!("test_testapp_config" => TestAppConfig);
    
    // Test that we can access the generated config
    let config = TestAppConfig::global();
    assert_eq!(config.debug, true);
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.host, "localhost");
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.timeout, 30);
}

#[test]
fn test_yamlapp_config_generation() {
    setup_config_environment();
    
    // This should generate a struct from test_yamlapp_config.yaml
    config!("test_yamlapp_config" => YamlAppConfig);
    
    let config = YamlAppConfig::global();
    assert_eq!(config.app.name, "YamlApp");
    assert_eq!(config.app.version, "1.0.0");
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.database.driver, "mysql");
    assert_eq!(config.database.port, 3306);
    assert_eq!(config.logging.level, "info");
}

#[test]
fn test_nested_config_generation() {
    setup_config_environment();
    
    // Test deeply nested configuration structures
    config!("test_nested_config" => NestedConfig);
    
    let config = NestedConfig::global();
    assert_eq!(config.api.endpoints.users, "/api/v1/users");
    assert_eq!(config.api.endpoints.auth, "/api/v1/auth");
    assert_eq!(config.api.rate_limits.per_minute, 100);
    assert_eq!(config.cache.redis.host, "localhost");
    assert_eq!(config.cache.redis.port, 6379);
    assert_eq!(config.cache.ttl, 3600);
    assert!(config.monitoring.enabled);
    assert!(config.monitoring.metrics.prometheus.enabled);
    assert_eq!(config.monitoring.metrics.prometheus.port, 9090);
}

#[test]
fn test_arrays_config_generation() {
    setup_config_environment();
    
    // Test configuration with arrays and complex structures
    config!("test_arrays_config" => ArrayConfig);
    
    let config = ArrayConfig::global();
    assert_eq!(config.services.len(), 4);
    assert!(config.services.contains(&"web".to_string()));
    assert!(config.services.contains(&"api".to_string()));
    
    assert_eq!(config.ports.len(), 3);
    assert!(config.ports.contains(&8080));
    assert!(config.ports.contains(&8081));
    
    assert_eq!(config.environments.len(), 2);
    assert_eq!(config.environments[0].name, "development");
    assert_eq!(config.environments[0].config.debug, true);
    assert_eq!(config.environments[1].name, "production");
    assert_eq!(config.environments[1].config.debug, false);
    
    assert!(config.feature_flags.new_ui);
    assert!(!config.feature_flags.beta_features);
    assert_eq!(config.feature_flags.experimental.len(), 2);
}

#[test]
fn test_performance_config_generation() {
    setup_config_environment();
    
    // Test performance with small config
    config!("test_performance_config" => PerfTestConfig);
    
    let _config1 = PerfTestConfig::global();
    let _config2 = PerfTestConfig::global(); // Should be instant (LazyLock cached)
    
    // Verify values
    let config = PerfTestConfig::global();
    assert_eq!(config.name, "PerfTest");
    assert_eq!(config.threads, 8);
    assert_eq!(config.batch_size, 1000);
    assert!(config.enabled);
    assert_eq!(config.timeout_ms, 5000);
}

#[test] 
fn test_nonexistent_config_handling() {
    // This should compile but use default values since file doesn't exist
    config!("nonexistent" => NonExistentConfig);
    
    // Should not panic, should use defaults
    let config = NonExistentConfig::global();
    // Since file doesn't exist, this will use Default::default()
    // We can't assert specific values since they're defaults
    let _ = config; // Just ensure it doesn't panic
}