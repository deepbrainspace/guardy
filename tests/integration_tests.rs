use guardy::git::GitRepo;
use guardy::scanner::{Scanner, SecretPatterns};
use guardy::config::GuardyConfig;
use anyhow::Result;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_git_operations() -> Result<()> {
    let git_repo = GitRepo::discover()?;
    
    // Test file discovery operations
    let staged_files = git_repo.get_staged_files()?;
    let unstaged_files = git_repo.get_unstaged_files()?;
    let uncommitted_files = git_repo.get_uncommitted_files()?;
    
    // Should not panic and return valid results
    assert!(staged_files.len() <= uncommitted_files.len());
    assert!(unstaged_files.len() <= uncommitted_files.len());
    
    // Test git repo properties
    assert!(git_repo.current_branch().is_ok());
    assert!(git_repo.workdir().is_some());
    
    println!("✓ Git operations working: {} staged, {} unstaged, {} total uncommitted", 
             staged_files.len(), unstaged_files.len(), uncommitted_files.len());
    
    Ok(())
}

#[test]
fn test_scanner_creation_and_patterns() -> Result<()> {
    let config = GuardyConfig::load()?;
    let scanner = Scanner::new(&config)?;
    let patterns = SecretPatterns::new(&config)?;
    
    // Should have loaded patterns successfully
    assert!(patterns.pattern_count() > 20, "Should have at least 20 patterns");
    
    // Test that we have modern AI patterns
    let pattern_names = patterns.get_pattern_names();
    let has_openai = pattern_names.iter().any(|name| name.contains("OpenAI"));
    let has_claude = pattern_names.iter().any(|name| name.contains("Claude"));
    let has_generic = pattern_names.iter().any(|name| name.contains("Generic"));
    
    assert!(has_openai, "Should have OpenAI patterns");
    assert!(has_claude, "Should have Claude patterns");
    assert!(has_generic, "Should have generic secret pattern");
    
    println!("✓ Scanner created with {} patterns including modern AI keys", patterns.pattern_count());
    
    Ok(())
}

#[test]
fn test_scanner_with_git_integration() -> Result<()> {
    let config = GuardyConfig::load()?;
    let scanner = Scanner::new(&config)?;
    let git_repo = GitRepo::discover()?;
    
    // Test scanning git-discovered files
    let uncommitted_files = git_repo.get_uncommitted_files()?;
    
    if !uncommitted_files.is_empty() {
        let result = scanner.scan_paths(&uncommitted_files)?;
        
        // Should complete without errors
        assert!(result.stats.files_scanned <= uncommitted_files.len());
        
        println!("✓ Git-Scanner integration: scanned {} files from git, found {} matches", 
                 result.stats.files_scanned, result.stats.total_matches);
    } else {
        println!("✓ Git-Scanner integration: no uncommitted files to scan");
    }
    
    Ok(())
}

#[test]
fn test_scanner_with_test_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = GuardyConfig::load()?;
    let scanner = Scanner::new(&config)?;
    
    // Create test files with various secret patterns
    let test_file1 = temp_dir.path().join("secrets.env");
    let test_file2 = temp_dir.path().join("config.json");
    
    fs::write(&test_file1, r#"
# Test environment file  
STRIPE_KEY=sk_live_4eC39HqLyjWDarjtT1zdp7dc
API_SECRET=J8fH9ks2Xm4pB7qN5rG8dF3vC6wA9zE2
GITHUB_TOKEN=ghp_wJbFxR9mK3qL7sP2vN8dH5zC4gY6tA1e
"#)?;
    
    fs::write(&test_file2, r#"{
  "openai_api_key": "sk-proj-K9mR7xL3qF8bN2vG5sH4jD6pA8cE7wZ1",
  "database_url": "postgres://user:Hs7Gf9Kp2Xm@localhost/db",
  "jwt_secret": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
}"#)?;
    
    // Test scanning individual files
    let result1 = scanner.scan_file(&test_file1)?;
    let result2 = scanner.scan_file(&test_file2)?;
    
    assert!(!result1.is_empty(), "Should detect secrets in .env file");
    assert!(!result2.is_empty(), "Should detect secrets in .json file");
    
    // Test scanning directory
    let dir_result = scanner.scan_directory(temp_dir.path())?;
    assert!(dir_result.stats.files_scanned >= 2, "Should scan both test files");
    assert!(dir_result.stats.total_matches > 0, "Should find secrets in test files");
    
    println!("✓ Scanner detected {} secrets across {} files", 
             dir_result.stats.total_matches, dir_result.stats.files_scanned);
    
    Ok(())
}

#[test]
fn test_entropy_analysis() -> Result<()> {
    use guardy::scanner::is_likely_secret;
    
    // Test entropy analysis with known patterns - use realistic values  
    assert!(is_likely_secret(b"sk_live_4eC39HqLyjWDarjtT1zdp7dc", 1.0 / 1e5), "Should detect Stripe live key");
    assert!(is_likely_secret(b"ghp_wJbFxR9mK3qL7sP2vN8dH5zC4gY6tA1e", 1.0 / 1e5), "Should detect GitHub token");
    
    // Test with realistic OpenAI key format (base64-like random characters)
    assert!(is_likely_secret(b"sk-proj-K9mR7xL3qF8bN2vG5sH4jD6pA8cE7wZ1", 1.0 / 1e5), "Should detect OpenAI API key");
    
    // Should reject obvious non-secrets
    assert!(!is_likely_secret(b"API_KEY_CONSTANT", 1.0 / 1e5), "Should reject constant string");
    assert!(!is_likely_secret(b"hello_world_test", 1.0 / 1e5), "Should reject simple words");
    assert!(!is_likely_secret(b"123456789", 1.0 / 1e5), "Should reject simple numbers");
    
    println!("✓ Entropy analysis correctly filtering secrets vs non-secrets");
    
    Ok(())
}

#[test]
fn test_gitignore_intelligence() -> Result<()> {
    use guardy::scanner::{GitignoreIntelligence, ProjectType};
    
    let temp_dir = TempDir::new()?;
    
    // Test Rust project detection
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"")?;
    let intelligence = GitignoreIntelligence::new(temp_dir.path());
    let project_type = intelligence.detect_project_type()?;
    assert_eq!(project_type, ProjectType::Rust);
    
    // Test suggestions
    let suggestions = intelligence.suggest_improvements()?;
    assert!(suggestions.iter().any(|s| s.pattern.contains("target/")), "Should suggest target/ for Rust");
    assert!(suggestions.iter().any(|s| s.pattern.contains(".env")), "Should suggest .env");
    
    println!("✓ Gitignore intelligence correctly detected Rust project and provided {} suggestions", 
             suggestions.len());
    
    Ok(())
}