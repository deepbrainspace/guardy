//! Tool management and auto-installation
//!
//! This module handles detection, installation, and execution of development tools
//! like formatters and linters across different languages and package managers.

use crate::config::{FormatterConfig, InstallConfig, LinterConfig, ToolsConfig};
use anyhow::{Context, Result};
use std::process::Command;

#[cfg(test)]
mod tests;

/// Tool manager for handling formatters and linters
pub struct ToolManager {
    config: ToolsConfig,
    auto_install: bool,
}

/// Status of a tool installation
#[derive(Debug, PartialEq)]
pub enum ToolStatus {
    Available,
    Missing,
    InstallationFailed,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new(config: ToolsConfig, auto_install: bool) -> Self {
        Self {
            config,
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

    /// Check if a linter is available, installing if needed
    pub fn ensure_linter_available(&self, linter: &LinterConfig) -> Result<()> {
        self.ensure_tool_available(
            &linter.name,
            linter.check_command.as_deref(),
            linter.install.as_ref(),
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
            Err(anyhow::anyhow!("No compatible package manager found"))
        };

        match install_result {
            Ok(_) => {
                println!("✅ Successfully installed {}", tool_name);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to auto-install {}: {}", tool_name, e);
                eprintln!("📖 Manual installation: {}", install_config.manual);
                anyhow::bail!("Auto-installation failed. Please install manually.");
            }
        }
    }

    /// Run an installation command using a specific package manager
    fn run_install_command(&self, command: &str, package_manager: &str) -> Result<()> {
        println!("   Using {} to install...", package_manager);

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .with_context(|| format!("Failed to execute install command: {}", command))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Installation command failed: {}", stderr);
        }

        Ok(())
    }

    /// Fail gracefully with installation instructions
    fn fail_with_install_instructions(
        &self,
        tool_name: &str,
        install_config: Option<&InstallConfig>,
    ) -> Result<()> {
        eprintln!("❌ Tool '{}' is not installed", tool_name);

        if let Some(config) = install_config {
            eprintln!("📦 To install automatically, use: --auto-install flag");
            eprintln!("📖 Or install manually:");

            if let Some(cargo_cmd) = &config.cargo {
                eprintln!("   Cargo:  {}", cargo_cmd);
            }
            if let Some(npm_cmd) = &config.npm {
                eprintln!("   NPM:    {}", npm_cmd);
            }
            if let Some(brew_cmd) = &config.brew {
                eprintln!("   Brew:   {}", brew_cmd);
            }
            if let Some(apt_cmd) = &config.apt {
                eprintln!("   APT:    {}", apt_cmd);
            }
            eprintln!("   Manual: {}", config.manual);
        } else {
            eprintln!(
                "📖 Please install {} manually (no installation instructions available)",
                tool_name
            );
        }

        anyhow::bail!("Required tool not available")
    }

    /// Fail when no installation information is available
    fn fail_with_no_install_info(&self, tool_name: &str) -> Result<()> {
        eprintln!("❌ Tool '{}' is not installed", tool_name);
        eprintln!("📖 No installation instructions available for this tool");
        eprintln!("📦 Please install {} manually", tool_name);
        anyhow::bail!("Required tool not available")
    }

    /// Check if cargo is available
    fn has_cargo(&self) -> bool {
        self.is_tool_available("cargo --version")
    }

    /// Check if npm is available
    fn has_npm(&self) -> bool {
        self.is_tool_available("npm --version")
    }

    /// Check if homebrew is available
    fn has_brew(&self) -> bool {
        self.is_tool_available("brew --version")
    }

    /// Check if apt is available
    fn has_apt(&self) -> bool {
        self.is_tool_available("apt --version")
    }
}

/// Example configuration showing trusted installation sources
pub fn create_example_tools_config() -> ToolsConfig {
    use crate::config::{FormatterConfig, InstallConfig, LinterConfig};

    ToolsConfig {
        auto_detect: true,
        auto_install: false, // Require explicit opt-in
        formatters: vec![
            FormatterConfig {
                name: "rustfmt".to_string(),
                command: "cargo fmt".to_string(),
                patterns: vec!["**/*.rs".to_string()],
                check_command: Some("rustfmt --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("rustup component add rustfmt".to_string()),
                    npm: None,
                    brew: None,
                    apt: None,
                    manual: "Install Rust toolchain: https://rustup.rs/".to_string(),
                }),
            },
            FormatterConfig {
                name: "prettier".to_string(),
                command: "npx prettier --write".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.json".to_string(),
                ],
                check_command: Some("npx prettier --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g prettier".to_string()),
                    brew: Some("brew install prettier".to_string()),
                    apt: None,
                    manual: "Install Node.js then run: npm install -g prettier".to_string(),
                }),
            },
        ],
        linters: vec![
            LinterConfig {
                name: "clippy".to_string(),
                command: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
                patterns: vec!["**/*.rs".to_string()],
                check_command: Some("cargo clippy --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("rustup component add clippy".to_string()),
                    npm: None,
                    brew: None,
                    apt: None,
                    manual: "Install Rust toolchain: https://rustup.rs/".to_string(),
                }),
            },
            LinterConfig {
                name: "eslint".to_string(),
                command: "npx eslint".to_string(),
                patterns: vec!["**/*.js".to_string(), "**/*.ts".to_string()],
                check_command: Some("npx eslint --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g eslint".to_string()),
                    brew: None,
                    apt: None,
                    manual: "Install Node.js then run: npm install -g eslint".to_string(),
                }),
            },
        ],
    }
}
