use guardy::scan::filters::content::entropy::{EntropyFilter, EntropyFilterStats};
use guardy::scan::types::{ScannerConfig, SecretMatch};
use anyhow::Result;

fn create_test_config(enabled: bool, threshold: f64) -> ScannerConfig {
    ScannerConfig {
        enable_entropy_analysis: enabled,
        min_entropy_threshold: threshold,
        ..ScannerConfig::default()
    }
}

fn create_test_match(matched_text: &str) -> SecretMatch {
    SecretMatch {
        file_path: "test.rs".to_string(),
        line_number: 1,
        line_content: format!("const secret = '{}';", matched_text),
        matched_text: matched_text.to_string(),
        start_pos: 15,
        end_pos: 15 + matched_text.len(),
        secret_type: "Test Secret".to_string(),
        pattern_description: "Test pattern".to_string(),
    }
}

#[test]
fn test_entropy_filter_creation() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    assert!(filter.is_enabled());
    let (enabled, threshold) = filter.get_config_info();
    assert!(enabled);
    assert_eq!(threshold, 1.0 / 1e5);
    Ok(())
}

#[test]
fn test_entropy_filter_disabled() -> Result<()> {
    let config = create_test_config(false, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    assert!(!filter.is_enabled());

    // Should not filter any matches when disabled
    let low_entropy_match = create_test_match("hello_world");
    assert!(!filter.should_filter_match(&low_entropy_match)?);

    let high_entropy_match = create_test_match("random_high_entropy_string_ABC123xyz");
    assert!(!filter.should_filter_match(&high_entropy_match)?);
    Ok(())
}

#[test]
fn test_low_entropy_constants() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    // Test common constants/variables that should be filtered out
    let low_entropy_constants = [
        "hello_world",
        "API_KEY_CONSTANT",
        "PROJECT_NAME_ALIAS",
        "TEST_VALUE_123",
        "DATABASE_URL",
    ];

    for constant in &low_entropy_constants {
        let secret_match = create_test_match(constant);
        let should_filter = filter.should_filter_match(&secret_match)?;
        assert!(should_filter, "Low entropy constant should be filtered: {}", constant);
    }
    Ok(())
}

#[test]
fn test_filter_matches_list() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    let matches = vec![
        create_test_match("AbC123XyZ789QwErTy456UiOpAs"),        // Should pass (high entropy)
        create_test_match("hello_world"),                        // Should be filtered
        create_test_match("rAnDoM987654321ZzXxCvBnMqWeRt"),     // Should pass (high entropy)
        create_test_match("API_KEY_CONSTANT"),                   // Should be filtered
    ];

    let filtered = filter.filter_matches(&matches);

    // Should keep only high-entropy matches
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].matched_text, "AbC123XyZ789QwErTy456UiOpAs");
    assert_eq!(filtered[1].matched_text, "rAnDoM987654321ZzXxCvBnMqWeRt");
    Ok(())
}

#[test]
fn test_validate_string_entropy() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    // Test high entropy strings
    assert!(filter.validate_string_entropy("AbC123XyZ789QwErTy456UiOpAs")?);
    assert!(filter.validate_string_entropy("random_b64_ABC123xyz789")?);

    // Test low entropy strings
    assert!(!filter.validate_string_entropy("hello_world")?);
    assert!(!filter.validate_string_entropy("API_KEY_CONSTANT")?);
    Ok(())
}

#[test]
fn test_entropy_probability_calculation() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    // Test that probability calculation works
    let high_entropy_prob = filter.calculate_entropy_probability("AbC123XyZ789QwErTy456UiOpAs");
    let low_entropy_prob = filter.calculate_entropy_probability("hello_world");

    assert!(high_entropy_prob > low_entropy_prob);
    assert!(high_entropy_prob > 1.0 / 1e5);
    assert!(low_entropy_prob < 1.0 / 1e5);
    Ok(())
}

#[test]
fn test_statistics() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    let matches = vec![
        create_test_match("AbC123XyZ789QwErTy456UiOpAs"),        // High entropy
        create_test_match("hello_world"),                        // Low entropy
        create_test_match("rAnDoM987654321ZzXxCvBnMqWeRt"),     // High entropy
    ];

    // Process matches to generate stats
    let _ = filter.filter_matches(&matches);

    let stats = filter.get_stats();
    assert_eq!(stats.matches_checked, 3);
    assert_eq!(stats.matches_passed_entropy, 2);
    assert_eq!(stats.matches_failed_entropy, 1);
    assert!(stats.total_entropy_analysis_time_ms >= 0);
    assert!(stats.average_entropy_check_time_ms >= 0.0);
    Ok(())
}

#[test]
fn test_reset_stats() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    // Generate some stats
    let test_match = create_test_match("AbC123XyZ789QwErTy456UiOpAs");
    let _ = filter.should_filter_match(&test_match)?;

    assert!(filter.get_stats().matches_checked > 0);

    // Reset stats
    filter.reset_stats();
    let stats = filter.get_stats();
    assert_eq!(stats.matches_checked, 0);
    assert_eq!(stats.matches_passed_entropy, 0);
    assert_eq!(stats.matches_failed_entropy, 0);
    assert_eq!(stats.total_entropy_analysis_time_ms, 0);
    assert_eq!(stats.average_entropy_check_time_ms, 0.0);
    Ok(())
}

#[test]
fn test_different_thresholds() -> Result<()> {
    // Test with very strict threshold (high value)
    let strict_config = create_test_config(true, 1.0 / 1e2);
    let strict_filter = EntropyFilter::new(&strict_config)?;

    // Test with lenient threshold (low value)
    let lenient_config = create_test_config(true, 1.0 / 1e8);
    let lenient_filter = EntropyFilter::new(&lenient_config)?;

    let test_string = "somewhat_random_123";
    let test_match = create_test_match(test_string);

    // Strict threshold should filter more aggressively
    let strict_result = strict_filter.should_filter_match(&test_match)?;
    let lenient_result = lenient_filter.should_filter_match(&test_match)?;

    // Strict should be more likely to filter (or at least not less likely)
    // The exact behavior depends on the entropy of the test string
    assert!(strict_result || !lenient_result || strict_result == lenient_result);
    Ok(())
}

#[test]
fn test_performance_characteristics() -> Result<()> {
    use std::time::Instant;

    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    // Create many matches to test performance
    let matches: Vec<SecretMatch> = (0..1000)
        .map(|i| create_test_match(
            if i % 2 == 0 {
                "AbC123XyZ789QwErTy456UiOpAs"
            } else {
                "hello_world_constant"
            }
        ))
        .collect();

    // Test that entropy filtering is fast
    let start = Instant::now();
    let _ = filter.filter_matches(&matches);
    let duration = start.elapsed();

    // Should complete 1000 entropy checks quickly
    assert!(duration.as_millis() < 100, "Entropy filtering too slow: {:?}", duration);

    // Verify stats were updated
    let stats = filter.get_stats();
    assert_eq!(stats.matches_checked, 1000);
    assert!(stats.total_entropy_analysis_time_ms > 0);
    assert!(stats.average_entropy_check_time_ms > 0.0);
    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    let config = create_test_config(true, 1.0 / 1e5);
    let filter = EntropyFilter::new(&config)?;

    // Test with empty string (edge case)
    let empty_match = create_test_match("");
    let result = filter.should_filter_match(&empty_match);

    // Should handle gracefully (either succeed or provide meaningful error)
    match result {
        Ok(_) => {
            // Success case - empty string handled gracefully
        }
        Err(e) => {
            // Error case - should be meaningful
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty());
        }
    }
    Ok(())
}
