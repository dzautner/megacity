//! Box Selection (UX-011): Shift+Left-Click drag to select multiple entities.
//!
//! When the player holds Shift and drags with the left mouse button, a
//! translucent rectangle is drawn on the ground plane. On release, all
//! buildings and road cells within the rectangle are added to the
//! `MultiSelectState` from the multi-select system.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid};
use simulation::multi_select::{MultiSelectState, SelectableItem};

use crate::input::{CursorGridPos, DrawPhase, RoadDrawState, StatusMessage};

// =============================================================================
// Constants
// =============================================================================

/// Minimum drag distance in world units before box selection activates.
const MIN_BOX_SIZE: f32 = CELL_SIZE * 0.5;

/// Color for the box selection rectangle outline.
const BOX_OUTLINE_COLOR: Color = Color::srgba(0.2, 0.8, 1.0, 0.9);

/// Color for the box selection fill (drawn as lines on ground).
const BOX_FILL_COLOR: Color = Color::srgba(0.2, 0.7, 1.0, 0.25);

// =============================================================================
// Resource
// =============================================================================

/// Tracks the state of the box selection drag operation.
#[derive(Resource, Default)]
pub struct BoxSelectionState {
    /// Whether a box selection drag is currently in progress.
    pub active: bool,
    /// World position (XZ plane) where the drag started.
    pub start_world_pos: Vec2,
    /// Current world position (XZ plane) of the cursor during drag.
    pub current_world_pos: Vec2,
}

impl BoxSelectionState {
    /// Returns the axis-aligned bounding rectangle as (min, max) in world XZ coords.
    pub fn bounds(&self) -> (Vec2, Vec2) {
        let min_x = self.start_world_pos.x.min(self.current_world_pos.x);
        let max_x = self.start_world_pos.x.max(self.current_world_pos.x);
        let min_y = self.start_world_pos.y.min(self.current_world_pos.y);
        let max_y = self.start_world_pos.y.max(self.current_world_pos.y);
        (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    /// Returns the size of the current selection box.
    pub fn size(&self) -> Vec2 {
        let (min, max) = self.bounds();
        max - min
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Detect Shift+LeftClick press to start box selection.
///
/// Guards:
/// - Only activates when Shift is held
/// - Does not activate during road drawing (Shift is used for angle snap)
pub fn box_selection_start(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorGridPos>,
    draw_state: Res<RoadDrawState>,
    mut box_state: ResMut<BoxSelectionState>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if !shift_held {
        return;
    }

    // Don't start box selection during road drawing (Shift is for angle snap)
    if draw_state.phase != DrawPhase::Idle {
        return;
    }

    if !cursor.valid {
        return;
    }

    box_state.active = true;
    box_state.start_world_pos = cursor.world_pos;
    box_state.current_world_pos = cursor.world_pos;
}

/// Update the box selection rectangle while dragging.
pub fn box_selection_update(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    mut box_state: ResMut<BoxSelectionState>,
) {
    if !box_state.active {
        return;
    }

    // If left button released, the release system will handle it
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    if cursor.valid {
        box_state.current_world_pos = cursor.world_pos;
    }
}

/// On release, select all entities within the box and populate `MultiSelectState`.
pub fn box_selection_release(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    grid: Res<WorldGrid>,
    mut box_state: ResMut<BoxSelectionState>,
    mut multi_select: ResMut<MultiSelectState>,
    mut status: ResMut<StatusMessage>,
) {
    if !box_state.active {
        return;
    }

    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // Finalize the selection
    let size = box_state.size();
    let is_meaningful = size.x >= MIN_BOX_SIZE || size.y >= MIN_BOX_SIZE;

    if is_meaningful {
        let (min, max) = box_state.bounds();

        // Convert world bounds to grid bounds
        let (grid_min_x, grid_min_y) = WorldGrid::world_to_grid(min.x, min.y);
        let (grid_max_x, grid_max_y) = WorldGrid::world_to_grid(max.x, max.y);

        // Clamp to valid grid range
        let gx_start = grid_min_x.max(0) as usize;
        let gy_start = grid_min_y.max(0) as usize;
        let gx_end = (grid_max_x as usize).min(grid.width.saturating_sub(1));
        let gy_end = (grid_max_y as usize).min(grid.height.saturating_sub(1));

        // If Shift is still held, add to existing selection; otherwise replace
        let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        if !shift_held {
            multi_select.clear();
        }

        let mut added = 0usize;

        // Scan all grid cells within the bounds
        for gy in gy_start..=gy_end {
            for gx in gx_start..=gx_end {
                if !grid.in_bounds(gx, gy) {
                    continue;
                }
                let cell = grid.get(gx, gy);

                // Select buildings
                if let Some(entity) = cell.building_id {
                    let item = SelectableItem::Building(entity);
                    if !multi_select.contains(&item) {
                        multi_select.add(item);
                        added += 1;
                    }
                }

                // Select road cells
                if cell.cell_type == CellType::Road {
                    let item = SelectableItem::RoadCell { x: gx, y: gy };
                    if !multi_select.contains(&item) {
                        multi_select.add(item);
                        added += 1;
                    }
                }
            }
        }

        if added > 0 {
            status.set(
                format!(
                    "Box selected {} item(s) ({} total)",
                    added,
                    multi_select.count()
                ),
                false,
            );
        }
    }

    // Always deactivate
    box_state.active = false;
}

/// Cancel box selection on Escape or right-click.
pub fn box_selection_cancel(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut box_state: ResMut<BoxSelectionState>,
) {
    if !box_state.active {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Right) {
        box_state.active = false;
    }
}

/// Draw the translucent selection rectangle on the ground plane using gizmos.
pub fn draw_box_selection_gizmo(box_state: Res<BoxSelectionState>, mut gizmos: Gizmos) {
    if !box_state.active {
        return;
    }

    let (min, max) = box_state.bounds();
    let size = max - min;

    // Skip drawing if too small
    if size.x < 1.0 && size.y < 1.0 {
        return;
    }

    // Ground-plane Y height for the rectangle
    let y = 0.5;

    // Four corners of the rectangle (in 3D, XZ plane)
    let c0 = Vec3::new(min.x, y, min.y);
    let c1 = Vec3::new(max.x, y, min.y);
    let c2 = Vec3::new(max.x, y, max.y);
    let c3 = Vec3::new(min.x, y, max.y);

    // Draw outline
    gizmos.line(c0, c1, BOX_OUTLINE_COLOR);
    gizmos.line(c1, c2, BOX_OUTLINE_COLOR);
    gizmos.line(c2, c3, BOX_OUTLINE_COLOR);
    gizmos.line(c3, c0, BOX_OUTLINE_COLOR);

    // Draw fill lines (horizontal hatching) for visual feedback
    let step = CELL_SIZE;
    let mut z = min.y;
    while z <= max.y {
        gizmos.line(
            Vec3::new(min.x, y, z),
            Vec3::new(max.x, y, z),
            BOX_FILL_COLOR,
        );
        z += step;
    }

    // Vertical fill lines too for grid effect
    let mut x = min.x;
    while x <= max.x {
        gizmos.line(
            Vec3::new(x, y, min.y),
            Vec3::new(x, y, max.y),
            BOX_FILL_COLOR,
        );
        x += step;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct BoxSelectionPlugin;

impl Plugin for BoxSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoxSelectionState>().add_systems(
            Update,
            (
                box_selection_start,
                box_selection_update,
                box_selection_release,
                box_selection_cancel,
                draw_box_selection_gizmo,
            ),
        );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_selection_bounds() {
        let state = BoxSelectionState {
            active: true,
            start_world_pos: Vec2::new(100.0, 200.0),
            current_world_pos: Vec2::new(50.0, 300.0),
        };
        let (min, max) = state.bounds();
        assert_eq!(min, Vec2::new(50.0, 200.0));
        assert_eq!(max, Vec2::new(100.0, 300.0));
    }

    #[test]
    fn test_box_selection_size() {
        let state = BoxSelectionState {
            active: true,
            start_world_pos: Vec2::new(0.0, 0.0),
            current_world_pos: Vec2::new(160.0, 320.0),
        };
        let size = state.size();
        assert_eq!(size, Vec2::new(160.0, 320.0));
    }

    #[test]
    fn test_box_selection_default() {
        let state = BoxSelectionState::default();
        assert!(!state.active);
        assert_eq!(state.start_world_pos, Vec2::ZERO);
        assert_eq!(state.current_world_pos, Vec2::ZERO);
    }
}
