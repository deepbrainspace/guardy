//! Partial configuration for layered overrides
//! 
//! This module provides a trait-based approach for partial configs
//! that avoids JSON serialization overhead.

use std::collections::HashMap;
use crate::Result;

/// Trait for types that can be partially configured
/// 
/// Implement this for your config struct to enable field-by-field overrides
/// without JSON serialization overhead.
pub trait PartialConfigurable {
    /// Apply a single field override by path
    fn set_field(&mut self, path: &str, value: &str) -> Result<()>;
    
    /// Extend an array field by path
    fn extend_array(&mut self, path: &str, values: Vec<String>) -> Result<()>;
}

/// Collection of configuration overrides
/// 
/// This stores overrides as strings and applies them directly
/// to config structs via the PartialConfigurable trait.
#[derive(Debug, Clone, Default)]
pub struct PartialConfig {
    /// Simple field overrides (path -> string value)
    field_overrides: HashMap<String, String>,
    /// Array extensions (path -> values to append)
    array_extensions: HashMap<String, Vec<String>>,
}

impl PartialConfig {
    /// Create a new empty partial configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set a field override
    /// 
    /// Values are stored as strings and parsed when applied
    pub fn set(&mut self, path: &str, value: impl ToString) {
        self.field_overrides.insert(path.to_string(), value.to_string());
    }
    
    /// Set a field only if the value is Some
    pub fn set_if_some<T: ToString>(&mut self, path: &str, value: Option<T>) {
        if let Some(v) = value {
            self.set(path, v);
        }
    }
    
    /// Extend an array field
    pub fn extend_array(&mut self, path: &str, values: Vec<String>) {
        self.array_extensions.insert(path.to_string(), values);
    }
    
    /// Apply all overrides to a configuration
    /// 
    /// This is fast because it directly calls methods on the config struct
    /// without any JSON serialization.
    pub fn apply_to<T: PartialConfigurable>(&self, config: &mut T) -> Result<()> {
        let start = std::time::Instant::now();
        
        // Apply field overrides
        for (path, value) in &self.field_overrides {
            tracing::trace!("Applying override: {} = {}", path, value);
            config.set_field(path, value)?;
        }
        
        // Apply array extensions
        for (path, values) in &self.array_extensions {
            tracing::trace!("Extending array: {} += {} items", path, values.len());
            config.extend_array(path, values.clone())?;
        }
        
        let duration = start.elapsed();
        tracing::trace!("Applied {} overrides in {:?}", 
            self.field_overrides.len() + self.array_extensions.len(), 
            duration
        );
        
        Ok(())
    }
    
    /// Check if there are any overrides
    pub fn is_empty(&self) -> bool {
        self.field_overrides.is_empty() && self.array_extensions.is_empty()
    }
    
    /// Get the number of overrides
    pub fn len(&self) -> usize {
        self.field_overrides.len() + self.array_extensions.len()
    }
}

/// Macro to implement PartialConfigurable for a config struct
/// 
/// This generates efficient match statements for direct field access,
/// avoiding any reflection or serialization overhead.
#[macro_export]
macro_rules! impl_partial_configurable {
    ($type:ty, {
        $(fields: {
            $($field_path:literal => $field_setter:expr),* $(,)?
        })?
        $(arrays: {
            $($array_path:literal => $array_extender:expr),* $(,)?
        })?
    }) => {
        impl $crate::PartialConfigurable for $type {
            fn set_field(&mut self, path: &str, value: &str) -> $crate::Result<()> {
                match path {
                    $($($field_path => {
                        $field_setter(self, value)?;
                        Ok(())
                    },)*)?
                    _ => Err(anyhow::anyhow!("Unknown field path: {}", path))
                }
            }
            
            fn extend_array(&mut self, path: &str, values: Vec<String>) -> $crate::Result<()> {
                match path {
                    $($($array_path => {
                        $array_extender(self, values);
                        Ok(())
                    },)*)?
                    _ => Err(anyhow::anyhow!("Unknown array path: {}", path))
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestConfig {
        debug: bool,
        max_threads: u32,
        name: String,
        patterns: Vec<String>,
    }
    
    impl PartialConfigurable for TestConfig {
        fn set_field(&mut self, path: &str, value: &str) -> Result<()> {
            match path {
                "debug" => {
                    self.debug = value.parse().map_err(|_| anyhow::anyhow!("Invalid bool"))?;
                    Ok(())
                }
                "max_threads" => {
                    self.max_threads = value.parse().map_err(|_| anyhow::anyhow!("Invalid u32"))?;
                    Ok(())
                }
                "name" => {
                    self.name = value.to_string();
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unknown field: {}", path))
            }
        }
        
        fn extend_array(&mut self, path: &str, values: Vec<String>) -> Result<()> {
            match path {
                "patterns" => {
                    self.patterns.extend(values);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unknown array: {}", path))
            }
        }
    }
    
    #[test]
    fn test_direct_field_updates() {
        let mut config = TestConfig {
            debug: false,
            max_threads: 4,
            name: "test".into(),
            patterns: vec!["pat1".into()],
        };
        
        let mut partial = PartialConfig::new();
        partial.set("debug", "true");
        partial.set("max_threads", "8");
        partial.extend_array("patterns", vec!["pat2".into(), "pat3".into()]);
        
        // Measure performance
        let start = std::time::Instant::now();
        partial.apply_to(&mut config).unwrap();
        let duration = start.elapsed();
        
        // Should be fast (no JSON serialization!)
        // More tolerant for debug builds vs release builds
        if cfg!(debug_assertions) {
            assert!(duration.as_millis() < 5, "Debug build took {}ms", duration.as_millis());
        } else {
            assert!(duration.as_micros() < 10, "Release build took {}μs", duration.as_micros());
        }
        
        assert_eq!(config.debug, true);
        assert_eq!(config.max_threads, 8);
        assert_eq!(config.patterns, vec!["pat1".to_string(), "pat2".to_string(), "pat3".to_string()]);
    }
}