//! Undo/Redo System for Player Actions (UX-001).
//!
//! Implements the command pattern for all player actions. An `ActionHistory`
//! resource maintains undo and redo stacks (capped at 100 entries). Actions
//! are recorded via the `RecordAction` event, and Ctrl+Z / Ctrl+Y (or
//! Ctrl+Shift+Z) trigger undo/redo respectively.
//!
//! Composite actions group drag operations (e.g., road drag = 1 undo step).
//! Treasury is restored on undo.

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::keybindings::KeyBinding;
use crate::road_segments::{RoadSegmentStore, SegmentId, SegmentNodeId};
use crate::roads::RoadNetwork;
use crate::services::{self, ServiceBuilding, ServiceType};
use crate::utilities::{UtilitySource, UtilityType};

/// Maximum number of actions kept in the undo stack.
pub const MAX_HISTORY: usize = 100;

// ---------------------------------------------------------------------------
// CityAction enum — each variant stores enough data to reverse the action
// ---------------------------------------------------------------------------

/// A single undoable/redoable player action.
#[derive(Debug, Clone, Event)]
pub enum CityAction {
    /// A road segment was placed via the freeform drawing tool.
    PlaceRoadSegment {
        segment_id: SegmentId,
        start_node: SegmentNodeId,
        end_node: SegmentNodeId,
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        road_type: RoadType,
        rasterized_cells: Vec<(usize, usize)>,
        cost: f64,
    },
    /// A road cell was placed via the legacy grid-snap tool.
    PlaceGridRoad {
        x: usize,
        y: usize,
        road_type: RoadType,
        cost: f64,
    },
    /// One or more zone cells were painted.
    PlaceZone {
        cells: Vec<(usize, usize, ZoneType)>,
        cost: f64,
    },
    /// A service building was placed.
    PlaceService {
        service_type: ServiceType,
        grid_x: usize,
        grid_y: usize,
        cost: f64,
    },
    /// A utility building was placed.
    PlaceUtility {
        utility_type: UtilityType,
        grid_x: usize,
        grid_y: usize,
        cost: f64,
    },
    /// A road cell was bulldozed.
    BulldozeRoad {
        x: usize,
        y: usize,
        road_type: RoadType,
        refund: f64,
    },
    /// A zone cell was bulldozed (cleared to None).
    BulldozeZone { x: usize, y: usize, zone: ZoneType },
    /// A service building was bulldozed.
    BulldozeService {
        service_type: ServiceType,
        grid_x: usize,
        grid_y: usize,
        refund: f64,
    },
    /// A utility building was bulldozed.
    BulldozeUtility {
        utility_type: UtilityType,
        grid_x: usize,
        grid_y: usize,
        refund: f64,
    },
    /// Multiple actions grouped as one (e.g., a drag operation).
    Composite(Vec<CityAction>),
}

// ---------------------------------------------------------------------------
// ActionHistory resource
// ---------------------------------------------------------------------------

/// Stores undo and redo stacks for player actions.
#[derive(Resource, Default)]
pub struct ActionHistory {
    pub undo_stack: Vec<CityAction>,
    pub redo_stack: Vec<CityAction>,
}

impl ActionHistory {
    /// Push a new action onto the undo stack, clearing the redo stack.
    /// If the stack exceeds `MAX_HISTORY`, the oldest action is dropped.
    pub fn push(&mut self, action: CityAction) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// Pop the most recent action from the undo stack for undoing.
    pub fn pop_undo(&mut self) -> Option<CityAction> {
        self.undo_stack.pop()
    }

    /// Pop the most recent action from the redo stack for redoing.
    pub fn pop_redo(&mut self) -> Option<CityAction> {
        self.redo_stack.pop()
    }

    /// Push an action onto the redo stack (after undo).
    pub fn push_redo(&mut self, action: CityAction) {
        self.redo_stack.push(action);
    }

    /// Push an action onto the undo stack (after redo), without clearing redo.
    pub fn push_undo_no_clear(&mut self, action: CityAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Events for triggering undo/redo from keyboard input
// ---------------------------------------------------------------------------

/// Marker event: the player wants to undo.
#[derive(Event)]
pub struct UndoRequested;

/// Marker event: the player wants to redo.
#[derive(Event)]
pub struct RedoRequested;

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_history_push_and_undo() {
        let mut history = ActionHistory::default();
        let action = CityAction::PlaceGridRoad {
            x: 10,
            y: 20,
            road_type: RoadType::Local,
            cost: 10.0,
        };
        history.push(action);
        assert_eq!(history.undo_stack.len(), 1);
        assert!(history.redo_stack.is_empty());

        let undone = history.pop_undo();
        assert!(undone.is_some());
        assert!(history.undo_stack.is_empty());
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let mut history = ActionHistory::default();
        // Push and undo to get something in the redo stack
        history.push(CityAction::PlaceGridRoad {
            x: 10,
            y: 20,
            road_type: RoadType::Local,
            cost: 10.0,
        });
        let action = history.pop_undo().unwrap();
        history.push_redo(action);
        assert_eq!(history.redo_stack.len(), 1);

        // New action should clear redo
        history.push(CityAction::PlaceGridRoad {
            x: 5,
            y: 5,
            road_type: RoadType::Avenue,
            cost: 20.0,
        });
        assert!(history.redo_stack.is_empty());
    }

    #[test]
    fn test_max_history_limit() {
        let mut history = ActionHistory::default();
        for i in 0..150 {
            history.push(CityAction::PlaceGridRoad {
                x: i,
                y: 0,
                road_type: RoadType::Local,
                cost: 10.0,
            });
        }
        assert_eq!(history.undo_stack.len(), MAX_HISTORY);
    }

    #[test]
    fn test_composite_action() {
        let mut history = ActionHistory::default();
        let composite = CityAction::Composite(vec![
            CityAction::PlaceGridRoad {
                x: 10,
                y: 10,
                road_type: RoadType::Local,
                cost: 10.0,
            },
            CityAction::PlaceGridRoad {
                x: 11,
                y: 10,
                road_type: RoadType::Local,
                cost: 10.0,
            },
        ]);
        history.push(composite);
        assert_eq!(history.undo_stack.len(), 1);
    }

    #[test]
    fn test_can_undo_can_redo() {
        let mut history = ActionHistory::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.push(CityAction::PlaceGridRoad {
            x: 0,
            y: 0,
            road_type: RoadType::Local,
            cost: 10.0,
        });
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let action = history.pop_undo().unwrap();
        history.push_redo(action);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_push_undo_no_clear_preserves_redo() {
        let mut history = ActionHistory::default();
        history.push_redo(CityAction::PlaceGridRoad {
            x: 0,
            y: 0,
            road_type: RoadType::Local,
            cost: 10.0,
        });
        history.push_undo_no_clear(CityAction::PlaceGridRoad {
            x: 1,
            y: 1,
            road_type: RoadType::Avenue,
            cost: 20.0,
        });
        assert!(history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_place_zone_action() {
        let mut history = ActionHistory::default();
        let action = CityAction::PlaceZone {
            cells: vec![
                (10, 10, ZoneType::ResidentialLow),
                (10, 11, ZoneType::ResidentialLow),
            ],
            cost: 10.0,
        };
        history.push(action);
        assert_eq!(history.undo_stack.len(), 1);
    }
}
