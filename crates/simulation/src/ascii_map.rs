//! ASCII map rendering for the city grid.
//!
//! Provides two views:
//! - **Overview** (64x64): each character represents a 4x4 block of grid cells
//! - **Detail** (full resolution): 1 character per grid cell, cropped to content
//!
//! Maps are built on-demand from `&WorldGrid` — no per-frame systems needed.

use bevy::prelude::*;

use crate::grid::{Cell, CellType, RoadType, WorldGrid, ZoneType};

/// Empty plugin — ASCII maps are generated on-demand, no systems required.
pub struct AsciiMapPlugin;

impl Plugin for AsciiMapPlugin {
    fn build(&self, _app: &mut App) {}
}

// -----------------------------------------------------------------------
// Character encoding
// -----------------------------------------------------------------------

/// Convert a single grid cell to its ASCII character representation.
///
/// Priority: Water > Road > Building > Zone > Grass.
///
/// Zone characters without a building use the "base" char defined by the spec.
/// When a building is present on a zoned cell, the uppercase version is used.
pub fn cell_to_char(cell: &Cell) -> char {
    // Water takes top priority
    if cell.cell_type == CellType::Water {
        return '~';
    }

    // Roads
    if cell.cell_type == CellType::Road {
        return road_char(cell.road_type);
    }

    // Building present — use zone-based uppercase if zoned, else generic `B`
    if cell.building_id.is_some() {
        return building_char(cell.zone);
    }

    // Zone painted but no building yet
    if cell.zone != ZoneType::None {
        return zone_char_no_building(cell.zone);
    }

    // Default: grass / empty
    '.'
}

fn road_char(road_type: RoadType) -> char {
    match road_type {
        RoadType::Local | RoadType::OneWay | RoadType::Path => '#',
        RoadType::Avenue => '=',
        RoadType::Boulevard => 'H',
        RoadType::Highway => '%',
    }
}

/// Character for a zone that has NO building on it.
fn zone_char_no_building(zone: ZoneType) -> char {
    match zone {
        ZoneType::None => '.',
        ZoneType::ResidentialLow => 'r',
        ZoneType::ResidentialMedium => 'm',
        ZoneType::ResidentialHigh => 'R',
        ZoneType::CommercialLow => 'c',
        ZoneType::CommercialHigh => 'C',
        ZoneType::Industrial => 'I',
        ZoneType::Office => 'O',
        ZoneType::MixedUse => 'M',
    }
}

/// Character for a cell that has a building_id.
/// If the cell is zoned, use the uppercase zone char.
/// If unzoned (service/utility building), use `B`.
fn building_char(zone: ZoneType) -> char {
    match zone {
        ZoneType::None => 'B',
        ZoneType::ResidentialLow => 'R',
        ZoneType::ResidentialMedium => 'M',
        ZoneType::ResidentialHigh => 'R',
        ZoneType::CommercialLow => 'C',
        ZoneType::CommercialHigh => 'C',
        ZoneType::Industrial => 'I',
        ZoneType::Office => 'O',
        ZoneType::MixedUse => 'M',
    }
}

// -----------------------------------------------------------------------
// Overview map (64x64, each char = 4x4 block)
// -----------------------------------------------------------------------

/// Numeric priority for tie-breaking in 4x4 overview blocks.
/// Higher value = wins the block.
fn char_priority(ch: char) -> u8 {
    match ch {
        '~' => 5,                                                  // Water
        '#' | '=' | 'H' | '%' => 4,                               // Roads
        'B' | 'S' | 'U' | 'R' | 'C' | 'I' | 'O' | 'M' => 3,    // Buildings
        'r' | 'm' | 'c' => 2,                                     // Zones (no building)
        '.' => 0,                                                  // Grass
        _ => 1,
    }
}

/// Build a 64x64 overview map of the full 256x256 grid.
///
/// Each character represents the dominant type in a 4x4 cell block.
/// Includes row/column coordinate headers and a legend.
pub fn build_overview_map(grid: &WorldGrid) -> String {
    const OVERVIEW: usize = 64;
    const BLOCK: usize = 4;

    let mut lines: Vec<String> = Vec::with_capacity(OVERVIEW + 8);

    // Column header — show real grid coordinate every 8 overview columns
    let mut col_header = String::from("       "); // left margin for row labels
    for col in (0..OVERVIEW).step_by(8) {
        let real_col = col * BLOCK;
        let label = format!("{real_col:<8}");
        col_header.push_str(&label);
    }
    lines.push(col_header.trim_end().to_string());

    // Grid rows
    for row in 0..OVERVIEW {
        let real_row = row * BLOCK;
        // Row label every 4 overview rows
        let label = if row.is_multiple_of(4) {
            format!("{real_row:>4} | ")
        } else {
            "     | ".to_string()
        };

        let mut line = label;
        for col in 0..OVERVIEW {
            let gx_start = col * BLOCK;
            let gy_start = row * BLOCK;
            let ch = dominant_char(grid, gx_start, gy_start, BLOCK);
            line.push(ch);
        }
        lines.push(line);
    }

    // Legend
    lines.push(String::new());
    append_legend(&mut lines);

    lines.join("\n")
}

/// Find the dominant character in a BLOCK x BLOCK region starting at (gx, gy).
fn dominant_char(grid: &WorldGrid, gx: usize, gy: usize, block: usize) -> char {
    let mut best_char = '.';
    let mut best_priority = 0u8;

    for dy in 0..block {
        for dx in 0..block {
            let x = gx + dx;
            let y = gy + dy;
            if x < grid.width && y < grid.height {
                let ch = cell_to_char(grid.get(x, y));
                let pri = char_priority(ch);
                if pri > best_priority {
                    best_priority = pri;
                    best_char = ch;
                }
            }
        }
    }
    best_char
}

// -----------------------------------------------------------------------
// Detail map (full resolution, cropped to content)
// -----------------------------------------------------------------------

/// Build a full-resolution detail map, cropped to the bounding box of
/// non-empty cells plus `margin` cells of padding on each side.
///
/// Returns a descriptive message if the grid is entirely empty.
pub fn build_detail_map(grid: &WorldGrid, margin: usize) -> String {
    // Find bounding box of non-empty cells
    let mut min_x = grid.width;
    let mut max_x: usize = 0;
    let mut min_y = grid.height;
    let mut max_y: usize = 0;
    let mut has_content = false;

    for y in 0..grid.height {
        for x in 0..grid.width {
            let ch = cell_to_char(grid.get(x, y));
            if ch != '.' {
                has_content = true;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    if !has_content {
        return "(empty grid — no roads, zones, or buildings)".to_string();
    }

    // Apply margin, clamped to grid bounds
    let x0 = min_x.saturating_sub(margin);
    let y0 = min_y.saturating_sub(margin);
    let x1 = (max_x + margin).min(grid.width - 1);
    let y1 = (max_y + margin).min(grid.height - 1);

    let width = x1 - x0 + 1;

    let mut lines: Vec<String> = Vec::with_capacity((y1 - y0 + 1) + 4);

    // Column header
    let col_header = build_col_header(x0, width);
    lines.push(col_header);

    // Grid rows
    for y in y0..=y1 {
        let label = format!("{y:>4} | ");
        let mut line = label;
        for x in x0..=x1 {
            line.push(cell_to_char(grid.get(x, y)));
        }
        lines.push(line);
    }

    // Legend
    lines.push(String::new());
    append_legend(&mut lines);

    lines.join("\n")
}

fn build_col_header(x0: usize, width: usize) -> String {
    let margin_str = "       "; // matches row label width "XXXX | "
    let interval = if width > 40 { 10 } else { 5 };

    let mut header = String::from(margin_str);
    let mut col = 0;
    while col < width {
        let real_x = x0 + col;
        if real_x.is_multiple_of(interval) || col == 0 {
            let label = format!("{real_x}");
            header.push_str(&label);
            col += label.len();
        } else {
            header.push(' ');
            col += 1;
        }
    }
    header.trim_end().to_string()
}

fn append_legend(lines: &mut Vec<String>) {
    lines.push("Legend:".to_string());
    lines.push("  .=Grass  ~=Water  #=Road(Local)  ==Road(Avenue)".to_string());
    lines.push("  H=Road(Boulevard)  %=Road(Highway)".to_string());
    lines.push(
        "  r=ResLow  m=ResMed  R=ResHigh  c=ComLow  C=ComHigh".to_string(),
    );
    lines.push(
        "  I=Industrial  O=Office  M=MixedUse  B=Building(unzoned)".to_string(),
    );
    lines.push("  Uppercase zone = zone with building".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_to_char_grass() {
        let cell = Cell::default();
        assert_eq!(cell_to_char(&cell), '.');
    }

    #[test]
    fn test_cell_to_char_water() {
        let mut cell = Cell::default();
        cell.cell_type = CellType::Water;
        assert_eq!(cell_to_char(&cell), '~');
    }

    #[test]
    fn test_cell_to_char_roads() {
        let mut cell = Cell::default();
        cell.cell_type = CellType::Road;

        cell.road_type = RoadType::Local;
        assert_eq!(cell_to_char(&cell), '#');

        cell.road_type = RoadType::Avenue;
        assert_eq!(cell_to_char(&cell), '=');

        cell.road_type = RoadType::Boulevard;
        assert_eq!(cell_to_char(&cell), 'H');

        cell.road_type = RoadType::Highway;
        assert_eq!(cell_to_char(&cell), '%');
    }

    #[test]
    fn test_cell_to_char_zones_no_building() {
        let mut cell = Cell::default();

        cell.zone = ZoneType::ResidentialLow;
        assert_eq!(cell_to_char(&cell), 'r');

        cell.zone = ZoneType::ResidentialMedium;
        assert_eq!(cell_to_char(&cell), 'm');

        cell.zone = ZoneType::ResidentialHigh;
        assert_eq!(cell_to_char(&cell), 'R');

        cell.zone = ZoneType::CommercialLow;
        assert_eq!(cell_to_char(&cell), 'c');

        cell.zone = ZoneType::CommercialHigh;
        assert_eq!(cell_to_char(&cell), 'C');

        cell.zone = ZoneType::Industrial;
        assert_eq!(cell_to_char(&cell), 'I');

        cell.zone = ZoneType::Office;
        assert_eq!(cell_to_char(&cell), 'O');

        cell.zone = ZoneType::MixedUse;
        assert_eq!(cell_to_char(&cell), 'M');
    }

    #[test]
    fn test_cell_to_char_building_with_zone() {
        let mut cell = Cell::default();
        cell.building_id = Some(Entity::from_raw(1));
        cell.zone = ZoneType::ResidentialLow;
        assert_eq!(cell_to_char(&cell), 'R');

        cell.zone = ZoneType::CommercialLow;
        assert_eq!(cell_to_char(&cell), 'C');
    }

    #[test]
    fn test_cell_to_char_building_no_zone() {
        let mut cell = Cell::default();
        cell.building_id = Some(Entity::from_raw(1));
        cell.zone = ZoneType::None;
        assert_eq!(cell_to_char(&cell), 'B');
    }

    #[test]
    fn test_water_beats_everything() {
        let mut cell = Cell::default();
        cell.cell_type = CellType::Water;
        cell.zone = ZoneType::Industrial;
        cell.building_id = Some(Entity::from_raw(1));
        assert_eq!(cell_to_char(&cell), '~');
    }
}
