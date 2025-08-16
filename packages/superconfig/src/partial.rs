//! Partial configuration for layered overrides
//! 
//! This module provides a PartialConfig type that can hold sparse configuration
//! overrides that get applied on top of a base configuration.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::{Error, Result};

/// Partial configuration for applying overrides
/// 
/// This allows you to specify only the fields you want to override
/// without needing to provide a complete configuration structure.
#[derive(Debug, Clone, Default)]
pub struct PartialConfig {
    /// Map of dot-notation paths to values
    overrides: HashMap<String, Value>,
}

impl PartialConfig {
    /// Create a new empty partial configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set a configuration value at the given path
    /// 
    /// # Example
    /// ```
    /// let mut partial = PartialConfig::new();
    /// partial.set("scanner.max_threads", 4);
    /// partial.set("general.debug", true);
    /// ```
    pub fn set<T: Into<Value>>(&mut self, path: &str, value: T) {
        self.overrides.insert(path.to_string(), value.into());
    }
    
    /// Set a configuration value only if it's Some
    /// 
    /// This is useful for CLI arguments that are optional
    pub fn set_if_some<T: Into<Value>>(&mut self, path: &str, value: Option<T>) {
        if let Some(v) = value {
            self.set(path, v);
        }
    }
    
    /// Extend an array at the given path
    /// 
    /// Instead of replacing the array, this appends to it
    pub fn extend_array(&mut self, path: &str, values: Vec<String>) {
        // Use special marker to indicate extension rather than replacement
        self.overrides.insert(
            format!("{}.__extend", path), 
            Value::Array(values.into_iter().map(Value::String).collect())
        );
    }
    
    /// Apply these overrides to a configuration struct
    /// 
    /// This modifies the configuration in-place with all overrides
    pub fn apply_to<T>(&self, config: &mut T) -> Result<()> 
    where
        T: Serialize + for<'de> Deserialize<'de>
    {
        // Convert config to JSON value for manipulation
        let mut value = serde_json::to_value(&*config)
            .map_err(|e| Error::ParseError(e.to_string()))?;
        
        // Apply each override
        for (path, override_value) in &self.overrides {
            if path.ends_with(".__extend") {
                // Handle array extension
                let base_path = path.trim_suffix(".__extend").unwrap();
                extend_array_at_path(&mut value, base_path, override_value)?;
            } else {
                // Normal value replacement
                apply_at_path(&mut value, path, override_value.clone())?;
            }
        }
        
        // Convert back to the original type
        *config = serde_json::from_value(value)
            .map_err(|e| Error::ParseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Check if there are any overrides
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }
    
    /// Get the number of overrides
    pub fn len(&self) -> usize {
        self.overrides.len()
    }
}

/// Apply a value at a dot-notation path
fn apply_at_path(root: &mut Value, path: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;
    
    // Navigate to the parent of the target
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let Value::Object(map) = current {
                map.insert(part.to_string(), value);
            } else {
                return Err(Error::ParseError(
                    format!("Cannot set field '{}' on non-object", part)
                ));
            }
        } else {
            // Intermediate part - navigate deeper
            if let Value::Object(map) = current {
                // Create intermediate objects if they don't exist
                if !map.contains_key(*part) {
                    map.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                }
                current = map.get_mut(*part).unwrap();
            } else {
                return Err(Error::ParseError(
                    format!("Cannot navigate through '{}' - not an object", part)
                ));
            }
        }
    }
    
    Ok(())
}

/// Extend an array at a dot-notation path
fn extend_array_at_path(root: &mut Value, path: &str, values: &Value) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;
    
    // Navigate to the target array
    for part in &parts {
        if let Value::Object(map) = current {
            if let Some(next) = map.get_mut(*part) {
                current = next;
            } else {
                // Array doesn't exist yet - create it
                map.insert(part.to_string(), Value::Array(vec![]));
                current = map.get_mut(*part).unwrap();
            }
        } else {
            return Err(Error::ParseError(
                format!("Cannot navigate through '{}' - not an object", part)
            ));
        }
    }
    
    // Extend the array
    if let Value::Array(target_array) = current {
        if let Value::Array(new_values) = values {
            target_array.extend(new_values.clone());
        }
    } else {
        return Err(Error::ParseError(
            format!("Target at path '{}' is not an array", path)
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        debug: bool,
        max_threads: u32,
        scanner: ScannerConfig,
    }
    
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ScannerConfig {
        enabled: bool,
        patterns: Vec<String>,
    }
    
    #[test]
    fn test_partial_config_basic() {
        let mut config = TestConfig {
            debug: false,
            max_threads: 4,
            scanner: ScannerConfig {
                enabled: true,
                patterns: vec!["pattern1".into()],
            },
        };
        
        let mut partial = PartialConfig::new();
        partial.set("debug", true);
        partial.set("max_threads", 8u32);
        partial.set("scanner.enabled", false);
        
        partial.apply_to(&mut config).unwrap();
        
        assert_eq!(config.debug, true);
        assert_eq!(config.max_threads, 8);
        assert_eq!(config.scanner.enabled, false);
    }
    
    #[test]
    fn test_partial_config_extend_array() {
        let mut config = TestConfig {
            debug: false,
            max_threads: 4,
            scanner: ScannerConfig {
                enabled: true,
                patterns: vec!["pattern1".into()],
            },
        };
        
        let mut partial = PartialConfig::new();
        partial.extend_array("scanner.patterns", vec![
            "pattern2".into(),
            "pattern3".into(),
        ]);
        
        partial.apply_to(&mut config).unwrap();
        
        assert_eq!(config.scanner.patterns, vec![
            "pattern1".into(),
            "pattern2".into(),
            "pattern3".into(),
        ]);
    }
}