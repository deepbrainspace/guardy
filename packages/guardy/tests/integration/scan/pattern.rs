use guardy::scan::pattern::Pattern;
use guardy::scan::types::ScannerConfig;
use anyhow::Result;

#[test]
fn test_pattern_loading() -> Result<()> {
    let config = ScannerConfig::default();
    let patterns = Pattern::load_patterns(&config)?;

    // Should load built-in patterns
    assert!(!patterns.is_empty(), "Should load at least some patterns");

    // Check that patterns have required fields
    for pattern in &patterns {
        assert!(!pattern.id.is_empty(), "Pattern ID should not be empty");
        assert!(!pattern.description.is_empty(), "Pattern description should not be empty");
        assert!(pattern.entropy >= 0.0, "Entropy should be non-negative");
        assert!(pattern.secret_group > 0, "Secret group should be positive");
    }

    Ok(())
}

#[test]
fn test_pattern_caching() -> Result<()> {
    use std::time::Instant;

    let config = ScannerConfig::default();

    // First load
    let start = Instant::now();
    let patterns1 = Pattern::load_patterns(&config)?;
    let first_load_time = start.elapsed();

    // Second load (should use cache)
    let start = Instant::now();
    let patterns2 = Pattern::load_patterns(&config)?;
    let second_load_time = start.elapsed();

    // Should get same patterns
    assert_eq!(patterns1.len(), patterns2.len());

    // Second load should be significantly faster (cached)
    assert!(second_load_time < first_load_time / 2,
           "Second load should be much faster due to caching");

    Ok(())
}

#[test]
fn test_pattern_fields() -> Result<()> {
    let config = ScannerConfig::default();
    let patterns = Pattern::load_patterns(&config)?;

    // Find a known pattern type to validate
    let api_patterns: Vec<_> = patterns.iter()
        .filter(|p| p.id.to_lowercase().contains("api") ||
                   p.description.to_lowercase().contains("api"))
        .collect();

    assert!(!api_patterns.is_empty(), "Should have API-related patterns");

    for pattern in api_patterns {
        // Validate regex compiles
        let test_text = "api_key = 'test123'";
        let _ = pattern.regex.is_match(test_text); // Should not panic

        // Validate keywords are reasonable
        assert!(!pattern.keywords.is_empty(),
               "Pattern {} should have keywords", pattern.id);

        for keyword in &pattern.keywords {
            assert!(!keyword.is_empty(), "Keywords should not be empty");
            assert!(keyword.len() >= 2, "Keywords should be at least 2 characters");
        }
    }

    Ok(())
}

#[test]
fn test_shared_pattern_cache() -> Result<()> {
    let config1 = ScannerConfig::default();
    let config2 = ScannerConfig::default();

    // Load patterns with different config instances
    let patterns1 = Pattern::load_patterns(&config1)?;
    let patterns2 = Pattern::load_patterns(&config2)?;

    // Should get identical results (same shared cache)
    assert_eq!(patterns1.len(), patterns2.len());

    // Validate they're actually the same patterns
    for (p1, p2) in patterns1.iter().zip(patterns2.iter()) {
        assert_eq!(p1.id, p2.id);
        assert_eq!(p1.description, p2.description);
        assert_eq!(p1.entropy, p2.entropy);
        assert_eq!(p1.secret_group, p2.secret_group);
    }

    Ok(())
}

#[test]
fn test_pattern_regex_compilation() -> Result<()> {
    let config = ScannerConfig::default();
    let patterns = Pattern::load_patterns(&config)?;

    // Test that all regex patterns compile and work
    for pattern in &patterns {
        // Test with some sample text
        let sample_texts = [
            "api_key = 'abcd1234567890'",
            "token: \"xyz789abc123\"",
            "password = secret123",
            "const key = 'test_value'",
        ];

        for text in &sample_texts {
            // Should not panic when matching
            let _matches = pattern.regex.is_match(text);

            // Test capture groups if regex matches
            if pattern.regex.is_match(text) {
                let captures = pattern.regex.captures(text);
                if let Some(caps) = captures {
                    // Should have at least the secret group
                    if pattern.secret_group <= caps.len() {
                        let secret = caps.get(pattern.secret_group);
                        if let Some(secret_match) = secret {
                            assert!(!secret_match.as_str().is_empty(),
                                   "Captured secret should not be empty for pattern {}",
                                   pattern.id);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_pattern_categories() -> Result<()> {
    let config = ScannerConfig::default();
    let patterns = Pattern::load_patterns(&config)?;

    // Group patterns by common categories
    let mut categories = std::collections::HashMap::new();

    for pattern in &patterns {
        let category = if pattern.id.contains("api") || pattern.description.to_lowercase().contains("api") {
            "api"
        } else if pattern.id.contains("token") || pattern.description.to_lowercase().contains("token") {
            "token"
        } else if pattern.id.contains("key") || pattern.description.to_lowercase().contains("key") {
            "key"
        } else if pattern.id.contains("secret") || pattern.description.to_lowercase().contains("secret") {
            "secret"
        } else if pattern.id.contains("password") || pattern.description.to_lowercase().contains("password") {
            "password"
        } else {
            "other"
        };

        categories.entry(category).or_insert_with(Vec::new).push(pattern);
    }

    // Should have multiple categories
    assert!(categories.len() >= 2, "Should have multiple pattern categories");

    // Each category should have reasonable patterns
    for (category, patterns_in_category) in categories {
        assert!(!patterns_in_category.is_empty(),
               "Category {} should have patterns", category);
    }

    Ok(())
}

#[test]
fn test_pattern_performance() -> Result<()> {
    use std::time::Instant;

    let config = ScannerConfig::default();
    let patterns = Pattern::load_patterns(&config)?;

    // Test performance with larger text
    let large_text = "const config = {\n".to_string() +
        &(0..1000)
            .map(|i| format!("  field_{}: 'value_{}',\n", i, i))
            .collect::<String>() +
        "  api_key: 'abcd1234567890abcd1234567890abcd',\n" +
        "  token: 'xyz789abc123xyz789abc123xyz789abc123'\n" +
        "};";

    let start = Instant::now();

    let mut matches_found = 0;
    for pattern in &patterns {
        if pattern.regex.is_match(&large_text) {
            matches_found += 1;
        }
    }

    let duration = start.elapsed();

    assert!(matches_found > 0, "Should find some matches in test text");
    assert!(duration.as_millis() < 100,
           "Pattern matching should be fast: {:?}", duration);

    Ok(())
}
