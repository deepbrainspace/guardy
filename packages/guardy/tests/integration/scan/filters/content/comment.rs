use guardy::scan::filters::content::CommentFilter;
use guardy::scan::types::{ScannerConfig, SecretMatch};
use anyhow::Result;

fn create_test_config() -> ScannerConfig {
    ScannerConfig {
        ignore_comments: vec![
            "guardy:ignore".to_string(),
            "guardy:ignore-line".to_string(),
            "guardy:allow".to_string(),
        ],
        ..ScannerConfig::default()
    }
}

fn create_test_match(line_number: usize, line_content: &str) -> SecretMatch {
    SecretMatch {
        file_path: "test.rs".to_string(),
        line_number,
        line_content: line_content.to_string(),
        matched_text: "secret123".to_string(),
        start_pos: 0,
        end_pos: 9,
        secret_type: "Test Secret".to_string(),
        pattern_description: "Test pattern".to_string(),
    }
}

#[test]
fn test_shared_regexes() -> Result<()> {
    let regexes = CommentFilter::get_ignore_regexes();
    assert!(!regexes.is_empty());
    
    // Test that default patterns are compiled
    let test_lines = [
        "const secret = 'test'; // guardy:ignore",
        "let key = 'secret'; // guardy:ignore-line",  
        "password = 'test'; // guardy:allow",
    ];
    
    for line in &test_lines {
        let found = regexes.iter().any(|regex| regex.is_match(line));
        assert!(found, "Should match ignore comment in: {}", line);
    }
    Ok(())
}

#[test]
fn test_comment_filter_creation() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    assert_eq!(filter.get_config_info(), 3);
    Ok(())
}

#[test]
fn test_line_ignore_detection() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Test positive cases
    assert!(filter.line_has_ignore_directive("const secret = 'test'; // guardy:ignore"));
    assert!(filter.line_has_ignore_directive("let key = 'secret'; # guardy:ignore-line"));
    assert!(filter.line_has_ignore_directive("password = 'test'; /* guardy:ignore */"));
    assert!(filter.line_has_ignore_directive("api_key = 'abc'; // guardy:allow"));
    
    // Test negative cases
    assert!(!filter.line_has_ignore_directive("const secret = 'test';"));
    assert!(!filter.line_has_ignore_directive("let key = 'secret';"));
    assert!(!filter.line_has_ignore_directive("// just a regular comment"));
    assert!(!filter.line_has_ignore_directive("/* block comment */"));
    Ok(())
}

#[test]
fn test_match_ignoring() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Test match that should be ignored
    let ignored_match = create_test_match(
        1, 
        "const secret = 'test123'; // guardy:ignore"
    );
    
    let file_content = "const secret = 'test123'; // guardy:ignore\n";
    assert!(filter.should_ignore_match(&ignored_match, file_content)?);
    
    // Test match that should not be ignored
    let normal_match = create_test_match(
        1,
        "const secret = 'test123';"
    );
    
    let file_content = "const secret = 'test123';\n";
    assert!(!filter.should_ignore_match(&normal_match, file_content)?);
    Ok(())
}

#[test]
fn test_filter_matches() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    let matches = vec![
        create_test_match(1, "const secret1 = 'test'; // guardy:ignore"),
        create_test_match(2, "const secret2 = 'test';"),
        create_test_match(3, "const secret3 = 'test'; // guardy:ignore-line"),
        create_test_match(4, "const secret4 = 'test';"),
    ];
    
    let file_content = "const secret1 = 'test'; // guardy:ignore\n\
                       const secret2 = 'test';\n\
                       const secret3 = 'test'; // guardy:ignore-line\n\
                       const secret4 = 'test';\n";
    
    let filtered = filter.filter_matches(&matches, file_content);
    
    // Should keep only matches without ignore comments
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].line_number, 2);
    assert_eq!(filtered[1].line_number, 4);
    Ok(())
}

#[test]
fn test_various_comment_formats() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Test different comment styles
    let test_cases = [
        "const key = 'secret'; // guardy:ignore",           // C++ style
        "const key = 'secret'; /* guardy:ignore */",        // C style  
        "const key = 'secret'; # guardy:ignore",            // Python/Ruby style
        "const key = 'secret'; -- guardy:ignore",           // SQL style
        "    const key = 'secret'; // guardy:ignore   ",    // With whitespace
        "const key = 'secret'; //guardy:ignore",            // No space after //
    ];
    
    for test_case in &test_cases {
        assert!(filter.line_has_ignore_directive(test_case),
               "Should detect ignore in: {}", test_case);
    }
    Ok(())
}

#[test]
fn test_case_sensitivity() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Test that matching is case-sensitive by default (as expected for precise directives)
    assert!(filter.line_has_ignore_directive("secret = 'test'; // guardy:ignore"));
    assert!(!filter.line_has_ignore_directive("secret = 'test'; // GUARDY:IGNORE"));
    assert!(!filter.line_has_ignore_directive("secret = 'test'; // Guardy:Ignore"));
    Ok(())
}

#[test]
fn test_statistics() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    let matches = vec![
        create_test_match(1, "const secret1 = 'test'; // guardy:ignore"),
        create_test_match(2, "const secret2 = 'test';"),
    ];
    
    let file_content = "const secret1 = 'test'; // guardy:ignore\n\
                       const secret2 = 'test';\n";
    
    // Process matches to generate stats
    let _ = filter.filter_matches(&matches, file_content);
    
    let stats = filter.get_stats();
    assert_eq!(stats.matches_checked, 2);
    assert_eq!(stats.matches_ignored_by_comment, 1);
    assert!(stats.lines_scanned_for_comments >= 2);
    assert!(stats.ignore_comments_found >= 1);
    Ok(())
}

#[test]
fn test_reset_stats() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Generate some stats
    let test_match = create_test_match(1, "secret = 'test'; // guardy:ignore");
    let _ = filter.should_ignore_match(&test_match, "secret = 'test'; // guardy:ignore\n");
    
    assert!(filter.get_stats().matches_checked > 0);
    
    // Reset stats
    filter.reset_stats();
    let stats = filter.get_stats();
    assert_eq!(stats.matches_checked, 0);
    assert_eq!(stats.matches_ignored_by_comment, 0);
    assert_eq!(stats.lines_scanned_for_comments, 0);
    assert_eq!(stats.ignore_comments_found, 0);
    Ok(())
}

#[test]
fn test_edge_cases() -> Result<()> {
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Test empty line
    assert!(!filter.line_has_ignore_directive(""));
    
    // Test line with just comment
    assert!(filter.line_has_ignore_directive("// guardy:ignore"));
    
    // Test multiple ignore directives on same line
    assert!(filter.line_has_ignore_directive("secret = 'test'; // guardy:ignore guardy:ignore-line"));
    
    // Test ignore directive as part of larger word (should not match)
    assert!(!filter.line_has_ignore_directive("const guardyIgnoreThis = 'test';"));
    Ok(())
}

#[test]
fn test_performance_characteristics() -> Result<()> {
    use std::time::Instant;
    
    let config = create_test_config();
    let filter = CommentFilter::new(&config)?;
    
    // Create many matches to test performance
    let matches: Vec<SecretMatch> = (0..1000)
        .map(|i| create_test_match(i + 1, "const secret = 'test';"))
        .collect();
    
    let file_content = (0..1000)
        .map(|_| "const secret = 'test';\n")
        .collect::<String>();
    
    // Test that comment filtering is fast
    let start = Instant::now();
    let _ = filter.filter_matches(&matches, &file_content);
    let duration = start.elapsed();
    
    // Should complete 1000 comment checks quickly
    assert!(duration.as_millis() < 50, "Comment filtering too slow: {:?}", duration);
    
    // Verify stats were updated
    let stats = filter.get_stats();
    assert_eq!(stats.matches_checked, 1000);
    Ok(())
}