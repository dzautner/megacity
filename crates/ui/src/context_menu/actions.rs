//! System for executing the chosen context menu action.

use bevy::prelude::*;

use rendering::input::{ActiveTool, SelectedBuilding, StatusMessage};
use simulation::grid::ZoneType;
use simulation::oneway::ToggleOneWayEvent;

use crate::citizen_info::{FollowCitizen, SelectedCitizen};

use super::types::{ContextMenuAction, ContextMenuState, ContextTarget, PendingAction};

/// Execute the chosen context menu action.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_context_menu_action(
    mut pending: ResMut<PendingAction>,
    mut tool: ResMut<ActiveTool>,
    mut selected_building: ResMut<SelectedBuilding>,
    mut selected_citizen: ResMut<SelectedCitizen>,
    mut follow_citizen: ResMut<FollowCitizen>,
    mut toggle_events: EventWriter<ToggleOneWayEvent>,
    state: Res<ContextMenuState>,
    mut status: ResMut<StatusMessage>,
) {
    let Some(action) = pending.0.take() else {
        return;
    };

    match action {
        ContextMenuAction::Inspect => {
            *tool = ActiveTool::Inspect;
            // If context was on a building/service, select it
            if let Some(
                ContextTarget::Building { entity, .. } | ContextTarget::Service { entity, .. },
            ) = &state.target
            {
                selected_building.0 = Some(*entity);
            }
        }
        ContextMenuAction::Bulldoze => {
            *tool = ActiveTool::Bulldoze;
            status.set("Bulldoze tool selected", false);
        }
        ContextMenuAction::SetToolZone(zone) => {
            *tool = match zone {
                ZoneType::ResidentialLow => ActiveTool::ZoneResidentialLow,
                ZoneType::ResidentialMedium => ActiveTool::ZoneResidentialMedium,
                ZoneType::ResidentialHigh => ActiveTool::ZoneResidentialHigh,
                ZoneType::CommercialLow => ActiveTool::ZoneCommercialLow,
                ZoneType::CommercialHigh => ActiveTool::ZoneCommercialHigh,
                ZoneType::Industrial => ActiveTool::ZoneIndustrial,
                ZoneType::Office => ActiveTool::ZoneOffice,
                ZoneType::MixedUse => ActiveTool::ZoneMixedUse,
                ZoneType::None => ActiveTool::ZoneResidentialLow,
            };
            status.set("Zone tool selected", false);
        }
        ContextMenuAction::SetToolPlaceService => {
            *tool = ActiveTool::PlaceFireStation;
            status.set("Place Service tool selected — choose from toolbar", false);
        }
        ContextMenuAction::ToggleOneWay(seg_id) => {
            toggle_events.send(ToggleOneWayEvent { segment_id: seg_id });
        }
        ContextMenuAction::FollowCitizen(entity) => {
            selected_citizen.0 = Some(entity);
            follow_citizen.0 = Some(entity);
        }
        ContextMenuAction::CitizenDetails(entity) => {
            selected_citizen.0 = Some(entity);
            *tool = ActiveTool::Inspect;
        }
    }
}
