//! Restore functions: reconstruct simulation resources from save structs.
//!
//! Split into focused sub-modules by domain.

mod economy;
mod infrastructure;
mod waste;
mod water;
mod weather;

pub use economy::*;
pub use infrastructure::*;
pub use waste::*;
pub use water::*;
pub use weather::*;
