use anyhow::Result;
use clap::{Parser, Subcommand};
use supercli::clap::create_help_styles;

pub mod config;
pub mod install;
pub mod run;
pub mod scan;
pub mod status;
pub mod sync;
pub mod uninstall;
pub mod version;

#[derive(Parser)]
#[command(
    name = "guardy",
    version = env!("CARGO_PKG_VERSION"),
    about = "Fast, secure git hooks in Rust with secret scanning and file synchronization",
    long_about = "Guardy provides native Rust implementations of git hooks with security scanning \
                  and protected file synchronization across repositories.",
    styles = create_help_styles()
)]
pub struct Cli {
    /// Run as if started in `<DIR>` instead of current working directory
    #[arg(short = 'C', long = "directory", global = true)]
    pub directory: Option<String>,

    /// Increase verbosity (can be repeated)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Use custom configuration file
    #[arg(long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install git hooks into the current repository
    Install(install::InstallArgs),
    /// Manually execute a specific hook for testing
    Run(run::RunArgs),
    /// Scan files or directories for secrets
    Scan(scan::ScanArgs),
    /// Configuration management
    Config(config::ConfigArgs),
    /// Show current installation and configuration status
    Status(status::StatusArgs),
    /// Remove all installed hooks
    Uninstall(uninstall::UninstallArgs),
    /// Protected file synchronization
    Sync(sync::SyncArgs),
    /// Show version information
    Version(version::VersionArgs),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Change directory if specified
        if let Some(dir) = &self.directory {
            std::env::set_current_dir(dir)?;
        }

        // Set up logging based on verbosity
        setup_logging(self.verbose, self.quiet);

        match self.command {
            Some(Commands::Install(args)) => install::execute(args, self.verbose).await,
            Some(Commands::Run(args)) => run::execute(args, self.verbose).await,
            Some(Commands::Scan(args)) => {
                tracing::debug!(config_path = ?self.config, "CLI config path");
                scan::execute(args, self.verbose, self.config.as_deref()).await
            }
            Some(Commands::Config(args)) => {
                config::execute(args, self.config.as_deref(), self.verbose).await
            }
            Some(Commands::Status(args)) => status::execute(args, self.verbose).await,
            Some(Commands::Uninstall(args)) => uninstall::execute(args).await,
            Some(Commands::Sync(args)) => sync::execute(args, self.config.as_deref()).await,
            Some(Commands::Version(args)) => version::execute(args).await,
            None => {
                // Default behavior - show status if in git repo, otherwise show help
                if crate::git::GitRepo::discover().is_ok() {
                    status::execute(status::StatusArgs::default(), self.verbose).await
                } else {
                    // TODO: Implement proper help display using clap's help system
                    println!("Run 'guardy --help' for usage information");
                    Ok(())
                }
            }
        }
    }
}

fn setup_logging(verbose: u8, quiet: bool) {
    if quiet {
        return;
    }

    // Create filter that suppresses debug from ignore/globset crates appropriately
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        match verbose {
            0 => tracing_subscriber::EnvFilter::new("warn"),
            1 => tracing_subscriber::EnvFilter::new("info,ignore=warn,globset=warn"),
            2 => tracing_subscriber::EnvFilter::new("debug,ignore=warn,globset=warn"),
            _ => tracing_subscriber::EnvFilter::new("trace"), // -vvv shows everything including globset
        }
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
