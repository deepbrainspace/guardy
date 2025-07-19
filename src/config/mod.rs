pub mod languages;

#[cfg(test)]
mod tests;

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
    
    /// Export configuration in specified format
    pub fn export_config(&self, format: ConfigFormat) -> Result<String> {
        let config: serde_json::Value = self.get_full_config()?;
        
        let output = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(&config)?,
            ConfigFormat::Toml => toml::to_string_pretty(&config)?,
            ConfigFormat::Yaml => serde_yml::to_string(&config)?,
        };
        
        Ok(output)
    }
    
    /// Export configuration with syntax highlighting
    pub fn export_config_highlighted(&self, format: ConfigFormat) -> Result<String> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::Style;
        use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
        use two_face::{syntax, theme};
        
        let output = self.export_config(format.clone())?;
        
        // Check if we're in a terminal that supports colors
        if !atty::is(atty::Stream::Stdout) {
            return Ok(output);
        }
        
        let ps = syntax::extra_newlines();
        let ts = theme::extra();
        
        let syntax = match format {
            ConfigFormat::Json => ps.find_syntax_by_extension("json").unwrap_or_else(|| ps.find_syntax_plain_text()),
            ConfigFormat::Toml => ps.find_syntax_by_extension("toml").unwrap_or_else(|| ps.find_syntax_plain_text()),
            ConfigFormat::Yaml => ps.find_syntax_by_extension("yaml").unwrap_or_else(|| ps.find_syntax_plain_text()),
        };
        
        use two_face::theme::EmbeddedThemeName;
        let theme = ts.get(EmbeddedThemeName::Base16OceanDark);
        let mut h = HighlightLines::new(syntax, theme);
        let mut highlighted = String::new();
        
        for line in LinesWithEndings::from(&output) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps)?;
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            highlighted.push_str(&escaped);
        }
        
        Ok(highlighted)
    }
    
    fn user_config_path() -> String {
        match std::env::var("HOME") {
            Ok(home) => format!("{}/.config/guardy/config.toml", home),
            Err(_) => "~/.config/guardy/config.toml".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}