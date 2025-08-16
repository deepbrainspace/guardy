//! CLI arguments static storage for config system access

use std::sync::OnceLock;
use clap::Parser;
use super::commands::Cli;

/// Global CLI - parsed once, accessed everywhere
pub static CLI: OnceLock<Cli> = OnceLock::new();

/// Initialize CLI (called once from main)
pub fn init() -> &'static Cli {
    if CLI.set(Cli::parse()).is_err() {
        tracing::debug!("CLI already initialized, returning existing instance");
    }
    CLI.get().unwrap()
}