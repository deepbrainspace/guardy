pub mod core;
pub mod entropy;
pub mod patterns;
pub mod ignore_intel;
pub mod test_detection;

// Re-export main types for easier access
pub use core::Scanner;
pub use patterns::SecretPatterns;