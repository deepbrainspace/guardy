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
        // For tags, we need to clone with tags to ensure they're available
        let mut clone_args = vec!["clone", "--depth", "1", "--quiet"];
        if self.is_immutable_version(version) && (version.starts_with('v') || version.chars().next().unwrap_or('a').is_ascii_digit()) {
            clone_args.push("--tags");
        }
        clone_args.push(repo_url);
        
        let output = Command::new("git")
            .args(&clone_args)
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

    /// Check if version is immutable (tag or commit SHA)
    fn is_immutable_version(&self, version: &str) -> bool {
        // Tag pattern: v1.0.0, v2.1.3-beta, 1.0.0, etc.
        if version.starts_with('v') && version.len() > 1 {
            if let Some(first_char) = version.chars().nth(1) {
                if first_char.is_ascii_digit() {
                    return true;
                }
            }
        }
        
        // Commit SHA pattern: 7+ hex characters (short or full SHA)
        if version.len() >= 7 && version.len() <= 40 && version.chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }
        
        // Semantic version without 'v' prefix: 1.0.0, 2.1.3-beta
        if version.chars().next().unwrap_or('a').is_ascii_digit() && version.contains('.') {
            return true;
        }
        
        false // Assume it's a mutable branch
    }

    /// Fetch and reset to remote version (ensures cache matches remote exactly)
    pub fn fetch_and_reset(&self, repo_name: &str, version: &str) -> Result<()> {
        let repo_path = self.cache_dir.join(repo_name);
        
        tracing::trace!("Checking if cache needs update for {} @ {}", repo_name, version);
        
        // Fast path: For immutable versions (tags/commits), just check if we have it locally
        if self.is_immutable_version(version) {
            tracing::debug!("Version '{}' appears to be immutable (tag/commit)", version);
            
            // Check if we have this version locally
            let has_version_output = Command::new("git")
                .args(["rev-parse", "--verify", "--quiet", version])
                .current_dir(&repo_path)
                .output()?;
            
            if has_version_output.status.success() {
                // Get the SHA of this version
                let version_sha_output = Command::new("git")
                    .args(["rev-parse", version])
                    .current_dir(&repo_path)
                    .output()?;
                
                // Get current HEAD SHA
                let head_sha_output = Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .current_dir(&repo_path)
                    .output()?;
                
                if version_sha_output.status.success() && head_sha_output.status.success() {
                    let version_sha = String::from_utf8_lossy(&version_sha_output.stdout).trim().to_string();
                    let head_sha = String::from_utf8_lossy(&head_sha_output.stdout).trim().to_string();
                    
                    if version_sha == head_sha {
                        tracing::info!("Cache already has immutable version: {} ({})", version, &version_sha[..8]);
                        return Ok(());
                    } else {
                        tracing::debug!("Cache has version {} but HEAD is different, need to reset", version);
                        // Reset to the correct version (no fetch needed for immutable versions)
                        let reset_output = Command::new("git")
                            .args(["reset", "--hard", version])
                            .current_dir(&repo_path)
                            .output()?;
                        
                        if reset_output.status.success() {
                            tracing::info!("Reset cache to immutable version: {} ({})", version, &version_sha[..8]);
                            return Ok(());
                        }
                    }
                }
            }
            
            tracing::debug!("Don't have immutable version {} locally, need to fetch", version);
        } else {
            tracing::debug!("Version '{}' appears to be mutable (branch), checking remote", version);
            
            // Slow path: For mutable versions (branches), compare with remote
            let local_sha_output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&repo_path)
                .output()?;
                
            let remote_sha_output = Command::new("git")
                .args(["ls-remote", "origin", version])
                .current_dir(&repo_path)
                .output()?;

            if local_sha_output.status.success() && remote_sha_output.status.success() {
                let local_sha = String::from_utf8_lossy(&local_sha_output.stdout).trim().to_string();
                let remote_output = String::from_utf8_lossy(&remote_sha_output.stdout);
                
                // Parse remote SHA (format: "commit_sha\trefs/heads/branch_name" or just "commit_sha")
                let remote_sha = remote_output.lines().next()
                    .and_then(|line| line.split_whitespace().next())
                    .unwrap_or("").to_string();
                
                tracing::trace!("Local SHA: {}, Remote SHA: {}", &local_sha[..8], &remote_sha[..8]);
                
                if local_sha == remote_sha && !local_sha.is_empty() {
                    tracing::info!("Cache already up to date: {} ({})", version, &local_sha[..8]);
                    return Ok(());
                }
                
                tracing::debug!("Cache needs update: local {} != remote {}", &local_sha[..8], &remote_sha[..8]);
            } else {
                tracing::trace!("Could not compare SHAs, proceeding with fetch");
            }
        }
        
        // Fetch only the specific branch/tag we need with depth 1 (just the latest commit)
        tracing::debug!("Fetching {} from origin", version);
        let mut fetch_args = vec!["fetch", "--depth", "1"];
        
        // For immutable versions that look like tags, fetch tags
        if self.is_immutable_version(version) && (version.starts_with('v') || version.chars().next().unwrap_or('a').is_ascii_digit()) {
            fetch_args.extend_from_slice(&["--tags", "origin"]);
        } else {
            fetch_args.extend_from_slice(&["origin", version]);
        }
        
        let output = Command::new("git")
            .args(&fetch_args)
            .current_dir(&repo_path)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to fetch from origin: {}", error_msg));
        }
        
        tracing::debug!("Fetch completed, resetting to fetched commit");

        // Reset to FETCH_HEAD (what we just fetched) - this is guaranteed to work
        let reset_output = Command::new("git")
            .args(["reset", "--hard", "FETCH_HEAD"])
            .current_dir(&repo_path)
            .output()?;

        if !reset_output.status.success() {
            let error_msg = String::from_utf8_lossy(&reset_output.stderr);
            return Err(anyhow!("Failed to reset to FETCH_HEAD after fetching '{}': {}", version, error_msg));
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

