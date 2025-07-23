// Re-export everything from supercli prelude for consistent CLI output
pub use supercli::prelude::*;

// Re-export starbase styling functions directly for backward compatibility
pub use supercli::starbase_styles::color::{
    file as file_path,
    property as property_name,
    hash as hash_value,
    id as id_value
};

