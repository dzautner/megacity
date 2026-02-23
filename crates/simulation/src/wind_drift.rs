//! Wind-driven pollution drift with 8-direction support and bilinear interpolation.
//!
//! Shifts the entire pollution grid based on wind direction and speed each update.
//! Uses a "pull" approach with bilinear interpolation for smooth fractional-cell offsets.

use crate::pollution::PollutionGrid;
use crate::wind::WindState;

/// Maximum drift per update in cells. At wind speed 1.0, pollution shifts
/// this many cells per tick.
const DRIFT_RATE: f32 = 1.5;

/// Wind speed below this threshold produces no drift (calm conditions).
const CALM_THRESHOLD: f32 = 0.1;

/// Shifts the entire pollution grid in the wind direction using bilinear
/// interpolation for fractional cell offsets.
///
/// Uses a "pull" approach: for each destination cell, we compute the upwind
/// source position and sample via bilinear interpolation across up to 4
/// source cells. This naturally supports all 8 cardinal/diagonal directions
/// plus arbitrary angles.
///
/// - Drift magnitude = `wind_speed * DRIFT_RATE` cells per update
/// - Out-of-bounds source samples are zero (boundary drain)
/// - O(n) single pass over all cells with one temporary buffer
pub fn apply_wind_drift(pollution: &mut PollutionGrid, wind: &WindState) {
    if wind.speed < CALM_THRESHOLD {
        return;
    }

    let (dx, dy) = wind.direction_vector();
    let drift = wind.speed * DRIFT_RATE;

    // Offset in the wind direction (how far pollution shifts)
    let shift_x = dx * drift;
    let shift_y = dy * drift;

    let w = pollution.width;
    let h = pollution.height;
    let total = w * h;

    // Temporary buffer to build the new shifted pollution grid.
    // For each destination cell (x, y), we sample the source at (x - shift_x, y - shift_y)
    // using bilinear interpolation.
    let mut result: Vec<f32> = vec![0.0; total];

    for y in 0..h {
        for x in 0..w {
            // Source position (upwind from this cell)
            let src_x = x as f32 - shift_x;
            let src_y = y as f32 - shift_y;

            // Bilinear interpolation: find the 4 surrounding integer cells
            let x0 = src_x.floor() as i32;
            let y0 = src_y.floor() as i32;
            let x1 = x0 + 1;
            let y1 = y0 + 1;

            // Fractional parts
            let fx = src_x - x0 as f32;
            let fy = src_y - y0 as f32;

            // Bilinear weights
            let w00 = (1.0 - fx) * (1.0 - fy);
            let w10 = fx * (1.0 - fy);
            let w01 = (1.0 - fx) * fy;
            let w11 = fx * fy;

            // Sample source cells (out-of-bounds = 0, boundary drain)
            let s00 = sample_grid(&pollution.levels, w, h, x0, y0);
            let s10 = sample_grid(&pollution.levels, w, h, x1, y0);
            let s01 = sample_grid(&pollution.levels, w, h, x0, y1);
            let s11 = sample_grid(&pollution.levels, w, h, x1, y1);

            let val = w00 * s00 + w10 * s10 + w01 * s01 + w11 * s11;
            result[y * w + x] = val;
        }
    }

    // Write back, clamping to u8 range
    for (level, &val) in pollution.levels.iter_mut().zip(result.iter()).take(total) {
        *level = val.clamp(0.0, 255.0) as u8;
    }
}

/// Samples the pollution grid at integer coordinates, returning 0.0 for
/// out-of-bounds positions (boundary drain).
#[inline]
fn sample_grid(levels: &[u8], width: usize, height: usize, x: i32, y: i32) -> f32 {
    if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
        levels[y as usize * width + x as usize] as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wind::WindState;

    /// Helper to create a small test pollution grid.
    fn make_grid(width: usize, height: usize) -> PollutionGrid {
        PollutionGrid {
            levels: vec![0; width * height],
            width,
            height,
        }
    }

    /// Helper to create a wind state with given direction and speed.
    fn make_wind(direction: f32, speed: f32) -> WindState {
        WindState {
            direction,
            speed,
            ..Default::default()
        }
    }

    #[test]
    fn test_calm_wind_no_drift() {
        let mut grid = make_grid(8, 8);
        grid.set(4, 4, 100);

        let wind = make_wind(0.0, 0.05); // below CALM_THRESHOLD
        apply_wind_drift(&mut grid, &wind);

        assert_eq!(grid.get(4, 4), 100, "calm wind should not move pollution");
    }

    #[test]
    fn test_east_wind_shifts_pollution_east() {
        let mut grid = make_grid(16, 16);
        grid.set(8, 8, 200);

        let wind = make_wind(0.0, 1.0);
        apply_wind_drift(&mut grid, &wind);

        let east = grid.get(9, 8) as u32 + grid.get(10, 8) as u32;
        let west = grid.get(7, 8) as u32 + grid.get(6, 8) as u32;
        assert!(
            east > west,
            "east wind should shift pollution east: east_sum={}, west_sum={}",
            east,
            west
        );
        assert!(
            grid.get(8, 8) < 200,
            "source should lose pollution, got {}",
            grid.get(8, 8)
        );
    }

    #[test]
    fn test_west_wind_shifts_pollution_west() {
        let mut grid = make_grid(16, 16);
        grid.set(8, 8, 200);

        let wind = make_wind(std::f32::consts::PI, 1.0);
        apply_wind_drift(&mut grid, &wind);

        let west = grid.get(7, 8) as u32 + grid.get(6, 8) as u32;
        let east = grid.get(9, 8) as u32 + grid.get(10, 8) as u32;
        assert!(
            west > east,
            "west wind should shift pollution west: west_sum={}, east_sum={}",
            west,
            east
        );
    }

    #[test]
    fn test_north_wind_shifts_pollution_north() {
        let mut grid = make_grid(16, 16);
        grid.set(8, 8, 200);

        let wind = make_wind(std::f32::consts::FRAC_PI_2, 1.0);
        apply_wind_drift(&mut grid, &wind);

        let north = grid.get(8, 9) as u32 + grid.get(8, 10) as u32;
        let south = grid.get(8, 7) as u32 + grid.get(8, 6) as u32;
        assert!(
            north > south,
            "north wind should shift pollution north: north_sum={}, south_sum={}",
            north,
            south
        );
    }

    #[test]
    fn test_diagonal_ne_wind() {
        let mut grid = make_grid(16, 16);
        grid.set(8, 8, 200);

        let wind = make_wind(std::f32::consts::FRAC_PI_4, 1.0);
        apply_wind_drift(&mut grid, &wind);

        let ne = grid.get(9, 9) as u32;
        let sw = grid.get(7, 7) as u32;
        assert!(ne > sw, "NE wind: ne={}, sw={}", ne, sw);
    }

    #[test]
    fn test_diagonal_sw_wind() {
        let mut grid = make_grid(16, 16);
        grid.set(8, 8, 200);

        let wind = make_wind(5.0 * std::f32::consts::FRAC_PI_4, 1.0);
        apply_wind_drift(&mut grid, &wind);

        let sw = grid.get(7, 7) as u32;
        let ne = grid.get(9, 9) as u32;
        assert!(sw > ne, "SW wind: sw={}, ne={}", sw, ne);
    }

    #[test]
    fn test_boundary_drain_east_edge() {
        let mut grid = make_grid(8, 8);
        grid.set(7, 4, 200);

        let wind = make_wind(0.0, 1.0);
        apply_wind_drift(&mut grid, &wind);

        assert!(
            grid.get(7, 4) < 200,
            "boundary drain should reduce edge pollution"
        );
    }

    #[test]
    fn test_boundary_drain_north_edge() {
        let mut grid = make_grid(8, 8);
        grid.set(4, 7, 200);

        let wind = make_wind(std::f32::consts::FRAC_PI_2, 1.0);
        apply_wind_drift(&mut grid, &wind);

        assert!(
            grid.get(4, 7) < 200,
            "boundary drain should reduce edge pollution"
        );
    }

    #[test]
    fn test_fractional_drift_distributes() {
        let mut grid = make_grid(16, 16);
        grid.set(8, 8, 200);

        let wind = make_wind(0.0, 0.5); // drift = 0.75 cells
        apply_wind_drift(&mut grid, &wind);

        let at_8 = grid.get(8, 8) as u32;
        let at_9 = grid.get(9, 8) as u32;
        assert!(
            at_8 > 0 && at_9 > 0,
            "fractional drift should distribute: at_8={}, at_9={}",
            at_8,
            at_9
        );
    }

    #[test]
    fn test_drift_speed_scaling() {
        let mut grid_slow = make_grid(16, 16);
        let mut grid_fast = make_grid(16, 16);
        grid_slow.set(8, 8, 200);
        grid_fast.set(8, 8, 200);

        apply_wind_drift(&mut grid_slow, &make_wind(0.0, 0.2));
        apply_wind_drift(&mut grid_fast, &make_wind(0.0, 0.8));

        let slow_shift = grid_slow.get(9, 8) as u32 + grid_slow.get(10, 8) as u32;
        let fast_shift = grid_fast.get(9, 8) as u32 + grid_fast.get(10, 8) as u32;
        assert!(
            fast_shift > slow_shift,
            "fast={}, slow={}",
            fast_shift,
            slow_shift
        );
    }

    #[test]
    fn test_sample_grid_out_of_bounds() {
        let levels = vec![100u8; 16];
        assert_eq!(sample_grid(&levels, 4, 4, -1, 0), 0.0);
        assert_eq!(sample_grid(&levels, 4, 4, 0, -1), 0.0);
        assert_eq!(sample_grid(&levels, 4, 4, 4, 0), 0.0);
        assert_eq!(sample_grid(&levels, 4, 4, 0, 4), 0.0);
    }

    #[test]
    fn test_sample_grid_in_bounds() {
        let levels = vec![42u8; 16];
        assert_eq!(sample_grid(&levels, 4, 4, 0, 0), 42.0);
        assert_eq!(sample_grid(&levels, 4, 4, 3, 3), 42.0);
    }

    #[test]
    fn test_all_eight_directions() {
        let directions: [(f32, i32, i32, &str); 8] = [
            (0.0, 1, 0, "E"),
            (std::f32::consts::FRAC_PI_4, 1, 1, "NE"),
            (std::f32::consts::FRAC_PI_2, 0, 1, "N"),
            (3.0 * std::f32::consts::FRAC_PI_4, -1, 1, "NW"),
            (std::f32::consts::PI, -1, 0, "W"),
            (5.0 * std::f32::consts::FRAC_PI_4, -1, -1, "SW"),
            (3.0 * std::f32::consts::FRAC_PI_2, 0, -1, "S"),
            (7.0 * std::f32::consts::FRAC_PI_4, 1, -1, "SE"),
        ];

        for (angle, expected_dx, expected_dy, label) in &directions {
            let mut grid = make_grid(16, 16);
            grid.set(8, 8, 200);

            let wind = make_wind(*angle, 1.0);
            apply_wind_drift(&mut grid, &wind);

            let downwind_x = (8 + expected_dx) as usize;
            let downwind_y = (8 + expected_dy) as usize;
            let upwind_x = (8 - expected_dx) as usize;
            let upwind_y = (8 - expected_dy) as usize;

            let downwind = grid.get(downwind_x, downwind_y);
            let upwind = grid.get(upwind_x, upwind_y);
            assert!(
                downwind > upwind,
                "{}: downwind({},{})={} should be > upwind({},{})={}",
                label,
                downwind_x,
                downwind_y,
                downwind,
                upwind_x,
                upwind_y,
                upwind
            );
        }
    }

    #[test]
    fn test_total_pollution_conserved_interior() {
        let mut grid = make_grid(32, 32);
        grid.set(16, 16, 200);

        let before_total: u32 = grid.levels.iter().map(|&v| v as u32).sum();

        apply_wind_drift(&mut grid, &make_wind(0.0, 0.3));

        let after_total: u32 = grid.levels.iter().map(|&v| v as u32).sum();
        let diff = (after_total as i32 - before_total as i32).unsigned_abs();
        assert!(
            diff <= 5,
            "interior drift should conserve pollution: before={}, after={}, diff={}",
            before_total,
            after_total,
            diff
        );
    }
}
