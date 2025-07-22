//! Extension traits for enhanced Figment functionality
//!
//! This module provides several extension traits that add powerful capabilities to regular Figment:
//!
//! ## Extension Traits
//!
//! - **`ExtendExt`** - Array merging with `_add` and `_remove` patterns
//! - **`FluentExt`** - Fluent builder methods (`with_file`, `with_env`, etc.)
//!   - ⚠️ **Note**: Automatically includes `ExtendExt` functionality (array merging)
//! - **`AccessExt`** - Convenience methods (`as_json`, `get_string`, etc.)
//!
//! ## Usage Examples
//!
//! ### With Regular Figment (Extension Traits)
//! ```rust
//! use figment::Figment;
//! use superfigment::prelude::*;  // Import all extension traits
//! 
//! let config = Figment::new()
//!     .with_file("config")    // FluentExt method
//!     .with_env("APP_");      // FluentExt method  
//! ```
//!
//! ### With SuperFigment (Built-in Methods)
//! ```rust
//! use superfigment::SuperFigment;  // No prelude needed
//! 
//! let config = SuperFigment::new()
//!     .with_file("config")    // Built-in method
//!     .with_env("APP_");      // Built-in method
//! ```
//!
//! ### Selective Import (Advanced)
//! ```rust
//! use superfigment::ExtendExt;  // Just array merging
//! use superfigment::{FluentExt, AccessExt};  // Builder + convenience
//! ```

pub mod extend;
pub mod fluent; 
pub mod access;

// Individual extension traits
pub use extend::ExtendExt;
pub use fluent::FluentExt;
pub use access::AccessExt;



/// Prelude module for convenient imports of all SuperFigment functionality
/// 
/// Import this module with `use superfigment::prelude::*` to get everything:
/// - Extension traits: `ExtendExt`, `FluentExt`, `AccessExt`
/// - Enhanced providers: `Universal`, `Nested`, `Empty`, `Hierarchical`
/// 
/// ## Example
/// ```rust,no_run
/// use figment::Figment;
/// use superfigment::prelude::*;  // Everything you need!
/// 
/// let config = Figment::new()
///     .merge(Universal::file("config"))  // Enhanced provider
///     .with_env("APP_")                  // Extension trait method
///     .merge_extend(Nested::prefixed("DB_")); // Extension trait method
/// let json = config.as_json()?;          // Extension trait method
/// # Ok::<(), figment::Error>(())
/// ```
pub mod prelude {
    // Extension traits - add methods to regular Figment
    pub use super::extend::ExtendExt;
    pub use super::fluent::FluentExt;
    pub use super::access::AccessExt;
    
    // Enhanced providers - drop-in replacements with superpowers
    pub use crate::providers::{Universal, Nested, Empty, Hierarchical};
}