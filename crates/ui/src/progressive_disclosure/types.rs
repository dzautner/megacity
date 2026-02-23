//! Tab identifiers and active-tab resource for the Building Inspector.

use bevy::prelude::*;

// =============================================================================
// Tab identifiers
// =============================================================================

/// Identifies a tab in the Building Inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildingTab {
    /// Type, level, occupancy, happiness at a glance.
    #[default]
    Overview,
    /// Power, water, and nearby service coverage/quality.
    Services,
    /// Land value, rent/salary, property value, taxes.
    Economy,
    /// List of residents or workers (clickable to follow).
    Residents,
    /// Pollution, noise, green space.
    Environment,
}

impl BuildingTab {
    /// Human-readable label for this tab.
    pub fn label(&self) -> &'static str {
        match self {
            BuildingTab::Overview => "Overview",
            BuildingTab::Services => "Services",
            BuildingTab::Economy => "Economy",
            BuildingTab::Residents => "Residents",
            BuildingTab::Environment => "Environment",
        }
    }

    /// All tab variants in display order.
    pub const ALL: [BuildingTab; 5] = [
        BuildingTab::Overview,
        BuildingTab::Services,
        BuildingTab::Economy,
        BuildingTab::Residents,
        BuildingTab::Environment,
    ];
}

// =============================================================================
// Active tab resource
// =============================================================================

/// Tracks which tab is currently active in the Building Inspector.
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedBuildingTab(pub BuildingTab);

// Keep backward compatibility: SectionStates is now an alias.
// (Nothing outside this file uses it, but this avoids breakage if something
// referenced the type via the re-export.)
/// Legacy alias kept for backward compatibility.
pub type SectionStates = SelectedBuildingTab;
