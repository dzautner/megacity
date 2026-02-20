//! Enhanced network visualization for power/water overlays.
//!
//! When the power or water overlay is active, this module draws:
//! - Pulsing glow circles around source buildings
//! - Animated pulse lines along network road paths
//! - Capacity fill bars on each source building
//! - Disconnection indicators (red X) on uncovered road cells
//!
//! Cell color-coding by source is handled via the terrain overlay system
//! using `NetworkVizData::power_source_color` / `water_source_color`.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::CellType;
use simulation::network_viz::{NetworkVizData, SourceInfo};
use simulation::utilities::UtilityType;

use crate::overlay::{OverlayMode, OverlayState};

/// Plugin for enhanced network visualization gizmos.
pub struct NetworkVizPlugin;

impl Plugin for NetworkVizPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_source_pulsing_glow,
                draw_network_pulse_lines,
                draw_capacity_fill_bars,
                draw_disconnection_indicators,
            ),
        );
    }
}

/// Convert grid coordinates to world position (center of cell).
fn grid_to_world(gx: usize, gy: usize) -> Vec3 {
    Vec3::new(
        gx as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        0.5, // slightly above ground
        gy as f32 * CELL_SIZE + CELL_SIZE * 0.5,
    )
}

/// Get a source color from its index, with alpha.
fn source_color(info: &SourceInfo, alpha: f32) -> Color {
    let hues = info_hue(info);
    Color::srgba(hues[0], hues[1], hues[2], alpha)
}

/// Get the RGB hue for a source based on its color index.
fn info_hue(info: &SourceInfo) -> [f32; 3] {
    const HUES: [[f32; 3]; 12] = [
        [0.30, 0.55, 0.95],
        [0.95, 0.55, 0.20],
        [0.30, 0.80, 0.45],
        [0.85, 0.30, 0.40],
        [0.60, 0.40, 0.85],
        [0.20, 0.80, 0.75],
        [0.90, 0.80, 0.20],
        [0.70, 0.35, 0.20],
        [0.55, 0.75, 0.30],
        [0.80, 0.45, 0.70],
        [0.35, 0.65, 0.55],
        [0.95, 0.65, 0.50],
    ];
    HUES[info.color_index % HUES.len()]
}

/// Draw pulsing glow circles around each source building.
fn draw_source_pulsing_glow(
    overlay: Res<OverlayState>,
    viz: Res<NetworkVizData>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let sources = match overlay.mode {
        OverlayMode::Power => &viz.power_sources,
        OverlayMode::Water => &viz.water_sources,
        _ => return,
    };

    let t = time.elapsed_secs();

    for info in sources {
        let pos = grid_to_world(info.grid_x, info.grid_y);

        // Pulsing effect: oscillate radius and alpha
        let pulse = (t * 2.5).sin() * 0.5 + 0.5; // 0..1 oscillation
        let inner_radius = CELL_SIZE * 0.6;
        let outer_radius = CELL_SIZE * (1.0 + pulse * 0.8);
        let alpha = 0.4 + pulse * 0.3;

        let color = source_color(info, alpha);

        // Inner bright circle
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            inner_radius,
            color,
        );

        // Outer pulsing circle
        let outer_color = source_color(info, alpha * 0.5);
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            outer_radius,
            outer_color,
        );

        // Range indicator circle (faint)
        let range_radius = info.effective_range as f32 * CELL_SIZE;
        let range_color = source_color(info, 0.08 + pulse * 0.04);
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            range_radius,
            range_color,
        );
    }
}

/// Draw animated pulse lines along network road paths.
///
/// Shows flowing "energy" along roads radiating from each source.
/// The animation uses distance-based phase offset to create a wave effect.
#[allow(clippy::too_many_arguments)]
fn draw_network_pulse_lines(
    overlay: Res<OverlayState>,
    viz: Res<NetworkVizData>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let (road_cells, sources) = match overlay.mode {
        OverlayMode::Power => (&viz.power_road_cells, &viz.power_sources),
        OverlayMode::Water => (&viz.water_road_cells, &viz.water_sources),
        _ => return,
    };

    if sources.is_empty() {
        return;
    }

    let t = time.elapsed_secs();

    // Draw pulse dots on road cells -- animate by distance from source
    // We sample every Nth road cell to avoid overdraw
    let step = if road_cells.len() > 2000 { 3 } else { 1 };

    for (i, &(x, y, dist, src_idx)) in road_cells.iter().enumerate() {
        if i % step != 0 {
            continue;
        }
        if (src_idx as usize) >= sources.len() {
            continue;
        }

        let info = &sources[src_idx as usize];

        // Wave animation: pulse travels outward from source
        let phase = dist as f32 * 0.3 - t * 4.0;
        let wave = (phase.sin() * 0.5 + 0.5).powi(3); // sharper peaks

        if wave < 0.15 {
            continue; // skip dim cells to reduce draw calls
        }

        let pos = grid_to_world(x, y);
        let alpha = wave * 0.6;
        let color = source_color(info, alpha);
        let size = CELL_SIZE * 0.3 * (0.5 + wave * 0.5);

        // Draw a small bright dot at the road cell
        gizmos.circle(
            Isometry3d::new(
                pos + Vec3::Y * 0.2,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            size,
            color,
        );
    }
}

/// Draw capacity fill bars on each source building.
///
/// Shows a horizontal bar above the source building indicating how much
/// of its range/coverage is being utilized.
fn draw_capacity_fill_bars(
    overlay: Res<OverlayState>,
    viz: Res<NetworkVizData>,
    mut gizmos: Gizmos,
) {
    let sources = match overlay.mode {
        OverlayMode::Power => &viz.power_sources,
        OverlayMode::Water => &viz.water_sources,
        _ => return,
    };

    for info in sources {
        let pos = grid_to_world(info.grid_x, info.grid_y);
        let bar_y = 3.0; // height above ground
        let bar_width = CELL_SIZE * 1.8;
        let bar_height = CELL_SIZE * 0.25;

        // Fill ratio based on coverage
        let fill = if info.max_coverage > 0 {
            (info.cells_covered as f32 / info.max_coverage as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let bar_center = Vec3::new(pos.x, bar_y, pos.z);

        // Background bar (dark)
        let bg_start = Vec3::new(
            bar_center.x - bar_width * 0.5,
            bar_y,
            bar_center.z - bar_height * 0.5,
        );
        let bg_end = Vec3::new(
            bar_center.x + bar_width * 0.5,
            bar_y,
            bar_center.z + bar_height * 0.5,
        );
        let bg_color = Color::srgba(0.15, 0.15, 0.15, 0.7);

        // Draw background rectangle using lines
        gizmos.line(
            Vec3::new(bg_start.x, bar_y, bg_start.z),
            Vec3::new(bg_end.x, bar_y, bg_start.z),
            bg_color,
        );
        gizmos.line(
            Vec3::new(bg_end.x, bar_y, bg_start.z),
            Vec3::new(bg_end.x, bar_y, bg_end.z),
            bg_color,
        );
        gizmos.line(
            Vec3::new(bg_end.x, bar_y, bg_end.z),
            Vec3::new(bg_start.x, bar_y, bg_end.z),
            bg_color,
        );
        gizmos.line(
            Vec3::new(bg_start.x, bar_y, bg_end.z),
            Vec3::new(bg_start.x, bar_y, bg_start.z),
            bg_color,
        );

        // Fill bar
        let fill_end_x = bg_start.x + bar_width * fill;
        let fill_color = fill_bar_color(fill, info.utility_type);

        // Draw filled portion with multiple horizontal lines for visibility
        let line_count = 3;
        for i in 0..line_count {
            let frac = (i as f32 + 0.5) / line_count as f32;
            let z = bg_start.z + (bg_end.z - bg_start.z) * frac;
            gizmos.line(
                Vec3::new(bg_start.x, bar_y, z),
                Vec3::new(fill_end_x, bar_y, z),
                fill_color,
            );
        }

        // Label: utility type name above bar
        // (Gizmos don't support text, but we draw a small icon indicator)
        let icon_pos = Vec3::new(bar_center.x, bar_y + 1.0, bar_center.z);
        let icon_color = source_color(info, 0.9);
        let icon_size = CELL_SIZE * 0.3;

        if info.utility_type.is_power() {
            // Lightning bolt shape (simplified as a zig-zag)
            gizmos.line(
                icon_pos + Vec3::new(-icon_size * 0.2, icon_size * 0.5, 0.0),
                icon_pos + Vec3::new(icon_size * 0.1, 0.0, 0.0),
                icon_color,
            );
            gizmos.line(
                icon_pos + Vec3::new(icon_size * 0.1, 0.0, 0.0),
                icon_pos + Vec3::new(-icon_size * 0.1, -0.1, 0.0),
                icon_color,
            );
            gizmos.line(
                icon_pos + Vec3::new(-icon_size * 0.1, -0.1, 0.0),
                icon_pos + Vec3::new(icon_size * 0.2, -icon_size * 0.5, 0.0),
                icon_color,
            );
        } else {
            // Water drop shape (simplified as a V + circle)
            gizmos.line(
                icon_pos + Vec3::new(0.0, icon_size * 0.4, 0.0),
                icon_pos + Vec3::new(-icon_size * 0.2, -icon_size * 0.1, 0.0),
                icon_color,
            );
            gizmos.line(
                icon_pos + Vec3::new(0.0, icon_size * 0.4, 0.0),
                icon_pos + Vec3::new(icon_size * 0.2, -icon_size * 0.1, 0.0),
                icon_color,
            );
        }
    }
}

/// Color for capacity fill bar based on utilization ratio.
fn fill_bar_color(fill: f32, utility_type: UtilityType) -> Color {
    // Green when low utilization, yellow at medium, red when near capacity
    let (r, g, b) = if utility_type.is_power() {
        if fill < 0.5 {
            (0.3, 0.85, 0.3) // green
        } else if fill < 0.8 {
            (0.9, 0.8, 0.2) // yellow
        } else {
            (0.9, 0.25, 0.2) // red
        }
    } else {
        // Water uses blue tones
        if fill < 0.5 {
            (0.2, 0.6, 0.9) // light blue
        } else if fill < 0.8 {
            (0.2, 0.85, 0.85) // cyan
        } else {
            (0.9, 0.25, 0.2) // red
        }
    };
    Color::srgba(r, g, b, 0.85)
}

/// Draw disconnection indicators on cells that lack coverage.
///
/// Shows small red markers on road cells near covered areas that are
/// themselves uncovered, highlighting network gaps.
fn draw_disconnection_indicators(
    overlay: Res<OverlayState>,
    viz: Res<NetworkVizData>,
    grid: Res<simulation::grid::WorldGrid>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let (cell_source, sources) = match overlay.mode {
        OverlayMode::Power => (&viz.power_cell_source, &viz.power_sources),
        OverlayMode::Water => (&viz.water_cell_source, &viz.water_sources),
        _ => return,
    };

    if sources.is_empty() {
        return;
    }

    let t = time.elapsed_secs();
    let blink = ((t * 3.0).sin() * 0.5 + 0.5) > 0.3; // blinking effect
    if !blink {
        return;
    }

    // Find road cells that are NOT covered but have a neighbor that IS covered.
    // These are disconnection boundary cells.
    let width = grid.width;
    let height = grid.height;

    // Sample to limit performance impact
    let mut count = 0u32;
    let max_indicators = 200;

    for y in 0..height {
        if count >= max_indicators {
            break;
        }
        for x in 0..width {
            if count >= max_indicators {
                break;
            }
            let idx = y * width + x;
            let cell = grid.get(x, y);

            // Only show on road cells that are NOT covered
            if cell.cell_type != CellType::Road {
                continue;
            }
            if cell_source[idx] != u16::MAX {
                continue;
            }

            // Check if any neighbor IS covered (this is a disconnection boundary)
            let (neighbors, ncount) = grid.neighbors4(x, y);
            let has_covered_neighbor = neighbors[..ncount]
                .iter()
                .any(|&(nx, ny)| cell_source[ny * width + nx] != u16::MAX);

            if !has_covered_neighbor {
                continue;
            }

            count += 1;
            let pos = grid_to_world(x, y);
            let marker_size = CELL_SIZE * 0.25;
            let color = Color::srgba(0.95, 0.15, 0.15, 0.8);

            // Draw X marker
            gizmos.line(
                pos + Vec3::new(-marker_size, 0.3, -marker_size),
                pos + Vec3::new(marker_size, 0.3, marker_size),
                color,
            );
            gizmos.line(
                pos + Vec3::new(marker_size, 0.3, -marker_size),
                pos + Vec3::new(-marker_size, 0.3, marker_size),
                color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_bar_color_power_ranges() {
        let low = fill_bar_color(0.2, UtilityType::PowerPlant);
        let mid = fill_bar_color(0.6, UtilityType::PowerPlant);
        let high = fill_bar_color(0.9, UtilityType::PowerPlant);
        // Low should be green-ish, high should be red-ish
        let low_s = low.to_srgba();
        let high_s = high.to_srgba();
        assert!(low_s.green > low_s.red, "low fill should be green");
        assert!(high_s.red > high_s.green, "high fill should be red");
        // Mid should differ from both
        let mid_s = mid.to_srgba();
        assert!(
            mid_s.red > 0.5 && mid_s.green > 0.5,
            "mid fill should be yellowish"
        );
    }

    #[test]
    fn fill_bar_color_water_ranges() {
        let low = fill_bar_color(0.2, UtilityType::WaterTower);
        let high = fill_bar_color(0.9, UtilityType::WaterTower);
        let low_s = low.to_srgba();
        let high_s = high.to_srgba();
        assert!(low_s.blue > low_s.red, "low water fill should be blue");
        assert!(high_s.red > high_s.green, "high water fill should be red");
    }

    #[test]
    fn grid_to_world_center_of_cell() {
        let pos = grid_to_world(0, 0);
        assert!((pos.x - CELL_SIZE * 0.5).abs() < 0.01);
        assert!((pos.z - CELL_SIZE * 0.5).abs() < 0.01);

        let pos2 = grid_to_world(10, 20);
        assert!((pos2.x - (10.0 * CELL_SIZE + CELL_SIZE * 0.5)).abs() < 0.01);
        assert!((pos2.z - (20.0 * CELL_SIZE + CELL_SIZE * 0.5)).abs() < 0.01);
    }
}
