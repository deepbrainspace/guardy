//! Package manager detection and utilities
//!
//! This module provides functionality to detect and work with various package managers
//! across different project types and platforms.

use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Supported package managers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    /// pnpm - Fast, disk space efficient package manager
    Pnpm,
    /// npm - Node.js package manager
    Npm,
    /// yarn - Fast, reliable, and secure dependency management
    Yarn,
    /// cargo - Rust package manager
    Cargo,
    /// pip - Python package installer
    Pip,
    /// poetry - Python dependency management
    Poetry,
    /// pipenv - Python development workflow
    Pipenv,
    /// go mod - Go modules
    GoMod,
    /// composer - PHP dependency manager
    Composer,
    /// bundle - Ruby gem bundler
    Bundle,
    /// maven - Java build automation
    Maven,
    /// gradle - Build automation tool
    Gradle,
    /// swift - Swift package manager
    Swift,
    /// deno - Deno package manager
    Deno,
    /// bun - JavaScript runtime and package manager
    Bun,
}

impl PackageManager {
    /// Get the command name for this package manager
    pub fn command(&self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm",
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Cargo => "cargo",
            PackageManager::Pip => "pip",
            PackageManager::Poetry => "poetry",
            PackageManager::Pipenv => "pipenv",
            PackageManager::GoMod => "go",
            PackageManager::Composer => "composer",
            PackageManager::Bundle => "bundle",
            PackageManager::Maven => "mvn",
            PackageManager::Gradle => "gradle",
            PackageManager::Swift => "swift",
            PackageManager::Deno => "deno",
            PackageManager::Bun => "bun",
        }
    }

    /// Get the display name for this package manager
    pub fn display_name(&self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm",
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "Yarn",
            PackageManager::Cargo => "Cargo",
            PackageManager::Pip => "pip",
            PackageManager::Poetry => "Poetry",
            PackageManager::Pipenv => "Pipenv",
            PackageManager::GoMod => "Go Modules",
            PackageManager::Composer => "Composer",
            PackageManager::Bundle => "Bundler",
            PackageManager::Maven => "Maven",
            PackageManager::Gradle => "Gradle",
            PackageManager::Swift => "Swift PM",
            PackageManager::Deno => "Deno",
            PackageManager::Bun => "Bun",
        }
    }

    /// Get the install command for this package manager
    pub fn install_command(&self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm install",
            PackageManager::Npm => "npm install",
            PackageManager::Yarn => "yarn install",
            PackageManager::Cargo => "cargo build",
            PackageManager::Pip => "pip install -r requirements.txt",
            PackageManager::Poetry => "poetry install",
            PackageManager::Pipenv => "pipenv install",
            PackageManager::GoMod => "go mod download",
            PackageManager::Composer => "composer install",
            PackageManager::Bundle => "bundle install",
            PackageManager::Maven => "mvn install",
            PackageManager::Gradle => "gradle build",
            PackageManager::Swift => "swift package resolve",
            PackageManager::Deno => "deno cache",
            PackageManager::Bun => "bun install",
        }
    }

    /// Get the lockfile name for this package manager
    pub fn lockfile(&self) -> Option<&'static str> {
        match self {
            PackageManager::Pnpm => Some("pnpm-lock.yaml"),
            PackageManager::Npm => Some("package-lock.json"),
            PackageManager::Yarn => Some("yarn.lock"),
            PackageManager::Cargo => Some("Cargo.lock"),
            PackageManager::Pip => None, // pip doesn't have a standard lockfile
            PackageManager::Poetry => Some("poetry.lock"),
            PackageManager::Pipenv => Some("Pipfile.lock"),
            PackageManager::GoMod => Some("go.sum"),
            PackageManager::Composer => Some("composer.lock"),
            PackageManager::Bundle => Some("Gemfile.lock"),
            PackageManager::Maven => None, // Maven doesn't have a standard lockfile
            PackageManager::Gradle => Some("gradle.lockfile"),
            PackageManager::Swift => Some("Package.resolved"),
            PackageManager::Deno => Some("deno.lock"),
            PackageManager::Bun => Some("bun.lockb"),
        }
    }

    /// Check if this package manager is available on the system
    pub fn is_available(&self) -> bool {
        crate::utils::SystemUtils::command_exists(self.command())
    }
}

/// Package manager detection result
#[derive(Debug, Clone)]
pub struct PackageManagerInfo {
    /// The primary package manager detected
    pub primary: PackageManager,
    /// Alternative package managers detected
    pub alternatives: Vec<PackageManager>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Reasoning for the detection
    pub reasoning: String,
}

/// Package manager detector
pub struct PackageManagerDetector {
    /// Preference order for package managers (higher index = higher preference)
    preference_order: HashMap<PackageManager, u8>,
}

impl Default for PackageManagerDetector {
    fn default() -> Self {
        let mut preference_order = HashMap::new();
        
        // JavaScript/TypeScript preferences (pnpm > yarn > npm)
        preference_order.insert(PackageManager::Pnpm, 10);
        preference_order.insert(PackageManager::Yarn, 8);
        preference_order.insert(PackageManager::Npm, 6);
        preference_order.insert(PackageManager::Bun, 9);
        preference_order.insert(PackageManager::Deno, 7);
        
        // Python preferences (poetry > pipenv > pip)
        preference_order.insert(PackageManager::Poetry, 10);
        preference_order.insert(PackageManager::Pipenv, 8);
        preference_order.insert(PackageManager::Pip, 6);
        
        // Other languages
        preference_order.insert(PackageManager::Cargo, 10);
        preference_order.insert(PackageManager::GoMod, 10);
        preference_order.insert(PackageManager::Composer, 10);
        preference_order.insert(PackageManager::Bundle, 10);
        preference_order.insert(PackageManager::Maven, 8);
        preference_order.insert(PackageManager::Gradle, 9);
        preference_order.insert(PackageManager::Swift, 10);
        
        Self { preference_order }
    }
}

impl PackageManagerDetector {
    /// Create a new package manager detector
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom preference order
    pub fn with_preferences(mut self, preferences: HashMap<PackageManager, u8>) -> Self {
        self.preference_order = preferences;
        self
    }

    /// Detect package managers in the given directory
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> Result<Vec<PackageManagerInfo>> {
        let path = path.as_ref();
        let mut results = Vec::new();
        
        // JavaScript/TypeScript projects
        if path.join("package.json").exists() {
            if let Some(info) = self.detect_js_package_manager(path)? {
                results.push(info);
            }
        }
        
        // Rust projects
        if path.join("Cargo.toml").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Cargo,
                alternatives: vec![],
                confidence: 1.0,
                reasoning: "Cargo.toml found".to_string(),
            });
        }
        
        // Python projects
        if let Some(info) = self.detect_python_package_manager(path)? {
            results.push(info);
        }
        
        // Go projects
        if path.join("go.mod").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::GoMod,
                alternatives: vec![],
                confidence: 1.0,
                reasoning: "go.mod found".to_string(),
            });
        }
        
        // PHP projects
        if path.join("composer.json").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Composer,
                alternatives: vec![],
                confidence: 1.0,
                reasoning: "composer.json found".to_string(),
            });
        }
        
        // Ruby projects
        if path.join("Gemfile").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Bundle,
                alternatives: vec![],
                confidence: 1.0,
                reasoning: "Gemfile found".to_string(),
            });
        }
        
        // Java projects
        if path.join("pom.xml").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Maven,
                alternatives: vec![PackageManager::Gradle],
                confidence: 0.9,
                reasoning: "pom.xml found".to_string(),
            });
        }
        
        if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Gradle,
                alternatives: vec![PackageManager::Maven],
                confidence: 0.9,
                reasoning: "build.gradle found".to_string(),
            });
        }
        
        // Swift projects
        if path.join("Package.swift").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Swift,
                alternatives: vec![],
                confidence: 1.0,
                reasoning: "Package.swift found".to_string(),
            });
        }
        
        // Deno projects
        if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            results.push(PackageManagerInfo {
                primary: PackageManager::Deno,
                alternatives: vec![],
                confidence: 1.0,
                reasoning: "deno.json found".to_string(),
            });
        }
        
        Ok(results)
    }

    /// Detect JavaScript/TypeScript package manager
    fn detect_js_package_manager<P: AsRef<Path>>(&self, path: P) -> Result<Option<PackageManagerInfo>> {
        let path = path.as_ref();
        let mut detected = Vec::new();
        let mut reasoning_parts = Vec::new();
        
        // Check for lockfiles (strongest indicator)
        if path.join("pnpm-lock.yaml").exists() {
            detected.push((PackageManager::Pnpm, 1.0));
            reasoning_parts.push("pnpm-lock.yaml found".to_string());
        }
        
        if path.join("yarn.lock").exists() {
            detected.push((PackageManager::Yarn, 1.0));
            reasoning_parts.push("yarn.lock found".to_string());
        }
        
        if path.join("package-lock.json").exists() {
            detected.push((PackageManager::Npm, 1.0));
            reasoning_parts.push("package-lock.json found".to_string());
        }
        
        if path.join("bun.lockb").exists() {
            detected.push((PackageManager::Bun, 1.0));
            reasoning_parts.push("bun.lockb found".to_string());
        }
        
        // Check for package.json packageManager field
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(package_manager) = package_json.get("packageManager") {
                    if let Some(pm_str) = package_manager.as_str() {
                        if pm_str.starts_with("pnpm") {
                            detected.push((PackageManager::Pnpm, 0.9));
                            reasoning_parts.push("packageManager field specifies pnpm".to_string());
                        } else if pm_str.starts_with("yarn") {
                            detected.push((PackageManager::Yarn, 0.9));
                            reasoning_parts.push("packageManager field specifies yarn".to_string());
                        } else if pm_str.starts_with("npm") {
                            detected.push((PackageManager::Npm, 0.9));
                            reasoning_parts.push("packageManager field specifies npm".to_string());
                        } else if pm_str.starts_with("bun") {
                            detected.push((PackageManager::Bun, 0.9));
                            reasoning_parts.push("packageManager field specifies bun".to_string());
                        }
                    }
                }
            }
        }
        
        // If no specific indicators, fall back to system availability and preferences
        if detected.is_empty() {
            for pm in &[PackageManager::Pnpm, PackageManager::Bun, PackageManager::Yarn, PackageManager::Npm] {
                if pm.is_available() {
                    let preference = self.preference_order.get(pm).unwrap_or(&5);
                    detected.push((pm.clone(), *preference as f64 / 10.0));
                    let reason = format!("{} is available", pm.display_name());
                    reasoning_parts.push(reason);
                }
            }
        }
        
        if detected.is_empty() {
            return Ok(None);
        }
        
        // Sort by confidence and preference
        detected.sort_by(|a, b| {
            let a_pref = self.preference_order.get(&a.0).unwrap_or(&5);
            let b_pref = self.preference_order.get(&b.0).unwrap_or(&5);
            
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b_pref.cmp(a_pref))
        });
        
        let primary = detected[0].0.clone();
        let alternatives = detected[1..].iter().map(|(pm, _)| pm.clone()).collect();
        let confidence = detected[0].1;
        
        Ok(Some(PackageManagerInfo {
            primary,
            alternatives,
            confidence,
            reasoning: reasoning_parts.join(", "),
        }))
    }

    /// Detect Python package manager
    fn detect_python_package_manager<P: AsRef<Path>>(&self, path: P) -> Result<Option<PackageManagerInfo>> {
        let path = path.as_ref();
        let mut detected = Vec::new();
        let mut reasoning_parts = Vec::new();
        
        // Check for specific files
        if path.join("pyproject.toml").exists() && path.join("poetry.lock").exists() {
            detected.push((PackageManager::Poetry, 1.0));
            reasoning_parts.push("poetry.lock found".to_string());
        } else if path.join("pyproject.toml").exists() {
            detected.push((PackageManager::Poetry, 0.8));
            reasoning_parts.push("pyproject.toml found".to_string());
        }
        
        if path.join("Pipfile").exists() {
            detected.push((PackageManager::Pipenv, 0.9));
            reasoning_parts.push("Pipfile found".to_string());
            
            if path.join("Pipfile.lock").exists() {
                // Increase confidence if lockfile exists
                if let Some(last) = detected.last_mut() {
                    last.1 = 1.0;
                }
                reasoning_parts.push("Pipfile.lock found".to_string());
            }
        }
        
        if path.join("requirements.txt").exists() {
            detected.push((PackageManager::Pip, 0.7));
            reasoning_parts.push("requirements.txt found".to_string());
        }
        
        if detected.is_empty() {
            return Ok(None);
        }
        
        // Sort by confidence and preference
        detected.sort_by(|a, b| {
            let a_pref = self.preference_order.get(&a.0).unwrap_or(&5);
            let b_pref = self.preference_order.get(&b.0).unwrap_or(&5);
            
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b_pref.cmp(a_pref))
        });
        
        let primary = detected[0].0.clone();
        let alternatives = detected[1..].iter().map(|(pm, _)| pm.clone()).collect();
        let confidence = detected[0].1;
        
        Ok(Some(PackageManagerInfo {
            primary,
            alternatives,
            confidence,
            reasoning: reasoning_parts.join(", "),
        }))
    }

    /// Get the best package manager for a project
    pub fn get_primary<P: AsRef<Path>>(&self, path: P) -> Result<Option<PackageManager>> {
        let results = self.detect(path)?;
        Ok(results.into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .map(|info| info.primary))
    }

    /// Get all available package managers on the system
    pub fn get_available_package_managers(&self) -> Vec<PackageManager> {
        let all_managers = vec![
            PackageManager::Pnpm,
            PackageManager::Npm,
            PackageManager::Yarn,
            PackageManager::Bun,
            PackageManager::Deno,
            PackageManager::Cargo,
            PackageManager::Pip,
            PackageManager::Poetry,
            PackageManager::Pipenv,
            PackageManager::GoMod,
            PackageManager::Composer,
            PackageManager::Bundle,
            PackageManager::Maven,
            PackageManager::Gradle,
            PackageManager::Swift,
        ];
        
        all_managers.into_iter()
            .filter(|pm| pm.is_available())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_package_manager_command() {
        assert_eq!(PackageManager::Pnpm.command(), "pnpm");
        assert_eq!(PackageManager::Npm.command(), "npm");
        assert_eq!(PackageManager::Yarn.command(), "yarn");
        assert_eq!(PackageManager::Cargo.command(), "cargo");
    }

    #[test]
    fn test_package_manager_lockfile() {
        assert_eq!(PackageManager::Pnpm.lockfile(), Some("pnpm-lock.yaml"));
        assert_eq!(PackageManager::Npm.lockfile(), Some("package-lock.json"));
        assert_eq!(PackageManager::Yarn.lockfile(), Some("yarn.lock"));
        assert_eq!(PackageManager::Cargo.lockfile(), Some("Cargo.lock"));
    }

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create Cargo.toml
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").expect("Failed to write Cargo.toml");
        
        let detector = PackageManagerDetector::new();
        let results = detector.detect(temp_dir.path()).expect("Failed to detect");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].primary, PackageManager::Cargo);
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn test_detect_node_project_with_pnpm_lock() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create package.json and pnpm-lock.yaml
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).expect("Failed to write package.json");
        fs::write(temp_dir.path().join("pnpm-lock.yaml"), "lockfileVersion: 5.4").expect("Failed to write pnpm-lock.yaml");
        
        let detector = PackageManagerDetector::new();
        let results = detector.detect(temp_dir.path()).expect("Failed to detect");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].primary, PackageManager::Pnpm);
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn test_detect_python_project_with_poetry() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create pyproject.toml and poetry.lock
        fs::write(temp_dir.path().join("pyproject.toml"), "[tool.poetry]\nname = \"test\"").expect("Failed to write pyproject.toml");
        fs::write(temp_dir.path().join("poetry.lock"), "[[package]]").expect("Failed to write poetry.lock");
        
        let detector = PackageManagerDetector::new();
        let results = detector.detect(temp_dir.path()).expect("Failed to detect");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].primary, PackageManager::Poetry);
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn test_detect_go_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create go.mod
        fs::write(temp_dir.path().join("go.mod"), "module test").expect("Failed to write go.mod");
        
        let detector = PackageManagerDetector::new();
        let results = detector.detect(temp_dir.path()).expect("Failed to detect");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].primary, PackageManager::GoMod);
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn test_package_manager_availability() {
        // This is a basic test - we can't guarantee what's available on the test system
        let available = PackageManagerDetector::new().get_available_package_managers();
        
        // The list should contain only available package managers
        for pm in available {
            assert!(pm.is_available());
        }
    }
}