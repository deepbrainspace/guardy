//! Git hooks management module
//!
//! This module provides functionality for managing and executing git hooks with
//! both built-in actions and custom commands. It features intelligent parallel
//! execution with workload profiling and comprehensive file filtering capabilities.
//!
//! ## Built-in Actions
//!
//! - `scan_secrets` - Scans staged files for secrets and sensitive data
//! - `validate_commit_msg` - Validates commit messages using conventional commits format
//!
//! ## Custom Commands
//!
//! Hooks can execute any shell command with advanced configuration options:
//!
//! ```yaml
//! hooks:
//!   pre-commit:
//!     enabled: true
//!     parallel: true  # Default: true - intelligent parallel execution
//!     builtin: ["scan_secrets"]
//!     custom:
//!       - command: "cargo fmt --check"
//!         description: "Check formatting"
//!         fail_on_error: true
//!         glob: ["*.rs"]  # Only run on Rust files
//!       - command: "eslint {files} --fix"
//!         description: "Fix linting issues"
//!         all_files: true  # Run on all files, not just staged
//!         glob: ["*.js", "*.jsx"]
//!         stage_fixed: true  # Auto-stage fixed files
//! ```
//!
//! ## Advanced Features
//!
//! ### Intelligent Parallel Execution
//! - **Workload Profiling**: Automatically determines optimal concurrency based on:
//!   - Number of commands to execute
//!   - Available system resources (CPU cores)
//!   - Hook-specific characteristics (I/O vs CPU-bound)
//! - **Adaptive Scaling**: 
//!   - Small workloads (≤3 commands): Sequential execution
//!   - Medium workloads (4-5 commands): Conservative parallelism
//!   - Large workloads (6+ commands): Full parallelism (capped at 8)
//! - **System-Aware**: Respects CPU core count and user-configured limits
//!
//! ### File Processing
//! - **Glob Filtering**: Use `glob` patterns to target specific file types
//! - **All Files Mode**: Set `all_files: true` to process all matching files in repository
//! - **Stage Integration**: Use `stage_fixed: true` to automatically stage modified files
//! - **File Substitution**: Use `{files}` placeholder for command file arguments
//!
//! ### Conventional Commits Validation
//! - Full specification compliance using `git-conventional` library
//! - Helpful error messages with format examples
//! - Comment filtering and scope validation
//! - Supports all conventional commit types (feat, fix, docs, chore, etc.)
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
