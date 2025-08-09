use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookConfig {
    #[serde(flatten)]
    pub hooks: HashMap<String, HookDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookDefinition {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    #[serde(default)]
    pub builtin: Vec<String>,
    
    #[serde(default)]
    pub custom: Vec<CustomCommand>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomCommand {
    pub command: String,
    
    #[serde(default)]
    pub description: String,
    
    #[serde(default = "default_fail_on_error")]
    pub fail_on_error: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_fail_on_error() -> bool {
    true
}

impl Default for HookConfig {
    fn default() -> Self {
        let mut hooks = HashMap::new();
        
        // Default pre-commit with secret scanning
        hooks.insert("pre-commit".to_string(), HookDefinition {
            enabled: true,
            builtin: vec!["scan_secrets".to_string()],
            custom: vec![],
        });
        
        Self { hooks }
    }
}