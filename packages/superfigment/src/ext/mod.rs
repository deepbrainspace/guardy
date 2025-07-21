//! Extension traits for enhanced Figment functionality
//!
//! This module provides several extension traits that add powerful capabilities to regular Figment:
//!
//! ## Individual Extension Traits
//!
//! - **`ExtendExt`** - Array merging with `_add` and `_remove` patterns
//! - **`FluentExt`** - Fluent builder methods (`with_file`, `with_env`, etc.)
//!   - ⚠️ **Note**: Automatically includes `ExtendExt` functionality (array merging)
//! - **`AccessExt`** - Convenience methods (`as_json`, `get_string`, etc.)
//!
//! ## All-in-One Trait
//!
//! - **`AllExt`** - Includes all extension functionality in a single import
//!   - Uses Rust's "blanket implementation" to automatically provide all methods
//!   - When you import `AllExt`, you get methods from all three individual traits
//!
//! ## Usage Examples
//!
//! ### Selective Import
//! ```rust
//! use superfigment::ExtendExt;  // Just array merging
//! use superfigment::{FluentExt, AccessExt};  // Builder + convenience (includes array merging)
//! ```
//!
//! ### Everything at Once
//! ```rust
//! use superfigment::AllExt;  // All extension functionality via blanket implementation
//! ```

pub mod extend;
pub mod fluent; 
pub mod access;

// Individual extension traits
pub use extend::ExtendExt;
pub use fluent::FluentExt;
pub use access::AccessExt;


/// All-in-one extension trait that includes all SuperFigment functionality
///
/// This trait combines `ExtendExt`, `FluentExt`, and `AccessExt` into a single
/// import for maximum convenience. Use this when you want all SuperFigment
/// enhancements available on regular Figment instances.
///
/// ## How It Works (Blanket Implementation)
///
/// ```rust
/// // AllExt inherits from all three traits
/// pub trait AllExt: ExtendExt + FluentExt + AccessExt {}
///
/// // Blanket implementation: any type that implements all three traits
/// // automatically gets AllExt for free
/// impl<T> AllExt for T where T: ExtendExt + FluentExt + AccessExt {}
/// ```
///
/// Since `Figment` implements all three individual traits, it automatically
/// gets `AllExt` as well. This means importing `AllExt` gives you all methods
/// from all three traits with a single `use` statement.
///
/// ## Examples
///
/// ```rust
/// use figment::Figment;
/// use superfigment::AllExt;
///
/// let config = Figment::new()
///     // FluentExt methods (includes ExtendExt array merging)
///     .with_file("config")           
///     .with_env("APP_")              
///     .with_hierarchical_config("myapp");
///
/// // AccessExt methods
/// let json = config.as_json()?;
/// let host = config.get_string("database.host")?;
///
/// // ExtendExt methods (also used automatically by FluentExt)
/// let custom_config = Figment::new()
///     .merge_extend(some_provider);
/// 
/// # Ok::<(), figment::Error>(())
/// ```
pub trait AllExt: ExtendExt + FluentExt + AccessExt {
    // Empty trait body - inherits all methods from the three parent traits
    // The "magic" happens via the blanket implementation below
}

/// Blanket implementation: automatically implement AllExt for any type 
/// that implements all three required traits
///
/// This means Figment gets AllExt "for free" since it implements 
/// ExtendExt, FluentExt, and AccessExt individually.
impl<T> AllExt for T where T: ExtendExt + FluentExt + AccessExt {}