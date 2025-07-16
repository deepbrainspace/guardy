//! Commit message hook implementation
//!
//! This hook validates commit messages according to conventional commit format.

use super::HookContext;
use anyhow::Result;
use regex::Regex;
use std::fs;

/// Execute commit-msg hook
/// TODO: Remove #[allow(dead_code)] when hook commands are implemented in Phase 1.5
#[allow(dead_code)]
pub async fn execute(context: HookContext) -> Result<()> {
    println!("📝 Validating commit message...");

    if context.args.is_empty() {
        anyhow::bail!("No commit message file provided");
    }

    let commit_msg_file = &context.args[0];
    let commit_msg = fs::read_to_string(commit_msg_file)?;

    validate_commit_message(&commit_msg)?;

    println!("✅ Commit message validation passed!");
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
