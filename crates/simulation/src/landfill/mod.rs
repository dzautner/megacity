//! Landfill Capacity and Environmental Effects (WASTE-005).
//!
//! Models individual landfill sites with finite capacity, environmental effects,
//! and post-closure requirements. Each landfill tracks its fill level, liner type,
//! gas collection status, and closure state.
//!
//! Key features:
//! - **Finite capacity**: Each landfill has `total_capacity_tons` and tracks
//!   `current_fill_tons`. `years_remaining()` estimates lifespan based on
//!   current daily input.
//! - **Environmental effects by liner type**:
//!   - Unlined: high groundwater pollution (0.8), large odor radius (15 cells),
//!     land value penalty -40%.
//!   - Lined: low groundwater pollution (0.2), moderate odor radius (10 cells),
//!     land value penalty -25%.
//!   - LinedWithCollection: minimal groundwater pollution (0.05), small odor
//!     radius (5 cells), land value penalty -15%.
//! - **Landfill gas**: ~1 MW per 1,000 tons/day if gas collection is enabled.
//! - **Post-closure**: When full, landfill must be capped. After capping, a
//!   30-year monitoring period begins. After 30+ years, the site can become a park.

pub mod constants;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests_state;
#[cfg(test)]
mod tests_types;

// Re-export all public items so callers don't need to change their imports.
pub use constants::*;
pub use state::*;
pub use types::*;
