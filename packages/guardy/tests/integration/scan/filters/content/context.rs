use guardy::scan::filters::content::ContextPrefilter;
use guardy::scan::pattern::Pattern;
use guardy::scan::types::ScannerConfig;
use anyhow::Result;

fn create_test_patterns() -> Vec<Pattern> {
    vec![
        Pattern {
            id: "api_key".to_string(),
            description: "API Key Pattern".to_string(),
            regex: regex::Regex::new(r"api[_-]?key\s*[:=]\s*['\"]([a-zA-Z0-9]{32})['\"]").unwrap(),
            keywords: vec!["api", "key".to_string()],
            entropy: 4.0,
            secret_group: 1,
        },
        Pattern {
            id: "token".to_string(),
            description: "Token Pattern".to_string(),
            regex: regex::Regex::new(r"token\s*[:=]\s*['\"]([a-zA-Z0-9]{40})['\"]").unwrap(),
            keywords: vec!["token".to_string()],
            entropy: 4.5,
            secret_group: 1,
        },
    ]
}

#[test]
fn test_context_prefilter_creation() -> Result<()> {
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Should successfully create prefilter
    assert!(prefilter.is_enabled());
    Ok(())
}

#[test]
fn test_keyword_extraction() -> Result<()> {
    let patterns = create_test_patterns();
    let keywords = ContextPrefilter::extract_keywords_from_patterns(&patterns);
    
    // Should extract keywords from patterns
    assert!(keywords.contains("api"));
    assert!(keywords.contains("key"));
    assert!(keywords.contains("token"));
    assert_eq!(keywords.len(), 3); // Deduplicated
    Ok(())
}

#[test]
fn test_content_prefiltering() -> Result<()> {
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Content that should pass prefiltering (contains keywords)
    let relevant_content = "const api_key = 'abcd1234567890abcd1234567890abcd';";
    assert!(prefilter.should_process_content(relevant_content)?);
    
    // Content that should be filtered out (no keywords)
    let irrelevant_content = "const username = 'john_doe';";
    assert!(!prefilter.should_process_content(irrelevant_content)?);
    
    Ok(())
}

#[test]
fn test_case_insensitive_matching() -> Result<()> {
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Test different cases
    let test_cases = [
        "const API_KEY = 'test';",
        "const api_key = 'test';", 
        "const Api_Key = 'test';",
        "TOKEN = 'something';",
        "token = 'something';",
        "Token = 'something';",
    ];
    
    for case in &test_cases {
        assert!(prefilter.should_process_content(case)?,
               "Should match case-insensitive keyword in: {}", case);
    }
    Ok(())
}

#[test]
fn test_multiple_keywords() -> Result<()> {
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Content with multiple keywords should pass
    let multi_keyword_content = "const config = { api: 'test', key: 'value', token: 'abc' };";
    assert!(prefilter.should_process_content(multi_keyword_content)?);
    
    Ok(())
}

#[test]
fn test_performance_characteristics() -> Result<()> {
    use std::time::Instant;
    
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Create large content to test performance
    let large_content = "const data = {\n".to_string() + 
        &(0..1000)
            .map(|i| format!("  field_{}: 'value_{}',\n", i, i))
            .collect::<String>() + 
        "  api_key: 'abcd1234567890abcd1234567890abcd'\n};";
    
    // Test that Aho-Corasick prefiltering is fast
    let start = Instant::now();
    let result = prefilter.should_process_content(&large_content)?;
    let duration = start.elapsed();
    
    assert!(result); // Should find the api_key
    assert!(duration.as_millis() < 10, "Prefiltering too slow: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_empty_patterns() -> Result<()> {
    let empty_patterns = Vec::new();
    
    // Should handle empty patterns gracefully
    let result = ContextPrefilter::new(&empty_patterns);
    assert!(result.is_ok());
    
    let prefilter = result?;
    
    // With no patterns, should not process any content
    assert!(!prefilter.should_process_content("any content")?);
    
    Ok(())
}

#[test]
fn test_statistics() -> Result<()> {
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Process some content to generate stats
    let _ = prefilter.should_process_content("api_key = 'test'")?;
    let _ = prefilter.should_process_content("username = 'test'")?;
    let _ = prefilter.should_process_content("token = 'test'")?;
    
    let stats = prefilter.get_stats();
    assert_eq!(stats.content_checked, 3);
    assert_eq!(stats.content_passed_prefilter, 2); // api_key and token
    assert_eq!(stats.content_filtered_out, 1); // username
    assert!(stats.total_prefilter_time_ms >= 0);
    
    Ok(())
}

#[test]
fn test_reset_stats() -> Result<()> {
    let patterns = create_test_patterns();
    let prefilter = ContextPrefilter::new(&patterns)?;
    
    // Generate some stats
    let _ = prefilter.should_process_content("api_key = 'test'")?;
    assert!(prefilter.get_stats().content_checked > 0);
    
    // Reset stats
    prefilter.reset_stats();
    let stats = prefilter.get_stats();
    assert_eq!(stats.content_checked, 0);
    assert_eq!(stats.content_passed_prefilter, 0);
    assert_eq!(stats.content_filtered_out, 0);
    assert_eq!(stats.total_prefilter_time_ms, 0);
    
    Ok(())
}