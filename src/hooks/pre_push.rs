//! Pre-push hook implementation
//!
//! This hook runs before pushes and performs final validation.

use super::HookContext;
use crate::git::GitOperations;
use anyhow::Result;

/// Execute pre-push hook
pub async fn execute(_context: HookContext) -> Result<()> {
    println!("🚀 Running pre-push checks...");

    let git = GitOperations::discover()?;

    // Validate working tree state
    validate_working_tree(&git)?;

    // TODO: Add optional test execution
    // TODO: Add lint checks
    // TODO: Add lockfile validation

    println!("✅ Pre-push checks passed!");
    Ok(())
}

/// Validate working tree state
fn validate_working_tree(git: &GitOperations) -> Result<()> {
    if !git.is_working_tree_clean()? {
        anyhow::bail!(
            "🚫 Working tree is not clean.\n\
            Please commit or stash your changes before pushing."
        );
    }

    println!("✅ Working tree validation passed");
    Ok(())
}
