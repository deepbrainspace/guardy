use guardy::scan::secret::SecretMatch;
use guardy::scan::pattern::Pattern;
use guardy::scan::types::ScannerConfig;
use anyhow::Result;

#[test]
fn test_secret_match_creation() {
    let secret_match = SecretMatch {
        rule_id: "test_rule".to_string(),
        description: "Test Secret".to_string(),
        match_text: "secret123".to_string(),
        file_path: "/path/to/file.txt".to_string(),
        line_number: 42,
        column_start: 10,
        column_end: 19,
        entropy: 3.5,
        commit_hash: Some("abcd1234".to_string()),
        author: Some("test@example.com".to_string()),
        timestamp: Some("2023-01-01T12:00:00Z".to_string()),
    };
    
    assert_eq!(secret_match.rule_id, "test_rule");
    assert_eq!(secret_match.description, "Test Secret");
    assert_eq!(secret_match.match_text, "secret123");
    assert_eq!(secret_match.line_number, 42);
    assert_eq!(secret_match.entropy, 3.5);
}

#[test]
fn test_secret_match_validation() -> Result<()> {
    let config = ScannerConfig::default();
    let patterns = Pattern::load_patterns(&config)?;
    
    // Find a pattern to test with
    let pattern = patterns.iter().find(|p| !p.keywords.is_empty())
        .expect("Should have at least one pattern with keywords");
    
    // Test content that should match
    let test_content = "const api_key = 'abcd1234567890abcd1234567890abcd';";
    
    if pattern.regex.is_match(test_content) {
        let captures = pattern.regex.captures(test_content).unwrap();
        let secret = captures.get(pattern.secret_group).unwrap();
        
        let secret_match = SecretMatch {
            rule_id: pattern.id.clone(),
            description: pattern.description.clone(),
            match_text: secret.as_str().to_string(),
            file_path: "test.rs".to_string(),
            line_number: 1,
            column_start: secret.start(),
            column_end: secret.end(),
            entropy: pattern.entropy,
            commit_hash: None,
            author: None,
            timestamp: None,
        };
        
        assert_eq!(secret_match.rule_id, pattern.id);
        assert_eq!(secret_match.description, pattern.description);
        assert!(!secret_match.match_text.is_empty());
        assert_eq!(secret_match.entropy, pattern.entropy);
    }
    
    Ok(())
}

#[test]
fn test_secret_match_with_git_info() {
    let secret_match = SecretMatch {
        rule_id: "github_token".to_string(),
        description: "GitHub Personal Access Token".to_string(),
        match_text: "ghp_1234567890abcdefghij1234567890ABCDEF".to_string(),
        file_path: "config.yaml".to_string(),
        line_number: 15,
        column_start: 8,
        column_end: 45,
        entropy: 4.8,
        commit_hash: Some("abc123def456".to_string()),
        author: Some("developer@company.com".to_string()),
        timestamp: Some("2023-12-01T14:30:00Z".to_string()),
    };
    
    // Validate git information is preserved
    assert_eq!(secret_match.commit_hash, Some("abc123def456".to_string()));
    assert_eq!(secret_match.author, Some("developer@company.com".to_string()));
    assert_eq!(secret_match.timestamp, Some("2023-12-01T14:30:00Z".to_string()));
}

#[test]
fn test_secret_match_without_git_info() {
    let secret_match = SecretMatch {
        rule_id: "api_key".to_string(),
        description: "API Key".to_string(),
        match_text: "ak_1234567890abcdefghij".to_string(),
        file_path: "app.py".to_string(),
        line_number: 8,
        column_start: 15,
        column_end: 37,
        entropy: 4.2,
        commit_hash: None,
        author: None,
        timestamp: None,
    };
    
    // Should handle None values gracefully
    assert!(secret_match.commit_hash.is_none());
    assert!(secret_match.author.is_none());
    assert!(secret_match.timestamp.is_none());
}

#[test]
fn test_secret_match_display() {
    let secret_match = SecretMatch {
        rule_id: "aws_access_key".to_string(),
        description: "AWS Access Key ID".to_string(),
        match_text: "AKIAIOSFODNN7EXAMPLE".to_string(),
        file_path: "src/config.rs".to_string(),
        line_number: 25,
        column_start: 20,
        column_end: 40,
        entropy: 4.1,
        commit_hash: Some("commit123".to_string()),
        author: Some("dev@example.com".to_string()),
        timestamp: Some("2023-11-15T09:30:00Z".to_string()),
    };
    
    // Test that all fields are accessible
    assert_eq!(secret_match.rule_id, "aws_access_key");
    assert_eq!(secret_match.description, "AWS Access Key ID");
    assert_eq!(secret_match.match_text, "AKIAIOSFODNN7EXAMPLE");
    assert_eq!(secret_match.file_path, "src/config.rs");
    assert_eq!(secret_match.line_number, 25);
    assert_eq!(secret_match.column_start, 20);
    assert_eq!(secret_match.column_end, 40);
    assert_eq!(secret_match.entropy, 4.1);
}

#[test]
fn test_secret_match_collections() {
    let mut secrets = Vec::new();
    
    // Add multiple secrets
    secrets.push(SecretMatch {
        rule_id: "rule1".to_string(),
        description: "First Secret".to_string(),
        match_text: "secret1".to_string(),
        file_path: "file1.rs".to_string(),
        line_number: 1,
        column_start: 0,
        column_end: 7,
        entropy: 3.0,
        commit_hash: None,
        author: None,
        timestamp: None,
    });
    
    secrets.push(SecretMatch {
        rule_id: "rule2".to_string(),
        description: "Second Secret".to_string(),
        match_text: "secret2".to_string(),
        file_path: "file2.rs".to_string(),
        line_number: 2,
        column_start: 5,
        column_end: 12,
        entropy: 3.5,
        commit_hash: None,
        author: None,
        timestamp: None,
    });
    
    assert_eq!(secrets.len(), 2);
    
    // Test filtering by entropy
    let high_entropy: Vec<_> = secrets.iter()
        .filter(|s| s.entropy > 3.2)
        .collect();
    assert_eq!(high_entropy.len(), 1);
    assert_eq!(high_entropy[0].rule_id, "rule2");
    
    // Test grouping by file
    let mut by_file = std::collections::HashMap::new();
    for secret in &secrets {
        by_file.entry(&secret.file_path).or_insert_with(Vec::new).push(secret);
    }
    assert_eq!(by_file.len(), 2);
}

#[test]
fn test_secret_match_edge_cases() {
    // Test with empty strings (should be allowed for some fields)
    let secret_match = SecretMatch {
        rule_id: "test".to_string(),
        description: "".to_string(), // Empty description
        match_text: "".to_string(),  // Empty match
        file_path: "test.txt".to_string(),
        line_number: 0, // Zero line number
        column_start: 0,
        column_end: 0,
        entropy: 0.0, // Zero entropy
        commit_hash: Some("".to_string()), // Empty commit hash
        author: Some("".to_string()),      // Empty author
        timestamp: Some("".to_string()),   // Empty timestamp
    };
    
    // Should handle edge cases gracefully
    assert_eq!(secret_match.description, "");
    assert_eq!(secret_match.match_text, "");
    assert_eq!(secret_match.line_number, 0);
    assert_eq!(secret_match.entropy, 0.0);
}

#[test] 
fn test_secret_match_sorting() {
    let mut secrets = vec![
        SecretMatch {
            rule_id: "rule1".to_string(),
            description: "High Entropy".to_string(),
            match_text: "secret1".to_string(),
            file_path: "file.rs".to_string(),
            line_number: 10,
            column_start: 0,
            column_end: 7,
            entropy: 5.0,
            commit_hash: None,
            author: None,
            timestamp: None,
        },
        SecretMatch {
            rule_id: "rule2".to_string(),
            description: "Low Entropy".to_string(),
            match_text: "secret2".to_string(),
            file_path: "file.rs".to_string(),
            line_number: 5,
            column_start: 0,
            column_end: 7,
            entropy: 2.0,
            commit_hash: None,
            author: None,
            timestamp: None,
        },
    ];
    
    // Sort by line number
    secrets.sort_by(|a, b| a.line_number.cmp(&b.line_number));
    assert_eq!(secrets[0].line_number, 5);
    assert_eq!(secrets[1].line_number, 10);
    
    // Sort by entropy (descending)
    secrets.sort_by(|a, b| b.entropy.partial_cmp(&a.entropy).unwrap());
    assert_eq!(secrets[0].entropy, 5.0);
    assert_eq!(secrets[1].entropy, 2.0);
}