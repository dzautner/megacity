//! Flood levee and seawall infrastructure (WATER-009).
//!
//! Implements flood protection infrastructure: levees, seawalls, and floodgates.
//!
//! **Levee**: Placeable along rivers, prevents flooding up to design height
//! (10 ft default). Water above the design height causes overtopping and
//! catastrophic failure, resulting in worse flooding than if the levee were absent.
//!
//! **Seawall**: Placeable along the coast, prevents coastal surge from flooding
//! inland cells. Seawalls have a fixed protection height of 15 ft.
//!
//! **Floodgate**: Allows controlled water release. When open, water flows freely;
//! when closed, acts like a levee with 12 ft protection height.
//!
//! **Maintenance**: Each protection cell costs $2,000/year. Neglected infrastructure
//! degrades over time, increasing failure probability. Failure probability rises
//! with age and lack of maintenance.
//!
//! The `update_flood_protection` system runs every slow tick and:
//!   1. Ages all protection structures
//!   2. Applies maintenance costs from the city budget
//!   3. Degrades unmaintained structures (condition decreases)
//!   4. Checks for overtopping during active floods
//!   5. Calculates failure probability based on age + condition
//!   6. Reduces flood depth in protected cells (or amplifies on failure)
//!   7. Updates aggregate protection statistics

pub mod systems;
pub mod types;

#[cfg(test)]
mod tests_systems;
#[cfg(test)]
mod tests_types;

// Re-export all public items for backward compatibility.
pub use systems::{
    can_place_floodgate, can_place_levee, can_place_seawall, daily_maintenance_cost,
    is_adjacent_to_water, should_fail, update_flood_protection, FloodProtectionPlugin,
};
pub use types::{FloodProtectionState, ProtectionStructure, ProtectionType};
