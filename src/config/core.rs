use anyhow::Result;
use figment::{Figment, providers::{Format, Toml, Json, Yaml, Env}};

// Embed the default config at compile time
const DEFAULT_CONFIG: &str = include_str!("../../default-config.toml");

pub struct GuardyConfig {
    figment: Figment,
}

impl GuardyConfig {
    pub fn load() -> Result<Self> {
        Self::load_with_custom_config(None)
    }
    
    pub fn load_with_custom_config(custom_config: Option<&str>) -> Result<Self> {
        let mut figment = Figment::new()
            .merge(Toml::string(DEFAULT_CONFIG));  // Embedded defaults
            
        // If custom config is specified, use only that + defaults + env vars
        if let Some(custom_path) = custom_config {
            figment = figment
                .merge(Toml::file(custom_path))
                .merge(Json::file(custom_path))
                .merge(Yaml::file(custom_path));
        } else {
            // Standard priority: user config -> repo config
            figment = figment
                // User config - support multiple formats
                .merge(Toml::file(Self::user_config_path()))
                .merge(Json::file(Self::user_config_path().replace(".toml", ".json")))
                .merge(Yaml::file(Self::user_config_path().replace(".toml", ".yaml")))
                .merge(Yaml::file(Self::user_config_path().replace(".toml", ".yml")))
                // Repository config - support multiple formats
                .merge(Toml::file("guardy.toml"))
                .merge(Json::file("guardy.json"))
                .merge(Yaml::file("guardy.yaml"))
                .merge(Yaml::file("guardy.yml"));
        }
        
        // Environment variables always have highest priority
        figment = figment.merge(Env::prefixed("GUARDY_"));
            
        Ok(GuardyConfig { figment })
    }
    
    /// Get a nested object/section as JSON
    pub fn get_section(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    /// Get the full merged configuration as a structured value
    pub fn get_full_config(&self) -> Result<serde_json::Value> {
        Ok(self.figment.extract()?)
    }
    
    /// Get a boolean value from config
    pub fn get_bool(&self, path: &str) -> Result<bool> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    /// Get a string value from config
    pub fn get_string(&self, path: &str) -> Result<String> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    /// Get a u16 value from config
    pub fn get_u16(&self, path: &str) -> Result<u16> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    /// Get a vector of strings from config
    pub fn get_vec(&self, path: &str) -> Result<Vec<String>> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    fn user_config_path() -> String {
        match std::env::var("HOME") {
            Ok(home) => format!("{}/.config/guardy/config.toml", home),
            Err(_) => "~/.config/guardy/config.toml".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = GuardyConfig::load();
        assert!(config.is_ok(), "Should load default config successfully");
    }

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

    #[test]
    fn test_config_methods() {
        let config = GuardyConfig::load().unwrap();
        
        // Test getting full config
        assert!(config.get_full_config().is_ok());
        
        // Test environment variable support
        unsafe { std::env::set_var("GUARDY_TEST_VALUE", "true"); }
        let test_config = GuardyConfig::load().unwrap();
        // Config should be loadable with environment variables
        assert!(test_config.get_full_config().is_ok());
    }

    #[test]
    fn test_custom_config_loading() {
        // Test with non-existent custom config (should fallback to defaults)
        let config = GuardyConfig::load_with_custom_config(Some("non_existent.toml"));
        assert!(config.is_ok(), "Should handle missing custom config gracefully");
    }
}