use bevy::prelude::*;

use simulation::app_state::AppState;
use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::urban_growth_boundary::UrbanGrowthBoundary;

use crate::input::{ActiveTool, CursorGridPos};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Cost per cell to zone (all zone types share the same cost).
pub const ZONE_COST_PER_CELL: f64 = 5.0;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Configurable zone brush size: 1x1, 3x3, or 5x5.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZoneBrushSize {
    /// Half-extent: 0 => 1x1, 1 => 3x3, 2 => 5x5.
    pub half_extent: i32,
}

impl ZoneBrushSize {
    /// Human-readable label, e.g. "1x1".
    pub fn label(self) -> &'static str {
        match self.half_extent {
            0 => "1x1",
            1 => "3x3",
            2 => "5x5",
            _ => "?",
        }
    }

    /// Cycle to the next size: 1 -> 3 -> 5 -> 1.
    pub fn cycle_up(&mut self) {
        self.half_extent = (self.half_extent + 1) % 3;
    }

    /// Cycle to the previous size: 1 -> 5 -> 3 -> 1.
    pub fn cycle_down(&mut self) {
        self.half_extent = (self.half_extent + 2) % 3;
    }
}

// ---------------------------------------------------------------------------
// Public helpers (used by both rendering gizmo system and UI cost display)
// ---------------------------------------------------------------------------

/// Returns the zone preview color for a given zone type (semi-transparent).
pub fn zone_color(zone: ZoneType) -> Color {
    match zone {
        ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
            Color::srgba(0.2, 0.75, 0.2, 0.45)
        }
        ZoneType::CommercialLow | ZoneType::CommercialHigh => Color::srgba(0.2, 0.35, 0.85, 0.45),
        ZoneType::Industrial => Color::srgba(0.85, 0.75, 0.1, 0.45),
        ZoneType::Office => Color::srgba(0.6, 0.5, 0.85, 0.45),
        ZoneType::MixedUse => Color::srgba(0.65, 0.55, 0.3, 0.45),
        ZoneType::None => Color::srgba(0.5, 0.5, 0.5, 0.3),
    }
}

/// Invalid cell color (red).
pub const INVALID_COLOR: Color = Color::srgba(0.9, 0.15, 0.1, 0.45);

/// Check if a cell is valid for the given zone type.
pub fn is_cell_valid_for_zone(
    grid: &WorldGrid,
    x: usize,
    y: usize,
    zone: ZoneType,
    ugb: &UrbanGrowthBoundary,
) -> bool {
    if !grid.in_bounds(x, y) {
        return false;
    }
    let cell = grid.get(x, y);
    // Must be grass (not water, not road)
    if cell.cell_type != CellType::Grass {
        return false;
    }
    // Must not already be this zone type
    if cell.zone == zone {
        return false;
    }
    // Must be within urban growth boundary
    if !ugb.allows_zoning(x, y) {
        return false;
    }
    // Must be adjacent to a road (cardinal neighbors only, matching try_zone in input.rs)
    let (n4, n4c) = grid.neighbors4(x, y);
    let has_road = n4[..n4c]
        .iter()
        .any(|(nx, ny)| grid.get(*nx, *ny).cell_type == CellType::Road);
    if !has_road {
        return false;
    }
    true
}

/// Iterate cells in the brush area centered at (cx, cy) with the given half_extent.
/// Returns (grid_x, grid_y) pairs that are within grid bounds.
pub fn brush_cells(cx: i32, cy: i32, half: i32, grid: &WorldGrid) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    for dy in -half..=half {
        for dx in -half..=half {
            let gx = cx + dx;
            let gy = cy + dy;
            if gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize) {
                cells.push((gx as usize, gy as usize));
            }
        }
    }
    cells
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Cycle brush size with `[` and `]` keys (only when a zone tool is active).
pub fn cycle_brush_size(
    keys: Res<ButtonInput<KeyCode>>,
    tool: Res<ActiveTool>,
    mut brush: ResMut<ZoneBrushSize>,
) {
    if tool.zone_type().is_none() {
        return;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        brush.cycle_up();
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        brush.cycle_down();
    }
}

/// Draw gizmo rectangles for each cell in the zone brush preview.
/// Valid cells get the zone color, invalid cells get red.
pub fn draw_zone_brush_preview(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    brush: Res<ZoneBrushSize>,
    grid: Res<WorldGrid>,
    ugb: Res<UrbanGrowthBoundary>,
    mut gizmos: Gizmos,
) {
    let Some(zone) = tool.zone_type() else {
        return;
    };
    if !cursor.valid {
        return;
    }

    let cells = brush_cells(cursor.grid_x, cursor.grid_y, brush.half_extent, &grid);
    let valid_color = zone_color(zone);
    let y = 0.6; // slightly above ground

    for (gx, gy) in &cells {
        let valid = is_cell_valid_for_zone(&grid, *gx, *gy, zone, &ugb);
        let color = if valid { valid_color } else { INVALID_COLOR };

        let (wx, _) = WorldGrid::grid_to_world(*gx, *gy);
        let wz = *gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

        // Draw a filled rectangle using gizmo lines
        let center = Vec3::new(wx, y, wz);
        let half = CELL_SIZE * 0.48; // slightly smaller than cell for visual gap
        let corners = [
            Vec3::new(center.x - half, y, center.z - half),
            Vec3::new(center.x + half, y, center.z - half),
            Vec3::new(center.x + half, y, center.z + half),
            Vec3::new(center.x - half, y, center.z + half),
        ];
        // Draw quad outline
        for i in 0..4 {
            gizmos.line(corners[i], corners[(i + 1) % 4], color);
        }
        // Draw diagonals for fill effect
        gizmos.line(corners[0], corners[2], color);
        gizmos.line(corners[1], corners[3], color);
    }

    // Draw brush boundary outline in white when brush > 1x1
    if brush.half_extent > 0 {
        let cx_w = cursor.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let cz_w = cursor.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let extent = (brush.half_extent as f32 + 0.5) * CELL_SIZE;
        let outline_color = Color::srgba(1.0, 1.0, 1.0, 0.6);
        let oy = y + 0.1;
        let tl = Vec3::new(cx_w - extent, oy, cz_w - extent);
        let tr = Vec3::new(cx_w + extent, oy, cz_w - extent);
        let br = Vec3::new(cx_w + extent, oy, cz_w + extent);
        let bl = Vec3::new(cx_w - extent, oy, cz_w + extent);
        gizmos.line(tl, tr, outline_color);
        gizmos.line(tr, br, outline_color);
        gizmos.line(br, bl, outline_color);
        gizmos.line(bl, tl, outline_color);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ZoneBrushPreviewPlugin;

impl Plugin for ZoneBrushPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneBrushSize>()
            .add_systems(
                Update,
                (cycle_brush_size, draw_zone_brush_preview)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_size_default() {
        let brush = ZoneBrushSize::default();
        assert_eq!(brush.half_extent, 0);
        assert_eq!(brush.label(), "1x1");
    }

    #[test]
    fn test_brush_size_cycle_up() {
        let mut brush = ZoneBrushSize::default();
        brush.cycle_up();
        assert_eq!(brush.half_extent, 1);
        assert_eq!(brush.label(), "3x3");
        brush.cycle_up();
        assert_eq!(brush.half_extent, 2);
        assert_eq!(brush.label(), "5x5");
        brush.cycle_up();
        assert_eq!(brush.half_extent, 0);
        assert_eq!(brush.label(), "1x1");
    }

    #[test]
    fn test_brush_size_cycle_down() {
        let mut brush = ZoneBrushSize::default();
        brush.cycle_down();
        assert_eq!(brush.half_extent, 2);
        assert_eq!(brush.label(), "5x5");
        brush.cycle_down();
        assert_eq!(brush.half_extent, 1);
        assert_eq!(brush.label(), "3x3");
        brush.cycle_down();
        assert_eq!(brush.half_extent, 0);
        assert_eq!(brush.label(), "1x1");
    }

    #[test]
    fn test_brush_cells_1x1() {
        let grid = WorldGrid::new(256, 256);
        let cells = brush_cells(10, 10, 0, &grid);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0], (10, 10));
    }

    #[test]
    fn test_brush_cells_3x3() {
        let grid = WorldGrid::new(256, 256);
        let cells = brush_cells(10, 10, 1, &grid);
        assert_eq!(cells.len(), 9);
    }

    #[test]
    fn test_brush_cells_5x5() {
        let grid = WorldGrid::new(256, 256);
        let cells = brush_cells(10, 10, 2, &grid);
        assert_eq!(cells.len(), 25);
    }

    #[test]
    fn test_brush_cells_at_corner() {
        let grid = WorldGrid::new(256, 256);
        // At corner (0,0) with 3x3 brush, should only get 4 cells (clipped)
        let cells = brush_cells(0, 0, 1, &grid);
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn test_brush_cells_at_edge() {
        let grid = WorldGrid::new(256, 256);
        // At edge (255, 128) with 3x3 brush
        let cells = brush_cells(255, 128, 1, &grid);
        assert_eq!(cells.len(), 6); // 2 columns x 3 rows
    }

    #[test]
    fn test_zone_color_returns_values() {
        // Just verify we get non-panicking colors for each zone type
        let _c1 = zone_color(ZoneType::ResidentialLow);
        let _c2 = zone_color(ZoneType::CommercialLow);
        let _c3 = zone_color(ZoneType::Industrial);
        let _c4 = zone_color(ZoneType::Office);
        let _c5 = zone_color(ZoneType::MixedUse);
    }

    #[test]
    fn test_is_cell_valid_for_zone_water() {
        use simulation::grid::CellType;
        let mut grid = WorldGrid::new(10, 10);
        // Set cell to water
        let idx = grid.index(5, 5);
        grid.cells[idx].cell_type = CellType::Water;
        let ugb = UrbanGrowthBoundary::default();
        assert!(!is_cell_valid_for_zone(
            &grid,
            5,
            5,
            ZoneType::ResidentialLow,
            &ugb
        ));
    }

    #[test]
    fn test_is_cell_valid_for_zone_no_road() {
        let grid = WorldGrid::new(10, 10);
        let ugb = UrbanGrowthBoundary::default();
        // No roads nearby, should be invalid
        assert!(!is_cell_valid_for_zone(
            &grid,
            5,
            5,
            ZoneType::ResidentialLow,
            &ugb
        ));
    }

    #[test]
    fn test_is_cell_valid_for_zone_with_road() {
        use simulation::grid::CellType;
        let mut grid = WorldGrid::new(10, 10);
        // Place a road adjacent to (5,5)
        let idx = grid.index(5, 4);
        grid.cells[idx].cell_type = CellType::Road;
        let ugb = UrbanGrowthBoundary::default();
        assert!(is_cell_valid_for_zone(
            &grid,
            5,
            5,
            ZoneType::ResidentialLow,
            &ugb
        ));
    }

    #[test]
    fn test_is_cell_valid_for_zone_already_zoned() {
        use simulation::grid::CellType;
        let mut grid = WorldGrid::new(10, 10);
        // Place a road and zone the cell
        let road_idx = grid.index(5, 4);
        grid.cells[road_idx].cell_type = CellType::Road;
        let zone_idx = grid.index(5, 5);
        grid.cells[zone_idx].zone = ZoneType::ResidentialLow;
        let ugb = UrbanGrowthBoundary::default();
        // Same zone type should be invalid
        assert!(!is_cell_valid_for_zone(
            &grid,
            5,
            5,
            ZoneType::ResidentialLow,
            &ugb
        ));
        // Different zone type should be valid
        assert!(is_cell_valid_for_zone(
            &grid,
            5,
            5,
            ZoneType::CommercialLow,
            &ugb
        ));
    }
}
