//! Procedural macros for fast-config using Typify
//! 
//! This crate provides the `config!` macro that auto-generates Rust structs
//! from configuration files using Typify and creates LazyLock static instances.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Ident, Token, Result as SynResult};
use syn::parse::{Parse, ParseStream};
use std::fs;
use std::path::{Path, PathBuf};

/// Input for the config! macro: "config_name" => StructName
struct ConfigInput {
    config_name: LitStr,
    _arrow: Token![=>],
    struct_name: Ident,
}

impl Parse for ConfigInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(ConfigInput {
            config_name: input.parse()?,
            _arrow: input.parse()?,
            struct_name: input.parse()?,
        })
    }
}

/// Auto-generate configuration struct and LazyLock static from config files using Typify
/// 
/// # Syntax
/// ```rust
/// use fast_config::config;
/// 
/// config!("myapp" => MyAppConfig);
/// ```
/// 
/// # Generated Code
/// - Complete struct hierarchy using Typify
/// - LazyLock static instance for zero-copy access
/// - `::global()` method for convenient access
#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ConfigInput);
    let config_name = input.config_name.value();
    let struct_name = &input.struct_name;
    
    // Find config file
    let config_file = match find_config_file(&config_name) {
        Some(path) => path,
        None => {
            return syn::Error::new_spanned(
                &input.config_name,
                format!("No config file found for '{config_name}'. Expected: {config_name}.json, {config_name}.yaml, or {config_name}.yml")
            ).to_compile_error().into();
        }
    };
    
    // Generate struct definitions using Typify
    let struct_definitions = match generate_structs_with_typify(&config_file, struct_name) {
        Ok(definitions) => definitions,
        Err(e) => {
            return syn::Error::new_spanned(
                &input.struct_name,
                format!("Failed to generate structs from config file {}: {}", config_file.display(), e)
            ).to_compile_error().into();
        }
    };
    
    // Generate LazyLock static instance
    let static_name = to_screaming_snake_case(&struct_name.to_string());
    let static_ident = syn::Ident::new(&static_name, struct_name.span());
    
    let expanded = quote! {
        #struct_definitions
        
        /// Auto-generated LazyLock static instance for zero-copy configuration access
        pub static #static_ident: ::std::sync::LazyLock<#struct_name> = ::std::sync::LazyLock::new(|| {
            ::fast_config::FastConfig::<#struct_name>::load(#config_name)
                .map(|config| config.clone_config())
                .unwrap_or_else(|e| {
                    ::tracing::warn!("Failed to load config {}: {}, using default", #config_name, e);
                    #struct_name::default()
                })
        });
        
        impl #struct_name {
            /// Get reference to the global configuration instance
            pub fn global() -> &'static #struct_name {
                &#static_ident
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Generate struct definitions using Typify from config file
fn generate_structs_with_typify(
    config_path: &Path,
    struct_name: &Ident,
) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(config_path)?;
    
    // Convert config file to JSON for Typify
    let config_json: serde_json::Value = match config_path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&content)?,
        Some("yaml") | Some("yml") => {
            let yaml_value: serde_yaml_bw::Value = serde_yaml_bw::from_str(&content)?;
            serde_json::to_value(yaml_value)?
        },
        _ => return Err("Unsupported file extension".into()),
    };
    
    // Generate JSON Schema with proper title for naming
    let schema = generate_json_schema_from_config(&config_json, &struct_name.to_string());
    
    // Convert to RootSchema that Typify expects
    let root_schema: schemars::schema::RootSchema = serde_json::from_value(schema)
        .map_err(|e| format!("Failed to convert to RootSchema: {e}"))?;
    
    // Use Typify to generate Rust structs with settings to avoid conflicts
    let settings = typify::TypeSpaceSettings::default();
    let mut typespace = typify::TypeSpace::new(&settings);
    typespace.add_root_schema(root_schema)?;
    
    // Get the generated code as TokenStream
    let generated_tokens = typespace.to_stream();
    
    // Parse the generated tokens to modify them
    let generated_string = generated_tokens.to_string();
    
    // Fix error module conflicts and add Default derives
    let fixed_code = fix_typify_output(&generated_string, struct_name);
    
    // Parse back to TokenStream
    let tokens: proc_macro2::TokenStream = fixed_code.parse()
        .map_err(|e| format!("Failed to parse generated code: {e}"))?;
    
    Ok(tokens)
}

/// Generate JSON Schema from configuration data
fn generate_json_schema_from_config(config: &serde_json::Value, struct_name: &str) -> serde_json::Value {
    match config {
        serde_json::Value::Object(map) => {
            let mut properties_obj = serde_json::Map::new();
            let mut required_fields = Vec::new();
            
            for (key, value) in map {
                properties_obj.insert(key.clone(), infer_schema_from_value(value));
                // Mark all fields as required (since they exist in the config)
                required_fields.push(key.clone());
            }
            
            serde_json::json!({
                "type": "object",
                "properties": properties_obj,
                "required": required_fields,  // Make fields required to avoid Option<T>
                "additionalProperties": false,
                "$schema": "http://json-schema.org/draft-07/schema#",
                "title": struct_name
            })
        },
        _ => {
            // For non-object root values, wrap in an object
            serde_json::json!({
                "type": "object",
                "properties": {
                    "value": infer_schema_from_value(config)
                },
                "required": ["value"],
                "additionalProperties": false,
                "$schema": "http://json-schema.org/draft-07/schema#",
                "title": struct_name
            })
        }
    }
}

/// Infer JSON Schema from a value
fn infer_schema_from_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(_) => serde_json::json!({
            "type": "string"
        }),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                serde_json::json!({ "type": "integer" })
            } else {
                serde_json::json!({ "type": "number" })
            }
        },
        serde_json::Value::Bool(_) => serde_json::json!({
            "type": "boolean"
        }),
        serde_json::Value::Array(arr) => {
            let item_schema = if let Some(first_item) = arr.first() {
                infer_schema_from_value(first_item)
            } else {
                serde_json::json!({ "type": "string" })
            };
            
            serde_json::json!({
                "type": "array",
                "items": item_schema
            })
        },
        serde_json::Value::Object(map) => {
            let mut properties_obj = serde_json::Map::new();
            let mut required_fields = Vec::new();
            
            for (key, nested_value) in map {
                properties_obj.insert(key.clone(), infer_schema_from_value(nested_value));
                required_fields.push(key.clone());
            }
            
            serde_json::json!({
                "type": "object",
                "properties": properties_obj,
                "required": required_fields,
                "additionalProperties": false
            })
        },
        serde_json::Value::Null => serde_json::json!({
            "type": "null"
        })
    }
}

/// Fix Typify output to avoid conflicts and use proper struct names
fn fix_typify_output(generated_code: &str, struct_name: &Ident) -> String {
    let mut result = generated_code.to_string();
    let struct_name_str = struct_name.to_string();
    
    // 1. Struct name should already be correct from the title field in JSON Schema
    // Typify generates based on the "title" field, which we set to struct_name
    
    // 2. Fix error module conflicts by making error module unique per struct
    let error_module_name = format!("{}_error", struct_name_str.to_lowercase());
    
    // The actual format in TokenStream has newline after "pub mod error"
    result = result.replace("pub mod error\n{", &format!("pub mod {error_module_name}\n{{"));
    result = result.replace("pub mod error\r\n{", &format!("pub mod {error_module_name}\r\n{{"));
    result = result.replace("pub mod error {", &format!("pub mod {error_module_name} {{"));
    // Also handle the case where there's no space/newline
    result = result.replace("pub mod error{", &format!("pub mod {error_module_name}{{"));
    result = result.replace("error::", &format!("{error_module_name}::"));
    
    // 3. Add Default derive to all structs
    // The actual format in the proc macro pipeline is different from the debug output
    let derive_pattern = "#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]";
    let enhanced_derive = "#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, Default)]";
    
    // Replace the derive pattern to add Default
    if result.contains(derive_pattern) {
        result = result.replace(derive_pattern, enhanced_derive);
    }
    
    result
}

/// Find config file for the given name in standard locations
fn find_config_file(config_name: &str) -> Option<PathBuf> {
    let search_paths = vec![
        // Current directory (highest priority)
        format!("{}.json", config_name),
        format!("{}.yaml", config_name),
        format!("{}.yml", config_name),
        
        // Tests directory (for integration tests)
        format!("tests/{}.json", config_name),
        format!("tests/{}.yaml", config_name),
        format!("tests/{}.yml", config_name),
        
        // Tests configs subdirectory (organized configs)
        format!("tests/configs/{}.json", config_name),
        format!("tests/configs/{}.yaml", config_name),
        format!("tests/configs/{}.yml", config_name),
        
        // Package-relative tests directory (when running from workspace root)
        format!("packages/fast-config/tests/{}.json", config_name),
        format!("packages/fast-config/tests/{}.yaml", config_name),
        format!("packages/fast-config/tests/{}.yml", config_name),
        
        // Package-relative tests configs subdirectory
        format!("packages/fast-config/tests/configs/{}.json", config_name),
        format!("packages/fast-config/tests/configs/{}.yaml", config_name),
        format!("packages/fast-config/tests/configs/{}.yml", config_name),
        
        // .config subdirectory
        format!(".config/{}/config.json", config_name),
        format!(".config/{}/config.yaml", config_name),
        format!(".config/{}/config.yml", config_name),
    ];
    
    for path_str in search_paths {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            return Some(path);
        }
    }
    
    // Try git repository root if available
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        && output.status.success() {
            let git_root_string = String::from_utf8_lossy(&output.stdout);
            let git_root = git_root_string.trim();
            let git_root_path = PathBuf::from(git_root);
            
            let git_paths = vec![
                git_root_path.join(format!("{config_name}.json")),
                git_root_path.join(format!("{config_name}.yaml")),
                git_root_path.join(format!("{config_name}.yml")),
            ];
            
            for path in git_paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }
    
    None
}

/// Convert string to SCREAMING_SNAKE_CASE
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() && chars.peek().is_some_and(|next| next.is_lowercase()) {
                result.push('_');
            }
            result.push(c);
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c.to_uppercase().next().unwrap());
        }
    }
    
    result
}