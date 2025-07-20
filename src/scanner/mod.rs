pub mod core;
pub mod entropy;
pub mod patterns;
pub mod ignore_intel;
pub mod test_detection;

// Re-export main types for easier access
pub use core::{Scanner, ScanResult, SecretMatch, ScanStats, Warning, WarningCategory};
pub use entropy::is_likely_secret;
pub use patterns::{SecretPatterns, SecretPattern};
pub use ignore_intel::{GitignoreIntelligence, ProjectType, GitignoreWarning, GitignoreSuggestion, WarningSeverity};
pub use test_detection::TestDetector;