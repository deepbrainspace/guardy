pub mod operations;
pub mod remote;
// TODO: Add hooks module for hook installation/management
// TODO: Add commit module for commit operations

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct GitRepo {
    pub path: PathBuf,
}

impl GitRepo {
    pub fn discover() -> Result<Self> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to execute git rev-parse --show-toplevel")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Not in a git repository"));
        }

        let stdout = String::from_utf8(output.stdout).context("Git output is not valid UTF-8")?;

        let path = PathBuf::from(stdout.trim());
        Ok(GitRepo { path })
    }

    pub fn current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.path)
            .output()
            .context("Failed to execute git branch --show-current")?;

        if !output.status.success() {
            return Ok("HEAD".to_string());
        }

        let stdout = String::from_utf8(output.stdout).context("Git output is not valid UTF-8")?;

        Ok(stdout.trim().to_string())
    }

    pub fn git_dir(&self) -> PathBuf {
        self.path.join(".git")
    }
}
