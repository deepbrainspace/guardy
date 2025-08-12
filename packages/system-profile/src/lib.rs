//! System Profile Crate
//! 
//! Provides cached system information and resource profiles for the entire application.
//! All values are computed once on first access and cached for the program lifetime.
//! 
//! Uses std::sync::LazyLock (Rust 1.80+) for lazy initialization.

use std::sync::{Arc, LazyLock};

/// Global system profile instance - computed once, cached forever
pub static SYSTEM: LazyLock<Arc<SystemProfile>> = LazyLock::new(|| {
    Arc::new(SystemProfile::detect())
});

/// System profile containing hardware and resource information
#[derive(Debug, Clone)]
pub struct SystemProfile {
    /// Total CPU cores (including hyperthreading)
    pub cpu_count: usize,
    
    /// Physical CPU cores (excluding hyperthreading)
    pub physical_cpu_count: usize,
    
    /// Total system memory in bytes
    pub total_memory: u64,
    
    /// Available system memory in bytes at startup
    pub available_memory: u64,
    
    /// Operating system name
    pub os_name: String,
    
    /// Operating system version
    pub os_version: String,
    
    /// System hostname
    pub hostname: String,
    
    /// Is this a Mac?
    pub is_macos: bool,
    
    /// Is this Windows?
    pub is_windows: bool,
    
    /// Is this Linux?
    pub is_linux: bool,
    
    /// Recommended worker count for I/O-bound tasks
    pub recommended_io_workers: usize,
    
    /// Recommended worker count for CPU-bound tasks
    pub recommended_cpu_workers: usize,
}

impl SystemProfile {
    /// Detect system profile (called once via LazyLock)
    fn detect() -> Self {
        use sysinfo::System;
        
        let cpu_count = num_cpus::get();
        let physical_cpu_count = num_cpus::get_physical();
        
        // Get system information
        let mut sys = System::new_with_specifics(
            sysinfo::RefreshKind::new()
                .with_memory(sysinfo::MemoryRefreshKind::everything())
        );
        sys.refresh_memory();
        
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();
        
        // Get OS information
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        
        // Detect OS type
        let is_macos = cfg!(target_os = "macos");
        let is_windows = cfg!(target_os = "windows");
        let is_linux = cfg!(target_os = "linux");
        
        // Calculate recommended worker counts
        // I/O-bound: can use more threads than cores (2x is common for file I/O)
        let recommended_io_workers = cpu_count * 2;
        
        // CPU-bound: typically matches physical cores
        let recommended_cpu_workers = physical_cpu_count;
        
        Self {
            cpu_count,
            physical_cpu_count,
            total_memory,
            available_memory,
            os_name,
            os_version,
            hostname,
            is_macos,
            is_windows,
            is_linux,
            recommended_io_workers,
            recommended_cpu_workers,
        }
    }
    
    /// Get the global system profile instance
    pub fn get() -> Arc<SystemProfile> {
        SYSTEM.clone()
    }
    
    /// Get optimal worker count based on percentage of available CPUs
    pub fn calculate_workers(&self, percentage: usize) -> usize {
        let percentage = percentage.min(100) as f32 / 100.0;
        ((self.cpu_count as f32 * percentage).ceil() as usize).max(1)
    }
    
    /// Get worker count with a maximum limit
    pub fn calculate_workers_with_limit(&self, percentage: usize, max_threads: usize) -> usize {
        if max_threads > 0 {
            self.calculate_workers(percentage).min(max_threads)
        } else {
            self.calculate_workers(percentage)
        }
    }
    
    /// Adapt worker count based on workload size (for file processing tasks)
    pub fn adapt_workers_for_workload(&self, item_count: usize, max_workers: usize) -> usize {
        match item_count {
            0..=10 => 1.min(max_workers),           // Minimal parallelism
            11..=50 => (max_workers / 2).max(1),    // Conservative parallelism  
            51..=100 => (max_workers * 3 / 4).max(1), // Moderate parallelism
            _ => max_workers,                        // Full parallelism
        }
    }
    
    /// Check if system has sufficient resources for parallel processing
    pub fn should_use_parallel(&self, min_memory_mb: u64) -> bool {
        self.cpu_count > 1 && self.available_memory > (min_memory_mb * 1024 * 1024)
    }
    
    /// Get a human-readable summary of system resources
    pub fn summary(&self) -> String {
        format!(
            "System: {} {} on {}\n\
             CPUs: {} ({} physical)\n\
             Memory: {:.2} GB ({:.2} GB available)\n\
             Host: {}",
            self.os_name,
            self.os_version,
            if self.is_macos { "macOS" } else if self.is_windows { "Windows" } else if self.is_linux { "Linux" } else { "Other" },
            self.cpu_count,
            self.physical_cpu_count,
            self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            self.available_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            self.hostname
        )
    }
    
    /// Get memory in GB
    pub fn total_memory_gb(&self) -> f64 {
        self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    /// Get available memory in GB
    pub fn available_memory_gb(&self) -> f64 {
        self.available_memory as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

/// Quick access functions
impl SystemProfile {
    /// Get CPU count directly
    pub fn cpu_count() -> usize {
        SYSTEM.cpu_count
    }
    
    /// Get physical CPU count directly  
    pub fn physical_cpu_count() -> usize {
        SYSTEM.physical_cpu_count
    }
    
    /// Check if running on a multi-core system
    pub fn is_multicore() -> bool {
        SYSTEM.cpu_count > 1
    }
    
    /// Check OS type quickly
    pub fn is_macos() -> bool {
        SYSTEM.is_macos
    }
    
    pub fn is_windows() -> bool {
        SYSTEM.is_windows
    }
    
    pub fn is_linux() -> bool {
        SYSTEM.is_linux
    }
}

#[cfg(feature = "gpu")]
pub mod gpu {
    use super::*;
    
    /// GPU information (if available)
    #[derive(Debug, Clone)]
    pub struct GpuInfo {
        pub name: String,
        pub memory_mb: u64,
        pub cuda_cores: Option<u32>,
    }
    
    /// Detect NVIDIA GPUs (requires 'gpu' feature)
    pub fn detect_nvidia_gpus() -> Vec<GpuInfo> {
        // TODO: Implement using nvml-wrapper
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_profile_initialization() {
        let profile = SystemProfile::get();
        assert!(profile.cpu_count > 0);
        assert!(profile.physical_cpu_count > 0);
        assert!(profile.total_memory > 0);
        
        // Test OS detection - at least one should be true
        assert!(profile.is_macos || profile.is_windows || profile.is_linux);
    }
    
    #[test]
    fn test_worker_calculation() {
        let profile = SystemProfile::get();
        
        // Test 50% workers
        let half_workers = profile.calculate_workers(50);
        assert!(half_workers >= 1);
        assert!(half_workers <= profile.cpu_count);
        
        // Test 100% workers
        let full_workers = profile.calculate_workers(100);
        assert_eq!(full_workers, profile.cpu_count);
        
        // Test with limit
        let limited = profile.calculate_workers_with_limit(100, 4);
        assert!(limited <= 4);
    }
    
    #[test]
    fn test_workload_adaptation() {
        let profile = SystemProfile::get();
        let max_workers = 8;
        
        // Small workload should use minimal workers
        assert_eq!(profile.adapt_workers_for_workload(5, max_workers), 1);
        
        // Medium workload should use conservative workers
        let medium = profile.adapt_workers_for_workload(30, max_workers);
        assert!(medium <= max_workers / 2);
        
        // Large workload should use all workers
        assert_eq!(profile.adapt_workers_for_workload(200, max_workers), max_workers);
    }
    
    #[test]
    fn test_static_access() {
        // Ensure multiple accesses return the same instance
        let profile1 = SystemProfile::get();
        let profile2 = SystemProfile::get();
        assert_eq!(profile1.cpu_count, profile2.cpu_count);
        
        // Test direct access methods
        assert_eq!(SystemProfile::cpu_count(), profile1.cpu_count);
        assert_eq!(SystemProfile::physical_cpu_count(), profile1.physical_cpu_count);
    }
    
    #[test]
    fn test_summary() {
        let profile = SystemProfile::get();
        let summary = profile.summary();
        
        // Summary should contain key information
        assert!(summary.contains("CPUs:"));
        assert!(summary.contains("Memory:"));
        assert!(summary.contains("System:"));
    }
}