use anyhow::Result;
use figment::{Figment, providers::{Format, Toml, Json, Yaml, Env, Serialized}};
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
        let mut figment = Figment::new()
            .merge(Toml::string(DEFAULT_CONFIG));  // 1. Defaults (lowest)
        
        // Debug: Show starting config load (only at trace level -vvv)
        tracing::trace!("CONFIG LOAD: Starting");

        // Standard configs first: json → yaml → yml → toml for each location
        figment = figment
            // User configs
            .merge(Json::file(Self::user_config_path().replace(".toml", ".json")))  // 2. User JSON
            .merge(Yaml::file(Self::user_config_path().replace(".toml", ".yaml")))  // 3. User YAML
            .merge(Yaml::file(Self::user_config_path().replace(".toml", ".yml")))   // 4. User YML
            .merge(Toml::file(Self::user_config_path()))                            // 5. User TOML
            // Repo configs  
            .merge(Json::file("guardy.json"))                                       // 6. Repo JSON
            .merge(Yaml::file("guardy.yaml"))                                       // 7. Repo YAML
            .merge(Yaml::file("guardy.yml"))                                        // 8. Repo YML
            .merge(Toml::file("guardy.toml"));                                      // 9. Repo TOML
        
        // Custom config overrides user/repo configs
        if let Some(custom_path) = custom_config {
            println!("DEBUG: Loading custom config from: {}", custom_path);
            figment = figment.merge(super::smart_load::auto(custom_path));          // 14. Custom (auto format)
            
            // Debug: Try to read the file directly
            if let Ok(content) = std::fs::read_to_string(custom_path) {
                println!("DEBUG: Custom config file content: {}", content);
            }
            
            // Debug: Check what figment extracted after loading custom config
            match figment.extract::<serde_json::Value>() {
                Ok(extracted) => {
                    println!("DEBUG: Figment extracted after custom config: {}", serde_json::to_string_pretty(&extracted).unwrap_or_else(|_| "Failed to serialize".to_string()));
                },
                Err(e) => {
                    println!("DEBUG: Figment extract failed: {:?}", e);
                    println!("DEBUG: Figment error kind: {:?}", e.kind);
                    println!("DEBUG: Figment error path: {:?}", e.path);
                    println!("DEBUG: Figment profile: {:?}", e.profile);
                }
            }
        }
        
        // Environment variables with custom mapping for nested keys
        let env_provider = Env::prefixed("GUARDY_")
            .map(|key| key.as_str().replace("_", ".").into());  // Map GUARDY_SCANNER_MODE -> scanner.mode
        figment = figment.merge(env_provider);                                      // 15. Environment
        
        // CLI overrides (highest priority) - filter out empty arrays first
        if let Some(cli_overrides) = cli_overrides {
            tracing::trace!("CONFIG LOAD: Applying CLI overrides");
            let filtered_overrides = super::overrides::filter_empty_arrays(cli_overrides);
            figment = figment.merge(Serialized::defaults(filtered_overrides));               // 16. CLI (highest)
        }
        
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