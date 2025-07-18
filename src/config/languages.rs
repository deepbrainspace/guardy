//! Language configurations and detection
//!
//! This module contains hardcoded language configurations that define
//! default package managers, formatters, and linters for each supported language.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Language configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub name: String,
    pub description: String,
    pub file_patterns: Vec<String>,
    pub package_managers: Vec<String>, // In order of preference
    pub formatters: Vec<ToolConfig>,
    pub linters: Vec<ToolConfig>,
}

/// Tool configuration (formatter or linter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub command: String,
    pub patterns: Vec<String>,
    pub check_command: String,
    pub install_commands: InstallCommands,
}

/// Installation commands for different package managers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallCommands {
    pub npm: Option<String>,
    pub pnpm: Option<String>,
    pub yarn: Option<String>,
    pub cargo: Option<String>,
    pub pip: Option<String>,
    pub brew: Option<String>,
    pub manual: String,
}

/// Get all hardcoded language configurations
pub fn get_language_configs() -> HashMap<String, LanguageConfig> {
    let mut configs = HashMap::new();
    
    // JavaScript/TypeScript
    configs.insert("javascript".to_string(), LanguageConfig {
        name: "JavaScript/TypeScript".to_string(),
        description: "Modern JavaScript/TypeScript projects".to_string(),
        file_patterns: vec![
            "**/*.js".to_string(),
            "**/*.ts".to_string(),
            "**/*.jsx".to_string(),
            "**/*.tsx".to_string(),
            "package.json".to_string(),
            "tsconfig.json".to_string(),
        ],
        package_managers: vec!["pnpm".to_string(), "bun".to_string(), "yarn".to_string(), "npm".to_string()],
        formatters: vec![
            ToolConfig {
                name: "prettier".to_string(),
                command: "npx prettier --write".to_string(),
                patterns: vec!["**/*.js".to_string(), "**/*.ts".to_string(), "**/*.jsx".to_string(), "**/*.tsx".to_string()],
                check_command: "npx prettier --version".to_string(),
                install_commands: InstallCommands {
                    npm: Some("npm install -g prettier".to_string()),
                    pnpm: Some("pnpm add -g prettier".to_string()),
                    yarn: Some("yarn global add prettier".to_string()),
                    cargo: None,
                    pip: None,
                    brew: Some("brew install prettier".to_string()),
                    manual: "Install Node.js then run: pnpm add -g prettier".to_string(),
                },
            },
            ToolConfig {
                name: "biome".to_string(),
                command: "biome format --write".to_string(),
                patterns: vec!["**/*.js".to_string(), "**/*.ts".to_string(), "**/*.jsx".to_string(), "**/*.tsx".to_string()],
                check_command: "biome --version".to_string(),
                install_commands: InstallCommands {
                    npm: Some("npm install -g @biomejs/biome".to_string()),
                    pnpm: Some("pnpm add -g @biomejs/biome".to_string()),
                    yarn: Some("yarn global add @biomejs/biome".to_string()),
                    cargo: Some("cargo install biome".to_string()),
                    pip: None,
                    brew: None,
                    manual: "Install with: pnpm add -g @biomejs/biome".to_string(),
                },
            },
        ],
        linters: vec![
            ToolConfig {
                name: "eslint".to_string(),
                command: "npx eslint".to_string(),
                patterns: vec!["**/*.js".to_string(), "**/*.ts".to_string(), "**/*.jsx".to_string(), "**/*.tsx".to_string()],
                check_command: "npx eslint --version".to_string(),
                install_commands: InstallCommands {
                    npm: Some("npm install -g eslint".to_string()),
                    pnpm: Some("pnpm add -g eslint".to_string()),
                    yarn: Some("yarn global add eslint".to_string()),
                    cargo: None,
                    pip: None,
                    brew: None,
                    manual: "Install Node.js then run: pnpm add -g eslint".to_string(),
                },
            },
            ToolConfig {
                name: "typescript".to_string(),
                command: "npx tsc --noEmit".to_string(),
                patterns: vec!["**/*.ts".to_string(), "**/*.tsx".to_string()],
                check_command: "npx tsc --version".to_string(),
                install_commands: InstallCommands {
                    npm: Some("npm install -g typescript".to_string()),
                    pnpm: Some("pnpm add -g typescript".to_string()),
                    yarn: Some("yarn global add typescript".to_string()),
                    cargo: None,
                    pip: None,
                    brew: None,
                    manual: "Install Node.js then run: pnpm add -g typescript".to_string(),
                },
            },
        ],
    });

    // Rust
    configs.insert("rust".to_string(), LanguageConfig {
        name: "Rust".to_string(),
        description: "Rust programming language".to_string(),
        file_patterns: vec![
            "**/*.rs".to_string(),
            "Cargo.toml".to_string(),
            "Cargo.lock".to_string(),
        ],
        package_managers: vec!["cargo".to_string()],
        formatters: vec![
            ToolConfig {
                name: "prettyplease".to_string(),
                command: "prettyplease".to_string(),
                patterns: vec!["**/*.rs".to_string()],
                check_command: "prettyplease --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: Some("cargo install prettyplease".to_string()),
                    pip: None,
                    brew: None,
                    manual: "Install Rust then run: cargo install prettyplease".to_string(),
                },
            },
            ToolConfig {
                name: "rustfmt".to_string(),
                command: "cargo fmt".to_string(),
                patterns: vec!["**/*.rs".to_string()],
                check_command: "rustfmt --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: Some("rustup component add rustfmt".to_string()),
                    pip: None,
                    brew: None,
                    manual: "Install Rust then run: rustup component add rustfmt".to_string(),
                },
            },
        ],
        linters: vec![
            ToolConfig {
                name: "clippy".to_string(),
                command: "cargo clippy".to_string(),
                patterns: vec!["**/*.rs".to_string()],
                check_command: "cargo clippy --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: Some("rustup component add clippy".to_string()),
                    pip: None,
                    brew: None,
                    manual: "Install Rust then run: rustup component add clippy".to_string(),
                },
            },
        ],
    });

    // Python
    configs.insert("python".to_string(), LanguageConfig {
        name: "Python".to_string(),
        description: "Python programming language".to_string(),
        file_patterns: vec![
            "**/*.py".to_string(),
            "requirements.txt".to_string(),
            "pyproject.toml".to_string(),
            "setup.py".to_string(),
        ],
        package_managers: vec!["uv".to_string(), "poetry".to_string(), "pip".to_string()],
        formatters: vec![
            ToolConfig {
                name: "black".to_string(),
                command: "black".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: "black --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: None,
                    pip: Some("pip install black".to_string()),
                    brew: Some("brew install black".to_string()),
                    manual: "Install Python then run: pip install black".to_string(),
                },
            },
            ToolConfig {
                name: "ruff".to_string(),
                command: "ruff format".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: "ruff --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: Some("cargo install ruff".to_string()),
                    pip: Some("pip install ruff".to_string()),
                    brew: Some("brew install ruff".to_string()),
                    manual: "Install Python then run: pip install ruff".to_string(),
                },
            },
        ],
        linters: vec![
            ToolConfig {
                name: "ruff".to_string(),
                command: "ruff check".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: "ruff --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: Some("cargo install ruff".to_string()),
                    pip: Some("pip install ruff".to_string()),
                    brew: Some("brew install ruff".to_string()),
                    manual: "Install Python then run: pip install ruff".to_string(),
                },
            },
            ToolConfig {
                name: "mypy".to_string(),
                command: "mypy".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: "mypy --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: None,
                    pip: Some("pip install mypy".to_string()),
                    brew: Some("brew install mypy".to_string()),
                    manual: "Install Python then run: pip install mypy".to_string(),
                },
            },
        ],
    });

    // Go
    configs.insert("go".to_string(), LanguageConfig {
        name: "Go".to_string(),
        description: "Go programming language".to_string(),
        file_patterns: vec![
            "**/*.go".to_string(),
            "go.mod".to_string(),
            "go.sum".to_string(),
        ],
        package_managers: vec!["go".to_string()],
        formatters: vec![
            ToolConfig {
                name: "gofmt".to_string(),
                command: "gofmt -w".to_string(),
                patterns: vec!["**/*.go".to_string()],
                check_command: "gofmt -h".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: None,
                    pip: None,
                    brew: Some("brew install go".to_string()),
                    manual: "Install Go from https://golang.org/dl/".to_string(),
                },
            },
            ToolConfig {
                name: "goimports".to_string(),
                command: "goimports -w".to_string(),
                patterns: vec!["**/*.go".to_string()],
                check_command: "goimports -h".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: None,
                    pip: None,
                    brew: None,
                    manual: "Install Go then run: go install golang.org/x/tools/cmd/goimports@latest".to_string(),
                },
            },
        ],
        linters: vec![
            ToolConfig {
                name: "golangci-lint".to_string(),
                command: "golangci-lint run".to_string(),
                patterns: vec!["**/*.go".to_string()],
                check_command: "golangci-lint --version".to_string(),
                install_commands: InstallCommands {
                    npm: None,
                    pnpm: None,
                    yarn: None,
                    cargo: None,
                    pip: None,
                    brew: Some("brew install golangci-lint".to_string()),
                    manual: "Install from https://golangci-lint.run/usage/install/".to_string(),
                },
            },
        ],
    });

    configs
}

/// Detect languages with file counts based on file patterns in the given directory
pub fn detect_languages<P: AsRef<std::path::Path>>(path: P) -> Vec<(String, usize)> {
    let path = path.as_ref();
    let configs = get_language_configs();
    let mut detected = Vec::new();

    for (lang_name, config) in configs {
        let mut total_files = 0;
        
        for pattern in &config.file_patterns {
            // Use proper glob pattern matching
            if crate::utils::glob::is_glob_pattern(pattern) {
                // For glob patterns, count matching files
                if let Ok(matches) = crate::utils::glob::expand_glob_pattern(pattern, path) {
                    total_files += matches.len();
                }
            } else {
                // For literal file patterns, check if file exists
                let file_path = path.join(pattern);
                if file_path.exists() {
                    total_files += 1;
                }
            }
        }
        
        if total_files > 0 {
            detected.push((lang_name, total_files));
        }
    }

    detected
}