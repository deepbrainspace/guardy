//! # SuperCLI
//!
//! **Universal CLI Management Platform supporting Multiple Popular Languages** - Built on starbase-styles foundation with enterprise-grade CLI patterns for output, help, prompts, and more.
//!
//! SuperCLI wraps [starbase-styles](https://github.com/moonrepo/starbase) to provide consistent, semantic CLI output patterns
//! across all your command-line tools while maintaining full compatibility with the underlying starbase styling system.
//!
//! Following the SuperConfig approach, SuperCLI is designed to become the universal CLI styling standard across popular
//! languages through WebAssembly bindings, REST APIs, and protocol standardization.
//!
//! ## 🚀 Why SuperCLI?
//!
//! While starbase-styles provides excellent terminal styling, modern CLI applications need more:
//!
//! - **Consistent output patterns** across all CLI tools with semantic functions
//! - **Fine-grained styling control** with the `styled!` macro
//! - **Output mode management** (color/monochrome/none) with environment variable support
//! - **Theme-aware output** that automatically adapts to light/dark terminals
//! - **100% starbase-styles compatibility** with enhanced convenience methods
//!
//! ## 🎯 Core Features
//!
//! ### 🎨 Semantic CLI Output Macros
//! - `success!()` - Success messages with checkmarks
//! - `warning!()` - Warning messages with caution symbols  
//! - `info!()` - Informational messages with info symbols
//! - `error!()` - Error messages with error symbols
//!
//! ### 🎯 Fine-Grained Styling Control
//! - `styled!()` - Mix different styles within a single line
//! - Support for unlimited styling parameters
//! - Automatic output mode adaptation
//!
//! ### 🎛️ Output Mode Management
//! - `GUARDY_OUTPUT_STYLE`: color, monochrome, none
//! - `NO_COLOR` standard compliance
//! - Theme-aware color selection
//!
//! ## Quick Start
//!
//! ```rust
//! use supercli::prelude::*;
//!
//! // Semantic output macros
//! success!("Operation completed successfully!");
//! warning!("This action cannot be undone");
//! info!("Processing files...");
//! error!("Configuration file not found");
//!
//! // Fine-grained styling control
//! styled!("Processing {} files in {}", 
//!     ("150", "number"),
//!     ("/home/user", "file_path")
//! );
//!
//! // Use starbase-styles functions directly
//! println!("Found {}", file("config.toml"));
//! ```

// Module declarations
pub mod output;
pub mod prelude;

#[cfg(feature = "clap")]
pub mod clap;

// Re-export starbase_styles for full compatibility
pub use starbase_styles;

// Re-export clap crate for full CLI functionality (when clap feature is enabled)
#[cfg(feature = "clap")]
pub use clap as clap_crate;

// Re-export our main functionality
pub use output::macros::{success_impl, warning_impl, info_impl, error_impl};
pub use output::styling::{apply_style};