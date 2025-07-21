use serde::Serialize;
use serde_json::Value;

/// Filter out empty arrays from any serializable structure
pub fn filter_empty_arrays<T: Serialize>(input: T) -> Value {
    let mut value = serde_json::to_value(input).unwrap_or(Value::Null);
    filter_empty_arrays_recursive(&mut value);
    value
}

fn filter_empty_arrays_recursive(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let keys_to_remove: Vec<String> = map.iter()
                .filter_map(|(k, v)| {
                    if let Value::Array(arr) = v {
                        if arr.is_empty() {
                            Some(k.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            
            for key in keys_to_remove {
                map.remove(&key);
            }
            
            // Recursively filter nested objects
            for (_, v) in map.iter_mut() {
                filter_empty_arrays_recursive(v);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                filter_empty_arrays_recursive(item);
            }
        }
        _ => {}
    }
}