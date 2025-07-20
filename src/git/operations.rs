use anyhow::{Result, Context};
use git2::{Status, StatusOptions, DiffOptions};
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
    
    /// Get list of unstaged modified files
    /// TODO: Will be used in pre-commit hooks to scan only modified files
    pub fn get_unstaged_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut status_opts = StatusOptions::new();
        status_opts.include_ignored(false);
        status_opts.include_untracked(true); // Include untracked files
        
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        let workdir = self.repo.workdir()
            .context("Repository has no working directory")?;
        
        for entry in statuses.iter() {
            let status = entry.status();
            
            // Check if file is modified in working tree but not staged
            if status.intersects(Status::WT_NEW | Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED | Status::WT_TYPECHANGE) {
                if let Some(path) = entry.path() {
                    files.push(workdir.join(path));
                }
            }
        }
        
        Ok(files)
    }
    
    /// Get all uncommitted files (staged + unstaged)
    /// TODO: Will be used in pre-commit hooks for comprehensive scanning
    pub fn get_uncommitted_files(&self) -> Result<Vec<PathBuf>> {
        let mut all_files = self.get_staged_files()?;
        all_files.extend(self.get_unstaged_files()?);
        
        // Remove duplicates
        all_files.sort();
        all_files.dedup();
        
        Ok(all_files)
    }
    
    /// Get files changed between two commits (useful for CI/CD)
    /// TODO: Will be used in CI/CD pipelines for scanning only changed files
    pub fn get_diff_files(&self, commit1: &str, commit2: &str) -> Result<Vec<PathBuf>> {
        // Get commits
        let commit1_obj = self.repo.revparse_single(commit1)?;
        let commit2_obj = self.repo.revparse_single(commit2)?;
        
        let commit1_tree = commit1_obj.peel_to_tree()?;
        let commit2_tree = commit2_obj.peel_to_tree()?;
        
        // Get diff
        let mut diff_opts = DiffOptions::new();
        diff_opts.context_lines(0); // We only need to know which files changed
        let diff = self.repo.diff_tree_to_tree(Some(&commit1_tree), Some(&commit2_tree), Some(&mut diff_opts))?;
        
        // Extract changed files
        let mut changed_files = Vec::new();
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(new_file) = delta.new_file().path() {
                    if let Ok(workdir) = self.repo.workdir().ok_or_else(|| git2::Error::from_str("No workdir")) {
                        changed_files.push(workdir.join(new_file));
                    }
                }
                true
            },
            None, None, None
        )?;
        
        Ok(changed_files)
    }
}