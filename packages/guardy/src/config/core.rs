use anyhow::Result;
use superfigment::SuperFigment;
use serde::Serialize;

// Embed the default config at compile time
const DEFAULT_CONFIG: &str = include_str!("../../default-config.toml");


pub struct GuardyConfig {
    config: SuperFigment,
}

impl GuardyConfig {
    pub fn load<T: Serialize>(
        custom_config: Option<&str>,
        cli_overrides: Option<T>
    ) -> Result<Self> {
        // Debug: Show starting config load (only at trace level -vvv)
        tracing::trace!("CONFIG LOAD: Starting");

        // Clean 4-stage configuration hierarchy using SuperFigment's explicit API
        let config = SuperFigment::new()
            .with_provider(superfigment::Universal::string(DEFAULT_CONFIG))       // 1. Defaults (lowest)
            .with_hierarchical_config("guardy")                                   // 2. Hierarchical: system→user→project
            .with_file_opt(custom_config)                                         // 3. Custom config file (if provided)
            .with_env_ignore_empty("GUARDY_")                                     // 4. Environment variables (with empty filtering)
            .with_cli_opt(cli_overrides);                                         // 5. CLI (highest priority)
        
        // Debug: Show final config (only at trace level -vvv)
        if let Ok(final_config) = config.extract::<serde_json::Value>() {
            tracing::trace!("CONFIG LOAD: Final scanner.mode = {:?}", 
                         final_config.get("scanner").and_then(|s| s.get("mode")));
        }
            
        Ok(GuardyConfig { config })
    }
    
    /// Get a nested object/section as JSON
    pub fn get_section(&self, path: &str) -> Result<serde_json::Value> {
        let value = self.config.extract_inner(path)?;
        Ok(value)
    }
    
    /// Get the full merged configuration as a structured value
    pub fn get_full_config(&self) -> Result<serde_json::Value> {
        let value = self.config.extract()?;
        Ok(value)
    }
    
    
    /// Get a vector of strings from config
    pub fn get_vec(&self, path: &str) -> Result<Vec<String>> {
        let vec: Vec<String> = self.config.extract_inner(path)?;
        Ok(vec)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = GuardyConfig::load(None, None::<&()>);
        assert!(config.is_ok(), "Should load default config successfully");
    }

    #[test]
    fn test_config_loads_defaults() {
        let config = GuardyConfig::load(None, None::<&()>).expect("Should load default config");
        
        // Test that we can load the full config
        let full_config = config.get_full_config().unwrap();
        assert!(full_config.get("general").is_some());
        assert!(full_config.get("scanner").is_some());
        
        // Test that we can get specific sections
        let scanner_section = config.get_section("scanner").unwrap();
        assert!(scanner_section.get("mode").is_some());
    }

    #[test]
    fn test_config_methods() {
        let config = GuardyConfig::load(None, None::<&()>).unwrap();
        
        // Test getting full config
        assert!(config.get_full_config().is_ok());
        
        // Test environment variable support
        unsafe { std::env::set_var("GUARDY_TEST_VALUE", "true"); }
        let test_config = GuardyConfig::load(None, None::<&()>).unwrap();
        // Config should be loadable with environment variables
        assert!(test_config.get_full_config().is_ok());
    }
}