//! Base pattern definitions for secret detection
//!
//! This module contains the default patterns as compile-time constants.
//! No runtime parsing or allocation - pure Rust data structures.

/// Base pattern definition - compile-time data
#[derive(Debug, Clone)]
pub struct BasePattern {
    pub name: &'static str,
    pub regex: &'static str,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
    pub priority: u8,
}

/// Base pattern definitions - compile-time data, zero runtime parsing
pub const BASE_PATTERNS: &[BasePattern] = &[
    // Modern AI API Keys (High Priority)
    BasePattern {
        name: "OpenAI API Key (New Format)",
        regex: r"sk-proj-[\dA-Za-z]{43,64}",
        description: "OpenAI API keys (new project-based format)",
        keywords: &["sk-proj-"],
        priority: 9,
    },
    BasePattern {
        name: "OpenAI API Key (Legacy)",
        regex: r"sk-[\dA-Za-z]{43,51}",
        description: "OpenAI API keys (legacy format)",
        keywords: &["sk-"],
        priority: 9,
    },
    BasePattern {
        name: "Anthropic Claude API Key",
        regex: r"sk-ant-api\d{2}-[\dA-Za-z_-]{43,95}",
        description: "Anthropic Claude API keys",
        keywords: &["sk-ant-api"],
        priority: 9,
    },
    BasePattern {
        name: "Anthropic Admin API Key",
        regex: r"sk-ant-admin-[\dA-Za-z_-]{43,95}",
        description: "Anthropic Admin API keys",
        keywords: &["sk-ant-admin"],
        priority: 9,
    },
    BasePattern {
        name: "Hugging Face Token",
        regex: r"hf_[\dA-Za-z]{37}",
        description: "Hugging Face API tokens",
        keywords: &["hf_"],
        priority: 9,
    },
    BasePattern {
        name: "Cohere API Key",
        regex: r"co\.[\dA-Za-z_-]{20,}",
        description: "Cohere API keys",
        keywords: &["co."],
        priority: 8,
    },
    BasePattern {
        name: "Replicate API Token",
        regex: r"r8_[\dA-Za-z]{40,}",
        description: "Replicate API tokens",
        keywords: &["r8_"],
        priority: 8,
    },

    // Version Control & Git
    BasePattern {
        name: "GitHub Token",
        regex: r"(?:gh[oprsu]|github_pat)_[\dA-Za-z_]{36}",
        description: "GitHub personal access tokens",
        keywords: &["ghp_", "gho_", "ghr_", "ghs_", "ghu_", "github_pat"],
        priority: 8,
    },
    BasePattern {
        name: "GitLab Token",
        regex: r"glpat-[\dA-Za-z_=-]{20,22}",
        description: "GitLab personal access tokens",
        keywords: &["glpat-"],
        priority: 8,
    },

    // Cloud Providers (High Priority)
    BasePattern {
        name: "AWS Access Key",
        regex: r"AKIA[0-9A-Z]{16}",
        description: "Amazon Web Services access keys",
        keywords: &["AKIA"],
        priority: 8,
    },
    BasePattern {
        name: "AWS Secret Key",
        regex: r#"(?i:aws.{0,20}secret.{0,20}key.{0,20}[:=]\s*['"]?[0-9a-zA-Z/+=]{40}['"]?)"#,
        description: "Amazon Web Services secret access keys",
        keywords: &["aws", "secret", "key"],
        priority: 8,
    },
    BasePattern {
        name: "GCP API Key",
        regex: r"AIzaSy[\dA-Za-z_-]{33}",
        description: "Google Cloud Platform API keys",
        keywords: &["AIzaSy"],
        priority: 8,
    },
    BasePattern {
        name: "Azure Storage Key",
        regex: r"AccountKey=[\d+/=A-Za-z]{88}",
        description: "Azure Storage account keys",
        keywords: &["AccountKey="],
        priority: 8,
    },
    BasePattern {
        name: "Azure Client Secret",
        regex: r#"(?i:azure.{0,20}client.{0,20}secret.{0,20}[:=]\s*['"]?[0-9a-zA-Z.~_-]{34,40}['"]?)"#,
        description: "Azure application client secrets",
        keywords: &["azure", "client", "secret"],
        priority: 7,
    },
    BasePattern {
        name: "Alibaba Access Key",
        regex: r"(LTAI)[\dA-Za-z]{12,20}",
        description: "Alibaba Cloud access keys",
        keywords: &["LTAI"],
        priority: 7,
    },

    // Payment Processors
    BasePattern {
        name: "Stripe API Key",
        regex: r"[rs]k_live_[\dA-Za-z]{24,247}",
        description: "Stripe API keys (live environment)",
        keywords: &["sk_live_", "rk_live_"],
        priority: 8,
    },
    BasePattern {
        name: "Square API Key",
        regex: r"sq0[ic][a-z]{2}-[\dA-Za-z_-]{22,50}",
        description: "Square API keys",
        keywords: &["sq0"],
        priority: 7,
    },
    BasePattern {
        name: "Square Token",
        regex: r"EAAA[\dA-Za-z+=-]{60}",
        description: "Square access tokens",
        keywords: &["EAAA"],
        priority: 7,
    },

    // Communication & Messaging
    BasePattern {
        name: "Slack Token",
        regex: r"xox[aboprs]-(?:\d+-)+[\da-z]+",
        description: "Slack API tokens",
        keywords: &["xox"],
        priority: 7,
    },
    BasePattern {
        name: "Slack Webhook",
        regex: r"https://hooks\.slack\.com/services/T[\dA-Za-z_]+/B[\dA-Za-z_]+/[\dA-Za-z_]+",
        description: "Slack incoming webhook URLs",
        keywords: &["hooks.slack.com"],
        priority: 7,
    },
    BasePattern {
        name: "SendGrid API Key",
        regex: r"SG\.[\dA-Za-z_-]{22}\.[\dA-Za-z_-]{43}",
        description: "SendGrid API keys",
        keywords: &["SG."],
        priority: 7,
    },
    BasePattern {
        name: "Twilio API Key",
        regex: r"(?:AC|SK)[\da-z]{32}",
        description: "Twilio API keys and tokens",
        keywords: &["AC", "SK"],
        priority: 6,
    },
    BasePattern {
        name: "Mailchimp API Key",
        regex: r"[\da-f]{32}-us\d{1,2}",
        description: "Mailchimp API keys",
        keywords: &["us"],
        priority: 5,
    },

    // Package Managers & Registries
    BasePattern {
        name: "npm Token (Modern)",
        regex: r"npm_[\dA-Za-z]{36}",
        description: "npm authentication tokens (modern format)",
        keywords: &["npm_"],
        priority: 7,
    },
    BasePattern {
        name: "npm Token (Legacy)",
        regex: r"//.+/:_authToken=[\dA-Za-z_-]+",
        description: "npm authentication tokens (legacy format)",
        keywords: &["_authToken="],
        priority: 7,
    },

    // Cryptographic Keys & Certificates
    BasePattern {
        name: "Private Key (Comprehensive)",
        regex: r"(?s)-----BEGIN[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----[\s\S]{64,}?-----END[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----",
        description: "Comprehensive private key detection including RSA, DSA, EC, OpenSSH, PGP with full content",
        keywords: &["-----BEGIN", "PRIVATE KEY"],
        priority: 8,
    },
    BasePattern {
        name: "SSL/TLS Certificate",
        regex: r"(?s)-----BEGIN[ A-Z0-9_-]{0,100}CERTIFICATE[ A-Z0-9_-]{0,100}-----[\s\S]{64,}?-----END[ A-Z0-9_-]{0,100}CERTIFICATE[ A-Z0-9_-]{0,100}-----",
        description: "SSL/TLS certificates and certificate signing requests with full content",
        keywords: &["-----BEGIN", "CERTIFICATE"],
        priority: 6,
    },
    BasePattern {
        name: "Certificate Signing Request",
        regex: r"(?s)-----BEGIN[ A-Z0-9_-]{0,100}CERTIFICATE REQUEST[ A-Z0-9_-]{0,100}-----[\s\S]{64,}?-----END[ A-Z0-9_-]{0,100}CERTIFICATE REQUEST[ A-Z0-9_-]{0,100}-----",
        description: "Certificate Signing Requests (CSR) with full content",
        keywords: &["-----BEGIN", "CERTIFICATE REQUEST"],
        priority: 6,
    },
    BasePattern {
        name: "SSH Public Key Content",
        regex: r"ssh-(?:rsa|dss|ed25519|ecdsa-sha2-nistp(?:256|384|521))\s+[A-Za-z0-9+/]{100,}={0,2}",
        description: "SSH public key content in authorized_keys format",
        keywords: &["ssh-rsa", "ssh-dss", "ssh-ed25519", "ssh-ecdsa"],
        priority: 6,
    },
    BasePattern {
        name: "Age Secret Key",
        regex: r"AGE-SECRET-KEY-1[\dA-Z]{58}",
        description: "Age encryption secret keys",
        keywords: &["AGE-SECRET-KEY"],
        priority: 7,
    },
    BasePattern {
        name: "PuTTY Private Key",
        regex: r"PuTTY-User-Key-File-\d+",
        description: "PuTTY private key files",
        keywords: &["PuTTY-User-Key"],
        priority: 6,
    },
    BasePattern {
        name: "1Password Secret Key",
        regex: r"op://[\dA-Za-z/\-]{10,}",
        description: "1Password secret references",
        keywords: &["op://"],
        priority: 7,
    },

    // JWT & Authentication Tokens
    BasePattern {
        name: "JWT/JWE Token",
        regex: r"\beyJ[\dA-Za-z=_-]+(?:\.[\dA-Za-z=_-]{3,}){1,4}",
        description: "JSON Web Tokens and JSON Web Encryption",
        keywords: &["eyJ"],
        priority: 7,
    },

    // Database Connection Strings
    BasePattern {
        name: "MongoDB Connection String",
        regex: r#"mongodb(\+srv)?://[^\s'"]+:[^\s'"]+@[^\s'"]+"#,
        description: "MongoDB connection strings with credentials",
        keywords: &["mongodb://", "mongodb+srv://"],
        priority: 7,
    },
    BasePattern {
        name: "PostgreSQL Connection String",
        regex: r#"postgres(ql)?://[^\s'"]+:[^\s'"]+@[^\s'"]+"#,
        description: "PostgreSQL connection strings with credentials",
        keywords: &["postgresql://", "postgres://"],
        priority: 7,
    },
    BasePattern {
        name: "MySQL Connection String",
        regex: r#"mysql://[^\s'"]+:[^\s'"]+@[^\s'"]+"#,
        description: "MySQL connection strings with credentials",
        keywords: &["mysql://"],
        priority: 7,
    },

    // URLs with Credentials
    BasePattern {
        name: "URL with Credentials",
        regex: r"[A-Za-z]+://\S{3,50}:(\S{8,50})@[\dA-Za-z#%&+./:=?_~-]+",
        description: "URLs containing embedded credentials",
        keywords: &["://"],
        priority: 6,
    },

    // Additional Services
    BasePattern {
        name: "Airtable API Key",
        regex: r"key[\dA-Za-z]{14}",
        description: "Airtable API keys",
        keywords: &["key"],
        priority: 5,
    },
    BasePattern {
        name: "Intra42 Token",
        regex: r"s-s4t2(?:af|ud)-[\da-f]{64}",
        description: "42 School Intra API tokens",
        keywords: &["s-s4t2"],
        priority: 6,
    },
    BasePattern {
        name: "Mistral AI API Key",
        regex: r"[\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12}",
        description: "Mistral AI API keys (UUID format)",
        keywords: &[],
        priority: 5,
    },

    // Private Key Header (legacy compatibility)
    BasePattern {
        name: "Private Key Header",
        regex: r"-----BEGIN[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----",
        description: "Private key headers (for backward compatibility)",
        keywords: &["-----BEGIN", "PRIVATE KEY"],
        priority: 3,
    },

    // Generic Secret Pattern (main workhorse for unknown formats)
    BasePattern {
        name: "Generic Secret Pattern",
        regex: r#"(?i:key|token|secret|password|api|auth|credential|pass)\w*["']?]?\s*(?:[:=]|:=|=>|<-|>)\s*[\t "'`]?([\w+./=~\-\\`^]{15,90})"#,
        description: "Generic pattern for detecting potential secrets based on context keywords",
        keywords: &["key", "token", "secret", "password", "api", "auth", "credential", "pass"],
        priority: 2,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base_patterns_available() {
        assert!(BASE_PATTERNS.len() > 40, "Should have all patterns converted");
        
        // Check a few key patterns exist
        let pattern_names: Vec<&str> = BASE_PATTERNS.iter().map(|p| p.name).collect();
        assert!(pattern_names.iter().any(|&name| name.contains("OpenAI")));
        assert!(pattern_names.iter().any(|&name| name.contains("GitHub")));
        assert!(pattern_names.iter().any(|&name| name.contains("AWS")));
    }
    
    #[test]
    fn test_all_patterns_have_valid_fields() {
        for pattern in BASE_PATTERNS {
            assert!(!pattern.name.is_empty(), "Pattern name should not be empty");
            assert!(!pattern.regex.is_empty(), "Pattern regex should not be empty");
            assert!(!pattern.description.is_empty(), "Pattern description should not be empty");
            assert!(pattern.priority >= 1 && pattern.priority <= 10, "Priority should be 1-10");
        }
    }
}