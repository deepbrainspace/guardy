//! Extensions for the Figment configuration framework.
//!
//! This module provides extension traits that add functionality to existing Figment types:
//!
//! - [`FigmentExt`]: Adds array merging capabilities to Figment chains

mod array_merge;

pub use array_merge::FigmentExt;