//! Commit message utilities
//!
//! This module provides functionality for validating commit messages,
//! checking conventional commit format, and other commit-related operations.

use regex::Regex;

/// Check if commit message follows conventional commit format
pub fn is_conventional_commit(message: &str) -> bool {
    // Basic conventional commit pattern: type(scope): description
    // or type: description
    let patterns = [
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .+",
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)!(\(.+\))?: .+", // breaking change
    ];
    
    for pattern in &patterns {
        if Regex::new(pattern).unwrap().is_match(message) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_conventional_commit() {
        // Valid conventional commits
        assert!(is_conventional_commit("feat: add new feature"));
        assert!(is_conventional_commit("fix: resolve bug"));
        assert!(is_conventional_commit("feat(auth): add login functionality"));
        assert!(is_conventional_commit("fix(api): handle error cases"));
        assert!(is_conventional_commit("chore: update dependencies"));
        assert!(is_conventional_commit("feat!: breaking change"));
        assert!(is_conventional_commit("fix!(api): breaking fix"));
        
        // Invalid conventional commits
        assert!(!is_conventional_commit("add new feature"));
        assert!(!is_conventional_commit("bug fix"));
        assert!(!is_conventional_commit("feat:"));
        assert!(!is_conventional_commit("invalid: message"));
        assert!(!is_conventional_commit(""));
    }
}