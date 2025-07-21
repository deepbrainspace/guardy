//! Access and convenience extension trait for enhanced Figment functionality
//!
//! The AccessExt trait provides convenient methods for accessing configuration values,
//! exporting to different formats, and debugging configuration state.

use figment::{Figment, Error};
use figment::error::Actual;

/// Extension trait that adds convenience methods for accessing and exporting Figment data
///
/// Provides methods for format conversion, value extraction, and debugging
/// that make working with Figment configurations more convenient.
///
/// ## Examples
///
/// ### Format Conversion
/// ```rust
/// use figment::Figment;
/// use superfigment::AccessExt;
/// 
/// let config = Figment::new()
///     .merge(/* providers */);
///     
/// let json_str = config.as_json()?;
/// let yaml_str = config.as_yaml()?;
/// let toml_str = config.as_toml()?;
/// # Ok::<(), figment::Error>(())
/// ```
///
/// ### Value Access
/// ```rust
/// use figment::Figment;
/// use superfigment::AccessExt;
/// 
/// let config = Figment::new()
///     .merge(/* providers */);
///     
/// let host = config.get_string("database.host")?;
/// let ports = config.get_array::<u16>("allowed_ports")?;
/// let has_redis = config.has_key("redis")?;
/// # Ok::<(), figment::Error>(())
/// ```
///
/// ### Debug Information
/// ```rust
/// use figment::Figment;
/// use superfigment::AccessExt;
/// 
/// let config = Figment::new()
///     .merge(/* providers */);
///     
/// println!("{}", config.debug_config()?);
/// let sources = config.debug_sources();
/// # Ok::<(), figment::Error>(())
/// ```
pub trait AccessExt {
    /// Export configuration as pretty-formatted JSON string
    fn as_json(&self) -> Result<String, Error>;

    /// Export configuration as YAML string
    fn as_yaml(&self) -> Result<String, Error>;

    /// Export configuration as TOML string
    fn as_toml(&self) -> Result<String, Error>;

    /// Get a string value from configuration
    fn get_string<K: AsRef<str>>(&self, key: K) -> Result<String, Error>;

    /// Get an array value from configuration
    fn get_array<T>(&self, key: &str) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned;

    /// Check if a configuration key exists
    fn has_key(&self, key: &str) -> Result<bool, Error>;

    /// Get all top-level configuration keys
    fn keys(&self) -> Result<Vec<String>, Error>;

    /// Debug configuration with pretty-printed values and source information
    fn debug_config(&self) -> Result<String, Error>;

    /// Get debug information about configuration sources
    fn debug_sources(&self) -> Vec<figment::Metadata>;
}

impl AccessExt for Figment {
    /// Export configuration as pretty-formatted JSON string
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let json_str = config.as_json()?;
    /// println!("{}", json_str); // Pretty-printed JSON
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn as_json(&self) -> Result<String, Error> {
        let value = self.extract::<serde_json::Value>()?;
        serde_json::to_string_pretty(&value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid JSON".into(),
            ))
        })
    }

    /// Export configuration as YAML string
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let yaml_str = config.as_yaml()?;
    /// println!("{}", yaml_str);
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn as_yaml(&self) -> Result<String, Error> {
        let value = self.extract::<serde_json::Value>()?;
        serde_yml::to_string(&value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid YAML".into(),
            ))
        })
    }

    /// Export configuration as TOML string
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let toml_str = config.as_toml()?;
    /// println!("{}", toml_str);
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn as_toml(&self) -> Result<String, Error> {
        let value = self.extract::<toml::Value>()?;
        toml::to_string_pretty(&value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid TOML".into(),
            ))
        })
    }

    /// Get a string value from configuration
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let host = config.get_string("database.host")?;
    /// println!("Database host: {}", host);
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn get_string<K: AsRef<str>>(&self, key: K) -> Result<String, Error> {
        self.extract_inner(key.as_ref())
    }

    /// Get an array value from configuration
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let origins = config.get_array::<String>("cors.allowed_origins")?;
    /// println!("CORS origins: {:?}", origins);
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn get_array<T>(&self, key: &str) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        self.extract_inner(key)
    }

    /// Check if a configuration key exists
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// if config.has_key("redis.enabled")? {
    ///     println!("Redis is configured");
    /// }
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn has_key(&self, key: &str) -> Result<bool, Error> {
        match self.find_value(key) {
            Ok(_) => Ok(true),
            Err(Error { kind: figment::error::Kind::MissingField(_), .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get all top-level configuration keys
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let keys = config.keys()?;
    /// println!("Config sections: {:?}", keys);
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn keys(&self) -> Result<Vec<String>, Error> {
        let value = self.extract::<serde_json::Value>()?;
        match value {
            serde_json::Value::Object(map) => {
                Ok(map.keys().cloned().collect())
            }
            _ => Ok(vec![]),
        }
    }

    /// Debug configuration with pretty-printed values and source information
    ///
    /// Returns a formatted string showing the final configuration values
    /// along with metadata about which providers contributed each value.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// println!("{}", config.debug_config()?);
    /// // Output shows merged config with provider information
    /// # Ok::<(), figment::Error>(())
    /// ```
    fn debug_config(&self) -> Result<String, Error> {
        let json_value = self.extract::<serde_json::Value>()?;
        let pretty_json = serde_json::to_string_pretty(&json_value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid JSON".into(),
            ))
        })?;

        Ok(format!(
            "=== Figment Configuration Debug ===\n\nFinal Configuration:\n{}\n\nProvider Chain:\n{:#?}",
            pretty_json,
            self
        ))
    }

    /// Get debug information about configuration sources
    ///
    /// Returns metadata about the providers that contributed to the configuration.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::AccessExt;
    /// 
    /// let config = Figment::new()
    ///     .merge(/* provider */);
    ///     
    /// let sources = config.debug_sources();
    /// println!("Configuration sources: {:#?}", sources);
    /// ```
    fn debug_sources(&self) -> Vec<figment::Metadata> {
        self.metadata().cloned().collect()
    }
}