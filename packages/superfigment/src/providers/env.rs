//! Enhanced environment variable provider with JSON parsing and caching
//!
//! The Nested provider creates nested configuration structures from environment variables
//! with intelligent caching and advanced value parsing capabilities.

use figment::{
    value::{Dict, Map, Tag, Value},
    Error, Metadata, Profile, Provider,
};
use serde_json;
use std::{collections::HashMap, env};

/// Environment variable provider with caching and nested structure creation
pub struct Nested {
    prefix: String,
    separator: String,
    env_vars: HashMap<String, String>,
    metadata: Metadata,
}

impl Nested {
    /// Create a Nested provider with the given prefix and underscore separator
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::Nested;
    /// 
    /// // Environment: APP_DATABASE_HOST=localhost
    /// let provider = Nested::prefixed("APP_");
    /// // Creates: { database: { host: "localhost" } }
    /// ```
    pub fn prefixed<S: AsRef<str>>(prefix: S) -> Self {
        Self::prefixed_with_separator(prefix, "_")
    }

    /// Create a Nested provider with custom prefix and separator
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::Nested;
    /// 
    /// // Environment: MYAPP.DATABASE.HOST=localhost
    /// let provider = Nested::prefixed_with_separator("MYAPP.", ".");
    /// // Creates: { database: { host: "localhost" } }
    /// ```
    pub fn prefixed_with_separator<S1: AsRef<str>, S2: AsRef<str>>(
        prefix: S1,
        separator: S2,
    ) -> Self {
        let prefix = prefix.as_ref().to_string();
        let separator = separator.as_ref().to_string();
        
        // Cache environment variables during construction for performance
        let env_vars = Self::collect_env_vars(&prefix);
        
        let metadata = Metadata::named("Env::Nested");

        Self {
            prefix,
            separator,
            env_vars,
            metadata,
        }
    }

    /// Create a provider that processes all environment variables (no prefix filtering)
    pub fn new() -> Self {
        Self::prefixed("")
    }

    /// Set custom metadata name for this provider
    pub fn named<S: AsRef<str>>(mut self, name: S) -> Self {
        self.metadata = Metadata::named(name.as_ref().to_string());
        self
    }

    /// Cache environment variables matching the prefix for performance optimization
    /// This avoids repeated calls to env::vars() during data() method
    fn collect_env_vars(prefix: &str) -> HashMap<String, String> {
        env::vars()
            .filter(|(key, _)| key.starts_with(prefix))
            .collect()
    }

    /// Parse environment variable value with smart type detection
    ///
    /// Supports:
    /// - JSON arrays: ["item1", "item2"] 
    /// - JSON objects: {"key": "value"}
    /// - Booleans: true, false, yes, no, 1, 0, on, off
    /// - Numbers: integers and floats
    /// - Strings: fallback for everything else
    fn parse_env_value(value: &str) -> Result<Value, Error> {
        let trimmed = value.trim();

        // Try parsing as JSON first (arrays and objects)
        if (trimmed.starts_with('[') && trimmed.ends_with(']'))
            || (trimmed.starts_with('{') && trimmed.ends_with('}'))
        {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                return Self::json_value_to_figment_value(json_val);
            }
        }

        // Parse booleans
        match trimmed.to_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => return Ok(Value::Bool(Tag::default(), true)),
            "false" | "no" | "0" | "off" => return Ok(Value::Bool(Tag::default(), false)),
            _ => {}
        }

        // Try parsing as number
        if let Ok(int_val) = trimmed.parse::<i64>() {
            return Ok(Value::from(int_val));
        }
        if let Ok(float_val) = trimmed.parse::<f64>() {
            return Ok(Value::from(float_val));
        }

        // Default to string
        Ok(Value::String(Tag::default(), trimmed.to_string()))
    }

    /// Convert serde_json::Value to figment::Value with proper type mapping
    fn json_value_to_figment_value(json_val: serde_json::Value) -> Result<Value, Error> {
        match json_val {
            serde_json::Value::Null => Ok(Value::String(Tag::default(), "null".to_string())),
            serde_json::Value::Bool(b) => Ok(Value::Bool(Tag::default(), b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::from(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::from(f))
                } else {
                    Ok(Value::String(Tag::default(), n.to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(Tag::default(), s)),
            serde_json::Value::Array(arr) => {
                let figment_array: Result<Vec<Value>, Error> = arr
                    .into_iter()
                    .map(Self::json_value_to_figment_value)
                    .collect();
                Ok(Value::Array(Tag::default(), figment_array?))
            }
            serde_json::Value::Object(obj) => {
                let mut figment_dict = Dict::new();
                for (k, v) in obj {
                    figment_dict.insert(k, Self::json_value_to_figment_value(v)?);
                }
                Ok(Value::Dict(Tag::default(), figment_dict))
            }
        }
    }

    /// Insert a value into a nested dictionary structure
    /// 
    /// Creates intermediate dictionaries as needed and handles conflicts
    /// by preferring the new value over existing ones.
    fn insert_nested_value(
        dict: &mut Dict,
        path_parts: &[&str],
        value: Value,
    ) -> Result<(), Error> {
        if path_parts.is_empty() {
            return Ok(());
        }

        if path_parts.len() == 1 {
            // Base case: insert the value
            dict.insert(path_parts[0].to_string(), value);
            return Ok(());
        }

        // Recursive case: create/navigate to nested dict
        let key = path_parts[0];
        let remaining_path = &path_parts[1..];

        let nested_dict = dict
            .entry(key.to_string())
            .or_insert_with(|| Value::Dict(Tag::default(), Dict::new()));

        match nested_dict {
            Value::Dict(_, ref mut nested) => {
                Self::insert_nested_value(nested, remaining_path, value)?;
            }
            _ => {
                // Conflict: replace existing non-dict value with new nested structure
                let mut new_dict = Dict::new();
                Self::insert_nested_value(&mut new_dict, remaining_path, value)?;
                *nested_dict = Value::Dict(Tag::default(), new_dict);
            }
        }

        Ok(())
    }
}

impl Default for Nested {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for Nested {
    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        let mut dict = Dict::new();

        // Process cached environment variables
        for (key, value) in &self.env_vars {
            // Remove prefix if present
            let key_without_prefix = if !self.prefix.is_empty() && key.starts_with(&self.prefix) {
                &key[self.prefix.len()..]
            } else {
                key
            };

            // Skip empty keys
            if key_without_prefix.is_empty() {
                continue;
            }

            // Split only on FIRST separator to create two-level nesting
            // Example: SCANNER_IGNORE_PATHS_ADD → ["SCANNER", "IGNORE_PATHS_ADD"]
            let path_parts: Vec<&str> = key_without_prefix.splitn(2, &self.separator).collect();
            
            // Convert to lowercase for consistent configuration structure
            let path_parts: Vec<String> = path_parts
                .iter()
                .map(|part| part.to_lowercase())
                .collect();
            
            let path_refs: Vec<&str> = path_parts.iter().map(|s| s.as_str()).collect();

            // Parse the environment variable value
            let parsed_value = Self::parse_env_value(value)?;

            // Insert into nested structure
            Self::insert_nested_value(&mut dict, &path_refs, parsed_value)?;
        }

        Ok(Map::from([(Profile::Default, dict)]))
    }

    fn profile(&self) -> Option<Profile> {
        None
    }
}