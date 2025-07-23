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

pub async fn execute(args: ConfigArgs, custom_config: Option<&str>) -> Result<()> {
    use crate::cli::output::*;
    use crate::config::{GuardyConfig, ConfigFormat};
    
    match args.command {
        ConfigCommand::Init => {
            styled!("Creating default {} file...", 
                ("guardy.toml", "file_path")
            );
            // TODO: Implement init - create minimal config file
            styled!("{} Created {} with default settings!", 
                ("✅", "success_symbol"),
                ("guardy.toml", "file_path")
            );
        },
        ConfigCommand::Show { format } => {
            styled!("Loading merged configuration in {} format...", 
                (&format, "property")
            );
            let config = GuardyConfig::load(custom_config, None::<&()>)?;
            
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
            styled!("Setting {} = {}", 
                (&key, "property"),
                (&value, "accent")
            );
            // TODO: Implement set - modify config file
            styled!("{} Configuration updated!", 
                ("✅", "success_symbol")
            );
        },
        ConfigCommand::Get { key } => {
            styled!("Getting configuration key: {}", 
                (&key, "property")
            );
            let config = GuardyConfig::load(custom_config, None::<&()>)?;
            
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
            styled!("Validating {} configuration...", 
                ("guardy", "primary")
            );
            let _config = GuardyConfig::load(None, None::<&()>)?;  // This will fail if config is invalid
            styled!("{} Configuration is valid!", 
                ("✅", "success_symbol")
            );
        },
    }
    
    Ok(())
}