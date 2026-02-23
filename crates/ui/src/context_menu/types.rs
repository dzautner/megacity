//! Types and helpers for the right-click context menu.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::grid::ZoneType;
use simulation::road_segments::SegmentId;

/// What kind of entity is under the cursor when the context menu opens.
#[derive(Debug, Clone)]
pub enum ContextTarget {
    /// A zoned building (residential, commercial, etc.)
    Building {
        entity: Entity,
        zone_type: ZoneType,
        level: u8,
        grid_x: usize,
        grid_y: usize,
    },
    /// A service building (fire station, hospital, etc.)
    Service {
        entity: Entity,
        name: String,
        grid_x: usize,
        grid_y: usize,
    },
    /// A road cell (may belong to a segment)
    Road {
        grid_x: usize,
        grid_y: usize,
        segment_id: Option<SegmentId>,
    },
    /// A citizen
    Citizen { entity: Entity },
    /// An empty grass cell
    Empty {
        grid_x: usize,
        grid_y: usize,
        zone_type: ZoneType,
    },
}

/// State of the right-click context menu.
#[derive(Resource, Default)]
pub struct ContextMenuState {
    /// Whether the menu is currently open.
    pub open: bool,
    /// Screen position (egui) where the menu should appear.
    pub screen_pos: egui::Pos2,
    /// What entity the menu targets.
    pub target: Option<ContextTarget>,
}

/// Action selected from the context menu, consumed by the action system.
#[derive(Debug, Clone)]
pub(crate) enum ContextMenuAction {
    Inspect,
    Bulldoze,
    SetToolZone(ZoneType),
    SetToolPlaceService,
    ToggleOneWay(SegmentId),
    FollowCitizen(Entity),
    CitizenDetails(Entity),
}

/// One-frame event carrying the chosen action.
#[derive(Resource, Default)]
pub(crate) struct PendingAction(pub Option<ContextMenuAction>);

/// Human-readable label for a zone type.
pub(crate) fn zone_label(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "Unzoned",
        ZoneType::ResidentialLow => "Low-Density Residential",
        ZoneType::ResidentialMedium => "Medium-Density Residential",
        ZoneType::ResidentialHigh => "High-Density Residential",
        ZoneType::CommercialLow => "Low-Density Commercial",
        ZoneType::CommercialHigh => "High-Density Commercial",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
        ZoneType::MixedUse => "Mixed-Use",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_state_default() {
        let state = ContextMenuState::default();
        assert!(!state.open);
        assert!(state.target.is_none());
    }

    #[test]
    fn test_pending_action_default() {
        let pending = PendingAction::default();
        assert!(pending.0.is_none());
    }

    #[test]
    fn test_zone_label() {
        assert_eq!(zone_label(ZoneType::None), "Unzoned");
        assert_eq!(
            zone_label(ZoneType::ResidentialLow),
            "Low-Density Residential"
        );
        assert_eq!(
            zone_label(ZoneType::ResidentialHigh),
            "High-Density Residential"
        );
        assert_eq!(
            zone_label(ZoneType::CommercialLow),
            "Low-Density Commercial"
        );
        assert_eq!(
            zone_label(ZoneType::CommercialHigh),
            "High-Density Commercial"
        );
        assert_eq!(zone_label(ZoneType::Industrial), "Industrial");
        assert_eq!(zone_label(ZoneType::Office), "Office");
        assert_eq!(zone_label(ZoneType::MixedUse), "Mixed-Use");
    }

    #[test]
    fn test_context_target_variants() {
        // Verify all target variants can be constructed
        let _building = ContextTarget::Building {
            entity: Entity::from_raw(1),
            zone_type: ZoneType::ResidentialLow,
            level: 1,
            grid_x: 10,
            grid_y: 20,
        };

        let _service = ContextTarget::Service {
            entity: Entity::from_raw(2),
            name: "Fire Station".to_string(),
            grid_x: 5,
            grid_y: 5,
        };

        let _road = ContextTarget::Road {
            grid_x: 3,
            grid_y: 3,
            segment_id: Some(SegmentId(42)),
        };

        let _citizen = ContextTarget::Citizen {
            entity: Entity::from_raw(3),
        };

        let _empty = ContextTarget::Empty {
            grid_x: 0,
            grid_y: 0,
            zone_type: ZoneType::None,
        };
    }
}
