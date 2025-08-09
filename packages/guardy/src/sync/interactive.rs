use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use std::fs;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use crate::cli::output;
use crate::sync::{SyncStatus, manager::SyncManager};

#[derive(Debug, Clone)]
pub enum FileAction {
    Update,
    Skip,
    Quit,
    UpdateAll,
    SkipAll,
}

pub struct InteractiveUpdater {
    manager: SyncManager,
    theme: ColorfulTheme,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl InteractiveUpdater {
    pub fn new(manager: SyncManager) -> Self {
        Self {
            manager,
            theme: ColorfulTheme::default(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        output::styled!("{} Analyzing sync status...", ("📋", "info_symbol"));
        
        let changed_files = self.get_changed_files()?;
        if changed_files.is_empty() {
            output::styled!("{} All files are already in sync", ("✅", "success_symbol"));
            return Ok(());
        }

        self.show_file_overview(&changed_files);
        self.process_files_interactively(changed_files).await
    }

    fn get_changed_files(&self) -> Result<Vec<PathBuf>> {
        match self.manager.check_sync_status()? {
            SyncStatus::InSync => Ok(vec![]),
            SyncStatus::OutOfSync { changed_files } => Ok(changed_files),
            SyncStatus::NotConfigured => {
                output::styled!("{} No sync configuration found", ("⚠️", "warning_symbol"));
                output::styled!("Run {} to bootstrap", 
                    ("guardy sync update --repo=<url> --version=<version>", "property"));
                Ok(vec![])
            }
        }
    }

    fn show_file_overview(&self, files: &[PathBuf]) {
        output::styled!("\n{} Files that have drifted from source:", ("📋", "info_symbol"));
        for (i, file) in files.iter().enumerate() {
            let protection_status = if self.manager.protection_manager.is_protected(file) {
                " 🔒"
            } else {
                ""
            };
            output::styled!("  {}. {}{}", 
                ((i + 1).to_string(), "muted"),
                (file.display().to_string(), "property"),
                (protection_status, "info_symbol")
            );
        }
    }


    async fn process_files_interactively(&mut self, files: Vec<PathBuf>) -> Result<()> {
        let mut updated_count = 0;
        let mut skipped_count = 0;
        let mut i = 0;

        while i < files.len() {
            let file = &files[i];
            
            println!();
            output::styled!("{}", ("─".repeat(60), "muted"));
            output::styled!("File {}/{}: {}", 
                ((i + 1).to_string(), "muted"),
                (files.len().to_string(), "muted"), 
                (file.display().to_string(), "property"));

            let (protection_status, status_style) = if self.manager.protection_manager.is_protected(file) {
                ("Protected file 🔒", "warning_symbol")
            } else {
                ("Unprotected file", "muted")
            };
            output::styled!("Status: {}", (protection_status, status_style));

            // Show diff immediately
            self.show_diff(file)?;

            match self.prompt_file_action(file)? {
                FileAction::Update => {
                    self.update_file(file)?;
                    updated_count += 1;
                    i += 1;
                },
                FileAction::Skip => {
                    output::styled!("{} Skipped {}", 
                        ("⏭️", "info_symbol"),
                        (file.display().to_string(), "property"));
                    skipped_count += 1;
                    i += 1;
                },
                FileAction::Quit => {
                    output::styled!("{} Update cancelled by user", ("ℹ️", "info_symbol"));
                    break;
                },
                FileAction::UpdateAll => {
                    let remaining = files.len() - i;
                    self.update_remaining_files(&files[i..])?;
                    updated_count += remaining;
                    break;
                },
                FileAction::SkipAll => {
                    let remaining = files.len() - i;
                    output::styled!("{} Skipped {} remaining files", 
                        ("⏭️", "info_symbol"),
                        (remaining.to_string(), "property"));
                    skipped_count += remaining;
                    break;
                },
            }
        }

        self.show_summary(updated_count, skipped_count);
        Ok(())
    }

    fn prompt_file_action(&self, _file: &Path) -> Result<FileAction> {
        let options = vec![
            "Yes - Update this file",
            "No - Skip this file", 
            "Quit - Stop processing",
            "Yes to all remaining files",
            "Skip all remaining files"
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => FileAction::Update,
            1 => FileAction::Skip,
            2 => FileAction::Quit,
            3 => FileAction::UpdateAll,
            4 => FileAction::SkipAll,
            _ => unreachable!(),
        })
    }

    fn show_diff(&self, file: &Path) -> Result<()> {
        // Get source and destination file contents
        let dest_content = fs::read_to_string(file).unwrap_or_default();

        // Get source content from cached repo
        let source_content = self.get_source_file_content(file)?;

        let repo_name = if let Some(repo) = self.manager.get_config().repos.first() {
            &repo.name
        } else {
            "unknown"
        };
        
        println!();
        output::styled!("{}", ("─".repeat(60), "muted"));
        output::styled!("● Update({}) - {}", 
            (repo_name, "info_symbol"),
            (file.display().to_string(), "property"));
        output::styled!("{}", ("─".repeat(60), "muted"));
        
        // Generate and show unified diff
        self.show_unified_diff(&source_content, &dest_content, file)?;
        
        Ok(())
    }

    fn get_source_file_content(&self, file: &Path) -> Result<String> {
        // Get actual source content from cached repo
        if let Some(repo) = self.manager.get_config().repos.first() {
            let repo_name = self.manager.extract_repo_name(&repo.repo);
            let cached_file = self.manager.get_cache_dir().join(&repo_name).join(file);
            match fs::read_to_string(&cached_file) {
                Ok(content) => Ok(content),
                Err(_) => Ok("[Source file not found in cache]".to_string()),
            }
        } else {
            Ok("[No repository configured]".to_string())
        }
    }

    fn show_unified_diff(&self, source: &str, dest: &str, file: &Path) -> Result<()> {
        let diff = TextDiff::from_lines(dest, source);
        
        // Set up syntax highlighting
        let syntax = self.syntax_set
            .find_syntax_for_file(file)?
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes["Solarized (dark)"];
        
        for group in diff.grouped_ops(3) {
            for op in group {
                for change in diff.iter_changes(&op) {
                    let line_number = change.new_index().unwrap_or(0);
                    let line_content = change.value().trim_end_matches('\n');
                    let mut highlighter = HighlightLines::new(syntax, theme);
                    
                    match change.tag() {
                        ChangeTag::Delete => {
                            // Claude Code red: RGB(120, 50, 50) background with white text
                            print!("\x1b[48;2;120;50;50;97m{line_number:>8} -  ");
                            if let Ok(ranges) = highlighter.highlight_line(line_content, &self.syntax_set) {
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                                println!("{escaped}\x1b[0m");
                            } else {
                                println!("{line_content}\x1b[0m");
                            }
                        },
                        ChangeTag::Insert => {
                            // Claude Code green: RGB(50, 120, 50) background with white text
                            print!("\x1b[48;2;50;120;50;97m{line_number:>8} +  ");
                            if let Ok(ranges) = highlighter.highlight_line(line_content, &self.syntax_set) {
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                                println!("{escaped}\x1b[0m");
                            } else {
                                println!("{line_content}\x1b[0m");
                            }
                        },
                        ChangeTag::Equal => {
                            // Context lines with normal formatting + syntax highlighting
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

    fn update_file(&mut self, file: &Path) -> Result<()> {
        // Get source content from cached repo
        let source_content = self.get_source_file_content(file)?;
        
        // Create parent directories if needed
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the source content to the destination file
        fs::write(file, source_content)?;
        
        output::styled!("{} Updated {}", 
            ("✅", "success_symbol"),
            (file.display().to_string(), "property"));
        Ok(())
    }

    fn update_remaining_files(&mut self, files: &[PathBuf]) -> Result<()> {
        output::styled!("{} Updating {} remaining files...", 
            ("⚡", "info_symbol"),
            (files.len().to_string(), "property"));
        
        for file in files {
            self.update_file(file)?;
        }
        
        Ok(())
    }

    fn show_summary(&self, updated_count: usize, skipped_count: usize) {
        println!();
        output::styled!("{}", ("═".repeat(60), "muted"));
        output::styled!("Summary: {} {} updated, {} {} skipped", 
            ("✅", "success_symbol"),
            (updated_count.to_string(), "property"),
            ("⏭️ ", "info_symbol"),  // Added space after emoji
            (skipped_count.to_string(), "property"));
    }
}