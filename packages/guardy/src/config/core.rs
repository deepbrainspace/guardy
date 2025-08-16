//! Core configuration module for Guardy
//!
//! This module provides the central configuration system with:
//! - Zero-cost static configuration via LazyLock
//! - CLI override support baked into defaults
//! - Direct field access for maximum performance

use std::sync::{Arc, LazyLock};
use serde::{Deserialize, Serialize};
use crate::scan::types::ScanMode;

/// Global configuration - initialized once, shared everywhere
pub static CONFIG: LazyLock<Arc<GuardyConfig>> = LazyLock::new(|| {
    // Default::default() handles CLI overrides internally
    Arc::new(GuardyConfig::default())
});

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
    #[serde(default)]
    pub parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub command: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub continue_on_error: bool,
    #[serde(default)]
    pub all_files: bool,
    #[serde(default)]
    pub glob: Arc<Vec<String>>,
    #[serde(default)]
    pub stage_fixed: bool,
}

/// Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub mode: ScanMode,
    pub max_threads: u16,
    pub include_binary: bool,
    pub follow_symlinks: bool,
    pub max_file_size_mb: u32,
    pub enable_entropy_analysis: bool,
    pub entropy_threshold: f32,
    pub ignore_test_code: bool,
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
    #[serde(default = "default_source_path")]
    pub source_path: String,
    #[serde(default = "default_dest_path")]
    pub dest_path: String,
    #[serde(default)]
    pub include: Arc<Vec<String>>,
    #[serde(default)]
    pub exclude: Arc<Vec<String>>,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProtectionConfig {
    pub auto_protect_synced: bool,
    pub block_modifications: bool,
}

impl Default for GuardyConfig {
    fn default() -> Self {
        // Get CLI args if available
        let cli = crate::cli::CLI.get();
        let scan_args = cli.and_then(|c| match &c.command {
            Some(crate::cli::commands::Commands::Scan(args)) => Some(args),
            _ => None,
        });
        
        GuardyConfig {
            general: Arc::new(GeneralConfig {
                debug: cli.map_or(false, |c| c.verbose > 1),
                color: cli.map_or(true, |c| !c.quiet),
                interactive: true,
            }),
            
            hooks: Arc::new(HooksConfig {
                pre_commit: HookConfig {
                    enabled: true,
                    builtin: Arc::new(vec!["scan_secrets".into()]),
                    custom: Arc::new(vec![]),
                    parallel: true,
                },
                commit_msg: HookConfig {
                    enabled: false,
                    builtin: Arc::new(vec![]),
                    custom: Arc::new(vec![]),
                    parallel: true,
                },
                post_checkout: HookConfig {
                    enabled: false,
                    builtin: Arc::new(vec![]),
                    custom: Arc::new(vec![]),
                    parallel: true,
                },
                pre_push: HookConfig {
                    enabled: false,
                    builtin: Arc::new(vec![]),
                    custom: Arc::new(vec![]),
                    parallel: true,
                },
            }),
            
            scanner: Arc::new(ScannerConfig {
                mode: scan_args.and_then(|a| a.mode).unwrap_or(ScanMode::Auto),
                max_threads: scan_args.map_or(0, |a| a.max_file_size as u16),  // 0 = auto-detect
                include_binary: scan_args.map_or(false, |a| a.include_binary),
                follow_symlinks: scan_args.map_or(false, |a| a.follow_symlinks),
                max_file_size_mb: scan_args.map_or(10, |a| a.max_file_size as u32),
                enable_entropy_analysis: scan_args.map_or(true, |a| !a.no_entropy),
                entropy_threshold: scan_args.and_then(|a| a.entropy_threshold)
                    .map_or(0.00001, |t| t as f32),
                ignore_test_code: true,
                thread_percentage: 75,
                min_files_for_parallel: 50,
                ignore_paths: Arc::new({
                    let mut paths = vec![
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
                    ];
                    if let Some(args) = scan_args {
                        paths.extend(args.ignore_paths.iter().cloned());
                    }
                    paths
                }),
                ignore_patterns: Arc::new({
                    let mut patterns = vec![
                        "# TEST_SECRET:".into(),
                        "DEMO_KEY_".into(),
                        "FAKE_".into(),
                    ];
                    if let Some(args) = scan_args {
                        patterns.extend(args.ignore_patterns.iter().cloned());
                    }
                    patterns
                }),
                ignore_comments: Arc::new({
                    let mut comments = vec![
                        "guardy:ignore".into(),
                        "guardy:ignore-line".into(),
                        "guardy:ignore-next".into(),
                    ];
                    if let Some(args) = scan_args {
                        comments.extend(args.ignore_comments.iter().cloned());
                    }
                    comments
                }),
                custom_patterns: Arc::new({
                    let mut patterns = vec![];
                    if let Some(args) = scan_args {
                        patterns.extend(args.custom_patterns.iter().cloned());
                    }
                    patterns
                }),
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

// Default functions for serde
fn default_source_path() -> String {
    ".".to_string()
}

fn default_dest_path() -> String {
    ".".to_string()
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_loads() {
        let config = GuardyConfig::default();
        assert_eq!(config.scanner.mode, ScanMode::Auto);
        assert_eq!(config.scanner.thread_percentage, 75);
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
    
    #[test]
    fn test_global_config_loads() {
        let config = CONFIG.clone();
        assert_eq!(config.scanner.mode, ScanMode::Auto);
        assert!(config.hooks.pre_commit.enabled);
    }
    
    #[test]
    fn test_config_is_singleton() {
        let config1 = CONFIG.clone();
        let config2 = CONFIG.clone();
        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&config1, &config2));
    }
}