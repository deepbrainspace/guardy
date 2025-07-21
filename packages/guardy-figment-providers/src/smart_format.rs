//! Smart format detection provider for Figment.
//!
//! Automatically detects configuration file formats based on content analysis,
//! returning the appropriate Figment provider (Json, Toml, or Yaml).

use std::path::Path;
use figment::providers::{Format, Json, Toml, Yaml};
use figment::{Provider, Metadata, Profile, Error, value::{Map, Dict}};

/// A smart format provider that automatically detects and loads configuration files.
/// 
/// This provider examines file content to detect the format (JSON, TOML, YAML)
/// and wraps the appropriate Figment provider internally. It works regardless
/// of file extension, making it perfect for config files with unknown or missing
/// extensions.
///
/// # Examples
///
/// ```rust,no_run
/// use figment::Figment;
/// use guardy_figment_providers::SmartFormat;
///
/// let figment = Figment::new()
///     .merge(SmartFormat::file("config.toml"))      // Auto-detects TOML
///     .merge(SmartFormat::file("settings.json"))    // Auto-detects JSON  
///     .merge(SmartFormat::file("app.yaml"))         // Auto-detects YAML
///     .merge(SmartFormat::file("mystery-config"));   // Works without extension!
/// ```
pub struct SmartFormat {
    inner: Box<dyn Provider>,
}

impl SmartFormat {
    /// Creates a new SmartFormat provider for the given file path.
    /// 
    /// The format is automatically detected by analyzing the file content.
    /// If detection fails, defaults to TOML format.
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        
        // Try extension first for performance
        if let Some(provider) = Self::try_extension_detection(path) {
            return provider;
        }
        
        // Fall back to content analysis
        match std::fs::read_to_string(path) {
            Ok(content) => Self::from_content(&content, Some(path)),
            Err(_) => {
                // File doesn't exist or can't be read - default to TOML
                // This allows the underlying provider to handle the error appropriately
                SmartFormat {
                    inner: Box::new(Toml::file(path)),
                }
            }
        }
    }

    /// Creates a new SmartFormat provider from a string with automatic format detection.
    /// 
    /// The format is detected by analyzing the content structure.
    /// If detection fails, defaults to TOML format.
    pub fn string(content: &str) -> Self {
        Self::from_content(content, None)
    }
    
    /// Try to detect format from file extension for performance
    fn try_extension_detection(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        let inner: Box<dyn Provider> = match extension.as_str() {
            "json" => Box::new(Json::file(path)),
            "toml" => Box::new(Toml::file(path)),
            "yaml" | "yml" => Box::new(Yaml::file(path)),
            _ => return None, // Unknown extension, try content detection
        };
        
        Some(SmartFormat { inner })
    }
    
    /// Create SmartFormat from content analysis
    fn from_content(content: &str, path: Option<&Path>) -> Self {
        let format = detect_format_from_content(content);
        
        let inner: Box<dyn Provider> = match format {
            DetectedFormat::Json => {
                if let Some(p) = path {
                    Box::new(Json::file(p))
                } else {
                    Box::new(Json::string(content))
                }
            }
            DetectedFormat::Yaml => {
                if let Some(p) = path {
                    Box::new(Yaml::file(p))
                } else {
                    Box::new(Yaml::string(content))
                }
            }
            DetectedFormat::Toml => {
                if let Some(p) = path {
                    Box::new(Toml::file(p))
                } else {
                    Box::new(Toml::string(content))
                }
            }
        };
        
        SmartFormat { inner }
    }
}

impl Provider for SmartFormat {
    fn metadata(&self) -> Metadata {
        self.inner.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        self.inner.data()
    }

    fn profile(&self) -> Option<Profile> {
        self.inner.profile()
    }
}

/// Detected configuration format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectedFormat {
    Json,
    Yaml,
    Toml,
}

/// Detect configuration format from file content.
/// 
/// This function analyzes content patterns to determine the most likely format.
/// The detection is designed to be robust and handle edge cases gracefully.
///
/// # Examples
///
/// ```
/// use guardy_figment_providers::smart_format::detect_format_from_content;
/// use guardy_figment_providers::smart_format::DetectedFormat;
/// 
/// // JSON detection
/// let json = r#"{"key": "value", "number": 42}"#;
/// assert_eq!(detect_format_from_content(json), DetectedFormat::Json);
/// 
/// // TOML detection  
/// let toml = r#"name = "guardy"
/// [section]
/// key = "value""#;
/// assert_eq!(detect_format_from_content(toml), DetectedFormat::Toml);
/// 
/// // YAML detection
/// let yaml = r#"name: guardy
/// section:
///   key: value"#;
/// assert_eq!(detect_format_from_content(yaml), DetectedFormat::Yaml);
/// 
/// // Empty content defaults to TOML
/// assert_eq!(detect_format_from_content(""), DetectedFormat::Toml);
/// ```
pub fn detect_format_from_content(content: &str) -> DetectedFormat {
    let trimmed = content.trim();
    
    if trimmed.is_empty() {
        return DetectedFormat::Toml; // Default for empty content
    }
    
    // JSON detection - most distinctive patterns first
    if is_json_format(trimmed) {
        return DetectedFormat::Json;
    }
    
    // TOML detection - check for TOML-specific patterns
    if is_toml_format(trimmed) {
        return DetectedFormat::Toml;
    }
    
    // YAML detection - check for YAML-specific patterns  
    if is_yaml_format(trimmed) {
        return DetectedFormat::Yaml;
    }
    
    // Default fallback
    DetectedFormat::Toml
}

/// Detect JSON format from content
fn is_json_format(content: &str) -> bool {
    // JSON must start and end with braces or brackets
    (content.starts_with('{') && content.ends_with('}')) ||
    (content.starts_with('[') && content.ends_with(']'))
}

/// Detect TOML format from content  
fn is_toml_format(content: &str) -> bool {
    let lines: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
        
    if lines.is_empty() {
        return false;
    }
    
    // Look for TOML-specific patterns
    for line in &lines {
        // TOML section headers: [section] or [[array.section]]
        if (line.starts_with('[') && line.ends_with(']')) && !line.contains(':') {
            return true;
        }
        
        // TOML key = value assignments (but not YAML key: value)
        if line.contains('=') && !line.contains(':') && !line.starts_with('[') {
            return true;
        }
    }
    
    false
}

/// Detect YAML format from content
fn is_yaml_format(content: &str) -> bool {
    // YAML document separator
    if content.contains("---") {
        return true;
    }
    
    let lines: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
        
    if lines.is_empty() {
        return false;
    }
    
    // Look for YAML key: value patterns
    let yaml_pattern_count = lines.iter()
        .filter(|line| {
            // YAML key: value pattern
            if let Some(colon_pos) = line.find(':') {
                // Make sure it's not inside quotes and has proper YAML spacing
                let after_colon = &line[colon_pos + 1..];
                return after_colon.starts_with(' ') || after_colon.is_empty();
            }
            false
        })
        .count();
    
    // If most non-comment lines look like YAML, it's probably YAML
    yaml_pattern_count > 0 && yaml_pattern_count >= lines.len() / 2
}

