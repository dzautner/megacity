//! Tabbed Building Info Panel (UX-005 + UX-061).
//!
//! Organizes the Building Inspector into tabs: Overview, Services, Economy,
//! Residents/Workers, and Environment. The active tab is tracked per session
//! via a Bevy [`Resource`]. Service and utility buildings retain their
//! simpler flat layouts since they have less information.
//!
//! # Module structure
//!
//! - [`types`] — Tab identifiers and active-tab resource.
//! - [`helpers`] — Shared colour / label / widget helpers.
//! - [`simple_tabs`] — Overview, Services, and Environment tab renderers.
//! - [`economy_tab`] — Economy tab renderer.
//! - [`citizen_tabs`] — Residents and Workers tab renderers.
//! - [`inspector`] — Main UI system and plugin registration.

mod citizen_tabs;
mod economy_tab;
pub(crate) mod helpers;
mod inspector;
mod simple_tabs;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so callers don't need to change their imports.
pub use inspector::{progressive_building_inspection_ui, ProgressiveDisclosurePlugin};
pub use types::{BuildingTab, SectionStates, SelectedBuildingTab};
