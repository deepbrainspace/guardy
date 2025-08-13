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
//! ## Git Hooks Integration
//!
//! Guardy provides flexible git hook management with both built-in actions and custom commands.
//! Hooks can be configured to run secret scanning, file synchronization, and custom commands.
//!
//! ### Hook Configuration Example
//!
//! ```yaml
//! hooks:
//!   pre-commit:
//!     enabled: true
//!     builtin: ["scan_secrets"]  # Built-in secret scanning
//!     custom:
//!       - command: "cargo fmt --check"
//!         description: "Check code formatting"
//!         fail_on_error: true
//!
//!   pre-push:
//!     enabled: true
//!     custom:
//!       - command: "guardy sync update --force --config ./guardy.yaml"
//!         description: "Sync protected files before push"
//!         fail_on_error: true
//! ```
//!
//! ## Repository Synchronization
//!
//! The sync feature allows you to keep files synchronized from upstream repositories.
//! This is particularly useful for maintaining consistent configurations across multiple projects.
//!
//! ### Automating Sync with Hooks
//!
//! You can integrate sync into your git workflow to ensure files stay synchronized:
//!
//! ```yaml
//! sync:
//!   repos:
//!     - name: "shared-configs"
//!       repo: "https://github.com/org/shared-configs"
//!       version: "v1.0.0"  # Can be tag, branch, or commit
//!       source_path: ".github"
//!       dest_path: "./.github"
//!       include: ["**/*"]
//!       exclude: ["*.md"]
//!
//! hooks:
//!   pre-push:
//!     enabled: true
//!     custom:
//!       - command: "guardy sync update --force --config ./guardy.yaml"
//!         description: "Ensure configs are synchronized"
//!         fail_on_error: true
//! ```
//!
//! This configuration ensures that protected files are restored to their canonical versions
//! before pushing changes, preventing drift from the upstream configuration.
//!
//! ## Library Usage
//!
//! Guardy can also be used as a library for building custom security tools:
//!
//! ```rust,no_run
//! use guardy::scan_v1::Scanner;
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
pub mod profiling;
pub mod reports;
pub mod scan;        // New optimized scanner (v3)
pub mod scan_v1;     // Legacy scanner (v1)
pub mod shared;
pub mod sync;
