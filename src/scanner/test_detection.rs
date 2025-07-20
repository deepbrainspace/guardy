use std::path::Path;
use crate::scanner::core::ScannerConfig;

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
        
        for i in start_line..lines.len() {
            let line = lines[i];
            
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
        for i in (start_line + 1)..lines.len() {
            let line = lines[i];
            
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
    
    fn create_test_config() -> GuardyConfig {
        GuardyConfig::load().unwrap()
    }
    
    fn create_scanner_config() -> ScannerConfig {
        let config = create_test_config();
        crate::scanner::core::Scanner::parse_scanner_config(&config).unwrap()
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
}