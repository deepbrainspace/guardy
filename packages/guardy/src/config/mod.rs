pub mod core;
pub mod formats;

// Re-export main types for easier access
pub use core::{GuardyConfig, CONFIG};
pub use formats::ConfigFormat;
