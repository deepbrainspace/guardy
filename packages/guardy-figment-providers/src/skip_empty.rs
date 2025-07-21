//! Skip empty values provider for Figment.
//!
//! Filters out empty values (empty strings, empty arrays, null values) to prevent
//! CLI overrides from masking meaningful configuration values from files.

use figment::{Provider, Metadata, Profile, Error, value::{Map, Dict}, providers::Serialized, error::Actual};
use serde::Serialize;
use serde_json::Value;

/// A provider that filters out empty values from another provider.
/// 
/// This is particularly useful for CLI arguments where empty values 
/// (like empty vectors from unused flags) should not override meaningful
/// configuration values from files.
///
/// # Examples
///
/// ```rust
/// use figment::{Figment, providers::Serialized};
/// use guardy_figment_providers::SkipEmpty;
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize)]
/// struct CliArgs {
///     name: Option<String>,
///     tags: Vec<String>,    // Empty when not specified
///     count: Option<u32>,
/// }
/// 
/// let cli_args = CliArgs {
///     name: Some("test-app".to_string()),
///     tags: Vec::new(),     // Empty - should be skipped
///     count: None,          // None - should be skipped  
/// };
/// 
/// let figment = Figment::new()
///     .merge(SkipEmpty::new(cli_args)); // Only 'name' will be included
/// 
/// let result: serde_json::Value = figment.extract().unwrap();
/// // Result only contains: {"name": "test-app"}
/// ```
pub struct SkipEmpty<T> {
    inner: T,
    metadata: Metadata,
}

impl<T> SkipEmpty<T> {
    /// Creates a new SkipEmpty provider that wraps the given data.
    /// 
    /// Empty values in the data will be filtered out when providing
    /// configuration to Figment.
    pub fn new(data: T) -> Self {
        SkipEmpty {
            inner: data,
            metadata: Metadata::named("skip empty values"),
        }
    }
    
    /// Sets a custom name for this provider in metadata.
    pub fn named<S: Into<String>>(mut self, name: S) -> Self {
        self.metadata = Metadata::named(name.into());
        self
    }
}

impl<T: Serialize> Provider for SkipEmpty<T> {
    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        // Use Figment's Serialized provider to handle the conversion properly
        let serialized = Serialized::defaults(&self.inner);
        let mut data_map = serialized.data()?;
        
        // Get the default profile data and filter it
        let dict = data_map.remove(&Profile::Default)
            .unwrap_or_else(Dict::new);
        
        // Convert Dict to serde_json::Value for filtering, then back to Dict
        let json_value = dict_to_json_value(dict)?;
        let filtered_json = filter_empty_values(json_value);
        let filtered_dict = json_value_to_dict(filtered_json)?;
        
        let mut result_map = Map::new();
        result_map.insert(Profile::Default, filtered_dict);
        Ok(result_map)
    }
}

/// Convert a figment Dict to serde_json::Value for processing
fn dict_to_json_value(dict: Dict) -> Result<Value, Error> {
    let mut map = serde_json::Map::new();
    
    for (key, figment_value) in dict {
        let json_value = figment_value_to_json_value(figment_value)?;
        map.insert(key.into(), json_value);
    }
    
    Ok(Value::Object(map))
}

/// Convert a figment::value::Value to serde_json::Value
fn figment_value_to_json_value(value: figment::value::Value) -> Result<Value, Error> {
    use figment::value::Value as FV;
    
    match value {
        FV::String(_, s) => Ok(Value::String(s)),
        FV::Char(_, c) => Ok(Value::String(c.to_string())),
        FV::Bool(_, b) => Ok(Value::Bool(b)),
        FV::Num(_, n) => {
            match n {
                figment::value::Num::U8(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::U16(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::U32(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::U64(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::U128(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::USize(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::I8(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::I16(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::I32(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::I64(v) => Ok(Value::Number(v.into())),
                figment::value::Num::I128(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::ISize(v) => Ok(Value::Number((v as i64).into())),
                figment::value::Num::F32(v) => {
                    serde_json::Number::from_f64(v as f64)
                        .map(Value::Number)
                        .ok_or_else(|| Error::from(figment::error::Kind::InvalidType(
                            Actual::Float(v as f64),
                            "valid number".into()
                        )))
                }
                figment::value::Num::F64(v) => {
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .ok_or_else(|| Error::from(figment::error::Kind::InvalidType(
                            Actual::Float(v),
                            "valid number".into()
                        )))
                }
            }
        }
        FV::Empty(_, _) => Ok(Value::Null),
        FV::Dict(_, dict) => dict_to_json_value(dict),
        FV::Array(_, vec) => {
            let json_vec: Result<Vec<Value>, Error> = vec.into_iter()
                .map(figment_value_to_json_value)
                .collect();
            Ok(Value::Array(json_vec?))
        }
    }
}

/// Convert serde_json::Value back to figment Dict
fn json_value_to_dict(value: Value) -> Result<Dict, Error> {
    match value {
        Value::Object(map) => {
            let mut dict = Dict::new();
            for (key, json_value) in map {
                let figment_value = json_value_to_figment_value(json_value)?;
                dict.insert(key.into(), figment_value);
            }
            Ok(dict)
        }
        _ => {
            return Err(Error::from(figment::error::Kind::InvalidType(
                Actual::from(serde::de::Unexpected::Str("non-object")),
                "object".into()
            )));
        }
    }
}

/// Convert serde_json::Value to figment::value::Value
fn json_value_to_figment_value(value: Value) -> Result<figment::value::Value, Error> {
    use figment::value::{Value as FV, Tag};
    
    let tag = Tag::Default;
    
    match value {
        Value::String(s) => Ok(FV::String(tag, s)),
        Value::Bool(b) => Ok(FV::Bool(tag, b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(FV::Num(tag, i.into()))
            } else if let Some(f) = n.as_f64() {
                Ok(FV::Num(tag, figment::value::Num::F64(f)))
            } else {
                Ok(FV::Empty(tag, figment::value::Empty::None))
            }
        }
        Value::Null => Ok(FV::Empty(tag, figment::value::Empty::None)),
        Value::Object(map) => {
            let mut dict = Dict::new();
            for (key, json_value) in map {
                let figment_value = json_value_to_figment_value(json_value)?;
                dict.insert(key.into(), figment_value);
            }
            Ok(FV::Dict(tag, dict))
        }
        Value::Array(vec) => {
            let figment_vec: Result<Vec<figment::value::Value>, Error> = vec.into_iter()
                .map(json_value_to_figment_value)
                .collect();
            Ok(FV::Array(tag, figment_vec?))
        }
    }
}

/// Recursively filter empty values from a serde_json::Value
fn filter_empty_values(value: Value) -> Value {
    match value {
        Value::Object(mut map) => {
            // Remove entries with empty values and recursively filter remaining values
            map.retain(|_, v| !is_empty_value(v));
            
            // Recursively filter remaining values
            for (_, v) in map.iter_mut() {
                *v = filter_empty_values(std::mem::take(v));
            }
            
            Value::Object(map)
        }
        Value::Array(vec) => {
            // Filter empty elements and recursively process remaining ones
            let filtered: Vec<Value> = vec.into_iter()
                .filter(|v| !is_empty_value(v))
                .map(filter_empty_values)
                .collect();
            Value::Array(filtered)
        }
        _ => value, // Primitive values are returned as-is
    }
}

/// Check if a value should be considered "empty" and filtered out
///
/// # Examples
///
/// ```
/// use guardy_figment_providers::skip_empty::is_empty_value;
/// use serde_json::Value;
/// 
/// // Empty values that get filtered
/// assert!(is_empty_value(&Value::Null));
/// assert!(is_empty_value(&Value::String("".to_string())));
/// assert!(is_empty_value(&Value::Array(vec![])));
/// assert!(is_empty_value(&Value::Object(serde_json::Map::new())));
/// 
/// // Non-empty values that are kept
/// assert!(!is_empty_value(&Value::String("hello".to_string())));
/// assert!(!is_empty_value(&Value::Number(42.into())));
/// assert!(!is_empty_value(&Value::Bool(false))); // false is not empty!
/// ```
pub fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(arr) => arr.is_empty(),
        Value::Object(obj) => obj.is_empty(),
        Value::Bool(_) => false, // false is a valid value, not empty
        Value::Number(_) => false, // 0 is a valid value, not empty
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;


    #[test]
    fn test_filter_empty_values() {
        let input = json!({
            "name": "test",
            "empty_string": "",
            "empty_array": [],
            "null_value": null,
            "valid_zero": 0,
            "valid_false": false,
            "nested": {
                "keep": "this",
                "remove": "",
                "also_keep": 42
            }
        });

        let expected = json!({
            "name": "test",
            "valid_zero": 0,
            "valid_false": false,
            "nested": {
                "keep": "this", 
                "also_keep": 42
            }
        });

        let result = filter_empty_values(input);
        assert_eq!(result, expected);
    }
}