//! POWER-020: Enhanced Power Grid Overlay (Coverage, Outages, Connections)
//!
//! Renders additional gizmo layers when the Power overlay is active:
//!
//! - **Outage zones**: Pulsing red highlight on cells affected by rolling
//!   blackouts (derived from `EnergyDispatchState`).
//! - **Power line connections**: Lines drawn between nearby power sources to
//!   show the interconnected grid topology.
//! - **Coverage boundaries**: Faint glow ring at the edge of each source's
//!   coverage area, indicating the reach limit.
//! - **Reserve margin warning**: Amber pulsing border when reserve margin is
//!   low (< 20%), intensifying as deficit approaches.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::energy_demand::EnergyGrid;
use simulation::energy_dispatch::EnergyDispatchState;
use simulation::grid::WorldGrid;
use simulation::network_viz::NetworkVizData;

use crate::overlay::{OverlayMode, OverlayState};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of outage cells to draw per frame (performance cap).
const MAX_OUTAGE_CELLS: u32 = 400;

/// Maximum distance (in grid cells) between sources to draw a connection line.
const SOURCE_CONNECTION_RANGE: f32 = 60.0;

/// Reserve margin threshold below which the warning indicator appears.
const LOW_RESERVE_THRESHOLD: f32 = 0.20;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PowerOverlayPlugin;

impl Plugin for PowerOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_outage_zones,
                draw_power_source_connections,
                draw_coverage_boundaries,
                draw_reserve_margin_warning,
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Convert grid coordinates to world position (center of cell).
fn grid_to_world(gx: usize, gy: usize) -> Vec3 {
    Vec3::new(
        gx as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        0.6,
        gy as f32 * CELL_SIZE + CELL_SIZE * 0.5,
    )
}

// ---------------------------------------------------------------------------
// Outage zone rendering
// ---------------------------------------------------------------------------

/// Draw pulsing red highlights on cells experiencing rolling blackout.
///
/// Uses the `EnergyDispatchState` blackout rotation offset to determine which
/// powered cells are currently affected. Cells are selected via a deterministic
/// hash of their position plus the rotation offset, ensuring the pattern
/// rotates each dispatch tick.
#[allow(clippy::too_many_arguments)]
fn draw_outage_zones(
    overlay: Res<OverlayState>,
    dispatch: Res<EnergyDispatchState>,
    grid: Res<WorldGrid>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    if overlay.mode != OverlayMode::Power {
        return;
    }
    if !dispatch.has_deficit || dispatch.blackout_cells == 0 {
        return;
    }

    let t = time.elapsed_secs();
    // Pulsing alpha for outage cells
    let pulse = ((t * 4.0).sin() * 0.5 + 0.5) * 0.6 + 0.2; // 0.2..0.8
    let outage_color = Color::srgba(0.95, 0.10, 0.10, pulse);

    let rotation = dispatch.blackout_rotation;
    let shed_fraction = dispatch.load_shed_fraction;
    let mut drawn: u32 = 0;

    for y in 0..grid.height {
        if drawn >= MAX_OUTAGE_CELLS {
            break;
        }
        for x in 0..grid.width {
            if drawn >= MAX_OUTAGE_CELLS {
                break;
            }
            let cell = grid.get(x, y);
            if !cell.has_power {
                continue;
            }
            // Deterministic selection: hash cell position with rotation
            let hash = (x.wrapping_mul(7919))
                .wrapping_add(y.wrapping_mul(6271))
                .wrapping_add(rotation as usize * 1301);
            let threshold = (shed_fraction * 1000.0) as usize;
            if (hash % 1000) >= threshold {
                continue;
            }

            drawn += 1;
            let pos = grid_to_world(x, y);
            let half = CELL_SIZE * 0.45;

            // Draw filled outage square using crossing lines
            for i in 0..3 {
                let frac = (i as f32 + 0.5) / 3.0;
                let offset = -half + CELL_SIZE * 0.9 * frac;
                gizmos.line(
                    Vec3::new(pos.x - half, pos.y, pos.z + offset),
                    Vec3::new(pos.x + half, pos.y, pos.z + offset),
                    outage_color,
                );
                gizmos.line(
                    Vec3::new(pos.x + offset, pos.y, pos.z - half),
                    Vec3::new(pos.x + offset, pos.y, pos.z + half),
                    outage_color,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Power source connections
// ---------------------------------------------------------------------------

/// Draw lines connecting nearby power sources to visualize the grid topology.
///
/// Sources within `SOURCE_CONNECTION_RANGE` grid cells are connected with
/// a faint line whose color shifts from green (both healthy) to orange/red
/// (one or both at capacity).
fn draw_power_source_connections(
    overlay: Res<OverlayState>,
    viz: Res<NetworkVizData>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    if overlay.mode != OverlayMode::Power {
        return;
    }
    let sources = &viz.power_sources;
    if sources.len() < 2 {
        return;
    }

    let t = time.elapsed_secs();
    // Slow pulse for connection lines
    let pulse_alpha = 0.25 + ((t * 1.5).sin() * 0.5 + 0.5) * 0.2;

    for i in 0..sources.len() {
        for j in (i + 1)..sources.len() {
            let a = &sources[i];
            let b = &sources[j];

            let dx = a.grid_x as f32 - b.grid_x as f32;
            let dy = a.grid_y as f32 - b.grid_y as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > SOURCE_CONNECTION_RANGE {
                continue;
            }

            // Color based on capacity utilization of both sources
            let util_a = source_utilization(a);
            let util_b = source_utilization(b);
            let max_util = util_a.max(util_b);

            let (r, g, b_ch) = connection_color(max_util);
            let color = Color::srgba(r, g, b_ch, pulse_alpha);

            let pos_a = grid_to_world(a.grid_x, a.grid_y) + Vec3::Y * 1.5;
            let pos_b = grid_to_world(b.grid_x, b.grid_y) + Vec3::Y * 1.5;

            // Draw main connection line
            gizmos.line(pos_a, pos_b, color);

            // Draw animated flow dot along the line
            let flow_t = ((t * 0.8 + i as f32 * 0.3) % 1.0).clamp(0.0, 1.0);
            let flow_pos = pos_a.lerp(pos_b, flow_t);
            let dot_color = Color::srgba(r, g, b_ch, pulse_alpha + 0.3);

            gizmos.circle(
                Isometry3d::new(
                    flow_pos,
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                ),
                CELL_SIZE * 0.3,
                dot_color,
            );
        }
    }
}

/// Compute utilization ratio for a source (0.0 = idle, 1.0 = fully utilized).
fn source_utilization(info: &simulation::network_viz::SourceInfo) -> f32 {
    if info.max_coverage == 0 {
        return 0.0;
    }
    (info.cells_covered as f32 / info.max_coverage as f32).clamp(0.0, 1.0)
}

/// Map utilization to connection line color.
/// Green (healthy) -> Yellow (moderate) -> Red (strained).
fn connection_color(utilization: f32) -> (f32, f32, f32) {
    if utilization < 0.5 {
        // Green to yellow
        let t = utilization / 0.5;
        (0.3 + t * 0.6, 0.85 - t * 0.15, 0.3 - t * 0.15)
    } else {
        // Yellow to red
        let t = (utilization - 0.5) / 0.5;
        (0.9, 0.7 - t * 0.5, 0.15 - t * 0.05)
    }
}

// ---------------------------------------------------------------------------
// Coverage boundaries
// ---------------------------------------------------------------------------

/// Draw faint boundary circles at the edge of each power source's coverage.
///
/// These circles help the player see exactly where coverage ends so they can
/// plan new power plant placement for maximum coverage.
fn draw_coverage_boundaries(
    overlay: Res<OverlayState>,
    viz: Res<NetworkVizData>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    if overlay.mode != OverlayMode::Power {
        return;
    }

    let t = time.elapsed_secs();

    for info in &viz.power_sources {
        let pos = grid_to_world(info.grid_x, info.grid_y);
        let range_radius = info.effective_range as f32 * CELL_SIZE;

        // Subtle breathing animation on the boundary
        let breath = ((t * 1.0 + info.color_index as f32 * 0.7).sin() * 0.5 + 0.5) * 0.06;
        let alpha = 0.12 + breath;

        // Dashed effect: draw multiple small arcs by using segmented circles
        let segments = 32;
        let segment_angle = std::f32::consts::TAU / segments as f32;

        for seg in 0..segments {
            // Skip every other segment for dashed effect
            if seg % 2 == 1 {
                continue;
            }
            let angle_start = seg as f32 * segment_angle;
            let angle_end = angle_start + segment_angle;

            let start = Vec3::new(
                pos.x + range_radius * angle_start.cos(),
                pos.y + 0.3,
                pos.z + range_radius * angle_start.sin(),
            );
            let end = Vec3::new(
                pos.x + range_radius * angle_end.cos(),
                pos.y + 0.3,
                pos.z + range_radius * angle_end.sin(),
            );

            let color = Color::srgba(0.9, 0.85, 0.2, alpha);
            gizmos.line(start, end, color);
        }
    }
}

// ---------------------------------------------------------------------------
// Reserve margin warning
// ---------------------------------------------------------------------------

/// When reserve margin is low, draw an amber pulsing warning indicator near
/// power sources. The indicator intensifies as the deficit threshold nears.
///
/// - 10%–20% reserve: gentle amber pulse
/// - 5%–10%: faster, brighter orange pulse
/// - <5% or deficit: rapid red pulse
fn draw_reserve_margin_warning(
    overlay: Res<OverlayState>,
    energy_grid: Res<EnergyGrid>,
    viz: Res<NetworkVizData>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    if overlay.mode != OverlayMode::Power {
        return;
    }

    let reserve = energy_grid.reserve_margin;
    if reserve >= LOW_RESERVE_THRESHOLD {
        return;
    }

    let t = time.elapsed_secs();

    // Determine severity and visual parameters
    let (color_rgb, pulse_speed, base_alpha) = if reserve < 0.0 {
        // Deficit: rapid red
        ((0.95, 0.10, 0.10), 6.0, 0.5)
    } else if reserve < 0.05 {
        // Critical: fast orange-red
        ((0.95, 0.30, 0.05), 4.5, 0.4)
    } else if reserve < 0.10 {
        // Warning: moderate orange
        ((0.95, 0.60, 0.10), 3.0, 0.3)
    } else {
        // Caution: gentle amber
        ((0.95, 0.80, 0.20), 2.0, 0.2)
    };

    let pulse = (t * pulse_speed).sin() * 0.5 + 0.5;
    let alpha = base_alpha + pulse * 0.25;

    let color = Color::srgba(color_rgb.0, color_rgb.1, color_rgb.2, alpha);

    // Draw warning rings around each power source
    for info in &viz.power_sources {
        let pos = grid_to_world(info.grid_x, info.grid_y);
        let radius = CELL_SIZE * (2.0 + pulse * 0.5);

        gizmos.circle(
            Isometry3d::new(
                pos + Vec3::Y * 0.4,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            radius,
            color,
        );

        // Inner ring for emphasis
        let inner_radius = CELL_SIZE * (1.2 + pulse * 0.3);
        let inner_color = Color::srgba(
            color_rgb.0,
            color_rgb.1,
            color_rgb.2,
            alpha * 0.6,
        );
        gizmos.circle(
            Isometry3d::new(
                pos + Vec3::Y * 0.4,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            inner_radius,
            inner_color,
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_color_green_at_low_utilization() {
        let (r, g, _b) = connection_color(0.0);
        assert!(g > r, "low utilization should be green-dominant");
    }

    #[test]
    fn connection_color_yellow_at_mid_utilization() {
        let (r, g, _b) = connection_color(0.5);
        assert!(r > 0.5 && g > 0.5, "mid utilization should be yellowish");
    }

    #[test]
    fn connection_color_red_at_high_utilization() {
        let (r, g, _b) = connection_color(1.0);
        assert!(r > g, "high utilization should be red-dominant");
    }

    #[test]
    fn source_utilization_zero_when_max_coverage_zero() {
        let info = simulation::network_viz::SourceInfo {
            entity: Entity::PLACEHOLDER,
            grid_x: 0,
            grid_y: 0,
            utility_type: simulation::utilities::UtilityType::PowerPlant,
            effective_range: 10,
            cells_covered: 0,
            max_coverage: 0,
            color_index: 0,
        };
        assert!((source_utilization(&info) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn source_utilization_clamped_to_one() {
        let info = simulation::network_viz::SourceInfo {
            entity: Entity::PLACEHOLDER,
            grid_x: 0,
            grid_y: 0,
            utility_type: simulation::utilities::UtilityType::PowerPlant,
            effective_range: 10,
            cells_covered: 100,
            max_coverage: 50,
            color_index: 0,
        };
        assert!((source_utilization(&info) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn grid_to_world_center_of_cell() {
        let pos = grid_to_world(5, 10);
        let expected_x = 5.0 * CELL_SIZE + CELL_SIZE * 0.5;
        let expected_z = 10.0 * CELL_SIZE + CELL_SIZE * 0.5;
        assert!((pos.x - expected_x).abs() < 0.01);
        assert!((pos.z - expected_z).abs() < 0.01);
    }

    #[test]
    fn connection_color_monotonic_red_increase() {
        let (r0, _, _) = connection_color(0.0);
        let (r5, _, _) = connection_color(0.5);
        let (r10, _, _) = connection_color(1.0);
        assert!(r5 >= r0, "red should increase with utilization");
        assert!(r10 >= r5, "red should increase with utilization");
    }
}
