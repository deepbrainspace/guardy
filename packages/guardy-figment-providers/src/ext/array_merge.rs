//! Array merging extension for Figment configuration chains.
//!
//! This module provides the [`FigmentExt`] trait that adds array merging functionality
//! to Figment, allowing configuration arrays to be extended rather than replaced.

use figment::{Figment, Provider};
use figment::providers::Serialized;

/// Extension trait for [`Figment`] that adds array merging capabilities.
///
/// This trait provides methods to merge configuration providers while also handling
/// array merging operations that allow adding/removing items from arrays rather
/// than replacing them entirely.
///
/// # Array Merging Pattern
///
/// Arrays can be extended using `_add` and `_remove` suffixes:
///
/// ```toml
/// # Base configuration
/// ignore_paths = ["base/*", "original/*"]
/// 
/// # Add items to the array
/// ignore_paths_add = ["custom/*", "temp/*"]
/// 
/// # Remove items from the array  
/// ignore_paths_remove = ["original/*"]
/// 
/// # Result: ignore_paths = ["base/*", "custom/*", "temp/*"]
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use figment::Figment;
/// use figment::providers::{Toml, Serialized};
/// use guardy_figment_providers::ext::FigmentExt;
///
/// let figment = Figment::new()
///     .merge(Serialized::defaults([("paths", vec!["default/*"])]))
///     .merge_extend(Toml::file("config.toml"));
/// ```
pub trait FigmentExt {
    /// Merges a provider into the Figment chain and applies array merging.
    ///
    /// This combines the functionality of [`Figment::merge`] with array merging,
    /// allowing configuration files to add/remove items from arrays rather than
    /// replacing them entirely.
    fn merge_extend<P: Provider>(self, provider: P) -> Self;
    
    /// Applies array merging to the current Figment chain without adding a new provider.
    ///
    /// This is useful when you want to apply array merging to an existing chain
    /// without adding additional configuration sources.
    fn merge_arrays(self) -> Self;
    
    /// Merges an optional provider if it contains a value.
    ///
    /// This handles `Option<Provider>` for conditional configuration sources.
    /// If the option is `None`, returns the figment unchanged.
    fn merge_extend_opt<P: Provider>(self, provider: Option<P>) -> Self;
}

impl FigmentExt for Figment {
    fn merge_extend<P: Provider>(self, provider: P) -> Self {
        // First, merge the provider into the chain
        let figment_with_provider = self.merge(provider);
        
        // Then apply array merging to the complete chain
        match figment_with_provider.extract::<serde_json::Value>() {
            Ok(mut config_data) => {
                // Apply array merging to the extracted config
                apply_array_merging(&mut config_data);
                
                // Create a new Figment with the merged data
                Figment::new().merge(Serialized::defaults(config_data))
            }
            Err(_) => {
                // If extraction fails, return the figment with provider
                figment_with_provider
            }
        }
    }
    
    fn merge_arrays(self) -> Self {
        // Convenience method for when no additional provider needed
        match self.extract::<serde_json::Value>() {
            Ok(mut config_data) => {
                // Apply array merging to the extracted config
                apply_array_merging(&mut config_data);
                
                // Create a new Figment with the merged data
                Figment::new().merge(Serialized::defaults(config_data))
            }
            Err(_) => {
                // If extraction fails, return original figment unchanged
                self
            }
        }
    }
    
    fn merge_extend_opt<P: Provider>(self, provider: Option<P>) -> Self {
        match provider {
            Some(p) => self.merge_extend(p),
            None => self,
        }
    }
}

/// Apply array merging to JSON configuration data
fn apply_array_merging(data: &mut serde_json::Value) {
        // Recursively find and merge arrays
        match data {
            serde_json::Value::Object(obj) => {
                merge_object_arrays(obj);
            }
            _ => {}
        }
}
    
fn merge_object_arrays(obj: &mut serde_json::Map<String, serde_json::Value>) {
    let keys: Vec<String> = obj.keys().cloned().collect();
    
    // First pass: recursively process nested objects
    for key in &keys {
        if let Some(serde_json::Value::Object(nested)) = obj.get_mut(key) {
            merge_object_arrays(nested);
        }
    }
    
    // Second pass: process arrays with add/remove
    for key in &keys {
        if key.ends_with("_add") || key.ends_with("_remove") {
            continue; // Skip helper fields
        }
        
        if let Some(serde_json::Value::Array(base_array)) = obj.get(key).cloned() {
            let add_key = format!("{}_add", key);
            let remove_key = format!("{}_remove", key);
            
            let mut merged = base_array;
            let mut changed = false;
            
            // Add items from {field}_add
            if let Some(serde_json::Value::Array(add_items)) = obj.get(&add_key) {
                merged.extend(add_items.clone());
                changed = true;
            }
            
            // Remove items from {field}_remove
            if let Some(serde_json::Value::Array(remove_items)) = obj.get(&remove_key) {
                merged.retain(|item| !remove_items.contains(item));
                changed = true;
            }
            
            // Update the array if we made changes
            if changed {
                obj.insert(key.clone(), serde_json::Value::Array(merged));
            }
        }
    }
    
    // Clean up helper fields after processing
    obj.retain(|k, _| !k.ends_with("_add") && !k.ends_with("_remove"));
}