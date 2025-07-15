//! Pre-commit hook implementation
//!
//! This hook runs before commits are created and performs:
//! - Branch protection checks
//! - Secret detection
//! - Staging validation
//! - Code formatting

use super::HookContext;
use crate::git::GitOperations;
use crate::security::SecretScanner;
use anyhow::Result;

/// Execute pre-commit hook
pub async fn execute(context: HookContext) -> Result<()> {
    println!("🔍 Running pre-commit checks...");

    let git = GitOperations::discover()?;

    // Check if we're on a protected branch
    if !context.config.security.protected_branches.is_empty() {
        check_branch_protection(&git, &context.config.security.protected_branches)?;
    }

    // Run secret detection if enabled
    if context.config.security.secret_detection {
        run_secret_detection(&git).await?;
    }

    // Validate staging area
    validate_staging(&git)?;

    println!("✅ Pre-commit checks passed!");
    Ok(())
}

/// Check if current branch is protected
fn check_branch_protection(git: &GitOperations, protected_branches: &[String]) -> Result<()> {
    if git.is_protected_branch(protected_branches)? {
        let current_branch = git.current_branch()?;
        anyhow::bail!(
            "🚫 Direct commits to protected branch '{}' are not allowed.\n\
            Please create a feature branch and submit a pull request.",
            current_branch
        );
    }

    println!("✅ Branch protection check passed");
    Ok(())
}

/// Run secret detection on staged files
async fn run_secret_detection(git: &GitOperations) -> Result<()> {
    let staged_files = git.get_staged_files()?;

    if staged_files.is_empty() {
        println!("ℹ️  No staged files to scan");
        return Ok(());
    }

    println!(
        "🔍 Scanning {} staged files for secrets...",
        staged_files.len()
    );

    let scanner = SecretScanner::new()?;
    let matches = scanner.scan_files(&staged_files)?;

    if !matches.is_empty() {
        eprintln!("🚫 Secrets detected in staged files:");
        for m in &matches {
            eprintln!(
                "  {} {}:{} [{}] {}",
                m.severity, m.file_path, m.line_number, m.pattern_name, m.content
            );
        }
        anyhow::bail!("Commit blocked due to secret detection");
    }

    println!("✅ Secret detection passed");
    Ok(())
}

/// Validate staging area
fn validate_staging(git: &GitOperations) -> Result<()> {
    let staged_files = git.get_staged_files()?;

    if staged_files.is_empty() {
        anyhow::bail!("🚫 No files staged for commit");
    }

    println!(
        "✅ Staging validation passed ({} files)",
        staged_files.len()
    );
    Ok(())
}
