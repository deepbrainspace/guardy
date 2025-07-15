//! Configuration management for Guardy
//!
//! This module handles loading, parsing, and validating Guardy configuration
//! from YAML files. It supports project-specific and global configuration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure for Guardy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardyConfig {
    /// Security configuration
    pub security: SecurityConfig,

    /// Git hooks configuration
    pub hooks: HooksConfig,

    /// MCP server configuration
    pub mcp: McpConfig,

    /// Tool integration settings
    pub tools: ToolsConfig,
}

/// Security-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable secret detection
    pub secret_detection: bool,

    /// Custom security patterns
    pub patterns: Vec<SecurityPatternConfig>,

    /// Additional file patterns to exclude from scanning (beyond gitignore)
    pub exclude_patterns: Vec<String>,

    /// Whether to automatically exclude patterns from .gitignore files
    #[serde(default = "default_use_gitignore")]
    pub use_gitignore: bool,

    /// Protected branches
    pub protected_branches: Vec<String>,

    /// Enable git-crypt integration
    pub git_crypt: bool,
}

/// Default value for use_gitignore
fn default_use_gitignore() -> bool {
    true
}

/// Security pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPatternConfig {
    /// Pattern name
    pub name: String,

    /// Regex pattern
    pub regex: String,

    /// Severity level (Critical or Info)
    #[serde(default = "default_severity")]
    pub severity: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Whether this pattern is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Default severity level for patterns
fn default_severity() -> String {
    "Critical".to_string()
}

/// Default enabled state for patterns
fn default_enabled() -> bool {
    true
}

/// Git hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Enable pre-commit hook
    pub pre_commit: bool,

    /// Enable commit-msg hook
    pub commit_msg: bool,

    /// Enable pre-push hook
    pub pre_push: bool,

    /// Timeout for hook execution (seconds)
    pub timeout: u64,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable MCP server
    pub enabled: bool,

    /// Server port
    pub port: u16,

    /// Server host
    pub host: String,

    /// Enable daemon mode
    pub daemon: bool,
}

/// Tool integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Auto-detect project type
    pub auto_detect: bool,

    /// Auto-install missing tools (can be overridden by CLI flag)
    #[serde(default)]
    pub auto_install: bool,

    /// Formatter configurations
    pub formatters: Vec<FormatterConfig>,

    /// Linter configurations
    pub linters: Vec<LinterConfig>,
}

/// Formatter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    /// Tool name (e.g., "prettier", "rustfmt")
    pub name: String,

    /// Command to run
    pub command: String,

    /// File patterns to match
    pub patterns: Vec<String>,

    /// Command to check if tool is installed
    #[serde(default)]
    pub check_command: Option<String>,

    /// Installation commands for different package managers
    #[serde(default)]
    pub install: Option<InstallConfig>,
}

/// Linter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterConfig {
    /// Tool name (e.g., "eslint", "clippy")
    pub name: String,

    /// Command to run
    pub command: String,

    /// File patterns to match
    pub patterns: Vec<String>,

    /// Command to check if tool is installed
    #[serde(default)]
    pub check_command: Option<String>,

    /// Installation commands for different package managers
    #[serde(default)]
    pub install: Option<InstallConfig>,
}

/// Installation configuration for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Cargo command (for Rust tools)
    pub cargo: Option<String>,

    /// NPM command (for Node.js tools)
    pub npm: Option<String>,

    /// Homebrew command (for macOS)
    pub brew: Option<String>,

    /// APT command (for Debian/Ubuntu)
    pub apt: Option<String>,

    /// Manual installation instructions
    pub manual: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            secret_detection: true,
            patterns: vec![],
            exclude_patterns: vec![
                // Only essential patterns that aren't typically in gitignore
                "*.tmp".to_string(),
                "*.temp".to_string(),
            ],
            use_gitignore: true,
            protected_branches: vec![
                "main".to_string(),
                "master".to_string(),
                "develop".to_string(),
            ],
            git_crypt: false,
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            pre_commit: true,
            commit_msg: true,
            pre_push: true,
            timeout: 300,
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            host: "localhost".to_string(),
            daemon: false,
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            auto_install: false, // Conservative default - require explicit opt-in
            formatters: vec![],
            linters: vec![],
        }
    }
}

#[allow(dead_code)]
impl GuardyConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: GuardyConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Get effective exclude patterns (combining config patterns with gitignore if enabled)
    pub fn get_effective_exclude_patterns(&self) -> Vec<String> {
        let mut patterns = self.security.exclude_patterns.clone();

        // Always load .guardyignore patterns (Guardy-specific exclusions)
        if let Ok(guardyignore_patterns) = Self::load_guardyignore_patterns() {
            patterns.extend(guardyignore_patterns);
        }

        if self.security.use_gitignore {
            if let Ok(gitignore_patterns) = Self::load_gitignore_patterns() {
                patterns.extend(gitignore_patterns);
            }
        }

        patterns
    }

    /// Load patterns from .gitignore files
    fn load_gitignore_patterns() -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        // Start from current directory and walk up to find .gitignore files
        let mut current_dir = std::env::current_dir()?;

        loop {
            let gitignore_path = current_dir.join(".gitignore");

            if gitignore_path.exists() {
                let content = std::fs::read_to_string(&gitignore_path).with_context(|| {
                    format!("Failed to read .gitignore: {}", gitignore_path.display())
                })?;

                for line in content.lines() {
                    let line = line.trim();

                    // Skip empty lines and comments
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    patterns.push(line.to_string());
                }
            }

            // Move to parent directory
            if !current_dir.pop() {
                break;
            }
        }

        Ok(patterns)
    }

    /// Load patterns from .guardyignore files
    fn load_guardyignore_patterns() -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        // Start from current directory and walk up to find .guardyignore files
        let mut current_dir = std::env::current_dir()?;

        loop {
            let guardyignore_path = current_dir.join(".guardyignore");

            if guardyignore_path.exists() {
                let content = std::fs::read_to_string(&guardyignore_path).with_context(|| {
                    format!(
                        "Failed to read .guardyignore: {}",
                        guardyignore_path.display()
                    )
                })?;

                for line in content.lines() {
                    let line = line.trim();

                    // Skip empty lines and comments
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    patterns.push(line.to_string());
                }
            }

            // Move to parent directory
            if !current_dir.pop() {
                break;
            }
        }

        Ok(patterns)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(self).context("Failed to serialize configuration")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Find configuration file in current directory or parent directories
    pub fn find_config_file() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            let config_path = current.join("guardy.yml");
            if config_path.exists() {
                return Some(config_path);
            }

            let config_path = current.join(".guardy.yml");
            if config_path.exists() {
                return Some(config_path);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Load configuration from found file or use defaults
    pub fn load_or_default() -> Self {
        if let Some(config_path) = Self::find_config_file() {
            Self::load_from_file(&config_path).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate MCP configuration
        if self.mcp.enabled {
            if self.mcp.port == 0 {
                anyhow::bail!("MCP server port cannot be 0");
            }
            if self.mcp.host.is_empty() {
                anyhow::bail!("MCP server host cannot be empty");
            }
        }

        // Validate hooks configuration
        if self.hooks.timeout == 0 {
            anyhow::bail!("Hooks timeout cannot be 0");
        }

        // Validate security configuration
        if self.security.protected_branches.is_empty() {
            anyhow::bail!("At least one protected branch must be specified");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
