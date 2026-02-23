//! Historic Preservation Districts (ZONE-008).
//!
//! Allows the player to designate districts as historic preservation zones.
//! Buildings in historic districts:
//! - Cannot be demolished, upgraded, or replaced
//! - Are frozen at their current building level
//! - Generate a +10% land value bonus for cells in the district
//! - Generate tourism visits (historic districts attract tourists)
//!
//! Removing historic preservation from a district triggers a happiness
//! penalty from preservationist citizens.

mod systems;
mod types;

#[cfg(test)]
mod tests;

pub use systems::*;
pub use types::*;
