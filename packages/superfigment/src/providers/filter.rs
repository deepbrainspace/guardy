//! Empty value filtering provider for clean CLI argument handling
//!
//! The Empty provider filters out empty values to prevent CLI overrides from 
//! masking meaningful configuration values from files.

use figment::{
    value::{Dict, Map, Value},
    Error, Metadata, Profile, Provider,
};

/// Provider wrapper that filters out empty values while preserving meaningful falsy values
pub struct Empty<T> {
    inner: T,
    metadata: Metadata,
}

impl<T: Provider> Empty<T> {
    /// Create a new Empty provider that wraps another provider
    ///
    /// # Examples
    /// ```rust
    /// use superfigment::Empty;
    /// use figment::providers::Serialized;
    /// 
    /// let cli_args = vec!["".to_string(), "value".to_string()]; // Empty first element
    /// let provider = Empty::new(Serialized::defaults(cli_args));
    /// // Result only contains "value", empty string is filtered out
    /// ```
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            metadata: Metadata::named("Filter::Empty"),
        }
    }

    /// Set custom metadata name for this provider
    pub fn named<S: Into<String>>(mut self, name: S) -> Self {
        self.metadata = Metadata::named(name.into());
        self
    }

    /// Check if a value should be considered "empty" and filtered out
    ///
    /// Empty values:
    /// - Null values
    /// - Empty strings ("")
    /// - Empty arrays ([])
    /// - Empty objects ({})
    ///
    /// Preserved values (not empty):
    /// - false (valid boolean)
    /// - 0 (valid number)
    /// - Non-empty strings, arrays, objects
    fn is_empty_value(value: &Value) -> bool {
        match value {
            Value::String(_, s) if s.is_empty() => true,
            Value::Array(_, arr) if arr.is_empty() => true,
            Value::Dict(_, dict) if dict.is_empty() => true,
            _ => false,
        }
    }

    /// Recursively filter empty values from a configuration value
    fn filter_empty_values(value: Value) -> Value {
        match value {
            Value::Dict(tag, dict) => {
                let filtered_dict: Dict = dict
                    .into_iter()
                    .filter_map(|(k, v)| {
                        let filtered_value = Self::filter_empty_values(v);
                        if Self::is_empty_value(&filtered_value) {
                            None // Remove empty values
                        } else {
                            Some((k, filtered_value))
                        }
                    })
                    .collect();
                Value::Dict(tag, filtered_dict)
            }
            Value::Array(tag, arr) => {
                let filtered_array: Vec<Value> = arr
                    .into_iter()
                    .filter_map(|v| {
                        let filtered_value = Self::filter_empty_values(v);
                        if Self::is_empty_value(&filtered_value) {
                            None // Remove empty values
                        } else {
                            Some(filtered_value)
                        }
                    })
                    .collect();
                Value::Array(tag, filtered_array)
            }
            // Pass through all other values unchanged (including false, 0, etc.)
            other => other,
        }
    }

}

impl<T: Provider> Provider for Empty<T> {
    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        // Get data from inner provider
        let inner_data = self.inner.data()?;

        // Filter empty values from each profile
        let filtered_data: Result<Map<Profile, Map<String, Value>>, Error> = inner_data
            .into_iter()
            .map(|(profile, profile_data)| {
                let filtered_profile_data: Map<String, Value> = profile_data
                    .into_iter()
                    .filter_map(|(key, value)| {
                        let filtered_value = Self::filter_empty_values(value);
                        if Self::is_empty_value(&filtered_value) {
                            None // Remove empty top-level values
                        } else {
                            Some((key, filtered_value))
                        }
                    })
                    .collect();
                Ok((profile, filtered_profile_data))
            })
            .collect();

        filtered_data
    }

    fn profile(&self) -> Option<Profile> {
        self.inner.profile()
    }
}