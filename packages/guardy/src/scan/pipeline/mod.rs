//! Pipeline modules for directory and file processing

mod directory;
mod file;

pub use directory::DirectoryPipeline;
pub use file::FilePipeline;