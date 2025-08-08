use std::path::Path;
use crate::scanner::types::ScannerConfig;

/// Handles intelligent test code detection across multiple programming languages
pub struct TestDetector<'a> {
    config: &'a ScannerConfig,
}

impl<'a> TestDetector<'a> {
    pub fn new(config: &'a ScannerConfig) -> Self {
        Self { config }
    }
    
    /// Build ranges of line numbers that should be ignored due to test blocks
    pub fn build_ignore_ranges(&self, lines: &[&str], path: &Path) -> Vec<std::ops::Range<usize>> {
        if !self.config.ignore_test_code {
            return vec![];
        }
        
        let mut ranges = Vec::new();
        let mut i = 0;
        
        while i < lines.len() {
            // Check if this line starts a test block
            if let Some(end_line) = self.find_test_block_end(lines, i, path) {
                ranges.push(i..end_line + 1);
                i = end_line + 1; // Skip past the block
            } else {
                i += 1;
            }
        }
        
        ranges
    }
    
    /// Find the end of a test block starting at the given line
    fn find_test_block_end(&self, lines: &[&str], start_line: usize, path: &Path) -> Option<usize> {
        let line = lines[start_line].trim();
        
        // Check if this line starts a test block
        let is_test_start = self.config.test_attributes.iter().any(|pattern| {
            self.matches_glob_pattern(line, pattern)
        }) || self.config.test_modules.iter().any(|pattern| {
            line.contains(pattern)
        });
        
        if !is_test_start {
            return None;
        }
        
        // Determine bracket style based on file extension
        let (open_bracket, close_bracket) = if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("py") => (":", ""), // Python uses indentation
                Some("rs") | Some("ts") | Some("js") => ("{", "}"), // Brace languages
                _ => ("{", "}")
            }
        } else {
            ("{", "}")
        };
        
        // Handle Python indentation-based blocks
        if open_bracket == ":" {
            return self.find_python_block_end(lines, start_line);
        }
        
        // Handle brace-based languages
        let mut brace_count = 0;
        let mut found_opening_brace = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            // Count opening braces
            let opens = line.matches(open_bracket).count();
            let closes = line.matches(close_bracket).count();
            
            brace_count += opens as i32;
            brace_count -= closes as i32;
            
            if opens > 0 {
                found_opening_brace = true;
            }
            
            // If we've found opening braces and count is back to 0, block is complete
            if found_opening_brace && brace_count == 0 {
                return Some(i);
            }
        }
        
        None
    }
    
    /// Find the end of a Python test block using indentation
    fn find_python_block_end(&self, lines: &[&str], start_line: usize) -> Option<usize> {
        let start_indent = self.get_line_indent(lines[start_line]);
        
        // Find the next line that has same or less indentation
        for (i, line) in lines.iter().enumerate().skip(start_line + 1) {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }
            
            let line_indent = self.get_line_indent(line);
            
            // If we find a line with same or less indentation, the block ends
            if line_indent <= start_indent {
                return Some(i - 1);
            }
        }
        
        // If we reach end of file, the block extends to the end
        Some(lines.len() - 1)
    }
    
    /// Get the indentation level of a line (number of leading spaces/tabs)
    fn get_line_indent(&self, line: &str) -> usize {
        line.len() - line.trim_start().len()
    }
    
    /// Simple glob pattern matching for test attributes
    fn matches_glob_pattern(&self, text: &str, pattern: &str) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::config::GuardyConfig;
    use crate::scanner::Scanner;
    use std::fs;
    
    fn create_test_config() -> GuardyConfig {
        GuardyConfig::load(None, None::<&()>, 0).unwrap()
    }
    
    fn create_scanner_config() -> ScannerConfig {
        let config = create_test_config();
        Scanner::parse_scanner_config(&config).unwrap()
    }
    
    #[test] 
    fn test_rust_block_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        let content = vec![
            "const API_KEY = \"sk_live_real_secret\";",
            "",
            "#[test]",
            "fn test_function() {",
            "    let secret = \"sk_live_test_secret\";", 
            "    assert_eq!(1, 1);",
            "}",
            "",
            "const ANOTHER_KEY = \"sk_live_another_secret\";",
        ];
        
        let scanner_config = create_scanner_config();
        let detector = TestDetector::new(&scanner_config);
        let ranges = detector.build_ignore_ranges(&content, &test_file);
        
        // Should ignore lines 2-6 (the test function)
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], 2..7);
    }
    
    #[test]
    fn test_typescript_block_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ts");
        
        let content = vec![
            "const apiKey = \"sk_live_real_secret\";",
            "",
            "describe(\"My test\", () => {",
            "    const testSecret = \"sk_live_test_secret\";",
            "    it(\"should work\", () => {",
            "        expect(true).toBe(true);",
            "    });",
            "});",
            "",
            "const finalKey = \"sk_live_final_secret\";",
        ];
        
        let scanner_config = create_scanner_config();
        let detector = TestDetector::new(&scanner_config);
        let ranges = detector.build_ignore_ranges(&content, &test_file);
        
        // Should ignore lines 2-7 (the describe block)
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], 2..8);
    }
    
    #[test]
    fn test_python_block_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        
        let content = vec![
            "api_key = \"sk_live_real_secret\"",
            "",
            "def test_function():",
            "    secret = \"sk_live_test_secret\"",
            "    assert True",
            "",
            "final_key = \"sk_live_final_secret\"",
        ];
        
        let scanner_config = create_scanner_config();
        let detector = TestDetector::new(&scanner_config);
        let ranges = detector.build_ignore_ranges(&content, &test_file);
        
        // Should ignore lines 2-5 (the test function including empty line)
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], 2..6);
    }
    
    // Integration tests that test full scanner + test detection
    #[test]
    fn test_rust_integration_test_block_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_blocks.rs");
        
        let test_content = r#"
use superconfig::Config;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

#[derive(Debug, Deserialize, Serialize)]
struct FakeTestValues {
    stripe_api_keys: Vec<String>,
    aws_keys: AwsKeys,
    github_tokens: Vec<String>,
    jwt_tokens: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AwsKeys {
    access_key: String,
    secret_key: String,
}

lazy_static! {
    static ref FAKE_VALUES: FakeTestValues = {
        Config::builder()
            .add_source(superconfig::File::with_name("fake_test_values"))
            .build()
            .expect("Failed to load fake_test_values.yml")
            .try_deserialize()
            .expect("Failed to parse fake test values")
    };
}

// Regular code that should be scanned - uses first stripe key
fn get_api_key() -> String {
    FAKE_VALUES.stripe_api_keys[0].clone()
}

#[test]
fn test_function() {
    // This secret should be ignored because it's in a test - uses second stripe key
    let secret = &FAKE_VALUES.stripe_api_keys[1];
    assert_eq!(1, 1);
}

// More regular code - uses AWS access key
fn get_another_key() -> String {
    FAKE_VALUES.aws_keys.access_key.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // This entire module should be ignored - uses github token
    fn get_test_secret() -> String {
        FAKE_VALUES.github_tokens[0].clone()
    }
    
    #[test]
    fn another_test() {
        // Uses JWT token
        let key = &FAKE_VALUES.jwt_tokens[0];
        assert!(!key.is_empty());
    }
}

// Back to regular code - uses AWS secret key
fn get_final_key() -> String {
    FAKE_VALUES.aws_keys.secret_key.clone()
}
"#;
        
        fs::write(&test_file, test_content).unwrap();
        
        let config = create_test_config();
        let scanner = Scanner::new(&config).unwrap();
        let result = scanner.scan_file(&test_file).unwrap();
        
        // Should only find secrets outside test blocks
        let found_secrets: Vec<&str> = result.iter()
            .map(|m| m.matched_text.as_str())
            .collect();
        
        // Should find regular secrets but not test secrets
        assert!(found_secrets.iter().any(|s| s.contains("4eC39HqLyjWDarjtT1zdp7dcGGTJ8XA5B9r2F3mQ")));
        assert!(found_secrets.iter().any(|s| s.contains("5xZ8jM3nK7qW2rT9vY4uL6pS1dF0hC8gA5bE3iO7")));  
        assert!(found_secrets.iter().any(|s| s.contains("8jL5nQ2tY9vX6rB1mF4sK7dA0cG3hE6iO9pZ2wU")));
        
        // Should NOT find test secrets
        assert!(!found_secrets.iter().any(|s| s.contains("7nX2mK9qY8dP5vL3wR6tF4uN8hG2cV1sA0eB7iO9")));
        assert!(!found_secrets.iter().any(|s| s.contains("9rB4mN7qX2sT6vY1uL8pF5dH0cG3kA9bE7iO2wZ")));
        assert!(!found_secrets.iter().any(|s| s.contains("3mQ6nR9tY2vL5xZ8jK1sF4dC7gA0bE9hO6pW3uN")));
    }
    
    #[test]
    fn test_typescript_integration_test_block_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_blocks.ts");
        
        let test_content = r#"
// Regular code that should be scanned
const apiKey = "***REMOVED***";

describe("My test suite", () => {
    // This entire block should be ignored
    const testSecret = "***REMOVED***";
    
    it("should do something", () => {
        const anotherSecret = "***REMOVED***";
        expect(true).toBe(true);
    });
    
    test("another test", () => {
        const testKey = "***REMOVED***";
    });
});

// Back to regular code
const finalKey = "***REMOVED***";
"#;
        
        fs::write(&test_file, test_content).unwrap();
        
        let config = create_test_config();
        let scanner = Scanner::new(&config).unwrap();
        let result = scanner.scan_file(&test_file).unwrap();
        
        let found_secrets: Vec<&str> = result.iter()
            .map(|m| m.matched_text.as_str())
            .collect();
        
        // Should find regular secrets but not test secrets
        assert!(found_secrets.iter().any(|s| s.contains("2xK8mQ5nR9tY6vL3zJ7dF0hC4gA1bE8iO5pW2uN")));
        assert!(found_secrets.iter().any(|s| s.contains("3sH6mQ9tY2vX5rB8jL1nF4dK7cA0gE3hO6pZ9wU")));
        
        // Should NOT find test secrets (entire describe block ignored)
        assert!(!found_secrets.iter().any(|s| s.contains("7qB4nX2sT9vY1uL8pF5dH0cG6kA3bE9hO2pZ5wU")));
        assert!(!found_secrets.iter().any(|s| s.contains("4mL7nQ0tY3vX6rB9mF2sK5dA8cG1hE4iO7pZ0wU")));
        assert!(!found_secrets.iter().any(|s| s.contains("6jN9qR2tY5vL8xZ1mK4sF7dC0gA3hE6iO9pW2uN")));
    }
    
    #[test]
    fn test_python_integration_test_block_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_blocks.py");
        
        let test_content = r#"
# Regular code that should be scanned
api_key = "***REMOVED***"

def test_function():
    # This entire function should be ignored
    secret = "***REMOVED***"
    assert True

class TestClass:
    # This entire class should be ignored
    def setUp(self):
        self.secret = "***REMOVED***"
    
    def test_method(self):
        key = "***REMOVED***"

# Back to regular code
final_key = "***REMOVED***"
"#;
        
        fs::write(&test_file, test_content).unwrap();
        
        let config = create_test_config();
        let scanner = Scanner::new(&config).unwrap();
        let result = scanner.scan_file(&test_file).unwrap();
        
        let found_secrets: Vec<&str> = result.iter()
            .map(|m| m.matched_text.as_str())
            .collect();
        
        // Should find regular secrets but not test secrets
        assert!(found_secrets.iter().any(|s| s.contains("9rB4mN7qX2sT6vY1uL8pF5dH0cG3kA9bE7iO2wZ")));
        assert!(found_secrets.iter().any(|s| s.contains("4eC39HqLyjWDarjtT1zdp7dcGGTJ8XA5B9r2F3mQ")));
        
        // Should NOT find test secrets (entire test function/class ignored)
        assert!(!found_secrets.iter().any(|s| s.contains("5xZ8jM3nK7qW2rT9vY4uL6pS1dF0hC8gA5bE3iO7")));
        assert!(!found_secrets.iter().any(|s| s.contains("2mQ6nR9tY7vL5xZ8jK1sF4dC7gA0bE9hO6pW3uN")));
        assert!(!found_secrets.iter().any(|s| s.contains("8jL5nQ2tY9vX6rB1mF4sK7dA0cG3hE6iO9pZ2wU")));
    }
}