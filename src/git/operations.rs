//! Git operations utilities
//!
//! This module provides low-level git operations like getting staged files,
//! current branch, and other git-related functionality.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Get list of staged files
pub fn get_staged_files(current_dir: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(current_dir)
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let files = String::from_utf8(output.stdout)?
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    
    Ok(files)
}

/// Get current git branch
pub fn get_current_branch(current_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(current_dir)
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to get current branch");
    }
    
    let branch = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    
    Ok(branch)
}