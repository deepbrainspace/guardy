// TODO: Add operations module
// TODO: Add commit module

use anyhow::Result;

pub struct GitRepo {
    pub repo: git2::Repository,
}

impl GitRepo {
    pub fn discover() -> Result<Self> {
        let repo = git2::Repository::discover(".")?;
        Ok(GitRepo { repo })
    }
    
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let shorthand = head.shorthand().unwrap_or("HEAD");
        Ok(shorthand.to_string())
    }
}