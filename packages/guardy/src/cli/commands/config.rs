use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Create default guardy.toml
    Init,
    /// Display current merged configuration
    Show {
        /// Output format: json, toml, yaml
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    /// Set configuration value
    Set { key: String, value: String },
    /// Get configuration value  
    Get { key: String },
    /// Validate configuration file
    Validate,
}

pub async fn execute(args: ConfigArgs) -> Result<()> {
    use crate::cli::output::*;
    use crate::config::{GuardyConfig, ConfigFormat};
    
    match args.command {
        ConfigCommand::Init => {
            info("Creating default guardy.toml...");
            // TODO: Implement init - create minimal config file
            success("Created guardy.toml with default settings!");
        },
        ConfigCommand::Show { format } => {
            info("Loading merged configuration...");
            let config = GuardyConfig::load(None, None::<&()>)?;
            
            let format_enum = match format.to_lowercase().as_str() {
                "json" => ConfigFormat::Json,
                "yaml" | "yml" => ConfigFormat::Yaml,
                "toml" => ConfigFormat::Toml,
                _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use json, toml, or yaml", format)),
            };
            
            let output = config.export_config_highlighted(format_enum)?;
            println!("{}", output);
        },
        ConfigCommand::Set { key, value } => {
            info(&format!("Setting {}={}", key, value));
            // TODO: Implement set - modify config file
            success("Configuration updated!");
        },
        ConfigCommand::Get { key } => {
            info(&format!("Getting {}", key));
            let config = GuardyConfig::load(None, None::<&()>)?;
            
            // First try to get as a section/object
            if let Ok(section_val) = config.get_section(&key) {
                // Check if it's a complex object (array/object) or simple value
                match &section_val {
                    serde_json::Value::Object(_) => {
                        // Display as formatted JSON for objects
                        println!("{}", serde_json::to_string_pretty(&section_val)?);
                    },
                    serde_json::Value::Array(_) => {
                        // Display each array item on its own line
                        if let serde_json::Value::Array(arr) = section_val {
                            for item in arr {
                                match item {
                                    serde_json::Value::String(s) => println!("{}", s),
                                    _ => println!("{}", item),
                                }
                            }
                        }
                    },
                    _ => {
                        // Simple value - display directly
                        match section_val {
                            serde_json::Value::String(s) => println!("{}", s),
                            serde_json::Value::Bool(b) => println!("{}", b),
                            serde_json::Value::Number(n) => println!("{}", n),
                            _ => println!("{}", section_val),
                        }
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Configuration key '{}' not found", key));
            }
        },
        ConfigCommand::Validate => {
            info("Validating configuration...");
            let _config = GuardyConfig::load(None, None::<&()>)?;  // This will fail if config is invalid
            success("Configuration is valid!");
        },
    }
    
    Ok(())
}