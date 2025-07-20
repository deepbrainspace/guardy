use anyhow::Result;
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Java,
    Mixed,
    Unknown,
}

#[derive(Debug)]
pub struct GitignoreWarning {
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug)]
pub enum WarningSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug)]
pub struct GitignoreSuggestion {
    pub pattern: String,
    pub reason: String,
}

pub struct GitignoreIntelligence {
    project_path: std::path::PathBuf,
}

impl GitignoreIntelligence {
    pub fn new<P: AsRef<Path>>(project_path: P) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
        }
    }
    
    /// Detect the primary project type based on key files present
    pub fn detect_project_type(&self) -> Result<ProjectType> {
        let mut rust_score = 0;
        let mut node_score = 0;
        let mut python_score = 0;
        let mut go_score = 0;
        let mut java_score = 0;
        
        // Check for key indicator files
        if self.project_path.join("Cargo.toml").exists() {
            rust_score += 10;
        }
        if self.project_path.join("Cargo.lock").exists() {
            rust_score += 5;
        }
        
        if self.project_path.join("package.json").exists() {
            node_score += 10;
        }
        if self.project_path.join("node_modules").exists() {
            node_score += 5;
        }
        
        if self.project_path.join("requirements.txt").exists() 
           || self.project_path.join("pyproject.toml").exists()
           || self.project_path.join("setup.py").exists() {
            python_score += 10;
        }
        
        if self.project_path.join("go.mod").exists() {
            go_score += 10;
        }
        
        if self.project_path.join("pom.xml").exists() 
           || self.project_path.join("build.gradle").exists() {
            java_score += 10;
        }
        
        // Determine primary type
        let scores = [rust_score, node_score, python_score, go_score, java_score];
        let max_score = *scores.iter().max().unwrap();
        
        if max_score == 0 {
            return Ok(ProjectType::Unknown);
        }
        
        let high_scores: Vec<_> = [
            (rust_score, ProjectType::Rust),
            (node_score, ProjectType::Node),
            (python_score, ProjectType::Python),
            (go_score, ProjectType::Go),
            (java_score, ProjectType::Java),
        ].into_iter().filter(|(score, _)| *score == max_score).collect();
        
        if high_scores.len() > 1 {
            Ok(ProjectType::Mixed)
        } else {
            Ok(high_scores[0].1.clone())
        }
    }
    
    /// Validate basic gitignore patterns for the detected project type
    pub fn validate_gitignore(&self) -> Result<Vec<GitignoreWarning>> {
        let mut warnings = Vec::new();
        let project_type = self.detect_project_type()?;
        
        let gitignore_path = self.project_path.join(".gitignore");
        if !gitignore_path.exists() {
            warnings.push(GitignoreWarning {
                message: "No .gitignore file found".to_string(),
                severity: WarningSeverity::High,
            });
            return Ok(warnings);
        }
        
        let gitignore_content = std::fs::read_to_string(&gitignore_path)?;
        
        match project_type {
            ProjectType::Rust => {
                if !gitignore_content.contains("target/") {
                    warnings.push(GitignoreWarning {
                        message: "Rust project should ignore 'target/' directory".to_string(),
                        severity: WarningSeverity::High,
                    });
                }
                if gitignore_content.contains("node_modules/") {
                    warnings.push(GitignoreWarning {
                        message: "Rust project has Node.js patterns in gitignore - consider reviewing".to_string(),
                        severity: WarningSeverity::Medium,
                    });
                }
            }
            ProjectType::Node => {
                if !gitignore_content.contains("node_modules/") {
                    warnings.push(GitignoreWarning {
                        message: "Node.js project should ignore 'node_modules/' directory".to_string(),
                        severity: WarningSeverity::High,
                    });
                }
                if gitignore_content.contains("target/") {
                    warnings.push(GitignoreWarning {
                        message: "Node.js project has Rust patterns in gitignore - consider reviewing".to_string(),
                        severity: WarningSeverity::Medium,
                    });
                }
            }
            ProjectType::Python => {
                if !gitignore_content.contains("__pycache__/") {
                    warnings.push(GitignoreWarning {
                        message: "Python project should ignore '__pycache__/' directory".to_string(),
                        severity: WarningSeverity::High,
                    });
                }
            }
            _ => {} // No validation for other/mixed project types
        }
        
        Ok(warnings)
    }
    
    /// Suggest basic gitignore improvements based on project type
    pub fn suggest_improvements(&self) -> Result<Vec<GitignoreSuggestion>> {
        let mut suggestions = Vec::new();
        let project_type = self.detect_project_type()?;
        let gitignore_content = self.read_gitignore_content()?;
        
        match project_type {
            ProjectType::Rust => {
                if !gitignore_content.contains("target/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "target/".to_string(),
                        reason: "Rust build artifacts".to_string(),
                    });
                }
            }
            ProjectType::Node => {
                if !gitignore_content.contains("node_modules/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "node_modules/".to_string(),
                        reason: "Node.js dependencies".to_string(),
                    });
                }
                if !gitignore_content.contains("dist/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "dist/".to_string(),
                        reason: "Build output directory".to_string(),
                    });
                }
            }
            ProjectType::Python => {
                if !gitignore_content.contains("__pycache__/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "__pycache__/".to_string(),
                        reason: "Python bytecode cache".to_string(),
                    });
                }
                if !gitignore_content.contains("*.pyc") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "*.pyc".to_string(),
                        reason: "Python bytecode files".to_string(),
                    });
                }
            }
            ProjectType::Go => {
                if !gitignore_content.contains("vendor/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "vendor/".to_string(),
                        reason: "Go vendor dependencies".to_string(),
                    });
                }
            }
            ProjectType::Java => {
                if !gitignore_content.contains("target/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "target/".to_string(),
                        reason: "Maven build artifacts".to_string(),
                    });
                }
                if !gitignore_content.contains("build/") {
                    suggestions.push(GitignoreSuggestion {
                        pattern: "build/".to_string(),
                        reason: "Gradle build artifacts".to_string(),
                    });
                }
            }
            _ => {} // No suggestions for mixed/unknown project types
        }
        
        // Always suggest basic patterns if missing
        if !gitignore_content.contains(".env") {
            suggestions.push(GitignoreSuggestion {
                pattern: ".env".to_string(),
                reason: "Environment variables and secrets".to_string(),
            });
        }
        
        if !gitignore_content.contains(".DS_Store") {
            suggestions.push(GitignoreSuggestion {
                pattern: ".DS_Store".to_string(),
                reason: "macOS system files".to_string(),
            });
        }
        
        Ok(suggestions)
    }
    
    fn read_gitignore_content(&self) -> Result<String> {
        let gitignore_path = self.project_path.join(".gitignore");
        if gitignore_path.exists() {
            Ok(std::fs::read_to_string(gitignore_path)?)
        } else {
            Ok(String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        let intelligence = GitignoreIntelligence::new(temp_dir.path());
        let project_type = intelligence.detect_project_type().unwrap();
        
        assert_eq!(project_type, ProjectType::Rust);
    }
    
    #[test]
    fn test_detect_node_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        
        let intelligence = GitignoreIntelligence::new(temp_dir.path());
        let project_type = intelligence.detect_project_type().unwrap();
        
        assert_eq!(project_type, ProjectType::Node);
    }
    
    #[test]
    fn test_suggest_basic_improvements() {
        let temp_dir = TempDir::new().unwrap();
        // Create empty gitignore
        fs::write(temp_dir.path().join(".gitignore"), "").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        let intelligence = GitignoreIntelligence::new(temp_dir.path());
        let suggestions = intelligence.suggest_improvements().unwrap();
        
        // Should suggest target/, .env, and .DS_Store
        assert!(suggestions.iter().any(|s| s.pattern.contains("target/")));
        assert!(suggestions.iter().any(|s| s.pattern.contains(".env")));
        assert!(suggestions.iter().any(|s| s.pattern.contains(".DS_Store")));
    }
}