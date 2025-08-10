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
