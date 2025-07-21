use anyhow::{Result, Context};
use git2::{Status, StatusOptions};
use std::path::PathBuf;
use super::GitRepo;

impl GitRepo {
    /// Get list of files that are staged for commit (primary use case for pre-commit hooks)
    pub fn get_staged_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut status_opts = StatusOptions::new();
        status_opts.include_ignored(false);
        status_opts.include_untracked(false);
        
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        let workdir = self.repo.workdir()
            .context("Repository has no working directory")?;
        
        for entry in statuses.iter() {
            let status = entry.status();
            
            // Check if file is staged
            if status.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED | Status::INDEX_RENAMED | Status::INDEX_TYPECHANGE) {
                if let Some(path) = entry.path() {
                    files.push(workdir.join(path));
                }
            }
        }
        
        Ok(files)
    }
    
    
    
}