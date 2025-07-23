//! SuperCLI Prelude
//! 
//! Import everything you need for CLI styling with a single use statement:
//! ```rust
//! use supercli::prelude::*;
//! ```

// SuperCLI semantic macros - core functionality
pub use crate::{success, warning, info, error, styled};

// Symbol constants
pub use crate::output::symbols;

// Re-export all starbase_styles functions directly
pub use starbase_styles::color::*;

// Re-export clap for complete CLI functionality (when clap feature is enabled)
#[cfg(feature = "clap")]
pub use clap;