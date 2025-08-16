//! Configuration builder for layered configuration
//! 
//! This module provides a builder pattern for constructing configurations
//! with multiple layers of overrides using clean serde-based approach.

use serde::{Deserialize, Serialize};
use crate::Result;

/// Builder for constructing layered configurations
/// 
/// Currently supports:
/// - Defaults: 0ms (native Rust)
/// 
/// Future phases will add:
/// - File config support
/// - Environment variable overrides
/// - CLI overrides
pub struct ConfigBuilder<T> {
    defaults: Option<T>,
}

impl<T> ConfigBuilder<T> 
where 
    T: Serialize + for<'de> Deserialize<'de> + Clone + Default + Send + Sync + 'static
{
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            defaults: None,
        }
    }
    
    /// Set the default configuration
    /// 
    /// Performance: 0ms (already in memory)
    pub fn with_defaults(mut self, defaults: T) -> Self {
        self.defaults = Some(defaults);
        self
    }
    
    /// Build the final configuration
    /// 
    /// Returns the configured defaults or T::default() if no defaults were provided
    pub fn build(self) -> Result<T> {
        Ok(self.defaults.unwrap_or_else(T::default))
    }
}

impl<T> Default for ConfigBuilder<T> 
where 
    T: Serialize + for<'de> Deserialize<'de> + Clone + Default + Send + Sync + 'static
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    struct TestConfig {
        debug: bool,
        max_threads: u32,
        name: String,
    }

    #[test]
    fn test_builder_with_defaults() {
        let defaults = TestConfig {
            debug: true,
            max_threads: 4,
            name: "test".to_string(),
        };

        let config = ConfigBuilder::new()
            .with_defaults(defaults.clone())
            .build()
            .unwrap();

        assert_eq!(config, defaults);
    }

    #[test]
    fn test_builder_default() {
        let config = ConfigBuilder::<TestConfig>::new()
            .build()
            .unwrap();

        assert_eq!(config, TestConfig::default());
    }
}