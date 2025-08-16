//! Content-level filters - Applied after regex pattern matching to refine results
//!
//! These filters operate on potential secret matches to reduce false positives
//! and improve detection accuracy. They are applied after initial regex matching
//! but before final result compilation.
//!
//! ## Filter Hierarchy
//!
//! Content filters are applied in the following order for optimal performance:
//! 1. **Context Filter** - Aho-Corasick keyword prefiltering (THE KEY OPTIMIZATION)
//! 2. **Comment Filter** - Inline ignore directives (guardy:allow)
//! 3. **Entropy Filter** - Statistical entropy validation
//!
//! ## Performance Impact
//!
//! - **Context Filter**: ~5x performance improvement through keyword prefiltering
//!   - Uses Aho-Corasick automaton for O(n) multi-keyword matching
//!   - Extracts keywords from regex patterns for fast content screening
//!   - Only runs full regex matching on files containing relevant keywords
//!
//! - **Comment Filter**: Fast inline directive processing
//!   - Supports `guardy:allow` comments for suppressing false positives
//!   - Regex-based pattern matching for ignore directives
//!
//! - **Entropy Filter**: Statistical validation for secret authenticity
//!   - Filters out low-entropy matches (common words, constants)
//!   - Uses bigram analysis and character class distribution
//!   - Configurable thresholds for different security levels
//!
//! ## Integration with Scanning Pipeline
//!
//! ```text
//! File Content → Context Filter → Regex Matching → Comment Filter → Entropy Filter → Results
//!                      ↓               ↓              ↓              ↓
//!               Fast keyword     Pattern       Ignore directive   Entropy
//!               screening       matching       processing        validation
//! ```

pub mod comment;
pub mod context;
pub mod entropy;

pub use comment::CommentFilter;
pub use context::ContextPrefilter;
pub use entropy::EntropyFilter;
