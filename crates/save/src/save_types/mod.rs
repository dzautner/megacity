// ---------------------------------------------------------------------------
// Save structs and version constants (split into submodules)
// ---------------------------------------------------------------------------

mod core_types;
mod infrastructure_types;
mod policy_types;
mod save_data;
mod version;

// Re-export everything so callers see the same flat namespace.
pub use core_types::*;
pub use infrastructure_types::*;
pub use policy_types::*;
pub use save_data::*;
pub use version::*;
