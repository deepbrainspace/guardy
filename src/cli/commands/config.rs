//! Configuration command implementations
//!
//! Commands for managing Guardy configuration.

use crate::cli::ConfigCommands;
use crate::cli::Output;
use crate::config::{GuardyConfig, ToolsConfig, FormatterConfig, LinterConfig, InstallConfig};
use crate::utils::{get_current_dir, detect_project_type, ProjectType};
use anyhow::Result;
use std::fs;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::util::as_24_bit_terminal_escaped;

/// Execute config commands
pub async fn execute(cmd: ConfigCommands, output: &Output) -> Result<()> {
    match cmd {
        ConfigCommands::Init => init(output).await,
        ConfigCommands::Validate => validate(output).await,
        ConfigCommands::Show => show(output).await,
    }
}

async fn init(output: &Output) -> Result<()> {
    output.header("🔧 Initializing Configuration");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    // Check if config already exists
    if config_path.exists() {
        output.warning("Configuration file already exists");
        if !output.confirm("Do you want to overwrite it?") {
            output.info("Configuration initialization cancelled");
            return Ok(());
        }
    }
    
    // Detect project type
    let project_type = detect_project_type(&current_dir);
    output.info(&format!("Detected project type: {:?}", project_type));
    
    // Create project-specific configuration
    let mut config = GuardyConfig::default();
    
    // Add project-specific formatters and linters
    config.tools = create_project_specific_tools(&project_type);
    
    // Save configuration
    config.save_to_file(&config_path)?;
    
    output.success("Configuration file created successfully");
    output.table_row("Config file", &config_path.display().to_string());
    output.info("Edit guardy.yml to customize your settings");
    
    Ok(())
}

async fn validate(output: &Output) -> Result<()> {
    output.header("✅ Validating Configuration");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    if !config_path.exists() {
        output.error("Configuration file not found");
        output.indent("Run 'guardy config init' to create a configuration file");
        return Ok(());
    }
    
    match GuardyConfig::load_from_file(&config_path) {
        Ok(config) => {
            output.success("Configuration is valid");
            output.blank_line();
            
            // Show configuration summary
            output.step("Configuration Summary");
            output.table_row("Security patterns", &config.security.patterns.len().to_string());
            output.table_row("Tool integrations", &(config.tools.formatters.len() + config.tools.linters.len()).to_string());
            output.table_row("MCP server enabled", &config.mcp.enabled.to_string());
            
            if config.mcp.enabled {
                output.table_row("MCP server port", &config.mcp.port.to_string());
            }
            
            // Validate security patterns
            output.blank_line();
            output.step("Security Patterns");
            for pattern in &config.security.patterns {
                if pattern.enabled {
                    output.success(&format!("✓ {} ({})", pattern.name, pattern.severity));
                } else {
                    output.info(&format!("○ {} (disabled)", pattern.name));
                }
            }
            
            // Validate tool configurations
            output.blank_line();
            output.step("Tool Integrations");
            
            // Show formatters
            for formatter in &config.tools.formatters {
                output.success(&format!("✓ {} (formatter)", formatter.name));
            }
            
            // Show linters
            for linter in &config.tools.linters {
                output.success(&format!("✓ {} (linter)", linter.name));
            }
            
            if config.tools.formatters.is_empty() && config.tools.linters.is_empty() {
                output.info("○ No tools configured");
            }
        }
        Err(err) => {
            output.error("Configuration file is invalid");
            output.indent(&format!("Error: {}", err));
        }
    }
    
    Ok(())
}

async fn show(output: &Output) -> Result<()> {
    output.header("📄 Current Configuration");
    
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join("guardy.yml");
    
    if !config_path.exists() {
        output.error("Configuration file not found");
        output.indent("Run 'guardy config init' to create a configuration file");
        return Ok(());
    }
    
    // Read and display the raw configuration file
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            output.success("Configuration file content:");
            output.blank_line();
            output.separator();
            
            // Add YAML syntax highlighting if possible
            if let Some(highlighted) = highlight_yaml(&content) {
                print!("{}", highlighted);
            } else {
                // Fallback to plain text
                println!("{}", content);
            }
            
            output.separator();
            output.blank_line();
            output.table_row("Config file", &config_path.display().to_string());
        }
        Err(err) => {
            output.error("Failed to read configuration file");
            output.indent(&format!("Error: {}", err));
        }
    }
    
    Ok(())
}

/// Highlight YAML content using syntect
fn highlight_yaml(content: &str) -> Option<String> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    
    let syntax = syntax_set.find_syntax_by_extension("yml")?;
    let theme = &theme_set.themes["base16-eighties.dark"];
    
    let mut highlighted = String::new();
    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    
    for line in content.lines() {
        let ranges = highlighter.highlight_line(line, &syntax_set).ok()?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        highlighted.push_str(&escaped);
        highlighted.push('\n');
    }
    
    Some(highlighted)
}

/// Create project-specific tools configuration
fn create_project_specific_tools(project_type: &ProjectType) -> ToolsConfig {
    let mut tools = ToolsConfig {
        auto_detect: true,
        auto_install: false,
        formatters: Vec::new(),
        linters: Vec::new(),
    };

    match project_type {
        ProjectType::Rust => {
            // Add Rust formatters
            tools.formatters.push(FormatterConfig {
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
            });
            
            // Add alternative Rust formatter (prettier-please)
            tools.formatters.push(FormatterConfig {
                name: "prettier-please".to_string(),
                command: "prettier-please --write".to_string(),
                patterns: vec!["**/*.rs".to_string()],
                check_command: Some("prettier-please --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("cargo install prettier-please".to_string()),
                    npm: None,
                    brew: None,
                    apt: None,
                    manual: "Install with: cargo install prettier-please".to_string(),
                }),
            });

            // Add Rust linters
            tools.linters.push(LinterConfig {
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
            });
        }
        
        ProjectType::NodeJs => {
            // Add JavaScript/TypeScript formatters
            tools.formatters.push(FormatterConfig {
                name: "prettier".to_string(),
                command: "npx prettier --write".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.jsx".to_string(),
                    "**/*.tsx".to_string(),
                    "**/*.json".to_string(),
                    "**/*.css".to_string(),
                    "**/*.html".to_string(),
                    "**/*.md".to_string(),
                ],
                check_command: Some("npx prettier --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g prettier".to_string()),
                    brew: Some("brew install prettier".to_string()),
                    apt: None,
                    manual: "Install Node.js then run: npm install -g prettier".to_string(),
                }),
            });
            
            // Add Biome as alternative
            tools.formatters.push(FormatterConfig {
                name: "biome".to_string(),
                command: "biome format --write".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.jsx".to_string(),
                    "**/*.tsx".to_string(),
                    "**/*.json".to_string(),
                ],
                check_command: Some("biome --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("cargo install biome".to_string()),
                    npm: Some("npm install -g @biomejs/biome".to_string()),
                    brew: None,
                    apt: None,
                    manual: "Install with: npm install -g @biomejs/biome".to_string(),
                }),
            });

            // Add linters
            tools.linters.push(LinterConfig {
                name: "eslint".to_string(),
                command: "npx eslint".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.jsx".to_string(),
                    "**/*.tsx".to_string(),
                ],
                check_command: Some("npx eslint --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g eslint".to_string()),
                    brew: None,
                    apt: None,
                    manual: "Install Node.js then run: npm install -g eslint".to_string(),
                }),
            });
        }
        
        ProjectType::Python => {
            // Add Python formatters
            tools.formatters.push(FormatterConfig {
                name: "black".to_string(),
                command: "black".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: Some("black --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("cargo install black".to_string()),
                    npm: None,
                    brew: Some("brew install black".to_string()),
                    apt: Some("apt install python3-black".to_string()),
                    manual: "Install with: pip install black".to_string(),
                }),
            });
            
            // Add Ruff as alternative
            tools.formatters.push(FormatterConfig {
                name: "ruff".to_string(),
                command: "ruff format".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: Some("ruff --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("cargo install ruff".to_string()),
                    npm: None,
                    brew: Some("brew install ruff".to_string()),
                    apt: Some("apt install ruff".to_string()),
                    manual: "Install with: pip install ruff".to_string(),
                }),
            });

            // Add linters
            tools.linters.push(LinterConfig {
                name: "ruff-lint".to_string(),
                command: "ruff check".to_string(),
                patterns: vec!["**/*.py".to_string()],
                check_command: Some("ruff --version".to_string()),
                install: Some(InstallConfig {
                    cargo: Some("cargo install ruff".to_string()),
                    npm: None,
                    brew: Some("brew install ruff".to_string()),
                    apt: Some("apt install ruff".to_string()),
                    manual: "Install with: pip install ruff".to_string(),
                }),
            });
        }
        
        ProjectType::Go => {
            // Add Go formatters
            tools.formatters.push(FormatterConfig {
                name: "gofmt".to_string(),
                command: "gofmt -w".to_string(),
                patterns: vec!["**/*.go".to_string()],
                check_command: Some("gofmt -help".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: None,
                    brew: Some("brew install go".to_string()),
                    apt: Some("apt install golang-go".to_string()),
                    manual: "Install Go: https://golang.org/dl/".to_string(),
                }),
            });

            // Add linters
            tools.linters.push(LinterConfig {
                name: "golangci-lint".to_string(),
                command: "golangci-lint run".to_string(),
                patterns: vec!["**/*.go".to_string()],
                check_command: Some("golangci-lint --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: None,
                    brew: Some("brew install golangci-lint".to_string()),
                    apt: None,
                    manual: "Install: https://golangci-lint.run/usage/install/".to_string(),
                }),
            });
        }
        
        ProjectType::NxMonorepo => {
            // Add formatters for monorepo (usually JavaScript/TypeScript)
            tools.formatters.push(FormatterConfig {
                name: "prettier".to_string(),
                command: "npx prettier --write".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.jsx".to_string(),
                    "**/*.tsx".to_string(),
                    "**/*.json".to_string(),
                    "**/*.css".to_string(),
                    "**/*.html".to_string(),
                    "**/*.md".to_string(),
                ],
                check_command: Some("npx prettier --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g prettier".to_string()),
                    brew: Some("brew install prettier".to_string()),
                    apt: None,
                    manual: "Install Node.js then run: npm install -g prettier".to_string(),
                }),
            });

            // Add linters
            tools.linters.push(LinterConfig {
                name: "eslint".to_string(),
                command: "npx eslint".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.jsx".to_string(),
                    "**/*.tsx".to_string(),
                ],
                check_command: Some("npx eslint --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g eslint".to_string()),
                    brew: None,
                    apt: None,
                    manual: "Install Node.js then run: npm install -g eslint".to_string(),
                }),
            });
        }
        
        ProjectType::Generic => {
            // Add basic formatters that work across many languages
            tools.formatters.push(FormatterConfig {
                name: "prettier".to_string(),
                command: "npx prettier --write".to_string(),
                patterns: vec![
                    "**/*.js".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.json".to_string(),
                    "**/*.css".to_string(),
                    "**/*.html".to_string(),
                    "**/*.md".to_string(),
                ],
                check_command: Some("npx prettier --version".to_string()),
                install: Some(InstallConfig {
                    cargo: None,
                    npm: Some("npm install -g prettier".to_string()),
                    brew: Some("brew install prettier".to_string()),
                    apt: None,
                    manual: "Install Node.js then run: npm install -g prettier".to_string(),
                }),
            });
        }
    }

    tools
}
