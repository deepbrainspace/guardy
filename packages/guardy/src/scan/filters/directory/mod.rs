//! Directory-level filters - Applied before content processing to reduce I/O operations
//!
//! These filters operate on file metadata and paths to quickly exclude files
//! that don't need content analysis, providing significant performance benefits
//! by reducing the number of files that require I/O operations.
//!
//! ## Filter Hierarchy
//!
//! Directory filters are applied in the following order for optimal performance:
//! 1. **Path Filter** - Ignore patterns and directory exclusions
//! 2. **Size Filter** - File size validation (metadata only)
//! 3. **Binary Filter** - Binary file detection (extension + content inspection)
//!
//! ## Performance Characteristics
//!
//! - **Path Filter**: O(1) HashSet lookup for ignore patterns
//! - **Size Filter**: O(1) metadata check, no file I/O
//! - **Binary Filter**: O(1) extension check, O(512 bytes) content inspection fallback
//!
//! These filters can eliminate 70-90% of files from content processing,
//! providing the majority of our performance improvement.

pub mod binary;
pub mod path;
pub mod size;

pub use binary::BinaryFilter;
pub use path::PathFilter;
pub use size::SizeFilter;