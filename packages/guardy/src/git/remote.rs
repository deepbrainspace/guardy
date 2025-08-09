use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RemoteOperations {
    cache_dir: PathBuf,
}

impl RemoteOperations {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Clone or fetch a repository to the cache directory using system git
    pub fn clone_or_fetch(&self, repo_url: &str, version: &str) -> Result<PathBuf> {
        let repo_name = self.extract_repo_name(repo_url);
        let repo_path = self.cache_dir.join(&repo_name);

        if repo_path.exists() {
            // Repository exists, try to fetch and checkout
            tracing::info!("Fetching updates for {} at {}", repo_name, repo_path.display());
            self.fetch_and_checkout_system_git(&repo_path, version)?;
        } else {
            // Clone the repository
            tracing::info!("Cloning {} to {}", repo_url, repo_path.display());
            self.clone_with_system_git(repo_url, &repo_path, version)?;
        }

        Ok(repo_path)
    }

    /// Clone repository using system git command
    fn clone_with_system_git(&self, repo_url: &str, repo_path: &Path, version: &str) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Clone with system git - uses all user's authentication methods
        let output = Command::new("git")
            .args(["clone", "--quiet", repo_url])
            .arg(repo_path)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to clone repository '{}': {}", repo_url, error_msg));
        }

        // Checkout the specified version
        self.checkout_version_system_git(repo_path, version)?;

        Ok(())
    }

    /// Fetch and checkout using system git
    fn fetch_and_checkout_system_git(&self, repo_path: &Path, version: &str) -> Result<()> {
        // Fetch from origin
        let output = Command::new("git")
            .args(["fetch", "--quiet", "origin"])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to fetch from origin: {}", error_msg));
        }

        // Checkout the specified version
        self.checkout_version_system_git(repo_path, version)?;
        Ok(())
    }

    /// Checkout a specific version using system git
    fn checkout_version_system_git(&self, repo_path: &Path, version: &str) -> Result<()> {
        // Try to checkout the version - git will try branches, tags, and commits
        let output = Command::new("git")
            .args(["checkout", "--quiet", version])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            // Try fetching the specific branch/tag if it's not local
            let fetch_output = Command::new("git")
                .args(["fetch", "--quiet", "origin", &format!("{version}:{version}")])
                .current_dir(repo_path)
                .output()?;

            if fetch_output.status.success() {
                // Try checkout again
                let output = Command::new("git")
                    .args(["checkout", "--quiet", version])
                    .current_dir(repo_path)
                    .output()?;

                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Could not checkout version '{}': {}", version, error_msg));
                }
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Could not find version '{}' in repository: {}", version, error_msg));
            }
        }

        tracing::info!("Checked out version: {}", version);
        Ok(())
    }

    fn extract_repo_name(&self, repo_url: &str) -> String {
        // Extract repo name from URL like "github.com/org/repo" -> "repo"
        repo_url
            .trim_end_matches(".git")
            .split('/')
            .next_back()
            .unwrap_or("unknown")
            .to_string()
    }
}

