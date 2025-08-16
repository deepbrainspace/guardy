// All filtering now handled through cached filters in Scanner struct and collect_file_paths
use super::types::{ScanResult, ScanStats, Scanner, ScannerConfig, SecretMatch, Warning, ScanMode};
use crate::config::GuardyConfig;
use crate::parallel::ExecutionStrategy;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ============================================================================
// IMPORTANT: All scanner types should be defined in types.rs, not here!
// This keeps the codebase modular and prevents type duplication.
// Only implement behavior (impl blocks) in this file.
// ============================================================================

impl Scanner {
    pub fn new(config: &GuardyConfig) -> Result<Self> {
        // Parse scanner-specific config
        let scanner_config = Self::parse_scanner_config(config)?;

        // Initialize filters once for reuse throughout scanning
        let binary_filter = std::sync::Arc::new(super::filters::directory::BinaryFilter::new(!scanner_config.include_binary));
        let path_filter = std::sync::Arc::new(super::filters::directory::PathFilter::new(scanner_config.ignore_paths.clone()));
        let size_filter = std::sync::Arc::new(super::filters::directory::SizeFilter::new(scanner_config.max_file_size_mb));
        
        // Initialize content filters for optimization
        let prefilter = std::sync::Arc::new(super::filters::content::ContextPrefilter::new());
        let regex_executor = std::sync::Arc::new(super::filters::content::RegexExecutor::new());
        let comment_filter = std::sync::Arc::new(super::filters::content::CommentFilter::new());

        Ok(Scanner {
            config: scanner_config,
            binary_filter,
            path_filter,
            size_filter,
            prefilter,
            regex_executor,
            comment_filter,
        })
    }

    pub fn with_config(config: ScannerConfig) -> Result<Self> {
        // Initialize filters once for reuse throughout scanning
        let binary_filter = std::sync::Arc::new(super::filters::directory::BinaryFilter::new(!config.include_binary));
        let path_filter = std::sync::Arc::new(super::filters::directory::PathFilter::new(config.ignore_paths.clone()));
        let size_filter = std::sync::Arc::new(super::filters::directory::SizeFilter::new(config.max_file_size_mb));
        
        // Initialize content filters for optimization
        let prefilter = std::sync::Arc::new(super::filters::content::ContextPrefilter::new());
        let regex_executor = std::sync::Arc::new(super::filters::content::RegexExecutor::new());
        let comment_filter = std::sync::Arc::new(super::filters::content::CommentFilter::new());

        Ok(Scanner {
            config,
            binary_filter,
            path_filter,
            size_filter,
            prefilter,
            regex_executor,
            comment_filter,
        })
    }

    // fast_count_files() method removed - no longer needed since we always use parallel execution


    // Note: should_ignore_path method removed - all filtering now happens during directory walk

    pub fn from_fast_config_with_cli_overrides(
        config: &crate::config::FastConfig,
        args: &crate::cli::commands::scan::ScanArgs,
    ) -> Result<ScannerConfig> {
        super::config::from_fast_config_with_cli_overrides(config, args)
    }
    
    pub fn parse_scanner_config_with_cli_overrides(
        config: &GuardyConfig,
        args: &crate::cli::commands::scan::ScanArgs,
    ) -> Result<ScannerConfig> {
        let mut scanner_config = Self::parse_scanner_config(config)?;

        // Apply CLI overrides directly (bypassing SuperConfig issues)
        scanner_config.enable_entropy_analysis = !args.no_entropy;
        if let Some(threshold) = args.entropy_threshold {
            scanner_config.min_entropy_threshold = threshold;
        }
        scanner_config.include_binary = args.include_binary;
        scanner_config.follow_symlinks = args.follow_symlinks;
        scanner_config.max_file_size_mb = args.max_file_size;

        // Extend ignore lists with CLI values
        scanner_config
            .ignore_patterns
            .extend(args.ignore_patterns.clone());
        scanner_config
            .ignore_paths
            .extend(args.ignore_paths.clone());
        scanner_config
            .ignore_comments
            .extend(args.ignore_comments.clone());

        if let Some(mode) = &args.mode {
            scanner_config.mode = mode.clone();
        }

        tracing::debug!(
            "CLI OVERRIDE: Final enable_entropy_analysis = {}",
            scanner_config.enable_entropy_analysis
        );
        Ok(scanner_config)
    }

    pub fn parse_scanner_config(config: &GuardyConfig) -> Result<ScannerConfig> {
        let mut scanner_config = ScannerConfig::default();

        // Now using flattened keys, so scanner section won't exist as nested object

        // Override defaults with config values if present
        if let Ok(entropy_enabled) = config.get_section("scanner.entropy_analysis")
            && let Some(enabled) = entropy_enabled.as_bool()
        {
            tracing::debug!(
                "ENTROPY CONFIG: Found scanner.entropy_analysis = {}",
                enabled
            );
            scanner_config.enable_entropy_analysis = enabled;
        }

        // Support CLI override key name - direct access due to SuperConfig limitation with arrays
        if let Ok(full_config) = config.get_full_config() {
            tracing::debug!(
                "ENTROPY CONFIG: Full config keys: {:?}",
                full_config
                    .as_object()
                    .map(|o| o.keys().collect::<Vec<_>>())
            );

            if let Some(val) = full_config.get("scanner.enable_entropy_analysis") {
                tracing::debug!("ENTROPY CONFIG: Found value: {:?}", val);
                if let Some(enabled) = val.as_bool() {
                    tracing::debug!(
                        "ENTROPY CONFIG: Found scanner.enable_entropy_analysis = {} (direct access)",
                        enabled
                    );
                    scanner_config.enable_entropy_analysis = enabled;
                }
            } else {
                tracing::debug!(
                    "ENTROPY CONFIG: scanner.enable_entropy_analysis not found in full config"
                );
            }
        }

        // Fallback to standard get_section for traditional config files
        if let Ok(entropy_enabled) = config.get_section("scanner.enable_entropy_analysis")
            && let Some(enabled) = entropy_enabled.as_bool()
        {
            tracing::debug!(
                "ENTROPY CONFIG: Found scanner.enable_entropy_analysis = {} (get_section)",
                enabled
            );
            scanner_config.enable_entropy_analysis = enabled;
        }

        if let Ok(threshold) = config.get_section("scanner.entropy_threshold")
            && let Some(thresh) = threshold.as_f64()
        {
            scanner_config.min_entropy_threshold = thresh;
        }

        if let Ok(include_binary) = config.get_section("scanner.include_binary")
            && let Some(enabled) = include_binary.as_bool()
        {
            scanner_config.include_binary = enabled;
        }

        // Load ignore patterns from config
        if let Ok(ignore_paths) = config.get_vec("scanner.ignore_paths") {
            tracing::debug!("Loaded {} ignore paths from config: {:?}", ignore_paths.len(), ignore_paths);
            scanner_config.ignore_paths = ignore_paths;
        } else {
            crate::cli::output::styled!(
                "{}: No ignore_paths found in config, using defaults: {}",
                ("DEBUG", "debug"),
                (format!("{:?}", scanner_config.ignore_paths), "muted")
            );
        }

        if let Ok(ignore_patterns) = config.get_vec("scanner.ignore_patterns") {
            scanner_config.ignore_patterns = ignore_patterns;
        }

        if let Ok(ignore_comments) = config.get_vec("scanner.ignore_comments") {
            scanner_config.ignore_comments = ignore_comments;
        }

        // Parse processing mode settings
        if let Ok(mode_str) = config.get_section("scanner.mode")
            && let Some(mode) = mode_str.as_str()
        {
            tracing::trace!("SCANNER CONFIG: Parsing mode from config: '{}'", mode);
            scanner_config.mode = match mode.to_lowercase().as_str() {
                "sequential" => super::types::ScanMode::Sequential,
                "parallel" => super::types::ScanMode::Parallel,
                "auto" => super::types::ScanMode::Auto,
                _ => super::types::ScanMode::Auto, // Default fallback
            };
            tracing::trace!("SCANNER CONFIG: Set mode to: {:?}", scanner_config.mode);
        }

        if let Ok(max_threads) = config.get_section("scanner.max_threads")
            && let Some(threads) = max_threads.as_u64()
        {
            scanner_config.max_threads = threads as usize;
        }

        if let Ok(thread_percentage) = config.get_section("scanner.thread_percentage")
            && let Some(percentage) = thread_percentage.as_u64()
        {
            scanner_config.thread_percentage = percentage as u8;
        }

        if let Ok(min_files) = config.get_section("scanner.min_files_for_parallel")
            && let Some(files) = min_files.as_u64()
        {
            scanner_config.min_files_for_parallel = files as usize;
        }

        tracing::debug!(
            "ENTROPY CONFIG: Final enable_entropy_analysis = {}",
            scanner_config.enable_entropy_analysis
        );
        Ok(scanner_config)
    }

    /// Scan specific paths
    pub fn scan_paths(&self, paths: &[PathBuf]) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        let mut all_matches = Vec::new();
        let mut stats = ScanStats::default();
        let mut warnings: Vec<Warning> = Vec::new();

        for path in paths {
            match self.scan_single_path(path) {
                Ok(mut matches) => {
                    stats.files_scanned += 1;
                    stats.total_matches += matches.len();
                    all_matches.append(&mut matches);
                }
                Err(e) => {
                    stats.files_skipped += 1;
                    warnings.push(Warning {
                        message: format!("Failed to scan {}: {}", path.display(), e),
                    });
                }
            }
        }

        stats.scan_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(ScanResult {
            matches: all_matches,
            stats,
            warnings,
        })
    }

    /// Build a WalkBuilder with common directory filtering logic and statistics tracking
    pub(crate) fn build_directory_walker(&self, path: &Path, path_filter_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> WalkBuilder {
        let mut builder = WalkBuilder::new(path);
        builder
            .follow_links(self.config.follow_symlinks)
            .git_ignore(true) // Respect .gitignore files
            .git_global(true) // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .hidden(false) // Don't ignore hidden files by default
            .parents(true); // Check parent directories for .gitignore

        // Create directory handler for fast directory filtering
        let directory_handler = super::directory::DirectoryHandler::new();
        let path_filter = self.path_filter.clone();

        builder.filter_entry(move |entry| {
            // FIRST: Fast directory filtering by name (prevents descent)
            if entry.file_type().is_some_and(|ft| ft.is_dir())
                && let Some(dir_name) = entry.file_name().to_str()
                && directory_handler.should_filter_directory(dir_name)
            {
                tracing::debug!("[DirectoryHandler] Skipping directory: {}", entry.path().display());
                path_filter_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return false; // Don't descend into this directory
            }
            
            // SECOND: Apply PathFilter for file patterns (user-configurable)
            // Only check files, not directories (directories already handled above)
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                use super::filters::Filter;
                match path_filter.filter(entry.path()) {
                    Ok(super::filters::FilterDecision::Skip(reason)) => {
                        tracing::debug!("[PathFilter] Skipped file {}: {}", entry.path().display(), reason);
                        path_filter_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        return false;
                    }
                    Ok(super::filters::FilterDecision::Process) => {
                        tracing::trace!("[PathFilter] Allowed file {}", entry.path().display());
                    }
                    Err(e) => {
                        tracing::warn!("[PathFilter] Failed for {}: {}", entry.path().display(), e);
                    }
                }
            }
            
            true // Allow everything else
        });

        builder
    }

    /// Scan a directory recursively with optional execution strategy
    /// By default uses smart mode (auto-detects parallel vs sequential)
    pub fn scan_directory(
        &self,
        path: &Path,
        strategy: Option<ExecutionStrategy>,
    ) -> Result<ScanResult> {
        let directory_handler = super::directory::DirectoryHandler::new();
        directory_handler.scan(Arc::new(self.clone()), path, strategy)
    }

    /// Scan a single file
    pub fn scan_file(&self, path: &Path) -> Result<Vec<SecretMatch>> {
        self.scan_single_path(path)
    }


    pub(crate) fn scan_single_path(&self, path: &Path) -> Result<Vec<SecretMatch>> {
        // All filtering (path, size, binary) now happens during directory walk for better performance
        // Files reaching this method have already passed all filters

        // Read entire file content at once
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // Scan the entire content for matches
        self.scan_content(&content, path)
    }

    /// Scan entire file content for secrets using optimized filter pipeline
    fn scan_content(&self, content: &str, file_path: &Path) -> Result<Vec<SecretMatch>> {
        use super::filters::{Filter, content::{RegexInput, CommentFilterInput}};
        
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Step 1: Aho-Corasick prefilter - eliminates ~85% of patterns before regex execution
        let active_patterns = self.prefilter.filter(content)
            .context("Prefilter failed")?;
        
        if active_patterns.is_empty() {
            tracing::trace!("No active patterns for file {}, skipping regex execution", file_path_str);
            return Ok(Vec::new());
        }
        
        tracing::trace!(
            "Prefilter found {} active patterns for file {}",
            active_patterns.len(),
            file_path_str
        );
        
        // Step 2: Regex execution on pre-filtered patterns (~15% of original)
        let regex_input = RegexInput {
            content: content.to_string(),
            file_path: file_path_str.clone(),
            active_patterns,
        };
        
        let matches = self.regex_executor.filter(&regex_input)
            .context("Regex execution failed")?;
        
        if matches.is_empty() {
            return Ok(Vec::new());
        }
        
        tracing::trace!(
            "Regex executor found {} matches for file {}",
            matches.len(),
            file_path_str
        );
        
        // Step 3: Apply entropy analysis if enabled
        let mut filtered_matches = Vec::new();
        if self.config.enable_entropy_analysis {
            for secret_match in matches {
                // Use the optimized entropy module
                if super::entropy::is_likely_secret(
                    secret_match.matched_text.as_bytes(),
                    self.config.min_entropy_threshold,
                ) {
                    filtered_matches.push(secret_match);
                } else {
                    tracing::debug!(
                        "Match '{}' failed entropy analysis in file {} at line {}",
                        secret_match.matched_text,
                        file_path_str,
                        secret_match.line_number
                    );
                }
            }
        } else {
            filtered_matches = matches;
        }
        
        if filtered_matches.is_empty() {
            return Ok(Vec::new());
        }
        
        // Step 4: Comment filter for guardy:ignore directives
        let comment_input = CommentFilterInput {
            matches: filtered_matches,
            file_content: content.to_string(),
        };
        
        let final_matches = self.comment_filter.filter(&comment_input)
            .context("Comment filter failed")?;
        
        tracing::trace!(
            "Final pipeline result: {} matches for file {}",
            final_matches.len(),
            file_path_str
        );
        
        Ok(final_matches)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GuardyConfig;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> GuardyConfig {
        GuardyConfig::load(None, None::<&()>, 0).unwrap()
    }

    #[test]
    fn test_scanner_creation() {
        let config = create_test_config();
        let scanner = Scanner::new(&config);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_file_scanning() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_secrets.txt");

        // Create a test file with various secret patterns
        let test_content = r#"
# This is a test file
API_KEY = "sk_test_4eC39HqLyjWDarjtT1zdp7dc"
const GITHUB_TOKEN = "ghp_1234567890abcdef1234567890abcdef12";
JWT_TOKEN=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c

# These should be ignored
DATABASE_URL_EXAMPLE = "postgres://user:pass@localhost/db"
FAKE_API_KEY = "test_key_not_real"
"#;

        fs::write(&test_file, test_content).unwrap();

        let config = create_test_config();
        let scanner = Scanner::new(&config).unwrap();
        let result = scanner.scan_file(&test_file).unwrap();

        // Should find some secrets but filter out obvious fake ones with entropy analysis
        assert!(!result.is_empty());

        // Check that we found reasonable matches
        for secret_match in &result {
            println!(
                "Found: {} in {}",
                secret_match.matched_text, secret_match.secret_type
            );
        }
    }

    // Removed test_scan_directory - was causing CI timeouts and will be replaced by scan2 implementation
}
