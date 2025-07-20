use anyhow::Result;
use regex::Regex;
use crate::config::GuardyConfig;

#[derive(Debug, Clone)]
pub struct SecretPattern {
    pub name: String,
    pub regex: Regex,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SecretPatterns {
    pub patterns: Vec<SecretPattern>,
}

impl SecretPatterns {
    pub fn new(config: &GuardyConfig) -> Result<Self> {
        let mut patterns = Vec::new();
        
        // Add predefined patterns (extracted from ripsecrets)
        patterns.extend(Self::predefined_patterns()?);
        
        // Add custom patterns from config
        if let Ok(custom_patterns) = config.get_section("scanner.custom_patterns") {
            if let Some(array) = custom_patterns.as_array() {
                for (i, pattern) in array.iter().enumerate() {
                    if let Some(pattern_str) = pattern.as_str() {
                        match Regex::new(pattern_str) {
                            Ok(regex) => {
                                patterns.push(SecretPattern {
                                    name: format!("Custom Pattern {}", i + 1),
                                    regex,
                                    description: "User-defined pattern".to_string(),
                                });
                            }
                            Err(e) => {
                                eprintln!("Warning: Invalid custom regex pattern '{}': {}", pattern_str, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(SecretPatterns { patterns })
    }
    
    /// Predefined secret patterns extracted from ripsecrets
    /// These patterns are designed to work with entropy analysis
    fn predefined_patterns() -> Result<Vec<SecretPattern>> {
        let patterns = vec![
            // URLs with credentials
            SecretPattern {
                name: "URL with Credentials".to_string(),
                regex: Regex::new(r"[A-Za-z]+://\S{3,50}:(\S{8,50})@[\dA-Za-z#%&+./:=?_~-]+")?,
                description: "URLs containing embedded credentials".to_string(),
            },
            
            // JWT/JWE tokens
            SecretPattern {
                name: "JWT/JWE Token".to_string(),
                regex: Regex::new(r"\beyJ[\dA-Za-z=_-]+(?:\.[\dA-Za-z=_-]{3,}){1,4}")?,
                description: "JSON Web Tokens and JSON Web Encryption".to_string(),
            },
            
            // GitHub tokens
            SecretPattern {
                name: "GitHub Token".to_string(),
                regex: Regex::new(r"(?:gh[oprsu]|github_pat)_[\dA-Za-z_]{36}")?,
                description: "GitHub personal access tokens".to_string(),
            },
            
            // GitLab tokens
            SecretPattern {
                name: "GitLab Token".to_string(),
                regex: Regex::new(r"glpat-[\dA-Za-z_=-]{20,22}")?,
                description: "GitLab personal access tokens".to_string(),
            },
            
            // Stripe API keys
            SecretPattern {
                name: "Stripe API Key".to_string(),
                regex: Regex::new(r"[rs]k_live_[\dA-Za-z]{24,247}")?,
                description: "Stripe API keys (live environment)".to_string(),
            },
            
            // Square API keys
            SecretPattern {
                name: "Square API Key".to_string(),
                regex: Regex::new(r"sq0[ic][a-z]{2}-[\dA-Za-z_-]{22,50}")?,
                description: "Square API keys".to_string(),
            },
            
            // Square additional format
            SecretPattern {
                name: "Square Token".to_string(),
                regex: Regex::new(r"EAAA[\dA-Za-z+=-]{60}")?,
                description: "Square access tokens".to_string(),
            },
            
            // Azure Storage
            SecretPattern {
                name: "Azure Storage Key".to_string(),
                regex: Regex::new(r"AccountKey=[\d+/=A-Za-z]{88}")?,
                description: "Azure Storage account keys".to_string(),
            },
            
            // Google Cloud Platform
            SecretPattern {
                name: "GCP API Key".to_string(),
                regex: Regex::new(r"AIzaSy[\dA-Za-z_-]{33}")?,
                description: "Google Cloud Platform API keys".to_string(),
            },
            
            // npm tokens
            SecretPattern {
                name: "npm Token (Modern)".to_string(),
                regex: Regex::new(r"npm_[\dA-Za-z]{36}")?,
                description: "npm authentication tokens (modern format)".to_string(),
            },
            
            // npm legacy tokens
            SecretPattern {
                name: "npm Token (Legacy)".to_string(),
                regex: Regex::new(r"//.+/:_authToken=[\dA-Za-z_-]+")?,
                description: "npm authentication tokens (legacy format)".to_string(),
            },
            
            // Slack tokens
            SecretPattern {
                name: "Slack Token".to_string(),
                regex: Regex::new(r"xox[aboprs]-(?:\d+-)+[\da-z]+")?,
                description: "Slack API tokens".to_string(),
            },
            
            // Slack webhooks
            SecretPattern {
                name: "Slack Webhook".to_string(),
                regex: Regex::new(r"https://hooks\.slack\.com/services/T[\dA-Za-z_]+/B[\dA-Za-z_]+/[\dA-Za-z_]+")?,
                description: "Slack incoming webhook URLs".to_string(),
            },
            
            // SendGrid
            SecretPattern {
                name: "SendGrid API Key".to_string(),
                regex: Regex::new(r"SG\.[\dA-Za-z_-]{22}\.[\dA-Za-z_-]{43}")?,
                description: "SendGrid API keys".to_string(),
            },
            
            // Twilio
            SecretPattern {
                name: "Twilio API Key".to_string(),
                regex: Regex::new(r"(?:AC|SK)[\da-z]{32}")?,
                description: "Twilio API keys and tokens".to_string(),
            },
            
            // Mailchimp
            SecretPattern {
                name: "Mailchimp API Key".to_string(),
                regex: Regex::new(r"[\da-f]{32}-us\d{1,2}")?,
                description: "Mailchimp API keys".to_string(),
            },
            
            // Intra42
            SecretPattern {
                name: "Intra42 Token".to_string(),
                regex: Regex::new(r"s-s4t2(?:af|ud)-[\da-f]{64}")?,
                description: "42 School Intra API tokens".to_string(),
            },
            
            // PuTTY private key
            SecretPattern {
                name: "PuTTY Private Key".to_string(),
                regex: Regex::new(r"PuTTY-User-Key-File-2")?,
                description: "PuTTY private key files".to_string(),
            },
            
            // Age secret key
            SecretPattern {
                name: "Age Secret Key".to_string(),
                regex: Regex::new(r"AGE-SECRET-KEY-1[\dA-Z]{58}")?,
                description: "Age encryption secret keys".to_string(),
            },
            
            // Private key headers
            SecretPattern {
                name: "DSA Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN DSA PRIVATE KEY-{5}")?,
                description: "DSA private key headers".to_string(),
            },
            
            SecretPattern {
                name: "EC Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN EC PRIVATE KEY-{5}")?,
                description: "Elliptic Curve private key headers".to_string(),
            },
            
            SecretPattern {
                name: "OpenSSH Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN OPENSSH PRIVATE KEY-{5}")?,
                description: "OpenSSH private key headers".to_string(),
            },
            
            SecretPattern {
                name: "PGP Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN PGP PRIVATE KEY BLOCK-{5}")?,
                description: "PGP private key headers".to_string(),
            },
            
            SecretPattern {
                name: "PKCS Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN PRIVATE KEY-{5}")?,
                description: "PKCS#8 private key headers".to_string(),
            },
            
            SecretPattern {
                name: "RSA Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN RSA PRIVATE KEY-{5}")?,
                description: "RSA private key headers".to_string(),
            },
            
            SecretPattern {
                name: "SSH2 Encrypted Private Key".to_string(),
                regex: Regex::new(r"-{5}BEGIN SSH2 ENCRYPTED PRIVATE KEY-{5}")?,
                description: "SSH2 encrypted private key headers".to_string(),
            },
            
            // Modern AI API Keys (2024-2025)
            SecretPattern {
                name: "OpenAI API Key (New Format)".to_string(),
                regex: Regex::new(r"sk-proj-[\dA-Za-z]{43,64}")?,
                description: "OpenAI API keys (new project-based format)".to_string(),
            },
            
            SecretPattern {
                name: "OpenAI API Key (Legacy)".to_string(),
                regex: Regex::new(r"sk-[\dA-Za-z]{43,51}")?,
                description: "OpenAI API keys (legacy format)".to_string(),
            },
            
            SecretPattern {
                name: "Anthropic Claude API Key".to_string(),
                regex: Regex::new(r"sk-ant-api\d{2}-[\dA-Za-z_-]{43,95}")?,
                description: "Anthropic Claude API keys".to_string(),
            },
            
            SecretPattern {
                name: "Hugging Face Token".to_string(),
                regex: Regex::new(r"hf_[\dA-Za-z]{37}")?,
                description: "Hugging Face API tokens".to_string(),
            },
            
            SecretPattern {
                name: "Cohere API Key".to_string(),
                regex: Regex::new(r"co\.[\dA-Za-z_-]{20,}")?,
                description: "Cohere API keys".to_string(),
            },
            
            SecretPattern {
                name: "Replicate API Token".to_string(),
                regex: Regex::new(r"r8_[\dA-Za-z]{40,}")?,
                description: "Replicate API tokens".to_string(),
            },
            
            SecretPattern {
                name: "Mistral AI API Key".to_string(),
                regex: Regex::new(r"[\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12}")?,
                description: "Mistral AI API keys (UUID format)".to_string(),
            },
            
            // Additional cloud providers
            SecretPattern {
                name: "AWS Access Key".to_string(),
                regex: Regex::new(r"AKIA[0-9A-Z]{16}")?,
                description: "Amazon Web Services access keys".to_string(),
            },
            
            SecretPattern {
                name: "AWS Secret Key".to_string(),
                regex: Regex::new(r"(?i:aws.{0,20}secret.{0,20}key.{0,20}[:=]\s*['\x22]?[0-9a-zA-Z/+=]{40}['\x22]?)")?,
                description: "Amazon Web Services secret access keys".to_string(),
            },
            
            SecretPattern {
                name: "Azure Client Secret".to_string(),
                regex: Regex::new(r"(?i:azure.{0,20}client.{0,20}secret.{0,20}[:=]\s*['\x22]?[0-9a-zA-Z.~_-]{34,40}['\x22]?)")?,
                description: "Azure application client secrets".to_string(),
            },
            
            // Database connection strings
            SecretPattern {
                name: "MongoDB Connection String".to_string(),
                regex: Regex::new(r"mongodb(\+srv)?://[^\s'\x22]+:[^\s'\x22]+@[^\s'\x22]+")?,
                description: "MongoDB connection strings with credentials".to_string(),
            },
            
            SecretPattern {
                name: "PostgreSQL Connection String".to_string(),
                regex: Regex::new(r"postgres(ql)?://[^\s'\x22]+:[^\s'\x22]+@[^\s'\x22]+")?,
                description: "PostgreSQL connection strings with credentials".to_string(),
            },
            
            SecretPattern {
                name: "MySQL Connection String".to_string(),
                regex: Regex::new(r"mysql://[^\s'\x22]+:[^\s'\x22]+@[^\s'\x22]+")?,
                description: "MySQL connection strings with credentials".to_string(),
            },
            
            // Generic high-entropy pattern (the main workhorse)
            // This is the key pattern that catches unknown secrets via context + entropy
            SecretPattern {
                name: "Generic Secret Pattern".to_string(),
                regex: Regex::new(r"(?i:key|token|secret|password|api|auth|credential|pass)\w*[\x22']?]?\s*(?:[:=]|:=|=>|<-|>)\s*[\t \x22'\x60]?([\w+./=~\-\\\x60\^]{15,90})")?,
                description: "Generic pattern for detecting potential secrets based on context keywords".to_string(),
            },
        ];
        
        Ok(patterns)
    }
    
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
    
    pub fn get_pattern_names(&self) -> Vec<&str> {
        self.patterns.iter().map(|p| p.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_predefined_patterns() {
        let patterns = SecretPatterns::predefined_patterns().unwrap();
        assert!(!patterns.is_empty());
        
        // Test that we have the key generic pattern
        let has_generic = patterns.iter().any(|p| p.name.contains("Generic"));
        assert!(has_generic, "Should have generic secret pattern");
    }
    
    #[test]
    fn test_jwt_pattern() {
        let patterns = SecretPatterns::predefined_patterns().unwrap();
        let jwt_pattern = patterns.iter().find(|p| p.name.contains("JWT")).unwrap();
        
        // Test valid JWT
        let test_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(jwt_pattern.regex.is_match(test_jwt));
    }
    
    #[test]
    fn test_github_pattern() {
        let patterns = SecretPatterns::predefined_patterns().unwrap();
        let github_pattern = patterns.iter().find(|p| p.name.contains("GitHub")).unwrap();
        
        // Test GitHub token format (36 chars after ghp_)
        let test_token = "ghp_wJbFxR9mK3qL7sP2vN8dH5zC4gY6tA1eXyZ9";
        assert!(github_pattern.regex.is_match(test_token));
    }
}