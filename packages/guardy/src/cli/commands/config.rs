use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args, Clone)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommand {
    /// Display current merged configuration
    Show {
        /// Output format: json, toml, yaml
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
}

pub async fn execute(
    args: ConfigArgs,
    _custom_config: Option<&str>,
    _verbosity_level: u8,
) -> Result<()> {
    use crate::cli::output::*;
    use crate::config::CONFIG;

    match args.command {
        ConfigCommand::Show { format } => {
            styled!(
                "Displaying current configuration in {} format...",
                (&format, "property")
            );
            
            let output = match format.to_lowercase().as_str() {
                "json" => serde_json::to_string_pretty(&**CONFIG)?,
                "yaml" | "yml" => serde_yaml_bw::to_string(&**CONFIG)?,
                "toml" => toml::to_string_pretty(&**CONFIG)?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported format: {}. Use json, toml, or yaml",
                        format
                    ));
                }
            };
            
            println!("{output}");
        }
    }

    Ok(())
}
