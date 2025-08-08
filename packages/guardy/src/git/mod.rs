pub mod operations;
pub mod remote;
// TODO: Add hooks module for hook installation/management
// TODO: Add commit module for commit operations

use anyhow::Result;
use git2::Repository;
use std::path::Path;

pub struct GitRepo {
    pub repo: Repository,
}

impl GitRepo {
    pub fn discover() -> Result<Self> {
        let repo = Repository::discover(".")?;
        Ok(GitRepo { repo })
    }
    
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let shorthand = head.shorthand().unwrap_or("HEAD");
        Ok(shorthand.to_string())
    }
    
    pub fn git_dir(&self) -> &Path {
        self.repo.path()
    }
}