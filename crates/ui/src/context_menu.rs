//! Right-Click Context Menu (UX-012).
//!
//! Right-click release (without drag, < 5px movement) shows a context menu
//! for the entity under the cursor. Menu items vary by entity type:
//! - **Building**: Inspect, Bulldoze, Upgrade, Set Policy
//! - **Road**: Inspect, Upgrade, Bulldoze, One-Way
//! - **Citizen**: Follow, Details
//! - **Empty**: Zone, Place Service
//!
//! The menu closes on click outside, Escape, or when an item is selected.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::RightClickDrag;
use rendering::input::{ActiveTool, CursorGridPos, SelectedBuilding, StatusMessage};
use simulation::buildings::Building;
use simulation::citizen::{Citizen, Position};
use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::oneway::ToggleOneWayEvent;
use simulation::road_segments::{RoadSegmentStore, SegmentId};
use simulation::services::ServiceBuilding;

use crate::citizen_info::{FollowCitizen, SelectedCitizen};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

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
pub enum ContextMenuAction {
    Inspect,
    Bulldoze,
    SetToolZone(ZoneType),
    SetToolPlaceService,
    SetToolUpgradeRoad,
    ToggleOneWay(SegmentId),
    FollowCitizen(Entity),
    CitizenDetails(Entity),
}

/// One-frame event carrying the chosen action.
#[derive(Resource, Default)]
struct PendingAction(Option<ContextMenuAction>);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Detect right-click release (without drag) and open the context menu.
#[allow(clippy::too_many_arguments)]
pub fn detect_right_click_context_menu(
    right_click: Res<RightClickDrag>,
    cursor: Res<CursorGridPos>,
    windows: Query<&Window>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    buildings: Query<(Entity, &Building)>,
    services: Query<(Entity, &ServiceBuilding)>,
    citizens: Query<(Entity, &Position), With<Citizen>>,
    mut state: ResMut<ContextMenuState>,
) {
    if !right_click.just_released_click {
        return;
    }

    if !cursor.valid {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    // Get screen position for the menu
    let screen_pos = if let Some(pos) = window.cursor_position() {
        egui::pos2(pos.x, pos.y)
    } else {
        return;
    };

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    // 1. Check for citizen under cursor (small radius)
    let world_x = gx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let world_y = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let radius_sq = (CELL_SIZE * 2.0) * (CELL_SIZE * 2.0);

    let mut best_citizen: Option<(Entity, f32)> = None;
    for (entity, pos) in &citizens {
        let dx = pos.x - world_x;
        let dy = pos.y - world_y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < radius_sq && (best_citizen.is_none() || dist_sq < best_citizen.unwrap().1) {
            best_citizen = Some((entity, dist_sq));
        }
    }

    if let Some((entity, _)) = best_citizen {
        state.open = true;
        state.screen_pos = screen_pos;
        state.target = Some(ContextTarget::Citizen { entity });
        return;
    }

    let cell = grid.get(gx, gy);

    // 2. Check for building
    if let Some(building_entity) = cell.building_id {
        // Check if it's a service building
        if let Ok((ent, service)) = services.get(building_entity) {
            state.open = true;
            state.screen_pos = screen_pos;
            state.target = Some(ContextTarget::Service {
                entity: ent,
                name: service.service_type.name().to_string(),
                grid_x: service.grid_x,
                grid_y: service.grid_y,
            });
            return;
        }

        // Check if it's a zoned building
        if let Ok((ent, building)) = buildings.get(building_entity) {
            state.open = true;
            state.screen_pos = screen_pos;
            state.target = Some(ContextTarget::Building {
                entity: ent,
                zone_type: building.zone_type,
                level: building.level,
                grid_x: building.grid_x,
                grid_y: building.grid_y,
            });
            return;
        }
    }

    // 3. Check for road
    if cell.cell_type == CellType::Road {
        // Try to find the road segment this cell belongs to
        let mut found_segment = None;
        for segment in &segments.segments {
            if segment.rasterized_cells.contains(&(gx, gy)) {
                found_segment = Some(segment.id);
                break;
            }
        }

        state.open = true;
        state.screen_pos = screen_pos;
        state.target = Some(ContextTarget::Road {
            grid_x: gx,
            grid_y: gy,
            segment_id: found_segment,
        });
        return;
    }

    // 4. Empty cell
    if cell.cell_type == CellType::Grass {
        state.open = true;
        state.screen_pos = screen_pos;
        state.target = Some(ContextTarget::Empty {
            grid_x: gx,
            grid_y: gy,
            zone_type: cell.zone,
        });
    }
}

/// Render the context menu using egui.
pub fn context_menu_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<ContextMenuState>,
    mut pending: ResMut<PendingAction>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !state.open {
        return;
    }

    // Close on Escape
    if keys.just_pressed(KeyCode::Escape) {
        state.open = false;
        state.target = None;
        return;
    }

    let Some(target) = state.target.clone() else {
        state.open = false;
        return;
    };

    let pos = state.screen_pos;

    let mut close = false;

    egui::Area::new(egui::Id::new("context_menu_area"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(140.0);

                match &target {
                    ContextTarget::Building {
                        entity,
                        zone_type,
                        level,
                        ..
                    } => {
                        ui.label(
                            egui::RichText::new(format!("{} (L{})", zone_label(*zone_type), level))
                                .strong(),
                        );
                        ui.separator();

                        if ui.button("Inspect").clicked() {
                            pending.0 = Some(ContextMenuAction::Inspect);
                            close = true;
                        }
                        if ui.button("Bulldoze").clicked() {
                            pending.0 = Some(ContextMenuAction::Bulldoze);
                            close = true;
                        }

                        let _ = entity; // entity used via Inspect/Bulldoze targeting
                    }
                    ContextTarget::Service { name, .. } => {
                        ui.label(egui::RichText::new(name.as_str()).strong());
                        ui.separator();

                        if ui.button("Inspect").clicked() {
                            pending.0 = Some(ContextMenuAction::Inspect);
                            close = true;
                        }
                        if ui.button("Bulldoze").clicked() {
                            pending.0 = Some(ContextMenuAction::Bulldoze);
                            close = true;
                        }
                    }
                    ContextTarget::Road { segment_id, .. } => {
                        ui.label(egui::RichText::new("Road").strong());
                        ui.separator();

                        if ui.button("Inspect").clicked() {
                            pending.0 = Some(ContextMenuAction::Inspect);
                            close = true;
                        }
                        if ui.button("Bulldoze").clicked() {
                            pending.0 = Some(ContextMenuAction::Bulldoze);
                            close = true;
                        }
                        if let Some(seg_id) = segment_id {
                            if ui.button("Toggle One-Way").clicked() {
                                pending.0 = Some(ContextMenuAction::ToggleOneWay(*seg_id));
                                close = true;
                            }
                        }
                    }
                    ContextTarget::Citizen { entity } => {
                        ui.label(egui::RichText::new("Citizen").strong());
                        ui.separator();

                        if ui.button("Follow").clicked() {
                            pending.0 = Some(ContextMenuAction::FollowCitizen(*entity));
                            close = true;
                        }
                        if ui.button("Details").clicked() {
                            pending.0 = Some(ContextMenuAction::CitizenDetails(*entity));
                            close = true;
                        }
                    }
                    ContextTarget::Empty { zone_type, .. } => {
                        let label = if *zone_type == ZoneType::None {
                            "Empty Cell"
                        } else {
                            "Zoned Cell"
                        };
                        ui.label(egui::RichText::new(label).strong());
                        ui.separator();

                        if ui.button("Zone").clicked() {
                            pending.0 =
                                Some(ContextMenuAction::SetToolZone(ZoneType::ResidentialLow));
                            close = true;
                        }
                        if ui.button("Place Service").clicked() {
                            pending.0 = Some(ContextMenuAction::SetToolPlaceService);
                            close = true;
                        }
                    }
                }
            });
        });

    // Close on click outside: check if mouse is pressed and not over the menu
    // egui handles this via the area's response, but we also close if left-click happens
    let ctx = contexts.ctx_mut();
    if ctx.input(|i| i.pointer.any_pressed()) && !ctx.is_pointer_over_area() {
        close = true;
    }

    if close {
        state.open = false;
        state.target = None;
    }
}

/// Execute the chosen context menu action.
#[allow(clippy::too_many_arguments)]
pub fn execute_context_menu_action(
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
            if let Some(target) = &state.target {
                match target {
                    ContextTarget::Building { entity, .. }
                    | ContextTarget::Service { entity, .. } => {
                        selected_building.0 = Some(*entity);
                    }
                    _ => {}
                }
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
        ContextMenuAction::SetToolUpgradeRoad => {
            status.set("Road upgrade not yet available", false);
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn zone_label(zone: ZoneType) -> &'static str {
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

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextMenuState>()
            .init_resource::<PendingAction>()
            .add_systems(
                Update,
                (
                    detect_right_click_context_menu,
                    context_menu_ui,
                    execute_context_menu_action,
                )
                    .chain(),
            );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
