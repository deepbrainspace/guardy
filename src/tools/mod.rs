//! Tool management and auto-installation
//!
//! This module handles detection, installation, and execution of development tools
//! like formatters and linters across different languages and package managers.

use crate::config::{FormatterConfig, InstallConfig, ToolsConfig};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[cfg(test)]
mod tests;

/// Tool manager for handling formatters and linters
pub struct ToolManager {
    auto_install: bool,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new(_config: ToolsConfig, auto_install: bool) -> Self {
        Self {
            auto_install,
        }
    }

    /// Check if a formatter is available, installing if needed
    pub fn ensure_formatter_available(&self, formatter: &FormatterConfig) -> Result<()> {
        self.ensure_tool_available(
            &formatter.name,
            formatter.check_command.as_deref(),
            formatter.install.as_ref(),
        )
    }

    /// Generic tool availability check with auto-install support
    fn ensure_tool_available(
        &self,
        tool_name: &str,
        check_command: Option<&str>,
        install_config: Option<&InstallConfig>,
    ) -> Result<()> {
        // If no check command provided, assume tool is available
        let check_cmd = match check_command {
            Some(cmd) => cmd,
            None => return Ok(()),
        };

        // Check if tool is already available
        if self.is_tool_available(check_cmd) {
            return Ok(());
        }

        // Tool is missing - handle based on auto_install flag
        if !self.auto_install {
            return self.fail_with_install_instructions(tool_name, install_config);
        }

        // Attempt auto-installation
        match install_config {
            Some(config) => self.auto_install_tool(tool_name, config),
            None => self.fail_with_no_install_info(tool_name),
        }
    }

    /// Check if a tool is available by running its check command
    fn is_tool_available(&self, check_command: &str) -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(check_command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Attempt to auto-install a tool using available package managers
    fn auto_install_tool(&self, tool_name: &str, install_config: &InstallConfig) -> Result<()> {
        println!("🔄 Auto-installing {}...", tool_name);

        // Try installation methods in order of preference
        let install_result = if let Some(cargo_cmd) = &install_config.cargo {
            if self.has_cargo() {
                self.run_install_command(cargo_cmd, "cargo")
            } else {
                Err(anyhow::anyhow!("Cargo not available"))
            }
        } else if let Some(npm_cmd) = &install_config.npm {
            if self.has_npm() {
                self.run_install_command(npm_cmd, "npm")
            } else {
                Err(anyhow::anyhow!("NPM not available"))
            }
        } else if let Some(brew_cmd) = &install_config.brew {
            if self.has_brew() {
                self.run_install_command(brew_cmd, "brew")
            } else {
                Err(anyhow::anyhow!("Homebrew not available"))
            }
        } else if let Some(apt_cmd) = &install_config.apt {
            if self.has_apt() {
                self.run_install_command(apt_cmd, "apt")
            } else {
                Err(anyhow::anyhow!("APT not available"))
            }
        } else {
            Err(anyhow::anyhow!("No supported package manager found"))
        };

        match install_result {
            Ok(()) => {
                println!("✅ Successfully installed {}", tool_name);
                Ok(())
            }
            Err(e) => {
                println!("❌ Failed to install {}: {}", tool_name, e);
                self.fail_with_install_instructions(tool_name, Some(install_config))
            }
        }
    }

    /// Run an installation command
    fn run_install_command(&self, command: &str, package_manager: &str) -> Result<()> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .with_context(|| format!("Failed to execute {} command", package_manager))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "{} installation failed: {}",
                package_manager,
                stderr.trim()
            );
        }
    }

    /// Fail with installation instructions
    fn fail_with_install_instructions(
        &self,
        tool_name: &str,
        install_config: Option<&InstallConfig>,
    ) -> Result<()> {
        let mut message = format!("❌ Tool '{}' is not available. Install it using:", tool_name);

        if let Some(config) = install_config {
            if let Some(cargo_cmd) = &config.cargo {
                message.push_str(&format!("\n  cargo: {}", cargo_cmd));
            }
            if let Some(npm_cmd) = &config.npm {
                message.push_str(&format!("\n  npm: {}", npm_cmd));
            }
            if let Some(brew_cmd) = &config.brew {
                message.push_str(&format!("\n  brew: {}", brew_cmd));
            }
            if let Some(apt_cmd) = &config.apt {
                message.push_str(&format!("\n  apt: {}", apt_cmd));
            }
            message.push_str(&format!("\n  manual: {}", config.manual));
        }

        message.push_str("\n\nOr set --auto-install to automatically install missing tools.");

        anyhow::bail!(message);
    }

    /// Fail with no installation info
    fn fail_with_no_install_info(&self, tool_name: &str) -> Result<()> {
        anyhow::bail!(
            "❌ Tool '{}' is not available and no installation instructions are configured.",
            tool_name
        );
    }

    /// Check if cargo is available
    fn has_cargo(&self) -> bool {
        self.is_tool_available("cargo --version")
    }

    /// Check if npm is available
    fn has_npm(&self) -> bool {
        self.is_tool_available("npm --version")
    }

    /// Check if brew is available
    fn has_brew(&self) -> bool {
        self.is_tool_available("brew --version")
    }

    /// Check if apt is available
    fn has_apt(&self) -> bool {
        self.is_tool_available("apt --version")
    }

    /// Detect development tools in the current project
    pub fn detect_tools<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let path = path.as_ref();
        let mut detected_tools = Vec::new();

        // Check for common configuration files
        if path.join("rustfmt.toml").exists() || path.join(".rustfmt.toml").exists() {
            detected_tools.push("rustfmt (config found)".to_string());
        }

        if path.join(".prettierrc").exists()
            || path.join(".prettierrc.json").exists()
            || path.join(".prettierrc.js").exists()
            || path.join("prettier.config.js").exists()
        {
            detected_tools.push("prettier (config found)".to_string());
        }

        if path.join(".eslintrc").exists()
            || path.join(".eslintrc.json").exists()
            || path.join(".eslintrc.js").exists()
            || path.join("eslint.config.js").exists()
        {
            detected_tools.push("eslint (config found)".to_string());
        }

        if path.join("clippy.toml").exists() || path.join(".clippy.toml").exists() {
            detected_tools.push("clippy (config found)".to_string());
        }

        if path.join(".editorconfig").exists() {
            detected_tools.push("EditorConfig (config found)".to_string());
        }

        Ok(detected_tools)
    }
}