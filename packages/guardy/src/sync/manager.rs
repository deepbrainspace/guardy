use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::fs;
use ignore::WalkBuilder;
use dialoguer::{theme::ColorfulTheme, Select};
use similar::{ChangeTag, TextDiff};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use crate::config::GuardyConfig;
use crate::cli::output;
use crate::git::remote::RemoteOperations;
use super::{SyncConfig, SyncStatus, SyncRepo};

pub struct SyncManager {
    pub config: SyncConfig,
    cache_dir: PathBuf,
    remote_ops: RemoteOperations,
    // For interactive mode
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

#[derive(Debug, Clone)]
enum FileAction {
    Update,
    Skip,
    UpdateAll,
    SkipAll,
    Quit,
}

impl SyncManager {
    pub fn with_config(sync_config: SyncConfig) -> Result<Self> {
        let cache_dir = PathBuf::from(".guardy/cache");
        std::fs::create_dir_all(&cache_dir)?;
        
        // Create .gitignore in .guardy directory to ignore all contents
        let guardy_gitignore = PathBuf::from(".guardy/.gitignore");
        if !guardy_gitignore.exists() {
            std::fs::write(&guardy_gitignore, "*\n")?;
        }
        
        let remote_ops = RemoteOperations::new(cache_dir.clone());

        Ok(Self { 
            config: sync_config, 
            cache_dir, 
            remote_ops,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        })
    }

    pub fn bootstrap(repo_url: &str, version: &str) -> Result<Self> {
        let sync_repo = SyncRepo {
            name: "bootstrap".to_string(), 
            repo: repo_url.to_string(), 
            version: version.to_string(),
            source_path: ".".to_string(), 
            dest_path: ".".to_string(),
            include: vec!["*".to_string()], 
            exclude: vec![".git".to_string()],
        };
        Self::with_config(SyncConfig { 
            repos: vec![sync_repo]
        })
    }

    /// Parse sync config from GuardyConfig
    pub fn parse_sync_config(config: &GuardyConfig) -> Result<SyncConfig> {
        let sync_value = config.get_section("sync")
            .map_err(|_| anyhow!("No sync configuration found"))?;
        
        let sync_config: SyncConfig = serde_json::from_value(sync_value)
            .map_err(|e| anyhow!("Failed to parse sync configuration: {}", e))?;
        
        Ok(sync_config)
    }

    /// Get files matching patterns using ignore crate  
    fn get_files(&self, source: &Path, repo: &SyncRepo) -> Result<Vec<PathBuf>> {
        let mut builder = WalkBuilder::new(source);
        
        // Disable automatic ignore file discovery - only use our custom patterns
        builder.standard_filters(false);
        
        // Create syncignore file in .guardy/ directory for patterns
        let syncignore_file = if !repo.exclude.is_empty() {
            let ignore_file = self.cache_dir.join(".syncignore");
            fs::write(&ignore_file, repo.exclude.join("\n"))?;
            // Copy the syncignore file to the source directory temporarily
            let source_ignore = source.join(".syncignore");
            fs::copy(&ignore_file, &source_ignore)?;
            builder.add_custom_ignore_filename(".syncignore");
            Some((ignore_file, source_ignore))
        } else {
            None
        };
        
        let result = builder.build()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| entry.path().strip_prefix(source).ok().map(|p| p.to_path_buf()))
            .filter(|path| path.file_name() != Some(".syncignore".as_ref())) // Filter out temp file
            .collect();
            
        // Cleanup syncignore files
        if let Some((ignore_file, source_ignore)) = syncignore_file {
            let _ = fs::remove_file(ignore_file);
            let _ = fs::remove_file(source_ignore);
        }
        
        Ok(result)
    }

    /// Check which files differ between source and destination
    fn files_differ(&self, files: &[PathBuf], src: &Path, dst: &Path) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        
        for f in files {
            let src_file = src.join(f);
            let dst_file = dst.join(f);
            
            tracing::trace!("Checking file: {:?}", f);
            tracing::trace!("  Source: {:?}", src_file);
            tracing::trace!("  Dest: {:?}", dst_file);
            
            if !dst_file.exists() {
                tracing::debug!("File {:?} doesn't exist in destination", f);
                changed.push(f.clone());
                continue;
            }
            
            let src_meta = match fs::metadata(&src_file) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to get metadata for source {:?}: {}", src_file, e);
                    continue;
                }
            };
            
            let dst_meta = match fs::metadata(&dst_file) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to get metadata for dest {:?}: {}", dst_file, e);
                    continue;
                }
            };
            
            let src_len = src_meta.len();
            let dst_len = dst_meta.len();
            
            if src_len != dst_len {
                tracing::debug!("File {:?} size differs: src={}, dst={}", f, src_len, dst_len);
                changed.push(f.clone());
            } else {
                tracing::trace!("File {:?} unchanged (size={})", f, src_len);
            }
        }
        
        changed
    }

    /// Update cache from remote repository using git pull
    fn update_cache(&self, repo: &SyncRepo) -> Result<PathBuf> {
        let repo_name = self.extract_repo_name(&repo.repo);
        let repo_path = self.cache_dir.join(&repo_name);

        if !repo_path.exists() {
            // Clone if doesn't exist - pass the version we actually want
            self.remote_ops.clone_repository(&repo.repo, &repo_name, &repo.version)?;
        } else {
            // Only fetch and reset if repo already exists
            self.remote_ops.fetch_and_reset(&repo_name, &repo.version)?;
        }
        
        Ok(repo_path)
    }

    /// Copy a single file from source to destination
    fn copy_file(&self, file: &Path, src: &Path, dst: &Path) -> Result<PathBuf> {
        let src_file = src.join(file);
        let dst_file = dst.join(file);
        if let Some(parent) = dst_file.parent() { 
            fs::create_dir_all(parent)?; 
        }
        fs::copy(&src_file, &dst_file)?;
        Ok(dst_file)
    }

    /// Check sync status of all repositories
    pub fn check_sync_status(&self) -> Result<SyncStatus> {
        if self.config.repos.is_empty() { 
            return Ok(SyncStatus::NotConfigured); 
        }
        
        let mut changed_files = Vec::new();
        for repo in &self.config.repos {
            let repo_path = self.cache_dir.join(self.extract_repo_name(&repo.repo));
            if repo_path.exists() {
                let src = repo_path.join(&repo.source_path);
                let dst = Path::new(&repo.dest_path);
                let files = self.get_files(&src, repo)?;
                let different = self.files_differ(&files, &src, dst);
                // Convert to absolute paths for display
                changed_files.extend(different.iter().map(|f| dst.join(f)));
            }
        }
        
        if changed_files.is_empty() {
            Ok(SyncStatus::InSync)
        } else {
            Ok(SyncStatus::OutOfSync { changed_files })
        }
    }

    /// Main update function that handles both interactive and force modes
    pub async fn update_all_repos(&mut self, interactive: bool) -> Result<Vec<PathBuf>> {
        let mut all_updated_files = Vec::new();
        let mut all_skipped_files = Vec::new();
        let mut update_all_remaining = false;
        let mut skip_all_remaining = false;

        output::styled!("<chart> Analyzing sync status...");

        // First check if there are any changes at all
        let mut has_any_changes = false;

        for repo in self.config.repos.clone() {
            tracing::info!("Processing repository: {}", repo.name);
            
            // Update cache from remote
            let repo_path = self.update_cache(&repo)?;
            
            // Get changed files
            let src = repo_path.join(&repo.source_path);
            let dst = Path::new(&repo.dest_path);
            let files = self.get_files(&src, &repo)?;
            tracing::debug!("Found {} files in source", files.len());
            let changed_files = self.files_differ(&files, &src, dst);
            tracing::debug!("Found {} changed files", changed_files.len());
            
            if changed_files.is_empty() {
                tracing::info!("No changes detected for repository: {}", repo.name);
                continue;
            }

            has_any_changes = true;

            // Show repository info
            output::styled!("\n{} Repository: {} ({} files changed)", 
                ("🔗", "info_symbol"),
                (&repo.name, "property"),
                (changed_files.len().to_string(), "property")
            );

            // Process each changed file
            for (i, file) in changed_files.iter().enumerate() {
                let dst_file = dst.join(file);
                
                // If we're in "update all" or "skip all" mode, handle accordingly
                if skip_all_remaining {
                    output::styled!("{} Skipped {}", 
                        ("⏭️", "info_symbol"),
                        (dst_file.display().to_string(), "property"));
                    all_skipped_files.push(dst_file.clone());
                    continue;
                }
                
                if update_all_remaining || !interactive {
                    // In force mode or "update all" mode, just update
                    self.copy_file(file, &src, dst)?;
                    all_updated_files.push(dst_file.clone());
                    if interactive {
                        output::styled!("{} Updated {}", 
                            ("✅", "success_symbol"),
                            (dst_file.display().to_string(), "property"));
                    }
                    continue;
                }

                // Interactive mode: show diff and ask
                println!();
                output::styled!("{}", ("─".repeat(60), "muted"));
                output::styled!("File {}/{}: {}", 
                    ((i + 1).to_string(), "muted"),
                    (changed_files.len().to_string(), "muted"), 
                    (dst_file.display().to_string(), "property"));

                // Show diff
                self.show_diff(&dst_file, &src.join(file))?;

                // Ask user what to do
                match self.prompt_file_action()? {
                    FileAction::Update => {
                        self.copy_file(file, &src, dst)?;
                        all_updated_files.push(dst_file.clone());
                        output::styled!("{} Updated {}", 
                            ("✅", "success_symbol"),
                            (dst_file.display().to_string(), "property"));
                    },
                    FileAction::Skip => {
                        output::styled!("{} Skipped {}", 
                            ("⏭️", "info_symbol"),
                            (dst_file.display().to_string(), "property"));
                        all_skipped_files.push(dst_file.clone());
                    },
                    FileAction::UpdateAll => {
                        self.copy_file(file, &src, dst)?;
                        all_updated_files.push(dst_file.clone());
                        output::styled!("{} Updated {}", 
                            ("✅", "success_symbol"),
                            (dst_file.display().to_string(), "property"));
                        update_all_remaining = true;
                    },
                    FileAction::SkipAll => {
                        output::styled!("{} Skipped {}", 
                            ("⏭️", "info_symbol"),
                            (dst_file.display().to_string(), "property"));
                        all_skipped_files.push(dst_file.clone());
                        skip_all_remaining = true;
                    },
                    FileAction::Quit => {
                        output::styled!("{} Update cancelled by user", ("ℹ️", "info_symbol"));
                        return Ok(all_updated_files);
                    },
                }
            }
        }

        // If no changes at all, show message early
        if !has_any_changes {
            if interactive {
                println!();
                output::styled!("{}", ("═".repeat(60), "muted"));
                output::styled!("{} Everything is up to date", 
                    ("✅", "success_symbol"));
            }
            return Ok(all_updated_files);
        }

        // Show summary
        if interactive {
            println!();
            output::styled!("{}", ("═".repeat(60), "muted"));
            
            if all_updated_files.is_empty() && all_skipped_files.is_empty() {
                // Nothing was changed and nothing was skipped = truly up to date
                output::styled!("{} Everything is up to date", 
                    ("✅", "success_symbol"));
            } else if all_updated_files.is_empty() && !all_skipped_files.is_empty() {
                // Nothing updated but files were skipped = files remain out of sync
                output::styled!("{}  {} files remain out of sync (skipped by user)", 
                    ("⚠️", "warning_symbol"),
                    (all_skipped_files.len().to_string(), "property"));
            } else if !all_updated_files.is_empty() && all_skipped_files.is_empty() {
                // Files updated and nothing skipped = all changes applied
                output::styled!("{}  {} files updated", 
                    ("✅", "success_symbol"),
                    (all_updated_files.len().to_string(), "property"));
            } else {
                // Both updated and skipped files
                output::styled!("{}  {} files updated, {} files remain out of sync (skipped)", 
                    ("⚠️", "warning_symbol"),
                    (all_updated_files.len().to_string(), "property"),
                    (all_skipped_files.len().to_string(), "property"));
            }
        }

        Ok(all_updated_files)
    }

    /// Show diff between source and destination files
    fn show_diff(&self, dest_file: &Path, source_file: &Path) -> Result<()> {
        let dest_content = fs::read_to_string(dest_file).unwrap_or_default();
        let source_content = fs::read_to_string(source_file)?;
        
        println!();
        output::styled!("{}", ("─".repeat(60), "muted"));
        
        let diff = TextDiff::from_lines(&dest_content, &source_content);
        
        // Set up syntax highlighting
        let syntax = self.syntax_set
            .find_syntax_for_file(dest_file)?
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes["Solarized (dark)"];
        
        for group in diff.grouped_ops(3) {
            for op in group {
                for change in diff.iter_changes(&op) {
                    let line_content = change.value().trim_end_matches('\n');
                    let mut highlighter = HighlightLines::new(syntax, theme);
                    
                    match change.tag() {
                        ChangeTag::Delete => {
                            // For deletions, use the old line number
                            let line_number = change.old_index().unwrap_or(0);
                            print!("\x1b[48;2;100;40;40;97m{line_number:>8} -  ");
                            if let Ok(ranges) = highlighter.highlight_line(line_content, &self.syntax_set) {
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                                println!("{escaped}\x1b[0m");
                            } else {
                                println!("{line_content}\x1b[0m");
                            }
                        },
                        ChangeTag::Insert => {
                            // For insertions, use the new line number
                            let line_number = change.new_index().unwrap_or(0);
                            print!("\x1b[48;2;50;120;50;97m{line_number:>8} +  ");
                            if let Ok(ranges) = highlighter.highlight_line(line_content, &self.syntax_set) {
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                                println!("{escaped}\x1b[0m");
                            } else {
                                println!("{line_content}\x1b[0m");
                            }
                        },
                        ChangeTag::Equal => {
                            // Context lines - show the new line number (destination)
                            let line_number = change.new_index().unwrap_or(0);
                            print!("{line_number:>8}    ");
                            if let Ok(ranges) = highlighter.highlight_line(line_content, &self.syntax_set) {
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                                println!("{escaped}");
                            } else {
                                println!("{line_content}");
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Prompt user for action on a file
    fn prompt_file_action(&self) -> Result<FileAction> {
        let options = vec![
            "Yes - Update this file",
            "No - Skip this file", 
            "Yes to all remaining files",
            "Skip all remaining files",
            "Quit - Stop processing",
        ];

        println!(); // Add newline before prompt
        
        // Create a theme without the prompt prefix to avoid the extra "?"
        let theme = ColorfulTheme { 
            prompt_prefix: dialoguer::console::style("".to_string()), 
            ..Default::default() 
        };
        
        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => FileAction::Update,
            1 => FileAction::Skip,
            2 => FileAction::UpdateAll,
            3 => FileAction::SkipAll,
            4 => FileAction::Quit,
            _ => unreachable!(),
        })
    }

    /// Show all diffs without any interactive prompts (read-only view)
    pub async fn show_all_diffs(&mut self) -> Result<()> {
        output::styled!("<chart> Analyzing sync status...");

        let mut has_any_changes = false;

        for repo in self.config.repos.clone() {
            tracing::info!("Processing repository: {}", repo.name);
            
            // Update cache from remote
            let repo_path = self.update_cache(&repo)?;
            
            // Get changed files
            let src = repo_path.join(&repo.source_path);
            let dst = Path::new(&repo.dest_path);
            let files = self.get_files(&src, &repo)?;
            tracing::debug!("Found {} files in source", files.len());
            let changed_files = self.files_differ(&files, &src, dst);
            tracing::debug!("Found {} changed files", changed_files.len());
            
            if changed_files.is_empty() {
                tracing::info!("No changes detected for repository: {}", repo.name);
                continue;
            }

            has_any_changes = true;

            // Show repository info
            output::styled!("\n{} Repository: {} ({} files changed)", 
                ("🔗", "info_symbol"),
                (&repo.name, "property"),
                (changed_files.len().to_string(), "property")
            );

            // Show diff for each changed file (no prompts)
            for (i, file) in changed_files.iter().enumerate() {
                let dst_file = dst.join(file);
                
                println!();
                output::styled!("{}", ("─".repeat(60), "muted"));
                output::styled!("File {}/{}: {}", 
                    ((i + 1).to_string(), "muted"),
                    (changed_files.len().to_string(), "muted"), 
                    (dst_file.display().to_string(), "property"));

                // Show diff (no prompts)
                self.show_diff(&dst_file, &src.join(file))?;
            }
        }

        // Show summary
        if !has_any_changes {
            println!();
            output::styled!("{}", ("═".repeat(60), "muted"));
            output::styled!("{} Everything is up to date", 
                ("✅", "success_symbol"));
        } else {
            println!();
            output::styled!("{}", ("═".repeat(60), "muted"));
            output::styled!("{} Showing diffs for {} repositories", 
                ("📝", "info_symbol"),
                (self.config.repos.len().to_string(), "property"));
        }

        Ok(())
    }

    /// Get the cache directory
    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
    
    /// Extract repository name from URL
    pub fn extract_repo_name(&self, repo_url: &str) -> String {
        repo_url
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .split('/')
            .next_back()
            .unwrap_or("unknown")
            .to_string()
    }
}