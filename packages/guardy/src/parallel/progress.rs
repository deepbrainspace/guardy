use std::io::Write;

/// Progress reporting strategies for parallel execution
pub trait ProgressReporter: Send + Sync {
    fn report(&self, current: usize, total: usize, worker_id: usize);
    fn clear(&self);
}

/// Console progress reporter with configurable display
#[derive(Clone)]
pub struct ConsoleProgressReporter {
    show_worker_id: bool,
    update_frequency: usize,
    progress_icon: &'static str,
    item_name: String,
}

impl ConsoleProgressReporter {
    pub fn new(item_name: &str) -> Self {
        Self {
            show_worker_id: false,
            update_frequency: 5,
            progress_icon: "⏳",
            item_name: item_name.to_string(),
        }
    }

    pub fn with_worker_id(mut self) -> Self {
        self.show_worker_id = true;
        self.progress_icon = "⚡";
        self
    }

    pub fn with_frequency(mut self, frequency: usize) -> Self {
        self.update_frequency = frequency;
        self
    }

    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.progress_icon = icon;
        self
    }
}

impl ProgressReporter for ConsoleProgressReporter {
    fn report(&self, current: usize, total: usize, worker_id: usize) {
        // Only report at specified frequency to reduce console spam
        if current % self.update_frequency == 0 || current == total {
            let percentage = current as f64 / total as f64 * 100.0;
            
            if self.show_worker_id {
                print!("\r{} Progress: {}/{} {} ({:.1}%) [worker-{}]", 
                       self.progress_icon, current, total, self.item_name, percentage, worker_id);
            } else {
                print!("\r{} Progress: {}/{} {} ({:.1}%)", 
                       self.progress_icon, current, total, self.item_name, percentage);
            }
            
            std::io::stdout().flush().ok();
        }
    }

    fn clear(&self) {
        print!("\r");
        std::io::stdout().flush().ok();
    }
}

/// No-op progress reporter for quiet operations
pub struct NoOpProgressReporter;

impl ProgressReporter for NoOpProgressReporter {
    fn report(&self, _current: usize, _total: usize, _worker_id: usize) {
        // Do nothing
    }

    fn clear(&self) {
        // Do nothing
    }
}


/// Factory functions for common progress reporters
pub mod factories {
    use super::*;

    /// Create a parallel progress reporter for any type of items
    pub fn parallel_reporter(item_name: &str) -> ConsoleProgressReporter {
        ConsoleProgressReporter::new(item_name)
            .with_worker_id()
            .with_frequency(5)
    }

    /// Create a sequential progress reporter for any type of items
    pub fn sequential_reporter(item_name: &str) -> ConsoleProgressReporter {
        ConsoleProgressReporter::new(item_name)
            .with_frequency(5)
    }

    /// Create a no-op progress reporter for quiet operations
    pub fn quiet_reporter() -> NoOpProgressReporter {
        NoOpProgressReporter
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_progress_reporter() {
        let reporter = ConsoleProgressReporter::new("items");
        
        // This test mainly ensures the reporter doesn't panic
        reporter.report(5, 10, 0);
        reporter.clear();
    }

    #[test]
    fn test_noop_progress_reporter() {
        let reporter = NoOpProgressReporter;
        
        // Should do nothing without panicking
        reporter.report(5, 10, 0);
        reporter.clear();
    }

    #[test]
    fn test_factory_functions() {
        let _parallel = factories::parallel_reporter("tasks");
        let _sequential = factories::sequential_reporter("items");
    }
}