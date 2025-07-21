//! Enhanced environment variable provider for Figment.
//!
//! Provides smart environment variable mapping with nested object creation
//! and flexible prefix/separator handling.

use figment::{Provider, Metadata, Profile, Error, value::{Map, Dict, Value, Tag}};
use std::collections::HashMap;
use std::env;

/// An enhanced environment variable provider that creates nested configuration structures.
/// 
/// This provider transforms environment variables with prefixes and separators into
/// nested configuration objects, making environment-based configuration more intuitive.
///
/// # Examples
///
/// ```rust
/// use figment::Figment;
/// use guardy_figment_providers::NestedEnv;
/// use serde::{Serialize, Deserialize};
/// use std::env;
/// 
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     database: DatabaseConfig,
///     server: ServerConfig,
/// }
/// 
/// #[derive(Serialize, Deserialize)]
/// struct DatabaseConfig {
///     host: String,
///     port: u16,
/// }
/// 
/// #[derive(Serialize, Deserialize)]
/// struct ServerConfig {
///     port: u16,
///     workers: u32,
/// }
/// 
/// // Set environment variables for the example
/// env::set_var("DOCTEST_APP_DATABASE_HOST", "localhost");
/// env::set_var("DOCTEST_APP_DATABASE_PORT", "5432");
/// env::set_var("DOCTEST_APP_SERVER_PORT", "8080");
/// env::set_var("DOCTEST_APP_SERVER_WORKERS", "4");
/// 
/// let figment = Figment::new()
///     .merge(NestedEnv::prefixed("DOCTEST_APP_")); // Creates nested structure
/// 
/// let config: Config = figment.extract().unwrap();
/// 
/// // Verify the nested structure was created correctly
/// assert_eq!(config.database.host, "localhost");
/// assert_eq!(config.database.port, 5432);
/// assert_eq!(config.server.port, 8080);
/// assert_eq!(config.server.workers, 4);
/// 
/// // Clean up environment variables
/// env::remove_var("DOCTEST_APP_DATABASE_HOST");
/// env::remove_var("DOCTEST_APP_DATABASE_PORT");
/// env::remove_var("DOCTEST_APP_SERVER_PORT");
/// env::remove_var("DOCTEST_APP_SERVER_WORKERS");
/// ```
pub struct NestedEnv {
    prefix: String,
    separator: String,
    env_vars: HashMap<String, String>,
    metadata: Metadata,
}

impl NestedEnv {
    /// Creates a new NestedEnv provider with the given prefix.
    /// 
    /// Environment variables starting with the prefix will be included,
    /// with the prefix stripped and the remaining parts used to create
    /// nested objects using underscore as separator.
    pub fn prefixed<S: Into<String>>(prefix: S) -> Self {
        let prefix = prefix.into();
        let env_vars = collect_env_vars(&prefix);
        
        NestedEnv {
            prefix: prefix.clone(),
            separator: "_".to_string(),
            env_vars,
            metadata: Metadata::named(format!("environment variables with prefix '{}'", prefix)),
        }
    }
    
    /// Creates a new NestedEnv provider with custom prefix and separator.
    /// 
    /// This allows for more flexible environment variable naming schemes.
    /// For example, with prefix "APP" and separator "__", the variable
    /// `APP__DATABASE__HOST` becomes `database.host` in the config.
    pub fn prefixed_with_separator<S: Into<String>>(prefix: S, separator: S) -> Self {
        let prefix = prefix.into();
        let separator = separator.into();
        let env_vars = collect_env_vars(&prefix);
        
        NestedEnv {
            prefix: prefix.clone(),
            separator: separator.clone(),
            env_vars,
            metadata: Metadata::named(format!("environment variables with prefix '{}' and separator '{}'", prefix, separator)),
        }
    }
    
    /// Sets a custom name for this provider in metadata.
    pub fn named<S: Into<String>>(mut self, name: S) -> Self {
        self.metadata = Metadata::named(name.into());
        self
    }
}

impl Provider for NestedEnv {
    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut dict = Dict::new();
        
        for (key, value) in &self.env_vars {
            if !key.starts_with(&self.prefix) {
                continue;
            }
            
            // Remove prefix and split into path components
            let key_without_prefix = &key[self.prefix.len()..];
            let path_parts: Vec<&str> = key_without_prefix
                .split(&self.separator)
                .filter(|part| !part.is_empty())
                .collect();
            
            if path_parts.is_empty() {
                continue;
            }
            
            // Convert value to appropriate type and insert into nested structure
            let figment_value = parse_env_value(value)?;
            insert_nested_value(&mut dict, &path_parts, figment_value)?;
        }
        
        let mut map = Map::new();
        map.insert(Profile::Default, dict);
        Ok(map)
    }
}

/// Collect environment variables that start with the given prefix
fn collect_env_vars(prefix: &str) -> HashMap<String, String> {
    env::vars()
        .filter(|(key, _)| key.starts_with(prefix))
        .collect()
}

/// Parse environment variable value into appropriate Figment value type
fn parse_env_value(value: &str) -> Result<Value, Error> {
    let tag = Tag::Default;
    
    // Try to parse as different types in order of specificity
    
    // Boolean values
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => return Ok(Value::Bool(tag, true)),
        "false" | "no" | "0" | "off" => return Ok(Value::Bool(tag, false)),
        _ => {}
    }
    
    // Numeric values
    if let Ok(int_val) = value.parse::<i64>() {
        return Ok(Value::Num(tag, int_val.into()));
    }
    
    if let Ok(float_val) = value.parse::<f64>() {
        return Ok(Value::Num(tag, figment::value::Num::F64(float_val)));
    }
    
    // Default to string
    Ok(Value::String(tag, value.to_string()))
}

/// Insert a value into a nested dictionary structure using path components
fn insert_nested_value(dict: &mut Dict, path_parts: &[&str], value: Value) -> Result<(), Error> {
    if path_parts.is_empty() {
        return Ok(());
    }
    
    if path_parts.len() == 1 {
        // Base case: insert the value
        dict.insert(path_parts[0].to_lowercase().into(), value);
        return Ok(());
    }
    
    // Recursive case: navigate/create nested structure
    let key = path_parts[0].to_lowercase();
    let remaining_path = &path_parts[1..];
    
    // Get or create nested dict
    let nested_dict = match dict.get_mut(&key) {
        Some(Value::Dict(_, existing_dict)) => existing_dict,
        Some(_) => {
            // Key exists but is not a dict - this is a conflict
            return Err(Error::from(figment::error::Kind::InvalidType(
                figment::error::Actual::from(serde::de::Unexpected::Other("non-object")),
                "object".into()
            )));
        }
        None => {
            // Key doesn't exist - create new nested dict
            let new_dict = Dict::new();
            dict.insert(key.clone().into(), Value::Dict(Tag::Default, new_dict));
            
            // Get the dict we just inserted
            match dict.get_mut(&key).unwrap() {
                Value::Dict(_, dict) => dict,
                _ => unreachable!(),
            }
        }
    };
    
    // Recursively insert into nested dict
    insert_nested_value(nested_dict, remaining_path, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_parse_env_value() {
        // Test boolean parsing
        assert!(matches!(parse_env_value("true").unwrap(), Value::Bool(_, true)));
        assert!(matches!(parse_env_value("false").unwrap(), Value::Bool(_, false)));
        assert!(matches!(parse_env_value("yes").unwrap(), Value::Bool(_, true)));
        assert!(matches!(parse_env_value("no").unwrap(), Value::Bool(_, false)));
        assert!(matches!(parse_env_value("1").unwrap(), Value::Bool(_, true)));
        assert!(matches!(parse_env_value("0").unwrap(), Value::Bool(_, false)));
        
        // Test numeric parsing
        assert!(matches!(parse_env_value("42").unwrap(), Value::Num(_, _)));
        assert!(matches!(parse_env_value("3.14").unwrap(), Value::Num(_, _)));
        
        // Test string parsing
        assert!(matches!(parse_env_value("hello").unwrap(), Value::String(_, _)));
    }
    
    #[test] 
    fn test_insert_nested_value() {
        let mut dict = Dict::new();
        let value = Value::String(Tag::Default, "test-value".to_string());
        
        insert_nested_value(&mut dict, &["db", "host"], value).unwrap();
        
        // Should create nested structure: {"db": {"host": "test-value"}}
        assert!(dict.contains_key("db"));
        match dict.get("db").unwrap() {
            Value::Dict(_, nested) => {
                assert!(nested.contains_key("host"));
                match nested.get("host").unwrap() {
                    Value::String(_, s) => assert_eq!(s, "test-value"),
                    _ => panic!("Expected string value"),
                }
            }
            _ => panic!("Expected dict value"),
        }
    }
    
    #[test]
    fn test_nested_env_with_mock_vars() {
        // Set up mock environment variables
        env::set_var("TEST_DB_HOST", "localhost");
        env::set_var("TEST_DB_PORT", "5432");
        env::set_var("TEST_SERVER_WORKERS", "4");
        env::set_var("TEST_DEBUG", "true");
        
        let provider = NestedEnv::prefixed("TEST_");
        let data = provider.data().unwrap();
        let dict = data.get(&Profile::Default).unwrap();
        
        // Verify nested structure was created
        assert!(dict.contains_key("db"));
        assert!(dict.contains_key("server"));
        assert!(dict.contains_key("debug"));
        
        // Clean up
        env::remove_var("TEST_DB_HOST");
        env::remove_var("TEST_DB_PORT");
        env::remove_var("TEST_SERVER_WORKERS");
        env::remove_var("TEST_DEBUG");
    }
}