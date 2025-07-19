pub mod languages;

#[cfg(test)]
mod tests;

use anyhow::Result;
use figment::{Figment, providers::{Format, Toml, Env}};

// Embed the default config at compile time
const DEFAULT_CONFIG: &str = include_str!("../../default-config.toml");

pub struct GuardyConfig {
    figment: Figment,
}

impl GuardyConfig {
    pub fn load() -> Result<Self> {
        let figment = Figment::new()
            .merge(Toml::string(DEFAULT_CONFIG))  // Embedded defaults
            .merge(Toml::file(Self::user_config_path()))  // User config
            .merge(Toml::file("guardy.toml"))     // Repo config
            .merge(Env::prefixed("GUARDY_"));     // Environment variables
            
        Ok(GuardyConfig { figment })
    }
    
    pub fn get_bool(&self, path: &str) -> Result<bool> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    pub fn get_string(&self, path: &str) -> Result<String> {
        Ok(self.figment.extract_inner(path)?)
    }
    
    pub fn get_u16(&self, path: &str) -> Result<u16> {
        Ok(self.figment.extract_inner(path)?)
    }
    
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