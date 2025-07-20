pub mod core;
pub mod entropy;
pub mod patterns;
pub mod test_detection;
pub mod types;

// Re-export main types for easier access
pub use types::Scanner;
pub use patterns::SecretPatterns;