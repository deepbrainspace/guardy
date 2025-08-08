use anyhow::{anyhow, Result};
use git2::{Repository, RepositoryOpenFlags};
use std::path::{Path, PathBuf};
use super::GitRepo;

pub struct RemoteOperations {
    cache_dir: PathBuf,
}

impl RemoteOperations {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Clone or fetch a repository to the cache directory
    pub fn clone_or_fetch(&self, repo_url: &str, version: &str) -> Result<PathBuf> {
        let repo_name = self.extract_repo_name(repo_url);
        let repo_path = self.cache_dir.join(&repo_name);

        if repo_path.exists() {
            // Repository exists, try to fetch and checkout
            tracing::info!("Fetching updates for {} at {}", repo_name, repo_path.display());
            let git_repo = GitRepo::open(&repo_path)?;
            git_repo.fetch_and_checkout(version)?;
        } else {
            // Clone the repository
            tracing::info!("Cloning {} to {}", repo_url, repo_path.display());
            let git_repo = GitRepo::clone(repo_url, &repo_path)?;
            git_repo.checkout_version(version)?;
        }

        Ok(repo_path)
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

impl GitRepo {
    /// Open an existing repository
    pub fn open(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open_ext(
            repo_path,
            RepositoryOpenFlags::empty(),
            &[] as &[&std::ffi::OsStr],
        )?;
        Ok(GitRepo { repo })
    }

    /// Clone a repository from URL
    pub fn clone(repo_url: &str, repo_path: &Path) -> Result<Self> {
        // Convert various URL formats to git URLs
        let git_url = if repo_url.starts_with("http") || repo_url.starts_with("git@") {
            repo_url.to_string()
        } else {
            // Assume it's github.com/org/repo format
            format!("https://{repo_url}.git")
        };

        tracing::info!("Cloning from: {}", git_url);

        let repo = Repository::clone(&git_url, repo_path)?;
        Ok(GitRepo { repo })
    }

    /// Fetch from origin and checkout a specific version
    pub fn fetch_and_checkout(&self, version: &str) -> Result<()> {
        // Fetch from origin
        let mut remote = self.repo.find_remote("origin")?;
        let refspecs = remote.fetch_refspecs()?;
        let refspecs: Vec<&str> = refspecs.iter().flatten().collect();
        
        remote.fetch(&refspecs, None, None)?;
        
        self.checkout_version(version)?;
        Ok(())
    }

    /// Checkout a specific version (tag, branch, or commit)
    pub fn checkout_version(&self, version: &str) -> Result<()> {
        // Try to find the version as a tag, branch, or commit
        let object = if let Ok(reference) = self.repo.find_reference(&format!("refs/tags/{version}")) {
            reference.peel_to_commit()?.into_object()
        } else if let Ok(reference) = self.repo.find_reference(&format!("refs/heads/{version}")) {
            reference.peel_to_commit()?.into_object()
        } else if let Ok(reference) = self.repo.find_reference(&format!("refs/remotes/origin/{version}")) {
            reference.peel_to_commit()?.into_object()
        } else if let Ok(oid) = git2::Oid::from_str(version) {
            self.repo.find_object(oid, None)?
        } else {
            return Err(anyhow!("Could not find version '{}' in repository", version));
        };

        self.repo.checkout_tree(&object, None)?;
        self.repo.set_head_detached(object.id())?;

        tracing::info!("Checked out version: {}", version);
        Ok(())
    }
}