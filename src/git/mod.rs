//! Git integration layer for Guardy
//!
//! This module provides a high-level interface for Git operations using git2.
//! It handles repository detection, branch operations, and hook management.

use anyhow::{Context, Result};
use git2::{Repository, Status, StatusOptions};
use std::path::Path;

/// Git operations handler
/// TODO: Remove #[allow(dead_code)] when CLI commands are implemented in Phase 1.7
#[allow(dead_code)]
pub struct GitOperations {
    repo: Repository,
}

#[allow(dead_code)]
impl GitOperations {
    /// Open a Git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path).context("Failed to open Git repository")?;

        Ok(Self { repo })
    }

    /// Discover and open a Git repository from current directory
    pub fn discover() -> Result<Self> {
        let repo = Repository::discover(".").context("No Git repository found")?;

        Ok(Self { repo })
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head().context("Failed to get HEAD reference")?;

        let branch_name = head.shorthand().context("Failed to get branch name")?;

        Ok(branch_name.to_string())
    }

    /// Check if current branch is protected
    pub fn is_protected_branch(&self, protected_branches: &[String]) -> Result<bool> {
        let current = self.current_branch()?;
        Ok(protected_branches.contains(&current))
    }

    /// Get staged files
    pub fn get_staged_files(&self) -> Result<Vec<String>> {
        let mut staged_files = Vec::new();
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);

        let statuses = self
            .repo
            .statuses(Some(&mut opts))
            .context("Failed to get repository status")?;

        for entry in statuses.iter() {
            if entry.status().contains(Status::INDEX_NEW)
                || entry.status().contains(Status::INDEX_MODIFIED)
                || entry.status().contains(Status::INDEX_DELETED)
            {
                if let Some(path) = entry.path() {
                    staged_files.push(path.to_string());
                }
            }
        }

        Ok(staged_files)
    }

    /// Check if working tree is clean
    pub fn is_working_tree_clean(&self) -> Result<bool> {
        let statuses = self
            .repo
            .statuses(None)
            .context("Failed to get repository status")?;

        Ok(statuses.is_empty())
    }

    /// Get repository root path
    pub fn repo_path(&self) -> &Path {
        self.repo.path()
    }

    /// Get working directory path
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// Install a git hook
    pub fn install_hook(&self, hook_name: &str, hook_content: &str) -> Result<()> {
        let hooks_dir = self.repo.path().join("hooks");
        let hook_path = hooks_dir.join(hook_name);

        // Create hooks directory if it doesn't exist
        std::fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;

        // Write hook content
        std::fs::write(&hook_path, hook_content).context("Failed to write hook file")?;

        // Make hook executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_path)
                .context("Failed to get hook file metadata")?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_path, perms)
                .context("Failed to set hook file permissions")?;
        }

        Ok(())
    }

    /// Remove a git hook
    pub fn remove_hook(&self, hook_name: &str) -> Result<()> {
        let hook_path = self.repo.path().join("hooks").join(hook_name);

        if hook_path.exists() {
            std::fs::remove_file(&hook_path).context("Failed to remove hook file")?;
        }

        Ok(())
    }

    /// Check if a hook exists
    pub fn hook_exists(&self, hook_name: &str) -> bool {
        self.repo.path().join("hooks").join(hook_name).exists()
    }
}
