use anyhow::Result;
use super::core::GuardyConfig;

#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}

impl GuardyConfig {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GuardyConfig;

    #[test]
    fn test_config_export_formats() {
        let config = GuardyConfig::load(None, None::<&()>, 0).unwrap();
        
        // Test JSON export
        let json_output = config.export_config(ConfigFormat::Json);
        assert!(json_output.is_ok());
        assert!(json_output.unwrap().contains("{"));
        
        // Test TOML export
        let toml_output = config.export_config(ConfigFormat::Toml);
        assert!(toml_output.is_ok());
        
        // Test YAML export
        let yaml_output = config.export_config(ConfigFormat::Yaml);
        assert!(yaml_output.is_ok());
    }

    #[test]
    fn test_syntax_highlighting() {
        let config = GuardyConfig::load(None, None::<&()>, 0).unwrap();
        
        // Test highlighted export (should fallback to plain text in non-TTY)
        let highlighted = config.export_config_highlighted(ConfigFormat::Json);
        assert!(highlighted.is_ok());
    }
}