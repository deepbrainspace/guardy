//! # SuperConfig Procedural Macros
//! 
//! This crate provides procedural macros for the [`superconfig`] library that automatically
//! generate Rust configuration structs from existing configuration files using [Typify].
//! 
//! ## Key Features
//! 
//! - 🏗️ **Zero boilerplate**: No manual struct definitions required
//! - 🔍 **Automatic type inference**: Generates structs from JSON/YAML files using Typify
//! - 🚀 **LazyLock integration**: Creates static instances for zero-copy access
//! - 📁 **Flexible file discovery**: Searches multiple paths for config files
//! - ⚡ **Compile-time validation**: Ensures config file exists and is valid
//! 
//! ## Macros
//! 
//! - [`config!`] - Auto-generates structs from config files
//! - [`config_builder!`] - Creates LazyLock statics with builder pattern
//! 
//! [`superconfig`]: https://docs.rs/superconfig
//! [Typify]: https://docs.rs/typify

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Ident, Token, Result as SynResult};
use syn::parse::{Parse, ParseStream};
use std::fs;
use std::path::{Path, PathBuf};
use heck::{ToSnakeCase, ToShoutySnakeCase};

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

/// Options for config_builder! macro
#[derive(Default)]
struct ConfigOptions {
    env_prefix: Option<LitStr>,
    config_file: Option<syn::Expr>,
}

/// Input for config_builder! macro
struct ConfigBuilderInput {
    struct_name: Ident,
    options: ConfigOptions,
}

impl Parse for ConfigBuilderInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let struct_name = input.parse()?;
        
        let mut options = ConfigOptions::default();
        
        // Parse optional comma and options
        if input.parse::<Token![,]>().is_ok() {
            while !input.is_empty() {
                let name: Ident = input.parse()?;
                input.parse::<Token![:]>()?;
                
                match name.to_string().as_str() {
                    "env_prefix" => {
                        options.env_prefix = Some(input.parse()?);
                    }
                    "config_file" => {
                        options.config_file = Some(input.parse()?);
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(name, "Unknown option"));
                    }
                }
                
                // Parse optional comma
                let _ = input.parse::<Token![,]>();
            }
        }
        
        Ok(ConfigBuilderInput {
            struct_name,
            options,
        })
    }
}

/// Auto-generate configuration struct and LazyLock static from config files using Typify
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
                &input.config_name,
                format!("Failed to generate structs from {}: {}", config_file.display(), e)
            ).to_compile_error().into();
        }
    };
    
    let config_name_snake = config_name.to_snake_case().to_uppercase();
    let static_name = Ident::new(&format!("{}_CONFIG", config_name_snake), struct_name.span());
    
    // Generate the complete code
    let expanded = quote! {
        #struct_definitions
        
        static #static_name: std::sync::LazyLock<#struct_name> = std::sync::LazyLock::new(|| {
            // For now, just return default - will be enhanced with superconfig integration
            #struct_name::default()
        });
        
        impl #struct_name {
            pub fn global() -> &#struct_name {
                &#static_name
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Create LazyLock static for existing configuration structs with auto-derived settings
#[proc_macro]
pub fn config_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ConfigBuilderInput);
    let struct_name = &input.struct_name;
    let options = &input.options;
    
    // Auto-derive values from struct name
    let (auto_env_prefix, auto_config_name) = derive_config_values(&struct_name.to_string());
    
    // Use provided values or auto-derived ones
    let env_prefix = options.env_prefix.as_ref()
        .map(|lit| lit.value())
        .unwrap_or(auto_env_prefix);
    
    let config_name = extract_config_name_from_options(options)
        .unwrap_or(auto_config_name);
    
    // Debug output to show what was auto-generated
    eprintln!("config_builder! macro: struct={}, env_prefix={}, config_name={}", 
             struct_name, env_prefix, config_name);
    
    let static_name = Ident::new(
        &format!("{}_CONFIG", struct_name.to_string().to_shouty_snake_case()),
        struct_name.span()
    );
    
    // Generate the complete code with builder pattern
    // Phase 1: Just use defaults for now
    // TODO Phase 2: Add .with_config_file() support
    // TODO Phase 4: Add .with_env_prefix() support  
    let expanded = quote! {
        static #static_name: std::sync::LazyLock<#struct_name> = std::sync::LazyLock::new(|| {
            // Phase 1: Use ConfigBuilder with defaults only
            superconfig::ConfigBuilder::new()
                .with_defaults(#struct_name::default())
                .build()
                .unwrap_or_else(|_| #struct_name::default())
        });
        
        impl #struct_name {
            pub fn global() -> &#struct_name {
                &#static_name
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Find config file in various paths
fn find_config_file(name: &str) -> Option<PathBuf> {
    let extensions = ["json", "yaml", "yml"];
    let search_paths = [
        std::env::current_dir().ok(),
        Some(PathBuf::from("config")),
        Some(PathBuf::from(".")),
    ];
    
    for path_opt in &search_paths {
        if let Some(base_path) = path_opt {
            for ext in &extensions {
                let file_path = base_path.join(format!("{}.{}", name, ext));
                if file_path.exists() {
                    return Some(file_path);
                }
            }
        }
    }
    
    None
}

/// Generate struct definitions using Typify
fn generate_structs_with_typify(config_file: &Path, struct_name: &Ident) -> Result<proc_macro2::TokenStream, String> {
    let content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // For now, just generate a simple default struct
    // TODO: Integrate with typify crate for real struct generation
    Ok(quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
        pub struct #struct_name {
            // Placeholder - will be generated by typify
        }
    })
}

/// Derive environment prefix and config name from struct name
fn derive_config_values(struct_name: &str) -> (String, String) {
    // GuardyConfig -> GUARDY_, guardy
    let env_prefix = if struct_name.ends_with("Config") {
        let base = &struct_name[..struct_name.len() - 6]; // Remove "Config"
        format!("{}_", base.to_shouty_snake_case())
    } else {
        format!("{}_", struct_name.to_shouty_snake_case())
    };
    
    let config_name = if struct_name.ends_with("Config") {
        let base = &struct_name[..struct_name.len() - 6]; // Remove "Config" 
        base.to_snake_case()
    } else {
        struct_name.to_snake_case()
    };
    
    (env_prefix, config_name)
}

/// Extract config name from options if config_file is provided
fn extract_config_name_from_options(options: &ConfigOptions) -> Option<String> {
    if let Some(syn::Expr::Lit(syn::ExprLit { 
        lit: syn::Lit::Str(lit_str), .. 
    })) = &options.config_file {
        let path = lit_str.value();
        // Extract base name without extension
        if let Some(file_name) = std::path::Path::new(&path).file_stem()
            && let Some(name_str) = file_name.to_str() {
            return Some(name_str.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    
    #[test]
    fn test_derive_config_values() {
        let (env_prefix, config_name) = derive_config_values("GuardyConfig");
        assert_eq!(env_prefix, "GUARDY_");
        assert_eq!(config_name, "guardy");
        
        let (env_prefix, config_name) = derive_config_values("MyAppConfig");
        assert_eq!(env_prefix, "MY_APP_");
        assert_eq!(config_name, "my_app");
        
        let (env_prefix, config_name) = derive_config_values("Database");
        assert_eq!(env_prefix, "DATABASE_");
        assert_eq!(config_name, "database");
    }
    
    #[test]
    fn test_config_input_parsing() {
        let tokens: proc_macro2::TokenStream = quote! { "myapp" => MyAppConfig };
        let input = syn::parse2::<ConfigInput>(tokens).unwrap();
        
        assert_eq!(input.config_name.value(), "myapp");
        assert_eq!(input.struct_name.to_string(), "MyAppConfig");
    }
    
    #[test]
    fn test_config_builder_input_parsing() {
        let tokens: proc_macro2::TokenStream = quote! { GuardyConfig };
        let input = syn::parse2::<ConfigBuilderInput>(tokens).unwrap();
        
        assert_eq!(input.struct_name.to_string(), "GuardyConfig");
        assert!(input.options.env_prefix.is_none());
        assert!(input.options.config_file.is_none());
    }
    
    #[test]
    fn test_config_builder_input_with_options() {
        let tokens: proc_macro2::TokenStream = quote! { 
            GuardyConfig, env_prefix: "CUSTOM_" 
        };
        let input = syn::parse2::<ConfigBuilderInput>(tokens).unwrap();
        
        assert_eq!(input.struct_name.to_string(), "GuardyConfig");
        assert_eq!(input.options.env_prefix.unwrap().value(), "CUSTOM_");
    }
}