//! # Guardy - Intelligent Git Workflows for Modern Developers
//!
//! A revolutionary MCP-first developer workflow intelligence tool written in pure Rust.
//! Guardy provides secure, fast, and intelligent git hooks and workflow management.
//!
//! ## Features
//!
//! - **MCP-first Architecture**: Built-in Model Context Protocol server for AI integration
//! - **Security-first**: Advanced secret detection and branch protection
//! - **Zero-config**: Intelligent project detection and configuration
//! - **High Performance**: <50ms cold start, <10MB memory usage
//! - **Cross-platform**: Works on Linux, macOS, and Windows
//!
//! ## Quick Start
//!
//! ```bash
//! # Install guardy
//! cargo install guardy
//!
//! # Initialize in your project
//! guardy init
//!
//! # Start MCP server for AI integration
//! guardy mcp start
//! ```

pub mod cli;
pub mod config;
pub mod git;
pub mod hooks;
pub mod mcp;
pub mod security;
pub mod tools;
pub mod utils;

pub use cli::{Cli, Output};
pub use config::GuardyConfig;

/// Result type alias for Guardy operations
pub type Result<T> = anyhow::Result<T>;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
