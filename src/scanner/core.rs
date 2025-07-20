use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
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
        })
    }
    
    pub fn with_config(patterns: SecretPatterns, config: ScannerConfig) -> Result<Self> {
        Ok(Scanner {
            patterns,
            config,
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
    
    /// Scan a directory recursively
    pub fn scan_directory(&self, path: &Path) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        let mut all_matches = Vec::new();
        let mut stats = ScanStats::default();
        let mut warnings: Vec<Warning> = Vec::new();
        
        // Build intelligent walker that respects .gitignore AND skips directories that should be ignored
        let mut builder = WalkBuilder::new(path);
        builder
            .follow_links(self.config.follow_symlinks)
            .git_ignore(true)        // Respect .gitignore files
            .git_global(true)        // Respect global gitignore
            .git_exclude(true)       // Respect .git/info/exclude
            .hidden(false)           // Don't ignore hidden files by default
            .parents(true);          // Check parent directories for .gitignore
            
        // Add intelligent skipping for directories that SHOULD be ignored based on project type
        builder.filter_entry(|entry| {
            if let Some(file_name) = entry.file_name().to_str() {
                // Skip directories that should always be ignored for security/performance
                !matches!(file_name,
                    // Rust build artifacts
                    "target" |
                    // Node.js dependencies and build artifacts  
                    "node_modules" | "dist" | "build" | ".next" | ".nuxt" |
                    // Python artifacts
                    "__pycache__" | ".pytest_cache" | "venv" | ".venv" | "env" | ".env" |
                    // Go artifacts
                    "vendor" |
                    // Java artifacts  
                    "out" |
                    // Generic build/cache directories
                    "cache" | ".cache" | "tmp" | ".tmp" | "temp" | ".temp" |
                    // Version control
                    ".git" | ".svn" | ".hg" |
                    // IDE directories
                    ".vscode" | ".idea" | ".vs" |
                    // Coverage reports
                    "coverage" | ".nyc_output"
                )
            } else {
                true
            }
        });
        
        let walker = builder.build();
        
        // Count total files for progress indication
        let mut file_count = 0;
        let mut processed_count = 0;
        
        // First pass: count files for progress reporting
        println!("📊 Analyzing directory structure...");
        let mut count_builder = WalkBuilder::new(path);
        count_builder
            .follow_links(self.config.follow_symlinks)
            .git_ignore(true)
            .git_global(true)  
            .git_exclude(true)
            .hidden(false)
            .parents(true);
            
        // Apply the same intelligent filtering for counting
        count_builder.filter_entry(|entry| {
            if let Some(file_name) = entry.file_name().to_str() {
                !matches!(file_name,
                    "target" | "node_modules" | "dist" | "build" | ".next" | ".nuxt" |
                    "__pycache__" | ".pytest_cache" | "venv" | ".venv" | "env" | ".env" |
                    "vendor" | "out" | "cache" | ".cache" | "tmp" | ".tmp" | "temp" | ".temp" |
                    ".git" | ".svn" | ".hg" | ".vscode" | ".idea" | ".vs" | "coverage" | ".nyc_output"
                )
            } else {
                true
            }
        });
        
        let count_walker = count_builder.build();
            
        for entry in count_walker {
            if let Ok(entry) = entry {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    file_count += 1;
                }
            }
        }
        
        println!("🔍 Scanning {} files...", file_count);
        
        // Check which directories actually exist and analyze their gitignore status
        let mut properly_ignored = Vec::new();
        let mut needs_gitignore = Vec::new();
        
        // Helper function to check if a pattern exists in gitignore
        let check_gitignore_pattern = |pattern: &str| -> bool {
            if let Ok(gitignore_content) = std::fs::read_to_string(path.join(".gitignore")) {
                gitignore_content.lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .any(|line| line == pattern || line == pattern.trim_end_matches('/'))
            } else {
                false
            }
        };
        
        // Check for Rust directories
        if path.join("target").exists() {
            if check_gitignore_pattern("target/") || check_gitignore_pattern("target") {
                properly_ignored.push(("target/", "Rust build directory"));
            } else {
                needs_gitignore.push(("target/", "Rust build directory"));
            }
        }
        
        // Check for Node.js directories
        if path.join("node_modules").exists() {
            if check_gitignore_pattern("node_modules/") || check_gitignore_pattern("node_modules") {
                properly_ignored.push(("node_modules/", "Node.js dependencies"));
            } else {
                needs_gitignore.push(("node_modules/", "Node.js dependencies"));
            }
        }
        if path.join("dist").exists() {
            if check_gitignore_pattern("dist/") || check_gitignore_pattern("dist") {
                properly_ignored.push(("dist/", "Build output directory"));
            } else {
                needs_gitignore.push(("dist/", "Build output directory"));
            }
        }
        if path.join("build").exists() {
            if check_gitignore_pattern("build/") || check_gitignore_pattern("build") {
                properly_ignored.push(("build/", "Build output directory"));
            } else {
                needs_gitignore.push(("build/", "Build output directory"));
            }
        }
        
        // Check for Python directories
        if path.join("__pycache__").exists() {
            if check_gitignore_pattern("__pycache__/") || check_gitignore_pattern("__pycache__") {
                properly_ignored.push(("__pycache__/", "Python cache directory"));
            } else {
                needs_gitignore.push(("__pycache__/", "Python cache directory"));
            }
        }
        if path.join("venv").exists() {
            if check_gitignore_pattern("venv/") || check_gitignore_pattern("venv") {
                properly_ignored.push(("venv/", "Python virtual environment"));
            } else {
                needs_gitignore.push(("venv/", "Python virtual environment"));
            }
        }
        if path.join(".venv").exists() {
            if check_gitignore_pattern(".venv/") || check_gitignore_pattern(".venv") {
                properly_ignored.push((".venv/", "Python virtual environment"));
            } else {
                needs_gitignore.push((".venv/", "Python virtual environment"));
            }
        }
        
        // Check for Go directories
        if path.join("vendor").exists() {
            if check_gitignore_pattern("vendor/") || check_gitignore_pattern("vendor") {
                properly_ignored.push(("vendor/", "Go dependencies"));
            } else {
                needs_gitignore.push(("vendor/", "Go dependencies"));
            }
        }
        
        // Show status of discovered directories
        if !properly_ignored.is_empty() || !needs_gitignore.is_empty() {
            let total_dirs = properly_ignored.len() + needs_gitignore.len();
            println!("📁 Discovered {} director{}:", 
                     total_dirs, 
                     if total_dirs == 1 { "y" } else { "ies" });
            
            // Show properly ignored directories
            for (dir, description) in &properly_ignored {
                println!("   ✅ {} ({})", 
                    console::style(dir).green(),
                    console::style(description).dim()
                );
            }
            
            // Show directories that need gitignore rules
            for (dir, description) in &needs_gitignore {
                println!("   ⚠️  {} ({})", 
                    console::style(dir).yellow(),
                    console::style(description).dim()
                );
            }
            
            // Only show gitignore recommendations for directories that need them
            if !needs_gitignore.is_empty() {
                let patterns: Vec<&str> = needs_gitignore.iter().map(|(dir, _)| *dir).collect();
                println!("💡 Consider adding to .gitignore: {}", 
                         console::style(patterns.join(", ")).cyan());
            }
        }
        
        // Second pass: actual scanning with progress
        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        processed_count += 1;
                        
                        // Show progress every 50 files or for large scans
                        if processed_count % 50 == 0 || file_count > 500 {
                            let progress = (processed_count as f64 / file_count as f64 * 100.0) as u32;
                            print!("\r⏳ Progress: {}/{} files ({}%)", processed_count, file_count, progress);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        }
                        
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
                    });
                }
            }
        }
        
        // Clear progress line
        if file_count > 0 {
            print!("\r");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        
        let scan_duration = start_time.elapsed();
        stats.scan_duration_ms = scan_duration.as_millis() as u64;
        
        // Show timing summary
        println!("⏱️  Scan completed in {:.2}s ({} files scanned, {} matches found)", 
                 scan_duration.as_secs_f64(), 
                 stats.files_scanned, 
                 stats.total_matches);
        
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
                    pattern_description: pattern.description.clone(),
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