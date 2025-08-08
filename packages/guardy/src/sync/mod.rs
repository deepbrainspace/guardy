pub mod manager;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    pub repos: Vec<SyncRepo>,
    pub protection: ProtectionConfig,
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
    #[serde(default = "default_protected")]
    pub protected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtectionConfig {
    #[serde(default = "default_auto_protect")]
    pub auto_protect_synced: bool,
    #[serde(default = "default_block_modifications")]
    pub block_modifications: bool,
}

#[derive(Debug)]
pub enum SyncStatus {
    InSync,
    OutOfSync { changed_files: Vec<PathBuf> },
    NotConfigured,
}

// Default values for serde
fn default_source_path() -> String {
    ".".to_string()
}

fn default_dest_path() -> String {
    ".".to_string()
}

fn default_protected() -> bool {
    true
}

fn default_auto_protect() -> bool {
    true
}

fn default_block_modifications() -> bool {
    true
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            auto_protect_synced: default_auto_protect(),
            block_modifications: default_block_modifications(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            protection: ProtectionConfig::default(),
        }
    }
}