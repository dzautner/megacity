//! Gaussian plume and isotropic dispersion functions.
//!
//! These operate on a floating-point accumulator buffer indexed by
//! `[y * GRID_WIDTH + x]` matching the `PollutionGrid` layout.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

/// Maximum plume radius in grid cells from source.
pub const PLUME_RADIUS: i32 = 12;

/// A single pollution source on the grid.
pub struct PollutionSource {
    pub x: usize,
    pub y: usize,
    pub emission_q: f32,
}

/// Applies Gaussian plume dispersion for a single source onto the pollution
/// buffer. The plume spreads downwind with concentration following a Gaussian
/// profile in the crosswind direction.
///
/// For each cell within [`PLUME_RADIUS`] of the source:
/// 1. Compute the vector from source to cell
/// 2. Project onto wind direction (downwind distance) and perpendicular
///    (crosswind distance)
/// 3. Cells in the downwind cone receive the bulk of pollution
/// 4. All cells near the source get some ambient contribution
/// 5. sigma_y grows with downwind distance (lateral spread)
pub fn apply_plume_source(
    levels: &mut [f32],
    src: &PollutionSource,
    wind_dx: f32,
    wind_dy: f32,
    wind_speed: f32,
) {
    let sx = src.x as f32;
    let sy = src.y as f32;
    let q = src.emission_q;

    let x_min = (src.x as i32 - PLUME_RADIUS).max(0) as usize;
    let x_max = (src.x as i32 + PLUME_RADIUS).min(GRID_WIDTH as i32 - 1) as usize;
    let y_min = (src.y as i32 - PLUME_RADIUS).max(0) as usize;
    let y_max = (src.y as i32 + PLUME_RADIUS).min(GRID_HEIGHT as i32 - 1) as usize;

    // Speed factor: stronger wind concentrates plume more narrowly downwind
    let speed_factor = 0.5 + wind_speed * 0.5;

    for cy in y_min..=y_max {
        for cx in x_min..=x_max {
            let dx = cx as f32 - sx;
            let dy = cy as f32 - sy;
            let dist_sq = dx * dx + dy * dy;

            // Downwind distance (projection onto wind direction)
            let downwind = dx * wind_dx + dy * wind_dy;

            // Crosswind distance (perpendicular to wind)
            let crosswind = -dx * wind_dy + dy * wind_dx;

            // --- Ambient near-source component ---
            // All cells near the source get some pollution regardless of wind
            // direction, modeling turbulent mixing in the source's vicinity.
            let ambient_radius = 6.0f32;
            let dist = dist_sq.sqrt();
            let ambient = if dist < ambient_radius {
                q * 0.3 * (1.0 - dist / ambient_radius)
            } else {
                0.0
            };

            // --- Directional plume component ---
            let plume = if downwind >= -0.5 {
                // sigma_y grows with downwind distance (turbulent diffusion)
                // Base sigma = 2.0 for near-source spread, growing at 0.3/cell
                let sigma_y =
                    (2.0 + 0.3 * downwind.max(0.0)) * (1.0 / speed_factor);

                // Gaussian crosswind profile
                let crosswind_factor =
                    (-0.5 * crosswind * crosswind / (sigma_y * sigma_y)).exp();

                // Downwind decay: concentration decreases with distance
                let downwind_dist = downwind.max(0.01);
                let downwind_factor = 1.0 / (1.0 + 0.12 * downwind_dist);

                q * 0.7 * crosswind_factor * downwind_factor
            } else {
                // Upwind: small leakage for very close cells
                if dist_sq < 4.0 {
                    q * 0.05
                } else {
                    0.0
                }
            };

            levels[cy * GRID_WIDTH + cx] += ambient + plume;
        }
    }
}

/// Applies isotropic (calm wind) dispersion for a single source.
/// Used when wind speed is below the calm threshold.
pub fn apply_isotropic_source(levels: &mut [f32], src: &PollutionSource) {
    let radius = 8i32;
    let q = src.emission_q;

    let x_min = (src.x as i32 - radius).max(0) as usize;
    let x_max = (src.x as i32 + radius).min(GRID_WIDTH as i32 - 1) as usize;
    let y_min = (src.y as i32 - radius).max(0) as usize;
    let y_max = (src.y as i32 + radius).min(GRID_HEIGHT as i32 - 1) as usize;

    for cy in y_min..=y_max {
        for cx in x_min..=x_max {
            let dx = (cx as i32 - src.x as i32).abs();
            let dy = (cy as i32 - src.y as i32).abs();
            let dist = dx + dy;
            let decay = (q - dist as f32).max(0.0);
            levels[cy * GRID_WIDTH + cx] += decay;
        }
    }
}
