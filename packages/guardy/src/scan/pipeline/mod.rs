//! Pipeline modules for directory and file processing

pub mod directory;
mod file;

pub use directory::DirectoryPipeline;
pub use file::FilePipeline;