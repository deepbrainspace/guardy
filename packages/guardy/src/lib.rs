//! # Guardy - Fast, secure git hooks in Rust
//!
//! Guardy is a high-performance git hooks framework written in Rust that provides:
//!
//! - **Fast Security Scanning**: Multi-threaded secret detection with entropy analysis
//! - **Protected File Synchronization**: Keep configuration files in sync across repositories  
//! - **Comprehensive Git Hook Support**: Pre-commit, pre-push, and other git hooks
//! - **Flexible Configuration**: YAML, TOML, and JSON configuration support
//!
//! ## Quick Start
//!
//! ### Installation
//!
//! ```bash
//! # Install from crates.io
//! cargo install guardy
//!
//! # Or clone and build
//! git clone https://github.com/deepbrainspace/guardy
//! cd guardy
//! cargo build --release
//! ```
//!
//! ### Basic Usage
//!
//! ```bash
//! # Install git hooks in your repository
//! guardy install
//!
//! # Scan files for secrets
//! guardy scan src/
//!
//! # Check status
//! guardy status
//!
//! # Sync protected files
//! guardy sync
//! ```
//!
//! ## Library Usage
//!
//! Guardy can also be used as a library for building custom security tools:
//!
//! ```rust,no_run
//! use guardy::scanner::Scanner;
//! use guardy::config::GuardyConfig;
//! use std::path::Path;
//!
//! // Load configuration
//! let config = GuardyConfig::load(None, None::<&()>, 0)?;
//! let scanner = Scanner::new(&config)?;
//!
//! // Scan files for secrets
//! let results = scanner.scan_directory(Path::new("src/"), None)?;
//!
//! // Process results
//! for finding in results.matches {
//!     println!("Found secret in {}: {}", finding.file_path, finding.secret_type);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!

//! ## Protected File Sync
//!
//! Keep configuration files synchronized across repositories:
//!
//! ```yaml
//! # guardy.yaml
//! sync:
//!   repos:
//!     - name: "shared-config"
//!       repo: "https://github.com/yourorg/shared-configs"
//!       version: "main"
//!       source_path: "."
//!       dest_path: "."
//!       include: ["*.yml", "*.json"]
//!       exclude: [".git"]
//! ```
//!
//! ```bash
//! # Show what has changed
//! guardy sync diff
//!
//! # Update files interactively
//! guardy sync
//!
//! # Force update all changes
//! guardy sync --force
//! ```
//!
//! ## Features
//!
//! - **Multi-threaded scanning** with configurable parallelism
//! - **Entropy-based secret detection** for high accuracy
//! - **Git integration** with hooks and remote operations  
//! - **File synchronization** with diff visualization
//! - **Multiple output formats** (JSON, HTML, plain text)
//! - **Comprehensive configuration** via YAML/TOML/JSON

pub mod cli;
pub mod config;
pub mod external;
pub mod git;
pub mod hooks;
pub mod parallel;
pub mod reports;
pub mod scanner;
pub mod shared;
pub mod sync;
