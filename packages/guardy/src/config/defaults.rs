//! Native Rust defaults for Guardy configuration
//! 
//! This module provides zero-cost defaults using native Rust types
//! with cache-line optimization for hot path fields.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::scan::types::ScanMode;

/// Main Guardy configuration with Arc-wrapped sub-configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardyConfig {
    pub general: Arc<GeneralConfig>,
    pub hooks: Arc<HooksConfig>,
    pub scanner: Arc<ScannerConfig>,
    pub security: Arc<SecurityConfig>,
    pub branch_protection: Arc<BranchProtectionConfig>,
    pub git_crypt: Arc<GitCryptConfig>,
    pub formatting: Arc<FormattingConfig>,
    pub package_manager: Arc<PackageManagerConfig>,
    pub mcp: Arc<McpConfig>,
    pub external_tools: Arc<ExternalToolsConfig>,
    pub sync: Arc<SyncConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub debug: bool,
    pub color: bool,
    pub interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(rename = "pre-commit")]
    pub pre_commit: HookConfig,
    #[serde(rename = "commit-msg")]
    pub commit_msg: HookConfig,
    #[serde(rename = "post-checkout")]
    pub post_checkout: HookConfig,
    #[serde(rename = "pre-push")]
    pub pre_push: HookConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub enabled: bool,
    pub builtin: Arc<Vec<String>>,
    pub custom: Arc<Vec<CustomCommand>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub command: String,
    pub description: String,
    pub fail_on_error: bool,
}

/// Scanner configuration with hot/cold field separation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Hot fields - accessed on every file scan (cache-line optimized)
    #[serde(flatten)]
    pub hot: ScannerHotConfig,
    /// Cold fields - rarely accessed (Arc-wrapped)
    #[serde(flatten)]
    pub cold: Arc<ScannerColdConfig>,
}

/// Hot path scanner config - fits in single 64-byte cache line
#[repr(C, align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerHotConfig {
    pub mode: ScanMode,               // 1 byte (enum)
    pub max_threads: u16,             // 2 bytes
    pub include_binary: bool,         // 1 byte
    pub follow_symlinks: bool,        // 1 byte
    pub max_file_size_mb: u32,        // 4 bytes
    pub enable_entropy_analysis: bool, // 1 byte
    pub entropy_threshold: f32,       // 4 bytes (f32 instead of f64)
    pub ignore_test_code: bool,       // 1 byte
    #[serde(skip)]
    _padding: [u8; 49],               // Pad to exactly 64 bytes
}

/// Cold scanner config - separate allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerColdConfig {
    pub thread_percentage: u8,
    pub min_files_for_parallel: usize,
    pub ignore_paths: Arc<Vec<String>>,
    pub ignore_patterns: Arc<Vec<String>>,
    pub ignore_comments: Arc<Vec<String>>,
    pub custom_patterns: Arc<Vec<String>>,
    pub test_attributes: Arc<Vec<String>>,
    pub test_modules: Arc<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub patterns: Arc<Vec<String>>,
    pub exclude_files: Arc<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionConfig {
    pub protected_branches: Arc<Vec<String>>,
    pub allow_direct_commits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCryptConfig {
    pub enabled: bool,
    pub required_files: Arc<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingConfig {
    pub enabled: bool,
    pub command: String,
    pub auto_fix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    pub preferred: String,
    pub auto_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub port: u16,
    pub host: String,
    pub enabled: bool,
    pub tools: Arc<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalToolsConfig {
    pub git_crypt: String,
    pub nx: String,
    pub pnpm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub repos: Arc<Vec<RepoConfig>>,
    pub protection: SyncProtectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub name: String,
    pub repo: String,
    pub version: String,
    pub source_path: String,
    pub dest_path: String,
    pub include: Arc<Vec<String>>,
    pub exclude: Arc<Vec<String>>,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProtectionConfig {
    pub auto_protect_synced: bool,
    pub block_modifications: bool,
}

impl Default for GuardyConfig {
    fn default() -> Self {
        GuardyConfig {
            general: Arc::new(GeneralConfig {
                debug: false,
                color: true,
                interactive: true,
            }),
            
            hooks: Arc::new(HooksConfig {
                pre_commit: HookConfig {
                    enabled: true,
                    builtin: Arc::new(vec!["scan_secrets".into()]),
                    custom: Arc::new(vec![]),
                },
                commit_msg: HookConfig {
                    enabled: false,
                    builtin: Arc::new(vec![]),
                    custom: Arc::new(vec![]),
                },
                post_checkout: HookConfig {
                    enabled: false,
                    builtin: Arc::new(vec![]),
                    custom: Arc::new(vec![]),
                },
                pre_push: HookConfig {
                    enabled: false,
                    builtin: Arc::new(vec![]),
                    custom: Arc::new(vec![]),
                },
            }),
            
            scanner: Arc::new(ScannerConfig {
                hot: ScannerHotConfig {
                    mode: ScanMode::Auto,
                    max_threads: 0,  // 0 = auto-detect
                    include_binary: false,
                    follow_symlinks: false,
                    max_file_size_mb: 10,
                    enable_entropy_analysis: true,
                    entropy_threshold: 0.00001,
                    ignore_test_code: true,
                    _padding: [0; 49],
                },
                cold: Arc::new(ScannerColdConfig {
                    thread_percentage: 75,
                    min_files_for_parallel: 50,
                    ignore_paths: Arc::new(vec![
                        // Test directories
                        "tests/*".into(),
                        "testdata/*".into(),
                        "__tests__/*".into(),
                        "test/*".into(),
                        // Test files
                        "*_test.rs".into(),
                        "test_*.rs".into(),
                        "test_*.py".into(),
                        "*_test.py".into(),
                        "*.test.ts".into(),
                        "*.test.js".into(),
                        "*.spec.ts".into(),
                        "*.spec.js".into(),
                        // Git internals
                        ".git/objects/**".into(),
                        ".git_disabled/**".into(),
                        ".git/refs/**".into(),
                        ".git/logs/**".into(),
                        ".git/index".into(),
                        "**/.git/objects/**".into(),
                        "**/.git_disabled/**".into(),
                    ]),
                    ignore_patterns: Arc::new(vec![
                        "# TEST_SECRET:".into(),
                        "DEMO_KEY_".into(),
                        "FAKE_".into(),
                    ]),
                    ignore_comments: Arc::new(vec![
                        "guardy:ignore".into(),
                        "guardy:ignore-line".into(),
                        "guardy:ignore-next".into(),
                    ]),
                    custom_patterns: Arc::new(vec![]),
                    test_attributes: Arc::new(vec![
                        // Rust
                        "#[*test]".into(),
                        "#[bench]".into(),
                        "#[cfg(test)]".into(),
                        // Python
                        "def test_*".into(),
                        "class Test*".into(),
                        "@pytest.*".into(),
                        // TypeScript/JavaScript
                        "it(*".into(),
                        "test(*".into(),
                        "describe(*".into(),
                    ]),
                    test_modules: Arc::new(vec![
                        // Rust
                        "mod tests {".into(),
                        "mod test {".into(),
                        // Python
                        "class Test".into(),
                        // TypeScript/JavaScript
                        "describe(".into(),
                        "__tests__".into(),
                    ]),
                }),
            }),
            
            security: Arc::new(SecurityConfig {
                patterns: Arc::new(vec![
                    "sk-[a-zA-Z0-9]{48}".into(),           // OpenAI API keys
                    "ghp_[a-zA-Z0-9]{36}".into(),          // GitHub personal access tokens
                    "ey[a-zA-Z0-9]{20,}".into(),           // JWT tokens
                    "['\"'][a-zA-Z0-9+/]{32,}['\"]".into(), // Base64 encoded secrets
                ]),
                exclude_files: Arc::new(vec![
                    "*.lock".into(),
                    "*.log".into(),
                    ".husky/*".into(),
                ]),
            }),
            
            branch_protection: Arc::new(BranchProtectionConfig {
                protected_branches: Arc::new(vec![
                    "main".into(),
                    "master".into(),
                    "develop".into(),
                ]),
                allow_direct_commits: false,
            }),
            
            git_crypt: Arc::new(GitCryptConfig {
                enabled: true,
                required_files: Arc::new(vec![]),
            }),
            
            formatting: Arc::new(FormattingConfig {
                enabled: true,
                command: "nx format:write --uncommitted".into(),
                auto_fix: false,
            }),
            
            package_manager: Arc::new(PackageManagerConfig {
                preferred: "pnpm".into(),
                auto_install: true,
            }),
            
            mcp: Arc::new(McpConfig {
                port: 8080,
                host: "127.0.0.1".into(),
                enabled: false,
                tools: Arc::new(vec![
                    "git-status".into(),
                    "hook-run".into(),
                    "security-scan".into(),
                ]),
            }),
            
            external_tools: Arc::new(ExternalToolsConfig {
                git_crypt: "git-crypt".into(),
                nx: "nx".into(),
                pnpm: "pnpm".into(),
            }),
            
            sync: Arc::new(SyncConfig {
                repos: Arc::new(vec![]),
                protection: SyncProtectionConfig {
                    auto_protect_synced: true,
                    block_modifications: true,
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    
    #[test]
    fn test_hot_config_cache_line_size() {
        // Ensure hot config fits in exactly 64 bytes
        assert_eq!(mem::size_of::<ScannerHotConfig>(), 64);
        assert_eq!(mem::align_of::<ScannerHotConfig>(), 64);
    }
    
    #[test]
    fn test_default_config_loads() {
        let config = GuardyConfig::default();
        assert_eq!(config.scanner.hot.mode, ScanMode::Auto);
        assert_eq!(config.scanner.cold.thread_percentage, 75);
        assert!(config.hooks.pre_commit.enabled);
    }
    
    #[test]
    fn test_arc_clone_is_cheap() {
        let config = GuardyConfig::default();
        let start = std::time::Instant::now();
        let _clone = config.clone();
        let duration = start.elapsed();
        // Should be <100ns (Arc increment only)
        assert!(duration.as_nanos() < 100);
    }
}