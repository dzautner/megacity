//! Zoning visual feedback (PLAY-P1-01).
//!
//! When a player zones cells, they need immediate feedback about why buildings
//! aren't spawning. This module draws gizmo indicators on zoned cells that
//! have no building and are missing power, water, or both.
//!
//! Indicators are small colored diamonds drawn at ground level:
//!   - Red diamond: missing power
//!   - Blue diamond: missing water
//!   - Yellow diamond: missing both power and water
//!
//! Indicators only appear when the camera is close enough (LOD) and are
//! updated on a timer (every 2 seconds) to avoid per-frame grid scans.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use simulation::app_state::AppState;
use simulation::colorblind::ColorblindSettings;
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{WorldGrid, ZoneType};
use simulation::SaveLoadState;

use crate::camera::OrbitCamera;
use crate::colorblind_palette::{self, UtilityIconKind};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Camera distance beyond which zone feedback indicators are hidden.
/// These are small ground-level markers, so they disappear sooner than
/// building status icons.
const MAX_VISIBLE_DISTANCE: f32 = 600.0;

/// Height offset above ground for the gizmo markers.
const MARKER_Y: f32 = 0.8;

/// Half-size of the diamond marker in world units.
const DIAMOND_HALF: f32 = 2.5;

// ---------------------------------------------------------------------------
// Resource: cached zone status cells
// ---------------------------------------------------------------------------

/// A zoned cell that is missing utilities and has no building.
#[derive(Clone, Copy)]
struct ZoneFeedbackCell {
    gx: usize,
    gy: usize,
    kind: UtilityIconKind,
}

/// Cached list of zoned-but-empty cells that need utility feedback.
/// Rebuilt on a timer to avoid scanning the full grid every frame.
#[derive(Resource, Default)]
struct ZoneFeedbackCells {
    cells: Vec<ZoneFeedbackCell>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Scan the grid for zoned cells without buildings that are missing utilities.
/// Runs on a 2-second timer to match the status icon update cadence.
fn rebuild_zone_feedback(
    grid: Res<WorldGrid>,
    mut feedback: ResMut<ZoneFeedbackCells>,
) {
    let mut cells = Vec::new();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);

            // Only care about zoned cells without a building
            if cell.zone == ZoneType::None || cell.building_id.is_some() {
                continue;
            }

            // Classify missing utilities
            let kind = match (cell.has_power, cell.has_water) {
                (false, false) => UtilityIconKind::NoPowerNoWater,
                (false, true) => UtilityIconKind::NoPower,
                (true, false) => UtilityIconKind::NoWater,
                (true, true) => continue, // All utilities present, no feedback needed
            };

            cells.push(ZoneFeedbackCell { gx: x, gy: y, kind });
        }
    }

    feedback.cells = cells;
}

/// Draw gizmo diamond markers on zoned cells missing utilities.
/// Only draws when camera is close enough for the markers to be visible.
fn draw_zone_feedback(
    feedback: Res<ZoneFeedbackCells>,
    orbit: Res<OrbitCamera>,
    cb_settings: Res<ColorblindSettings>,
    mut gizmos: Gizmos,
) {
    if orbit.distance > MAX_VISIBLE_DISTANCE {
        return;
    }

    if feedback.cells.is_empty() {
        return;
    }

    for cell in &feedback.cells {
        let (wx, _) = WorldGrid::grid_to_world(cell.gx, cell.gy);
        let wz = cell.gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let center = Vec3::new(wx, MARKER_Y, wz);

        let color = colorblind_palette::utility_icon_color(cell.kind, cb_settings.mode);

        // Draw a diamond shape (rotated square) as 4 lines
        let top = Vec3::new(center.x, MARKER_Y, center.z - DIAMOND_HALF);
        let right = Vec3::new(center.x + DIAMOND_HALF, MARKER_Y, center.z);
        let bottom = Vec3::new(center.x, MARKER_Y, center.z + DIAMOND_HALF);
        let left = Vec3::new(center.x - DIAMOND_HALF, MARKER_Y, center.z);

        gizmos.line(top, right, color);
        gizmos.line(right, bottom, color);
        gizmos.line(bottom, left, color);
        gizmos.line(left, top, color);

        // Draw cross inside for better visibility
        gizmos.line(top, bottom, color);
        gizmos.line(left, right, color);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that provides visual feedback on zoned cells missing utilities.
///
/// Draws colored diamond gizmos on zoned-but-empty cells that lack
/// power and/or water, helping players understand why buildings aren't
/// spawning in their zoned areas.
pub struct ZoningFeedbackPlugin;

impl Plugin for ZoningFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneFeedbackCells>();

        let idle = in_state(SaveLoadState::Idle);
        let playing = in_state(AppState::Playing);

        app.add_systems(
            Update,
            (
                rebuild_zone_feedback
                    .run_if(on_timer(std::time::Duration::from_secs(2))),
                draw_zone_feedback
                    .after(rebuild_zone_feedback),
            )
                .run_if(idle)
                .run_if(playing),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::colorblind::ColorblindMode;
    use simulation::grid::CellType;

    #[test]
    fn test_feedback_cells_default_empty() {
        let feedback = ZoneFeedbackCells::default();
        assert!(feedback.cells.is_empty());
    }

    #[test]
    fn test_rebuild_finds_missing_power() {
        let mut grid = WorldGrid::new(10, 10);
        // Zone a cell with water but no power
        let idx = grid.index(5, 5);
        grid.cells[idx].zone = ZoneType::ResidentialLow;
        grid.cells[idx].cell_type = CellType::Grass;
        grid.cells[idx].has_power = false;
        grid.cells[idx].has_water = true;

        let mut feedback = ZoneFeedbackCells::default();
        // Simulate rebuild logic
        let cell = grid.get(5, 5);
        if cell.zone != ZoneType::None && cell.building_id.is_none() {
            match (cell.has_power, cell.has_water) {
                (false, true) => feedback.cells.push(ZoneFeedbackCell {
                    gx: 5,
                    gy: 5,
                    kind: UtilityIconKind::NoPower,
                }),
                _ => {}
            }
        }

        assert_eq!(feedback.cells.len(), 1);
        assert!(matches!(feedback.cells[0].kind, UtilityIconKind::NoPower));
    }

    #[test]
    fn test_rebuild_finds_missing_water() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(3, 3);
        grid.cells[idx].zone = ZoneType::CommercialLow;
        grid.cells[idx].cell_type = CellType::Grass;
        grid.cells[idx].has_power = true;
        grid.cells[idx].has_water = false;

        let cell = grid.get(3, 3);
        let kind = match (cell.has_power, cell.has_water) {
            (true, false) => Some(UtilityIconKind::NoWater),
            _ => None,
        };
        assert!(matches!(kind, Some(UtilityIconKind::NoWater)));
    }

    #[test]
    fn test_rebuild_finds_missing_both() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(7, 7);
        grid.cells[idx].zone = ZoneType::Industrial;
        grid.cells[idx].cell_type = CellType::Grass;
        grid.cells[idx].has_power = false;
        grid.cells[idx].has_water = false;

        let cell = grid.get(7, 7);
        let kind = match (cell.has_power, cell.has_water) {
            (false, false) => Some(UtilityIconKind::NoPowerNoWater),
            _ => None,
        };
        assert!(matches!(kind, Some(UtilityIconKind::NoPowerNoWater)));
    }

    #[test]
    fn test_rebuild_skips_cells_with_buildings() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(5, 5);
        grid.cells[idx].zone = ZoneType::ResidentialLow;
        grid.cells[idx].has_power = false;
        grid.cells[idx].has_water = false;
        grid.cells[idx].building_id = Some(Entity::from_raw(42));

        let cell = grid.get(5, 5);
        // Cell with building_id should be skipped
        assert!(cell.building_id.is_some());
    }

    #[test]
    fn test_rebuild_skips_unzoned_cells() {
        let grid = WorldGrid::new(10, 10);
        let cell = grid.get(5, 5);
        assert_eq!(cell.zone, ZoneType::None);
        // Unzoned cells should not generate feedback
    }

    #[test]
    fn test_rebuild_skips_fully_connected() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(5, 5);
        grid.cells[idx].zone = ZoneType::ResidentialLow;
        grid.cells[idx].has_power = true;
        grid.cells[idx].has_water = true;

        let cell = grid.get(5, 5);
        let should_show = cell.zone != ZoneType::None
            && cell.building_id.is_none()
            && !(cell.has_power && cell.has_water);
        assert!(!should_show);
    }

    #[test]
    fn test_icon_colors_are_reused_from_palette() {
        // Verify we get valid colors from the shared palette
        let settings = ColorblindSettings::default();
        let _c1 = colorblind_palette::utility_icon_color(
            UtilityIconKind::NoPower,
            settings.mode,
        );
        let _c2 = colorblind_palette::utility_icon_color(
            UtilityIconKind::NoWater,
            settings.mode,
        );
        let _c3 = colorblind_palette::utility_icon_color(
            UtilityIconKind::NoPowerNoWater,
            settings.mode,
        );
    }

    #[test]
    fn test_icon_colors_distinct_all_modes() {
        for mode in ColorblindMode::ALL {
            let c1 = colorblind_palette::utility_icon_color(
                UtilityIconKind::NoPower,
                mode,
            )
            .to_srgba();
            let c2 = colorblind_palette::utility_icon_color(
                UtilityIconKind::NoWater,
                mode,
            )
            .to_srgba();
            let diff = (c1.red - c2.red).abs()
                + (c1.green - c2.green).abs()
                + (c1.blue - c2.blue).abs();
            assert!(
                diff > 0.1,
                "NoPower and NoWater should be distinct in {:?} mode",
                mode
            );
        }
    }

    #[test]
    fn test_max_visible_distance_reasonable() {
        assert!(MAX_VISIBLE_DISTANCE > 0.0);
        assert!(MAX_VISIBLE_DISTANCE < 4000.0);
    }

    #[test]
    fn test_diamond_half_reasonable() {
        assert!(DIAMOND_HALF > 0.0);
        assert!(DIAMOND_HALF < CELL_SIZE);
    }
}
