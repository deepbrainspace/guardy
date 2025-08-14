//! Global scanner configuration
//!
//! This provides a static configuration that can be initialized once
//! (typically from CLI) and then accessed globally throughout the scan.

use crate::scan::config::ScannerConfig;
use std::sync::{Arc, LazyLock, RwLock};

/// Container for the global configuration
struct ConfigContainer {
    config: Option<Arc<ScannerConfig>>,
}

/// Global scanner configuration
static SCANNER_CONFIG: LazyLock<RwLock<ConfigContainer>> = LazyLock::new(|| {
    RwLock::new(ConfigContainer { config: None })
});

/// Initialize the global scanner configuration
/// 
/// This should be called once at program startup, typically from the CLI
/// after parsing command-line arguments and loading configuration files.
/// 
/// # Example
/// ```ignore
/// use guardy::scan::static_data::configuration;
/// use guardy::scan::ScannerConfig;
/// 
/// let config = ScannerConfig::default();
/// configuration::init_config(config);
/// ```
pub fn init_config(config: ScannerConfig) {
    // Store the configuration
    // Note: We no longer initialize rayon since we're using ExecutionStrategy
    // which manages its own worker threads via crossbeam channels
    let container = &SCANNER_CONFIG;
    if let Ok(mut guard) = container.write() {
        guard.config = Some(Arc::new(config));
        tracing::info!("Scanner configuration initialized");
    } else {
        tracing::error!("Failed to initialize scanner configuration - lock poisoned");
    }
}

/// Get the global scanner configuration
/// 
/// Returns the configuration if initialized, or a default configuration
/// if not yet initialized.
/// 
/// # Example
/// ```ignore
/// use guardy::scan::static_data::configuration;
/// 
/// let config = configuration::get_config();
/// println!("Max file size: {} MB", config.max_file_size_mb);
/// ```
pub fn get_config() -> Arc<ScannerConfig> {
    let container = &SCANNER_CONFIG;
    
    // Try to read existing config
    if let Ok(guard) = container.read()
        && let Some(ref config) = guard.config {
        return config.clone();
    }
    
    // If not initialized or lock failed, initialize with default
    tracing::warn!("Scanner configuration not initialized, using defaults");
    let default_config = Arc::new(ScannerConfig::default());
    
    // Try to set the default
    if let Ok(mut guard) = container.write() {
        guard.config = Some(default_config.clone());
    }
    
    default_config
}

/// Check if configuration has been initialized
pub fn is_initialized() -> bool {
    if let Ok(guard) = SCANNER_CONFIG.read() {
        guard.config.is_some()
    } else {
        false
    }
}

