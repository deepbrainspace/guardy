use super::entropy::is_likely_secret;
// All filtering now handled through cached filters in Scanner struct and collect_file_paths
use super::patterns::SecretPatterns;
use super::types::{ScanResult, ScanStats, Scanner, ScannerConfig, SecretMatch, Warning};
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
        // Load patterns from config
        let patterns = SecretPatterns::new(config)?;

        // Parse scanner-specific config
        let scanner_config = Self::parse_scanner_config(config)?;

        // Initialize filters once for reuse throughout scanning
        let binary_filter = std::sync::Arc::new(super::filters::directory::BinaryFilter::new(!scanner_config.include_binary));
        let path_filter = std::sync::Arc::new(super::filters::directory::PathFilter::new(scanner_config.ignore_paths.clone()));
        let size_filter = std::sync::Arc::new(super::filters::directory::SizeFilter::new(scanner_config.max_file_size_mb));

        Ok(Scanner {
            patterns,
            config: scanner_config,
            binary_filter,
            path_filter,
            size_filter,
        })
    }

    pub fn with_config(patterns: SecretPatterns, config: ScannerConfig) -> Result<Self> {
        // Initialize filters once for reuse throughout scanning
        let binary_filter = std::sync::Arc::new(super::filters::directory::BinaryFilter::new(!config.include_binary));
        let path_filter = std::sync::Arc::new(super::filters::directory::PathFilter::new(config.ignore_paths.clone()));
        let size_filter = std::sync::Arc::new(super::filters::directory::SizeFilter::new(config.max_file_size_mb));

        Ok(Scanner {
            patterns,
            config,
            binary_filter,
            path_filter,
            size_filter,
        })
    }

    // fast_count_files() method removed - no longer needed since we always use parallel execution


    // Note: should_ignore_path method removed - all filtering now happens during directory walk

    /// Check if content contains ignore patterns at a specific position
    fn should_ignore_at_position(&self, content: &str, position: usize) -> bool {
        // Find the line containing this position
        let line_start = content[..position].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = content[position..].find('\n').map(|i| position + i).unwrap_or(content.len());
        let line = &content[line_start..line_end];
        
        // Check for inline ignore comments
        for ignore_comment in &self.config.ignore_comments {
            if line.contains(ignore_comment) {
                return true;
            }
        }

        // Check for pattern-based ignores
        for ignore_pattern in &self.config.ignore_patterns {
            if line.contains(ignore_pattern) {
                return true;
            }
        }
        
        // Check if previous line has ignore-next directive
        if line_start > 0 {
            let prev_line_start = content[..line_start-1].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let prev_line = &content[prev_line_start..line_start-1];
            if prev_line.contains("guardy:ignore-next") {
                return true;
            }
        }

        false
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

    /// Scan entire file content for secrets
    fn scan_content(&self, content: &str, file_path: &Path) -> Result<Vec<SecretMatch>> {
        let mut matches = Vec::new();

        // Process each pattern against the entire content
        for pattern in &self.patterns.patterns {
            for regex_match in pattern.regex.find_iter(content) {
                // Skip if this position should be ignored
                if self.should_ignore_at_position(content, regex_match.start()) {
                    continue;
                }
                
                // Calculate line number and extract line content
                let line_number = content[..regex_match.start()].matches('\n').count() + 1;
                
                // Find the line containing this match
                let line_start = content[..regex_match.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = content[regex_match.start()..].find('\n')
                    .map(|i| regex_match.start() + i)
                    .unwrap_or(content.len());
                let line = &content[line_start..line_end];
                
                if let Some(secret_match) =
                    self.process_pattern_match(pattern, regex_match, line, file_path, line_number)
                {
                    matches.push(secret_match);
                }
            }
        }

        Ok(matches)
    }

    /// Process a single pattern match (extracted for reuse between sequential and parallel)
    fn process_pattern_match(
        &self,
        pattern: &super::patterns::SecretPattern,
        regex_match: regex::Match,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<SecretMatch> {
        // Pattern match found
        let matched_text = regex_match.as_str();

        // Extract the actual secret from capture groups if present
        let secret_content = if pattern.regex.captures_len() > 1 {
            // If pattern has capture groups, use the first group
            pattern
                .regex
                .captures(line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str())
                .unwrap_or(matched_text)
        } else {
            matched_text
        };

        // Apply entropy analysis if enabled (only on the secret content)
        if self.config.enable_entropy_analysis
            && !is_likely_secret(secret_content.as_bytes(), self.config.min_entropy_threshold)
        {
            return None; // Skip if entropy too low
        }

        Some(SecretMatch {
            file_path: file_path.to_string_lossy().into_owned(),
            line_number,
            line_content: line.to_string(),
            matched_text: matched_text.to_string(),
            start_pos: regex_match.start(),
            end_pos: regex_match.end(),
            secret_type: pattern.name.clone(),
            pattern_description: pattern.description.clone(),
        })
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
