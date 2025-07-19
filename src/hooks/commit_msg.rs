//! Commit message hook implementation
//!
//! This hook validates commit messages according to conventional commit format.

use super::HookContext;
use crate::cli::Output;
use crate::git::commit::is_conventional_commit;
use anyhow::Result;
use regex::Regex;
use std::fs;

/// Execute commit-msg hook
pub async fn execute(_context: HookContext) -> Result<()> {
    let output = Output::new(false, false); // Default to non-verbose, non-quiet
    let current_dir = std::env::current_dir()?;
    
    let step1_start = std::time::Instant::now();
    
    // Get commit message from file (usually .git/COMMIT_EDITMSG)
    let commit_msg_path = current_dir.join(".git/COMMIT_EDITMSG");
    if !commit_msg_path.exists() {
        output.warning("No commit message file found, skipping validation");
        return Ok(());
    }
    
    let commit_msg = fs::read_to_string(&commit_msg_path)?;
    let first_line = commit_msg.lines().next().unwrap_or("").trim();
    
    if first_line.is_empty() {
        anyhow::bail!("Commit message cannot be empty");
    }
    
    // 1. Check conventional commit format
    if !is_conventional_commit(first_line) {
        anyhow::bail!(
            "Commit message must follow conventional commit format\n\
            Expected format: type(scope): description\n\
            Examples: feat: add new feature, fix(auth): resolve login issue"
        );
    }
    
    output.success("Conventional commit format is valid");
    
    let step1_duration = step1_start.elapsed();
    output.workflow_step_timed(1, 3, "Validating commit message format", "📝", step1_duration);
    
    let step2_start = std::time::Instant::now();
    
    // 2. Check commit message length
    if first_line.len() > 72 {
        anyhow::bail!(
            "Commit message subject line is too long (max 72 characters)\n\
            Current length: {} characters", first_line.len()
        );
    }
    
    output.success("Commit message length is appropriate");
    
    let step2_duration = step2_start.elapsed();
    output.workflow_step_timed(2, 3, "Checking commit message length", "📏", step2_duration);
    
    let step3_start = std::time::Instant::now();
    
    // 3. Check for breaking changes indication
    if first_line.contains("BREAKING CHANGE") || first_line.contains("!") {
        output.info("Breaking change detected in commit message");
    }
    
    output.success("Commit message validation passed");
    
    let step3_duration = step3_start.elapsed();
    output.workflow_step_timed(3, 3, "Checking for breaking changes", "⚠️", step3_duration);
    
    Ok(())
}

/// Validate commit message format
#[allow(dead_code)]
fn validate_commit_message(msg: &str) -> Result<()> {
    // Remove comments and empty lines
    let msg = msg
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    if msg.trim().is_empty() {
        anyhow::bail!("🚫 Empty commit message");
    }

    let lines: Vec<&str> = msg.lines().collect();
    let subject = lines[0];

    // Check subject line length
    if subject.len() > 72 {
        anyhow::bail!(
            "🚫 Subject line too long ({} characters). Maximum 72 characters allowed.",
            subject.len()
        );
    }

    // Check conventional commit format
    let conventional_regex = Regex::new(
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .+",
    )?;

    if !conventional_regex.is_match(subject) {
        anyhow::bail!(
            "🚫 Invalid commit message format.\n\
            Expected format: type(scope): description\n\
            \n\
            Valid types: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert\n\
            \n\
            Examples:\n\
            - feat: add user authentication\n\
            - fix(auth): resolve login timeout issue\n\
            - docs: update README with installation steps\n\
            \n\
            Your message: {}",
            subject
        );
    }

    // Check for imperative mood
    let imperative_words = ["added", "fixed", "updated", "changed", "removed", "deleted"];
    let subject_lower = subject.to_lowercase();

    for word in imperative_words {
        if subject_lower.contains(word) {
            anyhow::bail!(
                "🚫 Use imperative mood in commit message.\n\
                Use '{}' instead of '{}'",
                word.trim_end_matches('d').trim_end_matches("ed"),
                word
            );
        }
    }

    println!("✅ Conventional commit format validated");
    Ok(())
}
