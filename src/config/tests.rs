use super::GuardyConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads_defaults() {
        let config = GuardyConfig::load().expect("Should load default config");
        
        // Test some default values from our default-config.toml
        assert_eq!(config.get_bool("general.color").unwrap(), true);
        assert_eq!(config.get_string("package_manager.preferred").unwrap(), "pnpm");
        assert_eq!(config.get_u16("mcp.port").unwrap(), 8080);
        
        let patterns = config.get_vec("security.patterns").unwrap();
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("sk-")));
    }
}