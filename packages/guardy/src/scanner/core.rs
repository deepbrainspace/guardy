use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::parallel::ExecutionStrategy;
use ignore::WalkBuilder;
use crate::config::GuardyConfig;
use super::entropy::is_likely_secret;
use super::patterns::SecretPatterns;
use super::test_detection::TestDetector;
use super::types::{SecretMatch, ScanStats, Warning, ScanResult, Scanner, ScannerConfig};
use globset::{Glob, GlobSet, GlobSetBuilder};


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
        
        Ok(Scanner {
            patterns,
            config: scanner_config,
            cached_path_ignorer: std::sync::OnceLock::new(),
        })
    }
    
    pub fn with_config(patterns: SecretPatterns, config: ScannerConfig) -> Result<Self> {
        Ok(Scanner {
            patterns,
            config,
            cached_path_ignorer: std::sync::OnceLock::new(),
        })
    }
    

    /// Fast file counting using lightweight directory traversal
    /// This is much faster than full WalkBuilder traversal because it doesn't
    /// apply all the gitignore rules and filters - just basic directory filtering
    pub(crate) fn fast_count_files(&self, path: &Path) -> Result<usize> {
        use std::fs;
        
        let directory_handler = super::directory::DirectoryHandler::new();
        
        fn count_files_recursive(dir: &Path, config: &ScannerConfig, directory_handler: &super::directory::DirectoryHandler) -> Result<usize> {
            let mut count = 0;
            
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    if path.is_file() {
                        // Basic file size check (skip very large files)
                        if let Ok(metadata) = entry.metadata() {
                            let size_mb = metadata.len() / (1024 * 1024);
                            if size_mb <= config.max_file_size_mb as u64 {
                                count += 1;
                            }
                        } else {
                            count += 1; // Count if we can't get metadata
                        }
                    } else if path.is_dir() {
                        // Skip directories using shared filter logic
                        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                            if !directory_handler.should_filter_directory(dir_name) {
                                count += count_files_recursive(&path, config, directory_handler)?;
                            }
                        }
                    }
                }
            }
            
            Ok(count)
        }
        
        if path.is_file() {
            Ok(1)
        } else {
            count_files_recursive(path, &self.config, &directory_handler)
        }
    }

    /// Build globset for path ignoring
    fn build_path_ignorer(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        
        for pattern in &self.config.ignore_paths {
            let glob = Glob::new(pattern)
                .with_context(|| format!("Invalid glob pattern: {pattern}"))?;
            builder.add(glob);
        }
        
        builder.build()
            .with_context(|| "Failed to build path ignore globset")
    }
    
    /// Check if a file path should be ignored
    fn should_ignore_path(&self, path: &Path) -> Result<bool> {
        // Build and cache the GlobSet on first use, preserving errors
        let globset_result = self.cached_path_ignorer.get_or_init(|| {
            self.build_path_ignorer().map_err(|e| e.to_string())
        });
        
        match globset_result {
            Ok(globset) => Ok(globset.is_match(path)),
            Err(e) => Err(anyhow::anyhow!("Failed to build path ignorer: {}", e)),
        }
    }
    
    /// Check if a line contains ignore patterns
    fn should_ignore_line(&self, line: &str) -> bool {
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
        
        // Check for test code patterns (if enabled)
        if self.is_test_code_line(line) {
            return true;
        }
        
        false
    }
    
    /// Detect if a line is test code using config patterns
    fn is_test_code_line(&self, line: &str) -> bool {
        if !self.config.ignore_test_code {
            return false;
        }
        
        let trimmed = line.trim();
        
        // Check test attributes with glob patterns
        for pattern in &self.config.test_attributes {
            if Self::matches_glob_pattern(trimmed, pattern) {
                return true;
            }
        }
        
        // Check test module patterns
        for pattern in &self.config.test_modules {
            if trimmed.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Simple glob pattern matching for test attributes
    fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return text.starts_with(prefix) && text.ends_with(suffix);
            }
        }
        text == pattern
    }
    
    pub fn parse_scanner_config(config: &GuardyConfig) -> Result<ScannerConfig> {
        let mut scanner_config = ScannerConfig::default();
        
        // Override defaults with config values if present
        if let Ok(entropy_enabled) = config.get_section("scanner.entropy_analysis") {
            if let Some(enabled) = entropy_enabled.as_bool() {
                scanner_config.enable_entropy_analysis = enabled;
            }
        }
        
        if let Ok(threshold) = config.get_section("scanner.entropy_threshold") {
            if let Some(thresh) = threshold.as_f64() {
                scanner_config.min_entropy_threshold = thresh;
            }
        }
        
        if let Ok(include_binary) = config.get_section("scanner.include_binary") {
            if let Some(enabled) = include_binary.as_bool() {
                scanner_config.include_binary = enabled;
            }
        }
        
        // Load ignore patterns from config
        if let Ok(ignore_paths) = config.get_vec("scanner.ignore_paths") {
            crate::cli::output::styled!("{}: Loaded ignore_paths from config: {}", 
                ("DEBUG", "debug"),
                (format!("{:?}", ignore_paths), "muted")
            );
            scanner_config.ignore_paths = ignore_paths;
        } else {
            crate::cli::output::styled!("{}: No ignore_paths found in config, using defaults: {}", 
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
        
        if let Ok(ignore_test_code) = config.get_section("scanner.ignore_test_code") {
            if let Some(enabled) = ignore_test_code.as_bool() {
                scanner_config.ignore_test_code = enabled;
            }
        }
        
        if let Ok(test_attributes) = config.get_vec("scanner.test_attributes") {
            // Keep test patterns case-sensitive for proper class name matching
            scanner_config.test_attributes = test_attributes;
        }
        
        if let Ok(test_modules) = config.get_vec("scanner.test_modules") {
            // Keep test patterns case-sensitive for proper class name matching
            scanner_config.test_modules = test_modules;
        }
        
        // Parse processing mode settings
        if let Ok(mode_str) = config.get_section("scanner.mode") {
            if let Some(mode) = mode_str.as_str() {
                tracing::trace!("SCANNER CONFIG: Parsing mode from config: '{}'", mode);
                scanner_config.mode = match mode.to_lowercase().as_str() {
                    "sequential" => super::types::ScanMode::Sequential,
                    "parallel" => super::types::ScanMode::Parallel,
                    "auto" => super::types::ScanMode::Auto,
                    _ => super::types::ScanMode::Auto, // Default fallback
                };
                tracing::trace!("SCANNER CONFIG: Set mode to: {:?}", scanner_config.mode);
            }
        }
        
        if let Ok(max_threads) = config.get_section("scanner.max_threads") {
            if let Some(threads) = max_threads.as_u64() {
                scanner_config.max_threads = threads as usize;
            }
        }
        
        if let Ok(thread_percentage) = config.get_section("scanner.thread_percentage") {
            if let Some(percentage) = thread_percentage.as_u64() {
                scanner_config.thread_percentage = percentage as u8;
            }
        }
        
        if let Ok(min_files) = config.get_section("scanner.min_files_for_parallel") {
            if let Some(files) = min_files.as_u64() {
                scanner_config.min_files_for_parallel = files as usize;
            }
        }
        
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
    
    /// Build a WalkBuilder with common directory filtering logic
    pub(crate) fn build_directory_walker(&self, path: &Path) -> WalkBuilder {
        let mut builder = WalkBuilder::new(path);
        builder
            .follow_links(self.config.follow_symlinks)
            .git_ignore(true)        // Respect .gitignore files
            .git_global(true)        // Respect global gitignore
            .git_exclude(true)       // Respect .git/info/exclude
            .hidden(false)           // Don't ignore hidden files by default
            .parents(true);          // Check parent directories for .gitignore
            
        // Use shared directory handler for consistent filtering logic
        let directory_handler = super::directory::DirectoryHandler::new();
        
        // Build ignore patterns for use in filter
        let ignore_globset = self.build_path_ignorer().ok();
        
        builder.filter_entry(move |entry| {
            // Skip directories that should always be ignored for security/performance
            if let Some(file_name) = entry.file_name().to_str() {
                if directory_handler.should_filter_directory(file_name) {
                    return false;
                }
            }
            
            // Apply ignore_paths patterns
            if let Some(ref globset) = ignore_globset {
                if globset.is_match(entry.path()) {
                    return false;
                }
            }
            
            true
        });
        
        builder
    }


    /// Scan a directory recursively with optional execution strategy
    /// By default uses smart mode (auto-detects parallel vs sequential)
    pub fn scan_directory(&self, path: &Path, strategy: Option<ExecutionStrategy>) -> Result<ScanResult> {
        let directory_handler = super::directory::DirectoryHandler::new();
        directory_handler.scan(Arc::new(self.clone()), path, strategy)
    }
    
    /// Scan a single file
    pub fn scan_file(&self, path: &Path) -> Result<Vec<SecretMatch>> {
        self.scan_single_path(path)
    }
    
    /// Scan a large file using streaming approach to minimize memory usage
    fn scan_file_streaming(&self, path: &Path) -> Result<Vec<SecretMatch>> {
        use std::io::{BufRead, BufReader};
        use std::fs::File;
        
        let file = File::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);
        let mut matches = Vec::new();
        
        // Note: For large files, we sacrifice some test detection features 
        // (which require full file analysis) for memory efficiency
        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result
                .with_context(|| format!("Failed to read line {} in file: {}", line_number + 1, path.display()))?;
            
            // Skip ignored lines
            if self.should_ignore_line(&line) {
                continue;
            }
            
            // Scan this line for secrets
            let line_matches = self.scan_line(&line, path, line_number + 1);
            matches.extend(line_matches);
        }
        
        Ok(matches)
    }

    pub(crate) fn scan_single_path(&self, path: &Path) -> Result<Vec<SecretMatch>> {
        // Check if path should be ignored
        if self.should_ignore_path(path)? {
            return Ok(vec![]);
        }
        
        // Check file size
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.config.max_file_size_mb as u64 {
                return Ok(vec![]);
            }
        }
        
        // Binary file check is now handled at the directory level for better performance
        
        // Read file content - use streaming for large files
        const STREAMING_THRESHOLD_MB: u64 = 5; // Stream files larger than 5MB
        let mut matches = Vec::new();
        
        // Check file size to decide on reading strategy
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_mb = metadata.len() / (1024 * 1024);
            
            if size_mb > STREAMING_THRESHOLD_MB {
                // Use streaming approach for large files
                return self.scan_file_streaming(path);
            }
        }
        
        // Use in-memory approach for small files (original behavior)
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let lines: Vec<&str> = content.lines().collect();
        
        // Build ignore ranges for test blocks
        let detector = TestDetector::new(&self.config);
        let ignore_ranges = detector.build_ignore_ranges(&lines, path);
        
        for (line_number, line) in lines.iter().enumerate() {
            // Check if this line is in an ignored range
            if ignore_ranges.iter().any(|range| range.contains(&line_number)) {
                continue;
            }
            
            // Check for ignore patterns on this line and next line
            if self.should_ignore_line(line) {
                continue;
            }
            
            // Check for ignore-next directive on previous line
            if line_number > 0 {
                let prev_line = lines[line_number - 1];
                if prev_line.contains("guardy:ignore-next") {
                    continue;
                }
            }
            
            let line_matches = self.scan_line(line, path, line_number + 1);
            matches.extend(line_matches);
        }
        
        Ok(matches)
    }
    
    fn scan_line(&self, line: &str, file_path: &Path, line_number: usize) -> Vec<SecretMatch> {
        // Always use sequential pattern processing (parallel patterns proved to be 10x slower)
        self.scan_line_sequential(line, file_path, line_number)
    }
    
    /// Sequential pattern matching (original implementation)
    fn scan_line_sequential(&self, line: &str, file_path: &Path, line_number: usize) -> Vec<SecretMatch> {
        let mut matches = Vec::new();
        
        // Find potential secrets using sequential pattern matching
        for pattern in &self.patterns.patterns {
            for regex_match in pattern.regex.find_iter(line) {
                if let Some(secret_match) = self.process_pattern_match(pattern, regex_match, line, file_path, line_number) {
                    matches.push(secret_match);
                }
            }
        }
        
        matches
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
            pattern.regex.captures(line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str())
                .unwrap_or(matched_text)
        } else {
            matched_text
        };
        
        // Apply entropy analysis if enabled (only on the secret content)
        if self.config.enable_entropy_analysis
            && !is_likely_secret(secret_content.as_bytes(), self.config.min_entropy_threshold) {
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
    use tempfile::TempDir;
    use std::fs;
    use crate::config::GuardyConfig;
    
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
            println!("Found: {} in {}", secret_match.matched_text, secret_match.secret_type);
        }
    }
    
    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple test files
        let file1 = temp_dir.path().join("secrets1.env");
        let file2 = temp_dir.path().join("config.json");
        
        fs::write(&file1, "STRIPE_KEY=***REMOVED***").unwrap();
        fs::write(&file2, r#"{"api_key": "fake_key_for_testing"}"#).unwrap();
        
        let config = create_test_config();
        let scanner = Scanner::new(&config).unwrap();
        let result = scanner.scan_directory(temp_dir.path(), None).unwrap();
        
        // Should scan multiple files
        assert!(result.stats.files_scanned >= 2);
    }
    
}