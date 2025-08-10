//! Git hooks management module
//!
//! This module provides functionality for managing and executing git hooks with
//! both built-in actions and custom commands.
//!
//! ## Built-in Actions
//!
//! - `scan_secrets` - Scans staged files for secrets and sensitive data
//! - `validate_commit_msg` - Validates commit message format (placeholder)
//!
//! ## Custom Commands
//!
//! Hooks can execute any shell command with configurable error handling:
//!
//! ```yaml
//! hooks:
//!   pre-commit:
//!     enabled: true
//!     builtin: ["scan_secrets"]
//!     custom:
//!       - command: "cargo fmt --check"
//!         description: "Check formatting"
//!         fail_on_error: true
//! ```
//!
//! ## Integration with Sync
//!
//! Hooks can be used to automatically sync protected files:
//!
//! ```yaml
//! hooks:
//!   pre-push:
//!     enabled: true
//!     custom:
//!       - command: "guardy sync update --force --config ./guardy.yaml"
//!         description: "Sync protected files before push"
//!         fail_on_error: true
//! ```
//!
//! This ensures that protected configuration files are always synchronized
//! with their upstream sources before pushing changes.

mod config;
mod executor;

pub use executor::HookExecutor;
