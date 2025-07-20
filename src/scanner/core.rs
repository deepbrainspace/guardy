use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use crate::config::GuardyConfig;
use super::entropy::is_likely_secret;
use super::patterns::SecretPatterns;
use super::test_detection::TestDetector;
use globset::{Glob, GlobSet, GlobSetBuilder};

#[derive(Debug, Clone)]
pub struct SecretMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub matched_text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub secret_type: String,
}

#[derive(Debug, Default)]
pub struct ScanStats {
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub total_matches: usize,
    pub scan_duration_ms: u64,
}

#[derive(Debug)]
pub struct Warning {
    pub message: String,
    pub category: WarningCategory,
}

#[derive(Debug)]
pub enum WarningCategory {
    GitignoreMismatch,
    BinaryFileSkipped,
    PermissionDenied,
    UnknownFileType,
}

#[derive(Debug)]
pub struct ScanResult {
    pub matches: Vec<SecretMatch>,
    pub stats: ScanStats,
    pub warnings: Vec<Warning>,
}

#[derive(Clone)]
pub struct Scanner {
    patterns: SecretPatterns,
    config: ScannerConfig,
}

#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub enable_entropy_analysis: bool,
    pub min_entropy_threshold: f64,
    pub skip_binary_files: bool,
    pub follow_symlinks: bool,
    pub max_file_size_mb: usize,
    pub ignore_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub ignore_comments: Vec<String>,
    pub ignore_test_code: bool,
    pub test_attributes: Vec<String>,
    pub test_modules: Vec<String>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enable_entropy_analysis: true,
            min_entropy_threshold: 1.0 / 1e5,
            skip_binary_files: true,
            follow_symlinks: false,
            max_file_size_mb: 10,
            ignore_paths: vec![
                "tests/*".to_string(),
                "testdata/*".to_string(),
                "*_test.rs".to_string(),
                "test_*.rs".to_string(),
            ],
            ignore_patterns: vec![
                "# TEST_SECRET:".to_string(),
                "DEMO_KEY_".to_string(),
                "FAKE_".to_string(),
            ],
            ignore_comments: vec![
                "guardy:ignore".to_string(),
                "guardy:ignore-line".to_string(),
                "guardy:ignore-next".to_string(),
            ],
            ignore_test_code: true,
            test_attributes: vec![],
            test_modules: vec![],
        }
    }
}

impl Scanner {
    pub fn new(config: &GuardyConfig) -> Result<Self> {
        // Load patterns from config
        let patterns = SecretPatterns::new(config)?;
        
        // Parse scanner-specific config
        let scanner_config = Self::parse_scanner_config(config)?;
        
        Ok(Scanner {
            patterns,
            config: scanner_config,
        })
    }
    
    /// Build globset for path ignoring
    fn build_path_ignorer(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        
        for pattern in &self.config.ignore_paths {
            let glob = Glob::new(pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            builder.add(glob);
        }
        
        builder.build()
            .with_context(|| "Failed to build path ignore globset")
    }
    
    /// Check if a file path should be ignored
    fn should_ignore_path(&self, path: &Path) -> Result<bool> {
        let globset = self.build_path_ignorer()?;
        Ok(globset.is_match(path))
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
        
        // Load ignore patterns from config
        if let Ok(ignore_paths) = config.get_vec("scanner.ignore_paths") {
            scanner_config.ignore_paths = ignore_paths;
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
            scanner_config.test_attributes = test_attributes;
        }
        
        if let Ok(test_modules) = config.get_vec("scanner.test_modules") {
            scanner_config.test_modules = test_modules;
        }
        
        Ok(scanner_config)
    }
    
    
    /// Scan specific paths
    pub fn scan_paths(&self, paths: &[PathBuf]) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        let mut all_matches = Vec::new();
        let mut stats = ScanStats::default();
        let mut warnings = Vec::new();
        
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
                        category: WarningCategory::PermissionDenied,
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
    
    /// Scan a directory recursively
    pub fn scan_directory(&self, path: &Path) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        let mut all_matches = Vec::new();
        let mut stats = ScanStats::default();
        let mut warnings = Vec::new();
        
        let walker = WalkBuilder::new(path)
            .follow_links(self.config.follow_symlinks)
            .build();
        
        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        match self.scan_single_path(entry.path()) {
                            Ok(mut matches) => {
                                stats.files_scanned += 1;
                                stats.total_matches += matches.len();
                                all_matches.append(&mut matches);
                            }
                            Err(_) => {
                                stats.files_skipped += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    warnings.push(Warning {
                        message: format!("Walk error: {}", e),
                        category: WarningCategory::PermissionDenied,
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
    
    /// Scan a single file
    pub fn scan_file(&self, path: &Path) -> Result<Vec<SecretMatch>> {
        self.scan_single_path(path)
    }
    
    fn scan_single_path(&self, path: &Path) -> Result<Vec<SecretMatch>> {
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
        
        // Read file content
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let mut matches = Vec::new();
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
        let mut matches = Vec::new();
        
        // Debug output removed
        
        // Find potential secrets using pattern matching
        for pattern in &self.patterns.patterns {
            for regex_match in pattern.regex.find_iter(line) {
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
                if self.config.enable_entropy_analysis {
                    if !is_likely_secret(secret_content.as_bytes(), self.config.min_entropy_threshold) {
                        continue; // Skip if entropy too low
                    }
                }
                
                matches.push(SecretMatch {
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number,
                    line_content: line.to_string(),
                    matched_text: matched_text.to_string(),
                    start_pos: regex_match.start(),
                    end_pos: regex_match.end(),
                    secret_type: pattern.name.clone(),
                });
            }
        }
        
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::config::GuardyConfig;
    
    fn create_test_config() -> GuardyConfig {
        GuardyConfig::load().unwrap()
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
        let result = scanner.scan_directory(temp_dir.path()).unwrap();
        
        // Should scan multiple files
        assert!(result.stats.files_scanned >= 2);
    }
    
}