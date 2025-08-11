use crate::scan::types::{ScannerConfig, Warning};
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Directory - Handles all file system operations
///
/// Responsibilities:
/// - Directory traversal and walking
/// - File path collection  
/// - Directory analysis and warnings
/// - Fast file counting
/// - Gitignore integration
pub struct Directory {
    config: ScannerConfig,
}

impl Directory {
    /// Create a new Directory handler with configuration
    pub fn new(config: &ScannerConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Fast count files without full traversal
    pub fn fast_count_files(paths: &[String]) -> Result<usize> {
        // TODO: Implement lightweight file counting
        // - Quick traversal without full WalkBuilder setup
        // - Basic directory filtering only
        Ok(100) // Placeholder
    }

    /// Analyze directory structure and generate warnings
    pub fn analyze_paths(paths: &[String]) -> Result<Vec<Warning>> {
        // TODO: Implement directory analysis
        // - Check for large ignored directories
        // - Suggest .gitignore improvements
        // - Identify performance bottlenecks
        Ok(vec![]) // Placeholder
    }

    /// Collect all file paths for scanning
    pub fn collect_file_paths(paths: &[String], config: &ScannerConfig) -> Result<Vec<PathBuf>> {
        let mut all_paths = Vec::new();
        
        for path_str in paths {
            let path = Path::new(path_str);
            
            if path.is_file() {
                // Single file - add directly
                all_paths.push(path.to_path_buf());
            } else if path.is_dir() {
                // Directory - use WalkBuilder for traversal
                let walker = Self::build_walker(path, config);
                
                for entry in walker {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                                all_paths.push(entry.path().to_path_buf());
                            }
                        }
                        Err(e) => {
                            // Log but don't fail on permission errors
                            tracing::debug!("Error accessing path: {}", e);
                        }
                    }
                }
            }
        }
        
        Ok(all_paths)
    }

    /// Build a WalkBuilder with proper gitignore and filtering
    fn build_walker(path: &Path, config: &ScannerConfig) -> ignore::Walk {
        let mut builder = WalkBuilder::new(path);
        
        // Respect gitignore files
        builder
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true);
        
        // Follow symlinks if configured
        builder.follow_links(config.follow_symlinks);
        
        // Add standard exclusions (node_modules, target, etc.)
        Self::add_standard_exclusions(&mut builder);
        
        builder.build()
    }

    /// Add standard directory exclusions for performance
    fn add_standard_exclusions(builder: &mut WalkBuilder) {
        // Comprehensive exclusion list organized by category (from existing scanner)
        let rust_dirs = ["target"];
        let nodejs_dirs = ["node_modules", "dist", "build", ".next", ".nuxt"];
        let python_dirs = ["__pycache__", ".pytest_cache", "venv", ".venv", "env", ".env"];
        let go_dirs = ["vendor"];
        let java_dirs = ["out"];
        let generic_dirs = ["cache", ".cache", "tmp", ".tmp", "temp", ".temp"];
        let vcs_dirs = [".git", ".svn", ".hg"];
        let ide_dirs = [".vscode", ".idea", ".vs"];
        let coverage_dirs = ["coverage", ".nyc_output"];
        
        // Combine all exclusions
        let mut all_exclusions = Vec::new();
        all_exclusions.extend_from_slice(&rust_dirs);
        all_exclusions.extend_from_slice(&nodejs_dirs);
        all_exclusions.extend_from_slice(&python_dirs);
        all_exclusions.extend_from_slice(&go_dirs);
        all_exclusions.extend_from_slice(&java_dirs);
        all_exclusions.extend_from_slice(&generic_dirs);
        all_exclusions.extend_from_slice(&vcs_dirs);
        all_exclusions.extend_from_slice(&ide_dirs);
        all_exclusions.extend_from_slice(&coverage_dirs);
        
        // Add each pattern as an ignore rule
        for pattern in all_exclusions {
            builder.add_ignore(format!("**/{}", pattern));
        }
    }

    /// Scan specific paths (called by Core::scan())
    pub fn scan_paths(&self, paths: &[String]) -> Result<Vec<PathBuf>> {
        Self::collect_file_paths(paths, &self.config)
    }
}