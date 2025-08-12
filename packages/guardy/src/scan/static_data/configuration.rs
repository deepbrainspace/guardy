//! Global scanner configuration
//!
//! This provides a static configuration that can be initialized once
//! (typically from CLI) and then accessed globally throughout the scan.

use crate::scan::config::ScannerConfig;
use std::sync::{Arc, LazyLock, RwLock};
use system_profile::SYSTEM;

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
/// ```no_run
/// use guardy::scan::static_data::configuration;
/// use guardy::scan::ScannerConfig;
/// 
/// let config = ScannerConfig::default();
/// configuration::init_config(config);
/// ```
pub fn init_config(config: ScannerConfig) {
    // Configure rayon thread pool based on config (this happens once)
    let thread_count = if let Some(override_threads) = config.max_threads {
        // Use explicit override if provided
        override_threads
    } else {
        // Calculate based on percentage of system CPUs
        let cpu_count = SYSTEM.cpu_count;
        let calculated = (cpu_count * config.max_cpu_percentage as usize) / 100;
        std::cmp::max(1, calculated) // Ensure at least 1 thread
    };
    
    // Set global rayon thread pool once at initialization
    // This affects all par_iter operations in the application
    if let Err(e) = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global() {
        // Only fails if already initialized, which is fine
        tracing::debug!("Rayon thread pool already initialized: {}", e);
    } else {
        tracing::info!(
            "Rayon thread pool initialized with {} threads ({}% of {} CPUs)",
            thread_count,
            config.max_cpu_percentage,
            SYSTEM.cpu_count
        );
    }
    
    // Store the configuration
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
/// ```no_run
/// use guardy::scan::static_data::configuration;
/// 
/// let config = configuration::get_config();
/// println!("Max file size: {} MB", config.max_file_size_mb);
/// ```
pub fn get_config() -> Arc<ScannerConfig> {
    let container = &SCANNER_CONFIG;
    
    // Try to read existing config
    if let Ok(guard) = container.read() {
        if let Some(ref config) = guard.config {
            return config.clone();
        }
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

/// Reset configuration (mainly for testing)
#[cfg(test)]
pub fn reset_config() {
    if let Ok(mut guard) = SCANNER_CONFIG.write() {
        guard.config = None;
    }
}