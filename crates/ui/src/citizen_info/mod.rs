//! Citizen Info Panel (UX-063).
//!
//! When a citizen entity is clicked (in Inspect mode), displays:
//! - Name, age, gender
//! - Job type and workplace location
//! - Happiness with factor breakdown (needs)
//! - Current state (at home, commuting, at work, shopping, etc.)
//! - Home and work locations
//! - "Follow" button to enter camera follow mode

mod display;
mod names;
mod plugin;
mod resources;
mod systems;
#[cfg(test)]
mod tests;

pub use plugin::CitizenInfoPlugin;
pub use resources::{FollowCitizen, SelectedCitizen};
pub use systems::{camera_follow_citizen, citizen_info_panel_ui, detect_citizen_selection};
