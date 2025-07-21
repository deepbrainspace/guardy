//! Universal format provider for Figment.
//!
//! Automatically detects configuration file formats based on content analysis,
//! supporting JSON, TOML, and YAML with any file extension.

use std::path::Path;
use figment::providers::{Format, Json, Toml, Yaml};
use figment::{Provider, Metadata, Profile, Error, value::{Map, Dict}};

/// Universal format provider that automatically detects and loads configuration files.
/// 
/// This provider examines file content to detect the format (JSON, TOML, YAML)
/// and works with any file extension, making it perfect for config files with
/// unknown, missing, or unconventional extensions.
///
/// # Examples
///
/// ```rust,no_run
/// use figment::Figment;
/// use guardy_figment_providers::providers::Universal;
///
/// let figment = Figment::new()
///     .merge(Universal::file("config.toml"))      // Auto-detects TOML
///     .merge(Universal::file("settings.json"))    // Auto-detects JSON  
///     .merge(Universal::file("app.yaml"))         // Auto-detects YAML
///     .merge(Universal::file("mystery-config"));  // Works without extension!
/// ```
pub struct Universal {
    inner: Box<dyn Provider>,
}

impl Universal {
    /// Creates a new Universal provider for the given file path.
    /// 
    /// If the exact path doesn't exist, automatically tries common config extensions:
    /// .toml, .yaml, .yml, .json (in that order)
    /// 
    /// The format is automatically detected by analyzing the file content.
    /// If detection fails, defaults to TOML format.
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        
        // If exact file exists, use it
        if path.exists() {
            return Self::load_existing_file(path);
        }
        
        // Try common extensions if exact path doesn't exist
        let extensions = ["toml", "yaml", "yml", "json"];
        for ext in &extensions {
            let path_with_ext = path.with_extension(ext);
            if path_with_ext.exists() {
                return Self::load_existing_file(&path_with_ext);
            }
        }
        
        // Nothing found - default to original path and let underlying provider handle the error
        Universal {
            inner: Box::new(Toml::file(path)),
        }
    }
    
    /// Helper method to load an existing file with format detection
    fn load_existing_file(path: &Path) -> Self {
        // Try extension first for performance
        if let Some(provider) = Self::try_extension_detection(path) {
            return provider;
        }
        
        // Fall back to content analysis
        match std::fs::read_to_string(path) {
            Ok(content) => Self::from_content(&content, Some(path)),
            Err(_) => {
                // File exists but can't be read - default to TOML
                Universal {
                    inner: Box::new(Toml::file(path)),
                }
            }
        }
    }

    /// Creates a new Universal provider from a string with automatic format detection.
    /// 
    /// The format is detected by analyzing the content structure.
    /// If detection fails, defaults to TOML format.
    pub fn string(content: &str) -> Self {
        Self::from_content(content, None)
    }
    
    /// Creates a new Universal provider that tries common config file extensions.
    /// 
    /// Given a base filename like "guardy", this will try in order:
    /// - guardy.toml
    /// - guardy.yaml  
    /// - guardy.yml
    /// - guardy.json
    /// - guardy (no extension)
    /// 
    /// Returns the first file that exists, with automatic format detection.
    pub fn file_with_extensions<P: AsRef<Path>>(base_path: P) -> Self {
        let base = base_path.as_ref();
        let extensions = ["toml", "yaml", "yml", "json"];
        
        // Try each extension
        for ext in &extensions {
            let path_with_ext = base.with_extension(ext);
            if path_with_ext.exists() {
                return Self::file(&path_with_ext);
            }
        }
        
        // Try without extension as fallback
        if base.exists() {
            return Self::file(base);
        }
        
        // Nothing found - default to .toml and let the underlying provider handle the error
        Self::file(base.with_extension("toml"))
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
        
        Some(Universal { inner })
    }
    
    /// Create Universal from content analysis
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
        
        Universal { inner }
    }
}

impl Provider for Universal {
    fn metadata(&self) -> Metadata {
        self.inner.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        // Try the detected format first
        let data = match self.inner.data() {
            Ok(data) => data,
            Err(_) => {
                // If the detected format fails, try other formats as fallback
                match self.try_fallback_formats() {
                    Ok(data) => data,
                    Err(_) => {
                        // All formats failed - this is likely not a config file at all
                        // Return empty configuration data gracefully instead of failing
                        
                        // Only print warning once by checking if this is a file we recognize as non-config
                        if let Some(source) = self.inner.metadata().source {
                            if let figment::Source::File(path) = source {
                                let path_str = path.display().to_string();
                                if path_str.ends_with(".md") || path_str.ends_with(".txt") || path_str.ends_with(".rs") {
                                    eprintln!("WARNING: File '{}' does not appear to be a configuration file - ignoring", path_str);
                                } else {
                                    eprintln!("WARNING: Universal could not parse '{}' as any configuration format (JSON/YAML/TOML)", path_str);
                                    eprintln!("         This file will be ignored in the configuration chain.");
                                }
                            }
                        }
                        
                        // Return empty configuration data - this allows Figment chain to continue
                        return Ok(Map::new());
                    }
                }
            }
        };
        
        // Note: Array merging is now handled by ArrayMerge provider at chain level
        
        Ok(data)
    }

    fn profile(&self) -> Option<Profile> {
        self.inner.profile()
    }
}

impl Universal {
    /// Try alternative formats when the primary detection fails
    fn try_fallback_formats(&self) -> Result<Map<Profile, Dict>, Error> {
        // Extract the path from metadata if available
        let metadata = self.inner.metadata();
        
        if let Some(source) = metadata.source {
            if let figment::Source::File(path) = source {
                // Try TOML, YAML, then JSON as fallbacks
                let fallback_providers: Vec<Box<dyn Provider>> = vec![
                    Box::new(Toml::file(&path)),
                    Box::new(Yaml::file(&path)),
                    Box::new(Json::file(&path)),
                ];
                
                let mut last_error = None;
                for provider in fallback_providers {
                    match provider.data() {
                        Ok(data) => return Ok(data),
                        Err(e) => last_error = Some(e),
                    }
                }
                
                // All formats failed, return the last error
                Err(last_error.unwrap_or_else(|| Error::from("No formats could parse the file")))
            } else {
                // For non-file sources, return the original error
                self.inner.data()
            }
        } else {
            // No source metadata available, return original error
            self.inner.data()
        }
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
/// use guardy_figment_providers::providers::universal::detect_format_from_content;
/// use guardy_figment_providers::providers::universal::DetectedFormat;
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

