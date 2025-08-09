use anyhow::{Result, Context};
use std::path::PathBuf;
use std::process::Command;
use super::GitRepo;

impl GitRepo {
    /// Get list of files that are staged for commit (primary use case for pre-commit hooks)
    pub fn get_staged_files(&self) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&self.path)
            .output()
            .context("Failed to execute git diff --cached --name-only")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Git command failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Git output is not valid UTF-8")?;

        let files = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| self.path.join(line.trim()))
            .collect();

        Ok(files)
    }
}