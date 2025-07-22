use anyhow::Result;
use figment::Figment;
use guardy_figment_providers::providers::{Universal, SkipEmpty, NestedEnv};
use guardy_figment_providers::ext::FigmentExt;
use serde::Serialize;

// Embed the default config at compile time
const DEFAULT_CONFIG: &str = include_str!("../../default-config.toml");


pub struct GuardyConfig {
    figment: Figment,
}

impl GuardyConfig {
    pub fn load<T: Serialize>(
        custom_config: Option<&str>, 
        cli_overrides: Option<T>
    ) -> Result<Self> {
        // Debug: Show starting config load (only at trace level -vvv)
        tracing::trace!("CONFIG LOAD: Starting");

        // Complete configuration chain with array merging support
        let figment = Figment::new()
            .merge(Universal::string(DEFAULT_CONFIG))                              // 1. Defaults (lowest)
            .merge_extend(Universal::file(Self::user_config_base_path()))          // 2. User config (any format)
            .merge_extend(Universal::file("guardy"))                               // 3. Repo config (any format)
            .merge_extend_opt(custom_config.map(|path| Universal::file(path)))     // 4. Custom config (if provided)
            .merge_extend(NestedEnv::prefixed("GUARDY_"))                           // 5. Environment variables
            .merge_extend_opt(cli_overrides.map(|cli| {                            // 6. CLI (highest priority)
                tracing::trace!("CONFIG LOAD: Applying CLI overrides");
                SkipEmpty::new(cli)
            }));
        
        // Debug: Show final config (only at trace level -vvv)
        if let Ok(final_config) = figment.extract::<serde_json::Value>() {
            tracing::trace!("CONFIG LOAD: Final scanner.mode = {:?}", 
                         final_config.get("scanner").and_then(|s| s.get("mode")));
        }
            
        Ok(GuardyConfig { figment })
    }
    
    /// Get a nested object/section as JSON
    pub fn get_section(&self, path: &str) -> Result<serde_json::Value> {
        let value = self.figment.extract_inner(path)?;
        Ok(value)
    }
    
    /// Get the full merged configuration as a structured value
    pub fn get_full_config(&self) -> Result<serde_json::Value> {
        let value = self.figment.extract()?;
        Ok(value)
    }
    
    
    /// Get a vector of strings from config
    pub fn get_vec(&self, path: &str) -> Result<Vec<String>> {
        let vec: Vec<String> = self.figment.extract_inner(path)?;
        Ok(vec)
    }
    
    
    fn user_config_base_path() -> String {
        match std::env::var("HOME") {
            Ok(home) => format!("{}/.config/guardy/config", home),
            Err(_) => "~/.config/guardy/config".to_string(),
        }
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

    #[test]
    fn test_custom_config_loading() {
        // Test with non-existent custom config (should fallback to defaults)
        let config = GuardyConfig::load(Some("non_existent.toml"), None::<&()>);
        assert!(config.is_ok(), "Should handle missing custom config gracefully");
    }
}