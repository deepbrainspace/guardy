//! Configuration command implementations
//!
//! Commands for managing Guardy configuration.

use crate::cli::ConfigCommands;
use crate::cli::Output;
use crate::config::GuardyConfig;
use crate::config::languages::{get_language_configs, detect_languages};
use crate::shared::get_current_dir;
use anyhow::Result;
use std::fs;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::util::as_24_bit_terminal_escaped;
use console::style;

/// Execute config commands
pub async fn execute(cmd: ConfigCommands, output: &Output) -> Result<()> {
    match cmd {
        ConfigCommands::Init => init(output).await,
        ConfigCommands::Validate => validate(output).await,
        ConfigCommands::Show => show(output).await,
        ConfigCommands::Language { name } => show_language(output, &name).await,
        ConfigCommands::Languages { all } => show_languages(output, all).await,
    }
}

async fn init(output: &Output) -> Result<()> {
    output.info("🔧 Initializing Configuration");
    
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
    
    // Detect languages in the project
    let detected_languages = detect_languages(&current_dir);
    if !detected_languages.is_empty() {
        let lang_names: Vec<&str> = detected_languages.iter().map(|(name, _)| name.as_str()).collect();
        output.info(&format!("❯ Detected languages: {}", lang_names.join(", ")));
    } else {
        output.info("❯ No specific languages detected, using generic configuration");
    }
    
    // Create default configuration with examples
    let lang_names: Vec<String> = detected_languages.iter().map(|(name, _)| name.clone()).collect();
    let config_content = create_config_with_examples(&lang_names);
    
    // Save configuration
    fs::write(&config_path, config_content)?;
    
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

/// Create configuration file with examples and detected languages
fn create_config_with_examples(detected_languages: &[String]) -> String {
    let mut config = String::new();
    
    config.push_str("# Guardy Configuration File\n");
    config.push_str("# Auto-generated with examples\n\n");
    
    config.push_str("hooks:\n");
    config.push_str("  pre-commit:\n");
    config.push_str("    - format\n");
    config.push_str("    - lint\n");
    config.push_str("    - secrets\n\n");
    
    config.push_str("security:\n");
    config.push_str("  secret_detection: true\n\n");
    
    config.push_str("tools:\n");
    config.push_str("  auto_detect: true\n");
    config.push_str("  auto_install: false\n\n");
    
    // Add language-specific override examples
    config.push_str("# Language-specific overrides (optional)\n");
    config.push_str("# Uncomment and modify to customize behavior for specific languages\n");
    config.push_str("#\n");
    config.push_str("# languages:\n");
    
    // Add examples for detected languages
    for lang in detected_languages {
        match lang.as_str() {
            "javascript" => {
                config.push_str("#   javascript:\n");
                config.push_str("#     package_manager: pnpm      # Override auto-detected package manager\n");
                config.push_str("#     formatters:\n");
                config.push_str("#       - prettier\n");
                config.push_str("#       - biome\n");
                config.push_str("#     linters:\n");
                config.push_str("#       - eslint\n");
                config.push_str("#       - typescript\n");
                config.push_str("#\n");
            }
            "rust" => {
                config.push_str("#   rust:\n");
                config.push_str("#     formatters:\n");
                config.push_str("#       - rustfmt             # Override default prettyplease\n");
                config.push_str("#     linters:\n");
                config.push_str("#       - clippy\n");
                config.push_str("#\n");
            }
            "python" => {
                config.push_str("#   python:\n");
                config.push_str("#     package_manager: pip      # Override auto-detected uv\n");
                config.push_str("#     formatters:\n");
                config.push_str("#       - black\n");
                config.push_str("#       - ruff\n");
                config.push_str("#     linters:\n");
                config.push_str("#       - ruff\n");
                config.push_str("#       - mypy\n");
                config.push_str("#\n");
            }
            "go" => {
                config.push_str("#   go:\n");
                config.push_str("#     formatters:\n");
                config.push_str("#       - gofmt\n");
                config.push_str("#       - goimports\n");
                config.push_str("#     linters:\n");
                config.push_str("#       - golangci-lint\n");
                config.push_str("#\n");
            }
            _ => {}
        }
    }
    
    // Add manual tool configuration example
    config.push_str("# Manual tool configuration (for unsupported languages)\n");
    config.push_str("# formatters:\n");
    config.push_str("#   - name: \"custom-formatter\"\n");
    config.push_str("#     command: \"my-formatter --fix\"\n");
    config.push_str("#     patterns:\n");
    config.push_str("#       - \"**/*.custom\"\n");
    config.push_str("#     check_command: \"my-formatter --version\"\n");
    config.push_str("#     install:\n");
    config.push_str("#       manual: \"Install from https://example.com\"\n");
    
    config
}

/// Show configuration for a specific language
async fn show_language(output: &Output, language: &str) -> Result<()> {
    let configs = get_language_configs();
    let current_dir = get_current_dir()?;
    
    // Check if language is supported
    if let Some(lang_config) = configs.get(language) {
        // Check if language is detected in current project
        let detected_languages = detect_languages(&current_dir);
        let is_detected = detected_languages.iter().any(|(lang, _)| lang == language);
        
        output.info(&format!("❯ {} Configuration", lang_config.name));
        output.blank_line();
        
        // Detection status with badge
        output.info("❯ Detection Status:");
        if is_detected {
            output.badge("DETECTED", "green");
            println!(" Language detected in current project");
        } else {
            output.badge("NOT FOUND", "yellow");
            println!(" Language not detected in current project");
        }
        output.blank_line();
        
        // Package managers using tree structure
        output.info("❯ Package Managers:");
        for (i, pm) in lang_config.package_managers.iter().enumerate() {
            let is_last = i == lang_config.package_managers.len() - 1;
            output.tree_item(is_last, pm, "(preference order)");
        }
        output.blank_line();
        
        // Formatters
        output.info("❯ Formatters:");
        for formatter in &lang_config.formatters {
            output.success(&format!("{}", formatter.name));
            output.indent(&format!("Command: {}", formatter.command));
            output.indent(&format!("Check: {}", formatter.check_command));
            
            // Show installation options
            if let Some(npm) = &formatter.install_commands.npm {
                output.indent(&format!("Install (npm): {}", npm));
            }
            if let Some(pnpm) = &formatter.install_commands.pnpm {
                output.indent(&format!("Install (pnpm): {}", pnpm));
            }
            if let Some(cargo) = &formatter.install_commands.cargo {
                output.indent(&format!("Install (cargo): {}", cargo));
            }
            if let Some(brew) = &formatter.install_commands.brew {
                output.indent(&format!("Install (brew): {}", brew));
            }
            output.indent(&format!("Manual: {}", formatter.install_commands.manual));
            output.blank_line();
        }
        
        // Linters
        output.info("❯ Linters:");
        for linter in &lang_config.linters {
            output.success(&format!("{}", linter.name));
            output.indent(&format!("Command: {}", linter.command));
            output.indent(&format!("Check: {}", linter.check_command));
            
            // Show installation options
            if let Some(npm) = &linter.install_commands.npm {
                output.indent(&format!("Install (npm): {}", npm));
            }
            if let Some(pnpm) = &linter.install_commands.pnpm {
                output.indent(&format!("Install (pnpm): {}", pnpm));
            }
            if let Some(cargo) = &linter.install_commands.cargo {
                output.indent(&format!("Install (cargo): {}", cargo));
            }
            if let Some(brew) = &linter.install_commands.brew {
                output.indent(&format!("Install (brew): {}", brew));
            }
            output.indent(&format!("Manual: {}", linter.install_commands.manual));
            output.blank_line();
        }
        
        // Configuration instructions
        output.info("❯ Configuration:");
        output.indent(&format!("Add to .guardy.yml under languages.{} section", language));
        output.indent("Use 'guardy config init' to generate example configuration");
        
    } else {
        output.error(&format!("Language '{}' is not supported", language));
        output.blank_line();
        output.info("❯ Supported languages:");
        for lang_name in configs.keys() {
            output.indent(&format!("• {}", lang_name));
        }
        output.blank_line();
        output.info("Run 'guardy config languages' to see all supported languages");
    }
    
    Ok(())
}

/// Show supported languages
async fn show_languages(output: &Output, show_all: bool) -> Result<()> {
    let configs = get_language_configs();
    let current_dir = get_current_dir()?;
    let detected_languages = detect_languages(&current_dir);
    
    if show_all {
        output.banner("All Supported Languages", "Complete list of languages supported by Guardy");
        
        // Show detected languages first
        if !detected_languages.is_empty() {
            println!("{} {}", style("●").green().bold(), style("Detected in Current Project:").bold());
            for (lang, count) in &detected_languages {
                if let Some(config) = configs.get(lang) {
                    let file_text = if *count == 1 { "file" } else { "files" };
                    output.success(&format!("{} - {} ({} {})", config.name, config.description, count, file_text));
                }
            }
            output.blank_line();
        }
        
        // Show available (non-detected) languages
        let mut available_languages = Vec::new();
        for (lang_name, config) in configs {
            let is_detected = detected_languages.iter().any(|(l, _)| l == &lang_name);
            if !is_detected {
                available_languages.push((lang_name, config));
            }
        }
        
        if !available_languages.is_empty() {
            println!("{} {}", style("●").white().bold(), style("Supported Languages:").bold());
            for (_, config) in available_languages {
                output.list_item(&format!("{} - {}", config.name, config.description));
            }
        }
    } else {
        // Show only detected languages by default with their tools
        output.banner("Detected Languages", "Languages and tools found in your project");
        
        if !detected_languages.is_empty() {
            for (lang, count) in &detected_languages {
                if let Some(config) = configs.get(lang) {
                    // Language header with file count
                    let file_text = if *count == 1 { "file" } else { "files" };
                    let description_with_count = format!("{} ({} {})", config.description, count, file_text);
                    output.language_header(&config.name, &description_with_count);
                    
                    // Show package manager
                    if !config.package_managers.is_empty() {
                        output.package_manager(&config.package_managers[0]);
                    }
                    
                    // Show formatters
                    if !config.formatters.is_empty() {
                        let formatter_names: Vec<&str> = config.formatters.iter().map(|f| f.name.as_str()).collect();
                        output.tool_section("🎨", "Formatters", &formatter_names);
                    }
                    
                    // Show linters
                    if !config.linters.is_empty() {
                        let linter_names: Vec<&str> = config.linters.iter().map(|l| l.name.as_str()).collect();
                        output.tool_section("🔍", "Linters", &linter_names);
                    }
                    
                    output.blank_line();
                }
            }
        } else {
            output.warning("No languages detected in current project");
            output.info("Run 'guardy config languages --all' to see all supported languages");
        }
    }
    
    Ok(())
}
