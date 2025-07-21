//! Array merging extension trait for enhanced Figment functionality
//!
//! The ExtendExt trait provides array merging capabilities using _add and _remove patterns
//! with performance optimizations to avoid unnecessary processing.

use figment::{
    providers::Format,
    Figment, Provider,
};
use serde_json;
use std::collections::HashSet;

/// Extension trait that adds array merging capabilities to Figment
///
/// ## Array Merging Pattern
///
/// Arrays can be extended and modified using special suffixes:
/// - `field_add` - Items to add to the base array
/// - `field_remove` - Items to remove from the base array
///
/// ## Example
/// 
/// ```toml
/// # Base configuration
/// ignore_paths = ["*.tmp", "*.log"]
/// 
/// # Extensions  
/// ignore_paths_add = ["*.cache", "build/*"]
/// ignore_paths_remove = ["*.tmp"]
/// ```
///
/// **Result:** `ignore_paths = ["*.log", "*.cache", "build/*"]`
pub trait ExtendExt {
    /// Merge a provider and apply array merging with optimization
    fn merge_extend<P: Provider>(self, provider: P) -> Self;

    /// Apply array merging to current configuration without new providers
    fn merge_arrays(self) -> Self;

    /// Merge an optional provider with array merging
    fn merge_extend_opt<P: Provider>(self, provider: Option<P>) -> Self;
}

impl ExtendExt for Figment {
    /// Merge a provider and apply array merging
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::ExtendExt;
    /// 
    /// let config = Figment::new()
    ///     .merge_extend(provider); // Auto array merging applied
    /// ```
    fn merge_extend<P: Provider>(self, provider: P) -> Self {
        self.merge(provider).merge_arrays()
    }

    /// Apply array merging to current configuration
    ///
    /// This method optimizes by detecting if array merging is actually needed
    /// before performing expensive extraction and reconstruction.
    fn merge_arrays(self) -> Self {
        // Optimization: Check if array merging is needed before expensive operations
        if !ArrayMergeHelper::needs_array_merging(&self) {
            return self; // No merging needed - early return
        }

        ArrayMergeHelper::apply_array_merging(self)
    }

    /// Merge an optional provider with array merging
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superfigment::ExtendExt;
    /// 
    /// let config = Figment::new()
    ///     .merge_extend_opt(cli_args); // Only merged if Some()
    /// ```
    fn merge_extend_opt<P: Provider>(self, provider: Option<P>) -> Self {
        match provider {
            Some(p) => self.merge_extend(p),
            None => self,
        }
    }
}

/// Helper functions for array merging
struct ArrayMergeHelper;

impl ArrayMergeHelper {
    /// Performance optimization: Check if array merging is actually needed
    /// 
    /// Scans for keys ending with "_add" or "_remove" without full extraction
    fn needs_array_merging(figment: &Figment) -> bool {
        // Quick check: extract configuration and scan for merge patterns
        if let Ok(value) = figment.extract::<serde_json::Value>() {
            Self::contains_merge_patterns(&value)
        } else {
            false // If extraction fails, skip array merging
        }
    }

    /// Recursively check if JSON value contains array merge patterns
    fn contains_merge_patterns(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(obj) => {
                // Check if any keys end with _add or _remove
                for key in obj.keys() {
                    if key.ends_with("_add") || key.ends_with("_remove") {
                        return true;
                    }
                }
                // Recursively check nested objects
                obj.values().any(Self::contains_merge_patterns)
            }
            serde_json::Value::Array(arr) => {
                // Check array elements for nested objects with merge patterns
                arr.iter().any(Self::contains_merge_patterns)
            }
            _ => false,
        }
    }

    /// Apply array merging to the figment configuration
    fn apply_array_merging(figment: Figment) -> Figment {
        // Extract complete configuration as JSON for processing
        let json_config = match figment.extract::<serde_json::Value>() {
            Ok(config) => config,
            Err(_) => return figment, // Return original if extraction fails
        };

        // Apply array merging transformations
        let merged_config = Self::merge_object_arrays(json_config);

        // Create new figment with merged configuration
        Figment::new().merge(figment::providers::Json::string(&merged_config.to_string()))
    }

    /// Core array merging logic with _add and _remove pattern support
    fn merge_object_arrays(mut value: serde_json::Value) -> serde_json::Value {
        match &mut value {
            serde_json::Value::Object(obj) => {
                let mut fields_to_remove = Vec::new();
                let mut arrays_to_update: Vec<(String, serde_json::Value)> = Vec::new();

                // Identify base arrays and their _add/_remove operations
                let base_fields: HashSet<String> = obj
                    .keys()
                    .filter_map(|key| {
                        if key.ends_with("_add") {
                            Some(key.strip_suffix("_add").unwrap().to_string())
                        } else if key.ends_with("_remove") {
                            Some(key.strip_suffix("_remove").unwrap().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                // Process each base array that has merge operations
                for base_field in base_fields {
                    let add_key = format!("{}_add", base_field);
                    let remove_key = format!("{}_remove", base_field);

                    // Get base array (or create empty if not exists)
                    let mut result_array = obj
                        .get(&base_field)
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_else(Vec::new);

                    // Apply _add operations
                    if let Some(add_value) = obj.get(&add_key).and_then(|v| v.as_array()) {
                        result_array.extend(add_value.clone());
                        fields_to_remove.push(add_key);
                    }

                    // Apply _remove operations  
                    if let Some(remove_value) = obj.get(&remove_key).and_then(|v| v.as_array()) {
                        result_array.retain(|item| !remove_value.contains(item));
                        fields_to_remove.push(remove_key);
                    }

                    // Queue array for update
                    arrays_to_update.push((base_field, serde_json::Value::Array(result_array)));
                }

                // Apply updates and cleanup
                for (field, new_array) in arrays_to_update {
                    obj.insert(field, new_array);
                }
                for field in fields_to_remove {
                    obj.remove(&field);
                }

                // Recursively process nested objects
                for value in obj.values_mut() {
                    *value = Self::merge_object_arrays(value.clone());
                }

                serde_json::Value::Object(obj.clone())
            }
            serde_json::Value::Array(arr) => {
                // Recursively process array elements
                let processed_array: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|item| Self::merge_object_arrays(item.clone()))
                    .collect();
                serde_json::Value::Array(processed_array)
            }
            // Pass through all other value types unchanged
            other => other.clone(),
        }
    }
}