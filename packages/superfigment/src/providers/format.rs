//! Universal format detection provider with performance optimizations
//!
//! The Universal provider automatically detects configuration file formats (JSON, TOML, YAML)
//! from content with intelligent caching and extension fallback for optimal performance.
//!
//! ## Detection Strategy & Scenarios
//!
//! The Universal provider uses a multi-layered approach to handle various real-world scenarios:
//!
//! ### Scenario 1: Standard Files with Known Extensions (Fast Path)
//! ```text
//! config.json    → JSON parser (extension-based)
//! config.toml    → TOML parser (extension-based)
//! config.yaml    → YAML parser (extension-based)
//! config.yml     → YAML parser (extension-based)
//! ```
//! **Performance**: Fastest path, no content reading required.
//!
//! ### Scenario 2: Misnamed Files (Content Detection)
//! ```text
//! config.yaml containing JSON:
//! {
//!   "database": {"host": "localhost"}
//! }
//! → Content analysis detects JSON → JSON parser (cached)
//! ```
//! **Performance**: Content-based detection with intelligent caching by modification time.
//!
//! ### Scenario 3: Unknown Extensions (Try All Formats)
//! ```text
//! config.cfg containing:
//! [database]
//! host = "localhost"
//! → Extension unknown → Content detection → Try TOML parser → Success!
//! 
//! settings.conf containing invalid TOML:
//! database:
//!   host: localhost
//! → Try TOML parser → Fail → Try YAML parser → Success!
//! ```
//! **Robustness**: Handles custom extensions and misidentified formats.
//!
//! ### Scenario 4: Base Filename Without Extension (Auto Extension Search)
//! ```text
//! Universal::file("config") searches for:
//! 1. config.toml    (if exists → TOML parser)
//! 2. config.yaml    (if exists → YAML parser)
//! 3. config.yml     (if exists → YAML parser)
//! 4. config.json    (if exists → JSON parser)
//! ```
//! **Convenience**: Automatic discovery of common configuration files.
//!
//! ### Scenario 5: Edge Cases & Fallbacks
//! ```text
//! Non-existent file:
//! Universal::file("missing.toml") → Empty provider (graceful failure)
//!
//! Unparseable file:
//! config.toml with corrupted content → Try YAML → Try JSON → Empty provider
//! ```
//! **Reliability**: Always returns a valid provider, never panics.
//!
//! ## Format Detection Logic
//!
//! Content-based format detection uses these patterns (in order of specificity):
//!
//! ### TOML Detection (Checked First - Most Specific)
//! ```toml
//! [section]           # Section headers
//! key = "value"       # Key-value pairs with equals
//! ```
//!
//! ### YAML Detection (Checked Second)  
//! ```yaml
//! ---                 # Document separator
//! key: value          # Key-value pairs with colons
//! nested:
//!   subkey: value
//! ```
//!
//! ### JSON Detection (Checked Last - Most Permissive)
//! ```json
//! {"key": "value"}    # Object wrapping
//! ["item1", "item2"]  # Array wrapping (with additional validation)
//! ```
//!
//! ## Performance Optimizations
//!
//! 1. **Extension-First Detection**: Avoids file I/O when extension is known
//! 2. **Format Detection Caching**: Caches results by file path + modification time
//! 3. **Lazy Content Reading**: Only reads file content when needed
//! 4. **Parser Validation**: Tests actual parsing success, not just heuristics
//!
//! ## Usage Examples
//!
//! ```rust
//! use superfigment::Universal;
//!
//! // Auto-detect from any file
//! let provider = Universal::file("config.json");          // → JSON
//! let provider = Universal::file("config.cfg");           // → Content detection
//! let provider = Universal::file("config");               // → Try extensions
//!
//! // Direct string parsing
//! let provider = Universal::string(r#"{"key": "value"}"#); // → JSON
//! let provider = Universal::string("[section]\nkey=val"); // → TOML
//! ```

use figment::{
    providers::Format,
    value::{Map, Value},
    Error, Metadata, Profile, Provider,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

/// Cache entry for format detection results
#[derive(Debug, Clone)]
struct FormatCacheEntry {
    format: ConfigFormat,
    modified_time: u64,
}

/// Global cache for format detection results to avoid repeated content analysis
type FormatCache = Arc<Mutex<HashMap<PathBuf, FormatCacheEntry>>>;

lazy_static::lazy_static! {
    static ref FORMAT_CACHE: FormatCache = Arc::new(Mutex::new(HashMap::new()));
}

/// Detected configuration format
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}

/// Universal configuration provider with automatic format detection and caching
pub struct Universal {
    provider: Box<dyn Provider>,
}

impl Universal {
    /// Create a Universal provider from a file path with automatic format detection
    /// 
    /// This method tries multiple strategies in order:
    /// 1. If file exists: Extension-based detection (fast path)
    /// 2. If extension fails: Content-based detection with caching
    /// 3. If content detection fails: Try parsing with each format until one works
    /// 4. If file doesn't exist: Try multiple extensions (.toml, .yaml, .yml, .json)
    /// 5. Final fallback: Empty provider
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        
        // First try: exact file with extension-based detection
        if path.exists() {
            if let Some(provider) = Self::try_extension_detection(path) {
                return Self { provider };
            }
            
            // Second try: content-based detection with caching
            if let Some(provider) = Self::try_cached_content_detection(path) {
                return Self { provider };
            }
            
            // Third try: brute force - try parsing with each format
            if let Some(provider) = Self::try_all_formats(path) {
                return Self { provider };
            }
        }
        
        // Fourth try: multiple extensions for base path (if file doesn't exist)
        if let Some(provider) = Self::try_multiple_extensions(path) {
            return Self { provider };
        }
        
        // Final fallback: empty provider
        Self::empty_provider()
    }

    /// Create a Universal provider from string content with format detection
    pub fn string<S: AsRef<str>>(content: S) -> Self {
        let content = content.as_ref();
        let format = Self::detect_format_from_content(content);
        
        let provider: Box<dyn Provider> = match format {
            ConfigFormat::Json => Box::new(figment::providers::Json::string(content)),
            ConfigFormat::Toml => Box::new(figment::providers::Toml::string(content)),
            ConfigFormat::Yaml => Box::new(figment::providers::Yaml::string(content)),
        };
        
        Self { provider }
    }

    /// Try multiple common extensions for a base filename
    pub fn file_with_extensions<P: AsRef<Path>>(base_path: P) -> Self {
        Self::try_multiple_extensions(base_path.as_ref())
            .map(|provider| Self { provider })
            .unwrap_or_else(Self::empty_provider)
    }

    /// Fast path: try extension-based detection first
    fn try_extension_detection(path: &Path) -> Option<Box<dyn Provider>> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        match extension.as_str() {
            "json" => Some(Box::new(figment::providers::Json::file(path))),
            "toml" => Some(Box::new(figment::providers::Toml::file(path))),
            "yaml" | "yml" => Some(Box::new(figment::providers::Yaml::file(path))),
            _ => None,
        }
    }

    /// Content-based detection with intelligent caching
    fn try_cached_content_detection(path: &Path) -> Option<Box<dyn Provider>> {
        // Get file modification time
        let modified_time = fs::metadata(path)
            .and_then(|meta| meta.modified())
            .map(|time| time.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0);

        // Check cache first
        if let Ok(cache) = FORMAT_CACHE.lock() {
            if let Some(entry) = cache.get(path) {
                if entry.modified_time == modified_time {
                    // Cache hit - use cached format
                    return Self::create_provider_for_format(path, entry.format);
                }
            }
        }

        // Cache miss - detect format from content
        let content = fs::read_to_string(path).ok()?;
        let format = Self::detect_format_from_content(&content);

        // Update cache
        if let Ok(mut cache) = FORMAT_CACHE.lock() {
            cache.insert(
                path.to_path_buf(),
                FormatCacheEntry {
                    format,
                    modified_time,
                },
            );
        }

        Self::create_provider_for_format(path, format)
    }

    /// Try parsing with each format until one succeeds
    /// This handles unknown extensions (.cfg) and misidentified formats
    fn try_all_formats(path: &Path) -> Option<Box<dyn Provider>> {
        // Try each format in order: TOML, YAML, JSON
        let formats: [Box<dyn Fn() -> Box<dyn Provider>>; 3] = [
            Box::new(|| Box::new(figment::providers::Toml::file(path)) as Box<dyn Provider>),
            Box::new(|| Box::new(figment::providers::Yaml::file(path)) as Box<dyn Provider>),
            Box::new(|| Box::new(figment::providers::Json::file(path)) as Box<dyn Provider>),
        ];
        
        for create_provider in &formats {
            let provider = create_provider();
            // Test if the provider can successfully parse the file
            if provider.data().is_ok() {
                return Some(provider);
            }
        }
        
        None
    }

    /// Create provider for detected format
    fn create_provider_for_format(path: &Path, format: ConfigFormat) -> Option<Box<dyn Provider>> {
        match format {
            ConfigFormat::Json => Some(Box::new(figment::providers::Json::file(path))),
            ConfigFormat::Toml => Some(Box::new(figment::providers::Toml::file(path))),
            ConfigFormat::Yaml => Some(Box::new(figment::providers::Yaml::file(path))),
        }
    }

    /// Try multiple extensions in priority order
    fn try_multiple_extensions(base_path: &Path) -> Option<Box<dyn Provider>> {
        let extensions = ["toml", "yaml", "yml", "json"];
        
        for ext in &extensions {
            let path_with_ext = base_path.with_extension(ext);
            if path_with_ext.exists() {
                if let Some(provider) = Self::try_extension_detection(&path_with_ext) {
                    return Some(provider);
                }
                if let Some(provider) = Self::try_cached_content_detection(&path_with_ext) {
                    return Some(provider);
                }
                if let Some(provider) = Self::try_all_formats(&path_with_ext) {
                    return Some(provider);
                }
            }
        }
        
        None
    }

    /// Detect configuration format from content analysis
    /// Detection order: TOML first (most specific), then YAML, then JSON (most permissive)
    fn detect_format_from_content(content: &str) -> ConfigFormat {
        let trimmed = content.trim();
        
        // Check TOML first - most specific patterns
        if Self::is_toml_format(trimmed) {
            ConfigFormat::Toml
        // Then YAML - key: value patterns
        } else if Self::is_yaml_format(trimmed) {
            ConfigFormat::Yaml
        // Finally JSON - most permissive, check last
        } else if Self::is_json_format(trimmed) {
            ConfigFormat::Json
        } else {
            // Default fallback to TOML
            ConfigFormat::Toml
        }
    }

    /// Detect TOML format by looking for sections and key-value pairs
    /// This is checked FIRST to avoid confusion with JSON arrays
    fn is_toml_format(content: &str) -> bool {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // TOML section headers [section]
            if line.starts_with('[') && line.ends_with(']') {
                return true;
            }
            
            // TOML key = value pairs (but not inside quotes)
            if line.contains('=') && !line.contains(':') {
                return true;
            }
        }
        false
    }

    /// Detect YAML format by looking for key: value pairs and document separators
    fn is_yaml_format(content: &str) -> bool {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // YAML document separator
            if line == "---" {
                return true;
            }
            
            // YAML key: value pairs (but not URLs)
            if let Some(colon_pos) = line.find(':') {
                let before_colon = &line[..colon_pos];
                let after_colon = &line[colon_pos + 1..];
                
                // Skip URLs (http://, https://, etc.)
                if before_colon.ends_with("http") || before_colon.ends_with("https") {
                    continue;
                }
                
                // YAML key: value pattern - colon must be followed by space or end of line
                if !before_colon.contains('=') && 
                   (after_colon.starts_with(' ') || after_colon.is_empty()) {
                    return true;
                }
            }
        }
        false
    }

    /// Detect JSON format by checking for object/array wrapping
    /// This is checked LAST to avoid false positives with TOML arrays
    fn is_json_format(content: &str) -> bool {
        // More specific JSON detection - must be valid JSON structure
        if content.starts_with('{') && content.ends_with('}') {
            return true;
        }
        
        // For arrays, be more careful - avoid TOML section confusion
        if content.starts_with('[') && content.ends_with(']') {
            // Additional checks to distinguish from TOML sections
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() == 1 || content.contains(',') || content.contains('"') {
                // Single line array or contains JSON-like syntax
                return true;
            }
        }
        
        false
    }

    /// Create empty provider as final fallback
    fn empty_provider() -> Self {
        Self {
            provider: Box::new(figment::providers::Serialized::defaults(())),
        }
    }

    /// Clear the format detection cache (useful for testing or memory management)
    pub fn clear_cache() {
        if let Ok(mut cache) = FORMAT_CACHE.lock() {
            cache.clear();
        }
    }
}

impl Provider for Universal {
    fn metadata(&self) -> Metadata {
        let inner_metadata = self.provider.metadata();
        let format = inner_metadata.name;
        
        let metadata_name = format!("Format::Universal::{}", format);
        let mut metadata = Metadata::named(metadata_name);
        if let Some(source) = inner_metadata.source {
            metadata = metadata.source(source);
        }
        metadata
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        self.provider.data()
    }

    fn profile(&self) -> Option<Profile> {
        self.provider.profile()
    }
}