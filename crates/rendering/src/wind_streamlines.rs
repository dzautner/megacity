//! Wind direction streamlines overlay.
//!
//! When the Wind overlay is active, renders animated streamline particles
//! across the map using gizmos. Particles move in the wind direction with
//! speed proportional to wind speed. Useful for wind turbine placement and
//! pollution dispersal planning.
//!
//! LOD: streamlines are only drawn when the camera is close enough (distance < 1500).
//! Particle density decreases with camera distance for performance.

use bevy::prelude::*;

use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::wind::WindState;

use crate::camera::OrbitCamera;
use crate::overlay::{OverlayMode, OverlayState};

/// Height above ground at which streamline particles are drawn.
const STREAMLINE_Y: f32 = 0.3;

/// Maximum camera distance at which streamlines are rendered.
const MAX_DRAW_DISTANCE: f32 = 1500.0;

/// Grid spacing between streamline seed points at close zoom (in cells).
const BASE_GRID_SPACING: usize = 4;

/// Length of each streamline particle (gizmo line) in world units.
const PARTICLE_LENGTH: f32 = 8.0;

/// Number of particles along each streamline.
const PARTICLES_PER_STREAMLINE: usize = 4;

/// Spacing between particles along a streamline (world units).
const PARTICLE_SPACING: f32 = 12.0;

/// Base animation speed multiplier (world units per second at wind speed 1.0).
const BASE_ANIM_SPEED: f32 = 80.0;

/// Total cycle length for animation wrapping (world units).
const CYCLE_LENGTH: f32 = PARTICLE_SPACING * PARTICLES_PER_STREAMLINE as f32;

/// Arrow head length as a fraction of particle length.
const ARROW_HEAD_RATIO: f32 = 0.4;

/// Arrow head half-width in world units.
const ARROW_HEAD_WIDTH: f32 = 2.0;

/// Deterministic hash for seeding per-streamline variation.
fn streamline_hash(x: usize, y: usize) -> f32 {
    let seed =
        (x as u64).wrapping_mul(0x517cc1b727220a95) ^ (y as u64).wrapping_mul(0x6c62272e07bb0142);
    let mixed = seed.wrapping_mul(0x9e3779b97f4a7c15);
    let mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let mixed = mixed ^ (mixed >> 27);
    (mixed % 1000) as f32 / 1000.0
}

/// System: draw animated wind streamlines when Wind overlay is active.
///
/// Streamlines are seeded on a regular grid covering the map, with density
/// adjusted by camera distance. Each streamline consists of several small
/// arrow-headed particles that animate in the wind direction.
pub fn draw_wind_streamlines(
    overlay: Res<OverlayState>,
    wind: Res<WindState>,
    camera: Res<OrbitCamera>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    if overlay.mode != OverlayMode::Wind {
        return;
    }

    // LOD: skip rendering when zoomed too far out
    if camera.distance > MAX_DRAW_DISTANCE {
        return;
    }

    let (wind_dx, wind_dy) = wind.direction_vector();
    let wind_dir = Vec2::new(wind_dx, wind_dy);
    let wind_speed = wind.speed;

    // Skip drawing if wind is essentially calm
    if wind_speed < 0.02 {
        return;
    }

    // Perpendicular vector for arrow heads
    let perp = Vec2::new(-wind_dir.y, wind_dir.x);

    // Determine grid spacing based on camera distance (LOD)
    let spacing = if camera.distance < 400.0 {
        BASE_GRID_SPACING
    } else if camera.distance < 800.0 {
        BASE_GRID_SPACING * 2
    } else {
        BASE_GRID_SPACING * 3
    };

    // Animation offset: particles move along the wind direction over time
    let anim_offset = (time.elapsed_secs() * BASE_ANIM_SPEED * wind_speed) % CYCLE_LENGTH;

    // Compute visible area from camera focus to cull off-screen streamlines
    let view_radius = camera.distance * 1.2; // generous margin
    let focus_x = camera.focus.x;
    let focus_z = camera.focus.z;

    // World bounds
    let world_w = GRID_WIDTH as f32 * CELL_SIZE;
    let world_h = GRID_HEIGHT as f32 * CELL_SIZE;

    // Cell range to iterate (clamped to grid)
    let min_cx = ((focus_x - view_radius) / CELL_SIZE).max(0.0) as usize;
    let max_cx = ((focus_x + view_radius) / CELL_SIZE).min((GRID_WIDTH - 1) as f32) as usize;
    let min_cy = ((focus_z - view_radius) / CELL_SIZE).max(0.0) as usize;
    let max_cy = ((focus_z + view_radius) / CELL_SIZE).min((GRID_HEIGHT - 1) as f32) as usize;

    // Color: semi-transparent cyan/white, alpha scaled by wind speed
    let base_alpha = 0.3 + wind_speed * 0.5;
    let color = Color::srgba(0.7, 0.85, 1.0, base_alpha);
    let head_color = Color::srgba(0.9, 0.95, 1.0, base_alpha * 0.8);

    // Iterate over seed grid
    let mut cx = min_cx;
    while cx <= max_cx {
        let mut cy = min_cy;
        while cy <= max_cy {
            // World position of this seed point
            let wx = cx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
            let wz = cy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

            // Per-streamline phase offset for staggered animation
            let phase = streamline_hash(cx, cy) * CYCLE_LENGTH;

            // Draw particles along this streamline
            for p in 0..PARTICLES_PER_STREAMLINE {
                // Base offset along wind direction for this particle
                let base_along = p as f32 * PARTICLE_SPACING + anim_offset + phase;
                let along = base_along % CYCLE_LENGTH - CYCLE_LENGTH * 0.5;

                // Particle center in world space
                let center_x = wx + wind_dir.x * along;
                let center_z = wz + wind_dir.y * along;

                // Cull particles outside world bounds
                if center_x < 0.0 || center_x > world_w || center_z < 0.0 || center_z > world_h {
                    continue;
                }

                // Scale particle length by wind speed
                let len = PARTICLE_LENGTH * (0.3 + wind_speed * 0.7);
                let half_len = len * 0.5;

                // Particle line endpoints
                let tail = Vec3::new(
                    center_x - wind_dir.x * half_len,
                    STREAMLINE_Y,
                    center_z - wind_dir.y * half_len,
                );
                let tip = Vec3::new(
                    center_x + wind_dir.x * half_len,
                    STREAMLINE_Y,
                    center_z + wind_dir.y * half_len,
                );

                // Draw shaft
                gizmos.line(tail, tip, color);

                // Draw small arrow head at the tip
                let head_len = len * ARROW_HEAD_RATIO;
                let head_base_x = tip.x - wind_dir.x * head_len;
                let head_base_z = tip.z - wind_dir.y * head_len;
                let wing_l = Vec3::new(
                    head_base_x + perp.x * ARROW_HEAD_WIDTH,
                    STREAMLINE_Y,
                    head_base_z + perp.y * ARROW_HEAD_WIDTH,
                );
                let wing_r = Vec3::new(
                    head_base_x - perp.x * ARROW_HEAD_WIDTH,
                    STREAMLINE_Y,
                    head_base_z - perp.y * ARROW_HEAD_WIDTH,
                );

                gizmos.line(wing_l, tip, head_color);
                gizmos.line(wing_r, tip, head_color);
            }

            cy += spacing;
        }
        cx += spacing;
    }
}

pub struct WindStreamlinesPlugin;

impl Plugin for WindStreamlinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_wind_streamlines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streamline_hash_deterministic() {
        let a = streamline_hash(10, 20);
        let b = streamline_hash(10, 20);
        assert!((a - b).abs() < f32::EPSILON, "Hash should be deterministic");
    }

    #[test]
    fn test_streamline_hash_range() {
        for x in 0..100 {
            for y in 0..100 {
                let val = streamline_hash(x, y);
                assert!(
                    (0.0..1.0).contains(&val),
                    "Hash({},{}) = {} out of range",
                    x,
                    y,
                    val
                );
            }
        }
    }

    #[test]
    fn test_streamline_hash_variation() {
        // Different inputs should produce different outputs (mostly)
        let a = streamline_hash(0, 0);
        let b = streamline_hash(1, 0);
        let c = streamline_hash(0, 1);
        // At least two of three should differ
        let distinct = (a != b) as u32 + (b != c) as u32 + (a != c) as u32;
        assert!(distinct >= 2, "Hash should produce varied outputs");
    }

    #[test]
    fn test_cycle_length_positive() {
        assert!(
            CYCLE_LENGTH > 0.0,
            "Cycle length should be positive: {}",
            CYCLE_LENGTH
        );
    }

    #[test]
    fn test_particle_constants_consistent() {
        // Particle spacing * count should equal cycle length
        let expected = PARTICLE_SPACING * PARTICLES_PER_STREAMLINE as f32;
        assert!(
            (CYCLE_LENGTH - expected).abs() < f32::EPSILON,
            "CYCLE_LENGTH ({}) should equal PARTICLE_SPACING * PARTICLES_PER_STREAMLINE ({})",
            CYCLE_LENGTH,
            expected
        );
    }
}
