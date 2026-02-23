//! Systems and core logic for undo/redo processing.

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::keybindings::KeyBinding;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::services::{self, ServiceBuilding, ServiceType};
use crate::utilities::{UtilitySource, UtilityType};

use super::history::{ActionHistory, RedoRequested, UndoRequested};
use super::types::CityAction;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// System that collects `CityAction` events and pushes them onto the history.
pub fn collect_actions(mut events: EventReader<CityAction>, mut history: ResMut<ActionHistory>) {
    for action in events.read() {
        history.push(action.clone());
    }
}

/// Keyboard listener: Ctrl+Z -> UndoRequested, Ctrl+Y / Ctrl+Shift+Z -> RedoRequested.
///
/// Uses `Option<Res<...>>` so the system is a no-op in headless tests
/// where Bevy's InputPlugin (and thus ButtonInput<KeyCode>) is not present.
pub fn keyboard_undo_redo(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut undo_events: EventWriter<UndoRequested>,
    mut redo_events: EventWriter<RedoRequested>,
) {
    let Some(keys) = keys else { return };
    let undo_binding = KeyBinding::ctrl(KeyCode::KeyZ);
    let redo_binding_y = KeyBinding::ctrl(KeyCode::KeyY);
    let redo_binding_shift_z = KeyBinding {
        key: KeyCode::KeyZ,
        ctrl: true,
        shift: true,
    };

    // Check redo first (Ctrl+Shift+Z) before undo (Ctrl+Z) since the shift
    // variant is more specific.
    if redo_binding_shift_z.just_pressed(&keys) {
        redo_events.send(RedoRequested);
    } else if undo_binding.just_pressed(&keys) {
        undo_events.send(UndoRequested);
    }

    if redo_binding_y.just_pressed(&keys) {
        redo_events.send(RedoRequested);
    }
}

/// System that processes undo requests.
#[allow(clippy::too_many_arguments)]
pub fn process_undo(
    mut events: EventReader<UndoRequested>,
    mut history: ResMut<ActionHistory>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut budget: ResMut<CityBudget>,
    mut commands: Commands,
    service_q: Query<(Entity, &ServiceBuilding)>,
    utility_q: Query<(Entity, &UtilitySource)>,
) {
    for _ in events.read() {
        if let Some(action) = history.pop_undo() {
            undo_action(
                &action,
                &mut grid,
                &mut roads,
                &mut segments,
                &mut budget,
                &mut commands,
                &service_q,
                &utility_q,
            );
            history.push_redo(action);
        }
    }
}

/// System that processes redo requests.
#[allow(clippy::too_many_arguments)]
pub fn process_redo(
    mut events: EventReader<RedoRequested>,
    mut history: ResMut<ActionHistory>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut budget: ResMut<CityBudget>,
    mut commands: Commands,
    service_q: Query<(Entity, &ServiceBuilding)>,
    utility_q: Query<(Entity, &UtilitySource)>,
) {
    for _ in events.read() {
        if let Some(action) = history.pop_redo() {
            redo_action(
                &action,
                &mut grid,
                &mut roads,
                &mut segments,
                &mut budget,
                &mut commands,
                &service_q,
                &utility_q,
            );
            history.push_undo_no_clear(action);
        }
    }
}

// ---------------------------------------------------------------------------
// Undo logic — reverse the action
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn undo_action(
    action: &CityAction,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    segments: &mut RoadSegmentStore,
    budget: &mut CityBudget,
    commands: &mut Commands,
    service_q: &Query<(Entity, &ServiceBuilding)>,
    utility_q: &Query<(Entity, &UtilitySource)>,
) {
    match action {
        CityAction::PlaceRoadSegment {
            segment_id, cost, ..
        } => {
            // Remove the segment (un-rasterizes from grid)
            segments.remove_segment(*segment_id, grid, roads);
            budget.treasury += cost;
        }
        CityAction::PlaceGridRoad { x, y, cost, .. } => {
            roads.remove_road(grid, *x, *y);
            budget.treasury += cost;
        }
        CityAction::PlaceZone { cells, cost } => {
            // Restore each cell to ZoneType::None
            for &(x, y, _zone) in cells {
                if grid.in_bounds(x, y) {
                    grid.get_mut(x, y).zone = ZoneType::None;
                }
            }
            budget.treasury += cost;
        }
        CityAction::PlaceService {
            service_type,
            grid_x,
            grid_y,
            cost,
        } => {
            // Find and despawn the service entity at this location
            let (fw, fh) = ServiceBuilding::footprint(*service_type);
            for (entity, service) in service_q.iter() {
                if service.service_type == *service_type
                    && service.grid_x == *grid_x
                    && service.grid_y == *grid_y
                {
                    // Clear grid cells
                    for fy in *grid_y..*grid_y + fh {
                        for fx in *grid_x..*grid_x + fw {
                            if grid.in_bounds(fx, fy) {
                                grid.get_mut(fx, fy).building_id = None;
                                grid.get_mut(fx, fy).zone = ZoneType::None;
                            }
                        }
                    }
                    commands.entity(entity).despawn();
                    break;
                }
            }
            budget.treasury += cost;
        }
        CityAction::PlaceUtility {
            utility_type,
            grid_x,
            grid_y,
            cost,
        } => {
            for (entity, utility) in utility_q.iter() {
                if utility.utility_type == *utility_type
                    && utility.grid_x == *grid_x
                    && utility.grid_y == *grid_y
                {
                    if grid.in_bounds(*grid_x, *grid_y) {
                        grid.get_mut(*grid_x, *grid_y).building_id = None;
                    }
                    commands.entity(entity).despawn();
                    break;
                }
            }
            budget.treasury += cost;
        }
        CityAction::BulldozeRoad {
            x,
            y,
            road_type,
            refund,
        } => {
            // Re-place the road
            roads.place_road_typed(grid, *x, *y, *road_type);
            budget.treasury -= refund;
        }
        CityAction::BulldozeZone { x, y, zone } => {
            if grid.in_bounds(*x, *y) {
                grid.get_mut(*x, *y).zone = *zone;
            }
        }
        CityAction::BulldozeService {
            service_type,
            grid_x,
            grid_y,
            refund,
        } => {
            // Re-place the service building
            services::place_service(commands, grid, *service_type, *grid_x, *grid_y);
            budget.treasury -= refund;
        }
        CityAction::BulldozeUtility {
            utility_type,
            grid_x,
            grid_y,
            refund,
        } => {
            services::place_utility_source(commands, grid, *utility_type, *grid_x, *grid_y);
            budget.treasury -= refund;
        }
        CityAction::Composite(actions) => {
            // Undo in reverse order
            for sub in actions.iter().rev() {
                undo_action(
                    sub, grid, roads, segments, budget, commands, service_q, utility_q,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Redo logic — re-apply the action
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn redo_action(
    action: &CityAction,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    segments: &mut RoadSegmentStore,
    budget: &mut CityBudget,
    commands: &mut Commands,
    service_q: &Query<(Entity, &ServiceBuilding)>,
    utility_q: &Query<(Entity, &UtilitySource)>,
) {
    match action {
        CityAction::PlaceRoadSegment {
            start_node,
            end_node,
            p0,
            p1,
            p2,
            p3,
            road_type,
            cost,
            ..
        } => {
            segments.add_segment(
                *start_node,
                *end_node,
                *p0,
                *p1,
                *p2,
                *p3,
                *road_type,
                grid,
                roads,
            );
            budget.treasury -= cost;
        }
        CityAction::PlaceGridRoad {
            x,
            y,
            road_type,
            cost,
        } => {
            roads.place_road_typed(grid, *x, *y, *road_type);
            budget.treasury -= cost;
        }
        CityAction::PlaceZone { cells, cost } => {
            for &(x, y, zone) in cells {
                if grid.in_bounds(x, y) {
                    grid.get_mut(x, y).zone = zone;
                }
            }
            budget.treasury -= cost;
        }
        CityAction::PlaceService {
            service_type,
            grid_x,
            grid_y,
            cost,
        } => {
            services::place_service(commands, grid, *service_type, *grid_x, *grid_y);
            budget.treasury -= cost;
        }
        CityAction::PlaceUtility {
            utility_type,
            grid_x,
            grid_y,
            cost,
        } => {
            services::place_utility_source(commands, grid, *utility_type, *grid_x, *grid_y);
            budget.treasury -= cost;
        }
        CityAction::BulldozeRoad { x, y, refund, .. } => {
            roads.remove_road(grid, *x, *y);
            budget.treasury += refund;
        }
        CityAction::BulldozeZone { x, y, .. } => {
            if grid.in_bounds(*x, *y) {
                grid.get_mut(*x, *y).zone = ZoneType::None;
            }
        }
        CityAction::BulldozeService {
            service_type,
            grid_x,
            grid_y,
            refund,
        } => {
            // Find and despawn the service entity
            let (fw, fh) = ServiceBuilding::footprint(*service_type);
            for (entity, service) in service_q.iter() {
                if service.service_type == *service_type
                    && service.grid_x == *grid_x
                    && service.grid_y == *grid_y
                {
                    for fy in *grid_y..*grid_y + fh {
                        for fx in *grid_x..*grid_x + fw {
                            if grid.in_bounds(fx, fy) {
                                grid.get_mut(fx, fy).building_id = None;
                                grid.get_mut(fx, fy).zone = ZoneType::None;
                            }
                        }
                    }
                    commands.entity(entity).despawn();
                    break;
                }
            }
            budget.treasury += refund;
        }
        CityAction::BulldozeUtility {
            utility_type,
            grid_x,
            grid_y,
            refund,
        } => {
            for (entity, utility) in utility_q.iter() {
                if utility.utility_type == *utility_type
                    && utility.grid_x == *grid_x
                    && utility.grid_y == *grid_y
                {
                    if grid.in_bounds(*grid_x, *grid_y) {
                        grid.get_mut(*grid_x, *grid_y).building_id = None;
                    }
                    commands.entity(entity).despawn();
                    break;
                }
            }
            budget.treasury += refund;
        }
        CityAction::Composite(actions) => {
            for sub in actions {
                redo_action(
                    sub, grid, roads, segments, budget, commands, service_q, utility_q,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionHistory>()
            .add_event::<CityAction>()
            .add_event::<UndoRequested>()
            .add_event::<RedoRequested>()
            .add_systems(
                Update,
                (
                    keyboard_undo_redo,
                    collect_actions,
                    process_undo.after(keyboard_undo_redo),
                    process_redo.after(keyboard_undo_redo),
                ),
            );
    }
}
