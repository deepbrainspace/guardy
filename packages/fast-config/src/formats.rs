//! Multi-format configuration parsing (JSON, YAML)

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::path::Path;

/// Supported configuration formats
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Json,
    Yaml,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Result<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Ok(ConfigFormat::Json),
            Some("yaml") | Some("yml") => Ok(ConfigFormat::Yaml),
            Some(ext) => Err(anyhow::anyhow!(
                "Unsupported config file extension: {} (supported: .json, .yaml, .yml)",
                ext
            )),
            None => {
                // Default to YAML for extensionless files
                Ok(ConfigFormat::Yaml)
            }
        }
    }

    /// Parse config file in this format
    pub fn parse<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        match self {
            ConfigFormat::Json => {
                tracing::debug!("Parsing JSON config: {} (~10ms)", path.display());
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON config: {}", path.display()))
            }
            ConfigFormat::Yaml => {
                tracing::debug!("Parsing YAML config: {} (~30ms)", path.display());
                #[cfg(feature = "yaml")]
                {
                    serde_yaml_bw::from_str(&content)
                        .with_context(|| format!("Failed to parse YAML config: {}", path.display()))
                }
                #[cfg(not(feature = "yaml"))]
                {
                    Err(anyhow::anyhow!(
                        "YAML support not enabled. Enable the 'yaml' feature."
                    ))
                }
            }
        }
    }

    /// Get expected performance for this format
    pub fn expected_parse_time_ms(&self) -> u32 {
        match self {
            ConfigFormat::Json => 10,
            ConfigFormat::Yaml => 30,
        }
    }
}
