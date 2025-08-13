//! Integration tests for the scan module

mod data;
mod filters;
mod pipeline;
mod reports;
mod static_data;
mod tracking;

// Core scanner tests
#[cfg(test)]
mod tests {
    use guardy::scan::{Scanner, ScannerConfig};
    use std::path::Path;
    
    #[test]
    fn test_scanner_creation() {
        let config = ScannerConfig::default();
        let scanner = Scanner::new(config);
        assert!(scanner.is_ok());
    }
    
    #[test]
    fn test_scan_current_directory() {
        let config = ScannerConfig::default();
        let scanner = Scanner::new(config).unwrap();
        
        // Scan a small test directory
        let result = scanner.scan(Path::new("."));
        assert!(result.is_ok());
        
        let scan_result = result.unwrap();
        assert!(scan_result.stats.files_scanned > 0);
    }
}