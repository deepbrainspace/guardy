//! Data structures for scan results and statistics

mod coordinate;
mod file_result;
mod scan_result;
mod secret_match;
mod stats;

// Public exports
pub use coordinate::{Coordinate, FileSpan};
pub use file_result::FileResult;
pub use scan_result::ScanResult;
pub use secret_match::{MatchSeverity, SecretMatch};
pub use stats::{DirectoryStats, FileStats, ScanStats};