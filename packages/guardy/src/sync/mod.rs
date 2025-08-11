<<<<<<< HEAD
=======
//! Repository synchronization module
//!
//! This module provides functionality for keeping files synchronized across repositories.
//! It allows you to maintain consistent configurations by syncing files from upstream sources.
//!
//! ## Features
//!
//! - Version pinning to specific tags, branches, or commits
//! - Selective sync with include/exclude patterns
//! - Automatic restoration of modified protected files
//! - Multi-repository configuration support
//!
//! ## Hook Integration
//!
//! Sync can be integrated into git hooks for automatic synchronization:
//!
//! ```yaml
//! sync:
//!   repos:
//!     - name: "shared-configs"
//!       repo: "https://github.com/org/shared-configs"
//!       version: "v1.0.0"
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
//!         description: "Sync protected files before push"
//!         fail_on_error: true
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Check sync status
//! guardy sync status
//!
//! # Interactive sync with diffs
//! guardy sync
//!
//! # Force sync all changes
//! guardy sync update --force
//! ```

>>>>>>> feat/benchmark
pub mod manager;
pub mod status;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SyncConfig {
    pub repos: Vec<SyncRepo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncRepo {
    pub name: String,
    pub repo: String,
    pub version: String,
    #[serde(default = "default_source_path")]
    pub source_path: String,
    #[serde(default = "default_dest_path")]
    pub dest_path: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug)]
pub enum SyncStatus {
    InSync,
    OutOfSync {
        changed_files: Vec<std::path::PathBuf>,
    },
    NotConfigured,
}

// Default values for serde
fn default_source_path() -> String {
    ".".to_string()
}

fn default_dest_path() -> String {
    ".".to_string()
}
