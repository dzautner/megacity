//! District Selection and Inspection Panel (UX-062).
//!
//! Clicking a cell in Inspect mode shows its district assignment. When a district
//! is selected, an info panel displays:
//! - District name and color swatch
//! - Population and jobs (commercial, industrial, office)
//! - Average happiness
//! - Service coverage: counts of fire, police, health, education, parks, and
//!   transport services whose radius overlaps cells in the district
//! - District boundary highlighting via an optional overlay flag
//!
//! The panel appears as an egui window anchored to the left side of the screen.

mod helpers;
mod resources;
mod systems;
mod ui_panel;

#[cfg(test)]
mod tests;

pub use helpers::{
    grid_to_world_center, happiness_color, happiness_label, resolve_district_index,
    service_covers_cell,
};
pub use resources::{DistrictInspectCache, DistrictPanelOpen, SelectedDistrict};
pub use systems::{detect_district_selection, refresh_district_inspect, DistrictInspectPlugin};
pub use ui_panel::district_inspect_ui;
