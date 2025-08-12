//! Scanning filter system - Two-tier architecture for performance optimization
//!
//! The filter system provides a two-level hierarchy designed to maximize scanning
//! performance by applying fast filters first and more expensive analysis only
//! when necessary.
//!
//! ## Architecture Overview
//!
//! ### Directory-Level Filters (Pre-Processing)
//! Applied before file content is loaded to reduce I/O operations:
//! - **Path filtering**: Ignore patterns and directory exclusions
//! - **Size filtering**: File size limits and validation
//! - **Binary detection**: Extension-based + content inspection fallback
//!
//! ### Content-Level Filters (Post-Processing) 
//! Applied after regex pattern matching to refine results:
//! - **Context prefiltering**: Aho-Corasick keyword matching (KEY OPTIMIZATION)
//! - **Comment filtering**: Inline ignore directives (guardy:allow)
//! - **Entropy filtering**: Statistical validation for secret authenticity
//!
//! ## Performance Impact
//!
//! The filter system provides the core performance improvements for scan2:
//!
//! | Filter Type | Performance Gain | Mechanism |
//! |-------------|------------------|-----------|
//! | **Context Prefilter** | ~5x improvement | Aho-Corasick O(n) vs regex O(p*n) |
//! | **Directory Filters** | ~3x improvement | Eliminate 70-90% of files from processing |
//! | **Shared Data Structures** | ~2x improvement | Arc<LazyLock> zero-copy sharing |
//!
//! Combined expected improvement: **~30x faster** than naive regex-per-file approach
//!
//! ## Usage Pattern
//!
//! ```rust
//! use guardy::scan::filters::{directory, content};
//!
//! // Directory-level filtering (fast, metadata-based)
//! let path_filter = directory::PathFilter::new(&config)?;
//! let size_filter = directory::SizeFilter::new(&config)?;
//! let binary_filter = directory::BinaryFilter::new(&config)?;
//!
//! // Content-level filtering (applied to potential matches)
//! let context_filter = content::ContextPrefilter::new(&patterns)?;
//! let comment_filter = content::CommentFilter::new(&config)?;
//! let entropy_filter = content::EntropyFilter::new(&config)?;
//! ```
//!
//! ## Filter Implementation Requirements
//!
//! All filters implement consistent interfaces:
//! - **Configuration-based**: Accept ScannerConfig for behavior customization
//! - **Error handling**: Use anyhow::Result for comprehensive error reporting
//! - **Performance optimized**: Use Arc<LazyLock> for shared data structures
//! - **Statistics tracking**: Provide filtering statistics for debugging
//! - **Thread-safe**: Safe for parallel execution across multiple workers

pub mod content;
pub mod directory;

// Re-export all filters for convenient access
pub use content::{CommentFilter, ContextPrefilter, EntropyFilter};
pub use directory::{BinaryFilter, PathFilter, SizeFilter};