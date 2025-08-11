//! System Profile Module
//! 
//! Provides cached system information and resource profiles for the entire application.
//! All values are computed once on first access and cached for the program lifetime.

use lazy_static::lazy_static;
use std::sync::Arc;

lazy_static! {
    /// System profile information cached for the entire program lifetime
    pub static ref SYSTEM: Arc<SystemProfile> = Arc::new(SystemProfile::new());
}

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
    
    /// Recommended worker count for I/O-bound tasks
    pub recommended_io_workers: usize,
    
    /// Recommended worker count for CPU-bound tasks
    pub recommended_cpu_workers: usize,
}

impl SystemProfile {
    /// Create a new system profile (called once via lazy_static)
    fn new() -> Self {
        let cpu_count = num_cpus::get();
        let physical_cpu_count = num_cpus::get_physical();
        
        // Get memory information using sysinfo crate
        let mut sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new().with_memory()
        );
        sys.refresh_memory();
        
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();
        
        // Calculate recommended worker counts
        // I/O-bound: can use more threads than cores (2x is common)
        let recommended_io_workers = cpu_count * 2;
        
        // CPU-bound: typically matches physical cores
        let recommended_cpu_workers = physical_cpu_count;
        
        Self {
            cpu_count,
            physical_cpu_count,
            total_memory,
            available_memory,
            recommended_io_workers,
            recommended_cpu_workers,
        }
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
    
    /// Check if system has sufficient resources for parallel processing
    pub fn should_use_parallel(&self, min_memory_mb: u64) -> bool {
        self.cpu_count > 1 && self.available_memory > (min_memory_mb * 1024 * 1024)
    }
    
    /// Get a human-readable summary of system resources
    pub fn summary(&self) -> String {
        format!(
            "System Profile: {} CPUs ({} physical), {:.2} GB RAM ({:.2} GB available)",
            self.cpu_count,
            self.physical_cpu_count,
            self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            self.available_memory as f64 / (1024.0 * 1024.0 * 1024.0)
        )
    }
}

/// Quick access functions for common operations
impl SystemProfile {
    /// Get the global system profile instance
    pub fn get() -> Arc<SystemProfile> {
        SYSTEM.clone()
    }
    
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
    fn test_static_access() {
        // Ensure multiple accesses return the same instance
        let profile1 = SystemProfile::get();
        let profile2 = SystemProfile::get();
        assert_eq!(profile1.cpu_count, profile2.cpu_count);
        
        // Test direct access methods
        assert_eq!(SystemProfile::cpu_count(), profile1.cpu_count);
        assert_eq!(SystemProfile::physical_cpu_count(), profile1.physical_cpu_count);
    }
}