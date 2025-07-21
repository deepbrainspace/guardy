//! Empty value filtering provider for clean CLI argument handling
//!
//! The Empty provider filters out empty values to prevent CLI overrides from 
//! masking meaningful configuration values from files.

use figment::{
    value::{Dict, Empty as FigmentEmpty, Map, Num, Tag, Value},
    Error, Metadata, Profile, Provider,
};
use serde_json;

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

    /// Convert Figment Value to serde_json::Value for processing
    fn figment_value_to_json_value(value: Value) -> Result<serde_json::Value, Error> {
        match value {
            Value::String(_, s) => Ok(serde_json::Value::String(s)),
            Value::Char(_, c) => Ok(serde_json::Value::String(c.to_string())),
            Value::Bool(_, b) => Ok(serde_json::Value::Bool(b)),
            Value::Num(_, num) => Self::num_to_json_value(num),
            Value::Empty(_, _) => Ok(serde_json::Value::Null),
            Value::Dict(_, dict) => {
                let json_obj: Result<serde_json::Map<String, serde_json::Value>, Error> = dict
                    .into_iter()
                    .map(|(k, v)| Ok((k, Self::figment_value_to_json_value(v)?)))
                    .collect();
                Ok(serde_json::Value::Object(json_obj?))
            }
            Value::Array(_, arr) => {
                let json_arr: Result<Vec<serde_json::Value>, Error> = arr
                    .into_iter()
                    .map(Self::figment_value_to_json_value)
                    .collect();
                Ok(serde_json::Value::Array(json_arr?))
            }
        }
    }

    /// Convert Figment Num to serde_json::Value
    fn num_to_json_value(num: Num) -> Result<serde_json::Value, Error> {
        match num {
            Num::U8(n) => Ok(serde_json::Value::Number(n.into())),
            Num::U16(n) => Ok(serde_json::Value::Number(n.into())),
            Num::U32(n) => Ok(serde_json::Value::Number(n.into())),
            Num::U64(n) => Ok(serde_json::Value::Number(n.into())),
            Num::U128(n) => {
                if let Ok(parsed) = n.to_string().parse::<u64>() {
                    Ok(serde_json::Value::Number(serde_json::Number::from(parsed)))
                } else {
                    Ok(serde_json::Value::String(n.to_string()))
                }
            }
            Num::I8(n) => Ok(serde_json::Value::Number(n.into())),
            Num::I16(n) => Ok(serde_json::Value::Number(n.into())),
            Num::I32(n) => Ok(serde_json::Value::Number(n.into())),
            Num::I64(n) => Ok(serde_json::Value::Number(n.into())),
            Num::I128(n) => {
                if let Ok(parsed) = n.to_string().parse::<i64>() {
                    Ok(serde_json::Value::Number(serde_json::Number::from(parsed)))
                } else {
                    Ok(serde_json::Value::String(n.to_string()))
                }
            }
            Num::F32(n) => {
                if let Some(n) = serde_json::Number::from_f64(n as f64) {
                    Ok(serde_json::Value::Number(n))
                } else {
                    Ok(serde_json::Value::String(n.to_string()))
                }
            }
            Num::F64(n) => {
                if let Some(n) = serde_json::Number::from_f64(n) {
                    Ok(serde_json::Value::Number(n))
                } else {
                    Ok(serde_json::Value::String(n.to_string()))
                }
            }
            Num::USize(n) => Ok(serde_json::Value::Number((n as u64).into())),
            Num::ISize(n) => Ok(serde_json::Value::Number((n as i64).into())),
        }
    }

    /// Convert serde_json::Value back to Figment Value
    fn json_value_to_figment_value(json_val: serde_json::Value) -> Result<Value, Error> {
        match json_val {
            serde_json::Value::Null => Ok(Value::Empty(Tag::default(), FigmentEmpty::Unit)),
            serde_json::Value::Bool(b) => Ok(Value::Bool(Tag::default(), b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::from(i))
                } else if let Some(u) = n.as_u64() {
                    Ok(Value::from(u))
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
                let figment_dict: Result<Dict, Error> = obj
                    .into_iter()
                    .map(|(k, v)| Ok((k, Self::json_value_to_figment_value(v)?)))
                    .collect();
                Ok(Value::Dict(Tag::default(), figment_dict?))
            }
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