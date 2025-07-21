use std::path::Path;
use figment::providers::{Format, Toml, Json, Yaml};

/// Smart configuration file loader that chooses the right format based on file extension
/// Returns a provider that can be directly used with figment.merge()
pub fn auto<P: AsRef<Path>>(path: P) -> impl figment::Provider {
    let path = path.as_ref();
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "toml" => SmartProvider::Toml(Toml::file(path)),
        "json" => SmartProvider::Json(Json::file(path)),
        "yaml" | "yml" => SmartProvider::Yaml(Yaml::file(path)),
        _ => {
            // For unknown extensions, try to detect based on content or default to TOML
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Some(format) = detect_format_from_content(&content) {
                    println!("DEBUG: Smart loader detected format '{}' for unknown extension", format);
                    match format.as_str() {
                        "json" => SmartProvider::Json(Json::file(path)),
                        "yaml" => SmartProvider::Yaml(Yaml::file(path)),
                        "toml" => SmartProvider::Toml(Toml::file(path)),
                        _ => SmartProvider::Toml(Toml::file(path)), // Default to TOML
                    }
                } else {
                    println!("DEBUG: Smart loader could not detect format, defaulting to TOML");
                    SmartProvider::Toml(Toml::file(path)) // Default fallback
                }
            } else {
                println!("DEBUG: Smart loader could not read file, defaulting to TOML");
                SmartProvider::Toml(Toml::file(path)) // Default fallback
            }
        }
    }
}

/// Wrapper enum to handle different provider types
enum SmartProvider {
    Toml(figment::providers::Data<figment::providers::Toml>),
    Json(figment::providers::Data<figment::providers::Json>),
    Yaml(figment::providers::Data<figment::providers::Yaml>),
}

impl figment::Provider for SmartProvider {
    fn metadata(&self) -> figment::Metadata {
        match self {
            SmartProvider::Toml(p) => p.metadata(),
            SmartProvider::Json(p) => p.metadata(),
            SmartProvider::Yaml(p) => p.metadata(),
        }
    }

    fn data(&self) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, figment::Error> {
        match self {
            SmartProvider::Toml(p) => p.data(),
            SmartProvider::Json(p) => p.data(),
            SmartProvider::Yaml(p) => p.data(),
        }
    }
}

/// Attempt to detect configuration format from file content
fn detect_format_from_content(content: &str) -> Option<String> {
    let trimmed = content.trim();
    
    // JSON detection - starts with { or [
    if (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
       (trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return Some("json".to_string());
    }
    
    // YAML detection - contains common YAML patterns
    if trimmed.contains("---") || // YAML document separator
       trimmed.lines().any(|line| {
           let line = line.trim();
           // YAML key-value with colon (but not TOML table header)
           line.contains(':') && !line.starts_with('[') && !line.ends_with(']')
       }) {
        return Some("yaml".to_string());
    }
    
    // TOML detection - contains [section] headers or key = value
    if trimmed.lines().any(|line| {
        let line = line.trim();
        (line.starts_with('[') && line.ends_with(']')) || // TOML section
        (line.contains('=') && !line.contains(':')) // TOML key-value
    }) {
        return Some("toml".to_string());
    }
    
    None // Unable to detect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format_from_content(r#"{"key": "value"}"#), Some("json".to_string()));
        assert_eq!(detect_format_from_content("key: value"), Some("yaml".to_string()));
        assert_eq!(detect_format_from_content("[section]\nkey = value"), Some("toml".to_string()));
        assert_eq!(detect_format_from_content("key = value"), Some("toml".to_string()));
    }
}