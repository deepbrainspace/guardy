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


    /// Clone repository using system git command
    fn clone_with_system_git(&self, repo_url: &str, repo_path: &Path, version: &str) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Clone with system git - shallow clone for speed
        let output = Command::new("git")
            .args(["clone", "--depth", "1", "--quiet", repo_url])
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

    /// Clone repository (called when it doesn't exist in cache)
    pub fn clone_repository(&self, repo_url: &str, repo_name: &str, version: &str) -> Result<()> {
        let repo_path = self.cache_dir.join(repo_name);
        self.clone_with_system_git(repo_url, &repo_path, version)?;
        Ok(())
    }

    /// Fetch and reset to remote version (ensures cache matches remote exactly)
    pub fn fetch_and_reset(&self, repo_name: &str, version: &str) -> Result<()> {
        let repo_path = self.cache_dir.join(repo_name);
        
        // Fetch only the specific branch/tag we need with depth 1 (just the latest commit)
        let output = Command::new("git")
            .args(["fetch", "--depth", "1", "origin", version])
            .current_dir(&repo_path)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to fetch from origin: {}", error_msg));
        }

        // Reset hard to the specified version (discards any local changes in cache)
        // Try as remote branch first
        let remote_ref = format!("origin/{version}");
        let reset_output = Command::new("git")
            .args(["reset", "--hard", &remote_ref])
            .current_dir(&repo_path)
            .output()?;

        if !reset_output.status.success() {
            // Try as tag or commit
            let reset_output = Command::new("git")
                .args(["reset", "--hard", version])
                .current_dir(&repo_path)
                .output()?;

            if !reset_output.status.success() {
                let error_msg = String::from_utf8_lossy(&reset_output.stderr);
                return Err(anyhow!("Failed to reset to version '{}': {}", version, error_msg));
            }
        }

        // Clean any untracked files
        Command::new("git")
            .args(["clean", "-fd"])
            .current_dir(&repo_path)
            .output()?;

        // Get and log the current commit SHA
        let sha_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&repo_path)
            .output()?;
        
        if sha_output.status.success() {
            let sha = String::from_utf8_lossy(&sha_output.stdout).trim().to_string();
            tracing::info!("Reset cache to version: {} ({})", version, &sha[..8]);
            
            // Store the SHA in .guardy directory for later reference
            let guardy_dir = PathBuf::from(".guardy");
            std::fs::create_dir_all(&guardy_dir)?;
            let sha_file = guardy_dir.join(format!("sync_sha_{repo_name}"));
            std::fs::write(sha_file, format!("{version}\n{sha}"))?;
        } else {
            tracing::info!("Reset cache to version: {}", version);
        }
        
        Ok(())
    }

}

