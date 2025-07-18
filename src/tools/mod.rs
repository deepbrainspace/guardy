//! Tool management and auto-installation
//!
//! This module handles detection, installation, and execution of development tools
//! like formatters and linters across different languages and package managers.

use crate::config::{FormatterConfig, InstallConfig, LinterConfig, ToolsConfig};
use crate::utils::package_manager::{PackageManager, PackageManagerDetector};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[cfg(test)]
mod tests;

/// Tool manager for handling formatters and linters
pub struct ToolManager {
    #[allow(dead_code)]
    config: ToolsConfig,
    auto_install: bool,
    pm_detector: PackageManagerDetector,
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
            pm_detector: PackageManagerDetector::new(),
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

    /// Detect project-specific package managers and tools
    pub fn detect_project_tools<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let path = path.as_ref();
        let mut detected_tools = Vec::new();

        // Detect package managers
        let pm_results = self.pm_detector.detect(path)?;
        for pm_info in pm_results {
            detected_tools.push(format!("{} ({})", pm_info.primary.display_name(), pm_info.reasoning));
        }

        // Detect JavaScript/TypeScript tools
        if path.join("package.json").exists() {
            if path.join(".prettierrc").exists() || path.join(".prettierrc.json").exists() {
                detected_tools.push("Prettier (config found)".to_string());
            }
            if path.join(".eslintrc.js").exists() || path.join(".eslintrc.json").exists() {
                detected_tools.push("ESLint (config found)".to_string());
            }
            if path.join("biome.json").exists() {
                detected_tools.push("Biome (config found)".to_string());
            }
        }

        // Detect Rust tools
        if path.join("Cargo.toml").exists() {
            if path.join("rustfmt.toml").exists() {
                detected_tools.push("rustfmt (config found)".to_string());
            } else if self.has_cargo() {
                detected_tools.push("rustfmt (available)".to_string());
            }
            if self.has_cargo() {
                detected_tools.push("Clippy (available)".to_string());
            }
        }

        // Detect Python tools
        if path.join("pyproject.toml").exists() {
            if let Ok(content) = std::fs::read_to_string(path.join("pyproject.toml")) {
                if content.contains("[tool.black]") {
                    detected_tools.push("Black (config found)".to_string());
                }
                if content.contains("[tool.ruff]") {
                    detected_tools.push("Ruff (config found)".to_string());
                }
            }
        }

        // Detect Go tools
        if path.join("go.mod").exists() {
            if self.is_tool_available("gofmt -help") {
                detected_tools.push("gofmt (available)".to_string());
            }
            if self.is_tool_available("golint -help") {
                detected_tools.push("golint (available)".to_string());
            }
        }

        // Detect generic tools
        if path.join(".editorconfig").exists() {
            detected_tools.push("EditorConfig (config found)".to_string());
        }

        Ok(detected_tools)
    }

    /// Get the best package manager for a project
    pub fn get_primary_package_manager<P: AsRef<Path>>(&self, path: P) -> Result<Option<PackageManager>> {
        self.pm_detector.get_primary(path)
    }

    /// Smart installation using detected package managers
    pub fn smart_install_tool(&self, tool_name: &str, path: &Path) -> Result<()> {
        // Try to use the project's primary package manager first
        if let Some(pm) = self.get_primary_package_manager(path)? {
            match pm {
                PackageManager::Npm => {
                    if self.has_npm() {
                        return self.try_npm_install(tool_name);
                    }
                }
                PackageManager::Pnpm => {
                    if self.is_tool_available("pnpm --version") {
                        return self.try_pnpm_install(tool_name);
                    }
                }
                PackageManager::Yarn => {
                    if self.is_tool_available("yarn --version") {
                        return self.try_yarn_install(tool_name);
                    }
                }
                PackageManager::Cargo => {
                    if self.has_cargo() {
                        return self.try_cargo_install(tool_name);
                    }
                }
                PackageManager::Pip => {
                    if self.is_tool_available("pip --version") {
                        return self.try_pip_install(tool_name);
                    }
                }
                PackageManager::Poetry => {
                    if self.is_tool_available("poetry --version") {
                        return self.try_poetry_install(tool_name);
                    }
                }
                _ => {
                    // Fall back to generic installation
                }
            }
        }

        // Fall back to generic installation methods
        self.try_generic_install(tool_name)
    }

    /// Try npm installation
    fn try_npm_install(&self, tool_name: &str) -> Result<()> {
        let cmd = format!("npm install -g {}", tool_name);
        self.run_install_command(&cmd, "npm")
    }

    /// Try pnpm installation
    fn try_pnpm_install(&self, tool_name: &str) -> Result<()> {
        let cmd = format!("pnpm add -g {}", tool_name);
        self.run_install_command(&cmd, "pnpm")
    }

    /// Try yarn installation
    fn try_yarn_install(&self, tool_name: &str) -> Result<()> {
        let cmd = format!("yarn global add {}", tool_name);
        self.run_install_command(&cmd, "yarn")
    }

    /// Try cargo installation
    fn try_cargo_install(&self, tool_name: &str) -> Result<()> {
        let cmd = format!("cargo install {}", tool_name);
        self.run_install_command(&cmd, "cargo")
    }

    /// Try pip installation
    fn try_pip_install(&self, tool_name: &str) -> Result<()> {
        let cmd = format!("pip install {}", tool_name);
        self.run_install_command(&cmd, "pip")
    }

    /// Try poetry installation
    fn try_poetry_install(&self, tool_name: &str) -> Result<()> {
        let cmd = format!("poetry add {}", tool_name);
        self.run_install_command(&cmd, "poetry")
    }

    /// Try generic installation methods
    fn try_generic_install(&self, tool_name: &str) -> Result<()> {
        // Try common package managers in order of preference
        if self.has_brew() {
            let cmd = format!("brew install {}", tool_name);
            if self.run_install_command(&cmd, "brew").is_ok() {
                return Ok(());
            }
        }

        if self.has_apt() {
            let cmd = format!("apt install -y {}", tool_name);
            if self.run_install_command(&cmd, "apt").is_ok() {
                return Ok(());
            }
        }

        anyhow::bail!("No compatible package manager found for {}", tool_name)
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
