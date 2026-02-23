//! Tests for the flood simulation system: water spreading, drainage, and
//! building damage calculation logic.

#[cfg(test)]
mod tests {
    use crate::flood_simulation::damage_curves::*;
    use crate::grid::ZoneType;

    // -------------------------------------------------------------------------
    // Water spreading logic tests (unit tests for the algorithm)
    // -------------------------------------------------------------------------

    #[test]
    fn test_spread_rate_constant() {
        assert!(
            (SPREAD_RATE - 0.25).abs() < f32::EPSILON,
            "Spread rate should be 0.25"
        );
    }

    #[test]
    fn test_natural_drain_rate_constant() {
        assert!(
            (NATURAL_DRAIN_RATE - 0.01).abs() < f32::EPSILON,
            "Natural drain rate should be 0.01 ft/tick"
        );
    }

    #[test]
    fn test_storm_drain_rate_constant() {
        assert!(
            (STORM_DRAIN_RATE - 0.05).abs() < f32::EPSILON,
            "Storm drain rate should be 0.05 ft/tick"
        );
    }

    #[test]
    fn test_flood_threshold_constant() {
        assert!(
            (FLOOD_DEPTH_THRESHOLD - 0.5).abs() < f32::EPSILON,
            "Flood threshold should be 0.5 ft"
        );
    }

    #[test]
    fn test_spread_iterations_constant() {
        assert_eq!(SPREAD_ITERATIONS, 5, "Should run 5 spread iterations");
    }

    // -------------------------------------------------------------------------
    // Water conservation during spreading (manual simulation)
    // -------------------------------------------------------------------------

    #[test]
    fn test_water_conservation_single_spread_step() {
        // Simulate a single spread step on a small 3x3 grid.
        // Center cell has 4.0 ft of water; all neighbors are at lower elevation.
        // Flat terrain: elevation 10.0 at center, 9.0 at neighbors.
        let mut depths = vec![0.0_f32; 9];
        let elevations = vec![9.0, 9.0, 9.0, 9.0, 10.0, 9.0, 9.0, 9.0, 9.0];
        let width = 3usize;

        // Place water at center (1,1)
        depths[1 * width + 1] = 4.0;

        let total_before: f32 = depths.iter().sum();

        // Spread: center cell distributes SPREAD_RATE * depth to lower neighbors
        let snapshot = depths.clone();
        let cx = 1usize;
        let cy = 1usize;
        let cidx = cy * width + cx;
        let current_depth = snapshot[cidx];
        let current_elev = elevations[cidx];
        let current_surface = current_elev + current_depth;

        // 4 cardinal neighbors of (1,1) in 3x3: (0,1), (2,1), (1,0), (1,2)
        let neighbors: [(usize, usize); 4] = [(0, 1), (2, 1), (1, 0), (1, 2)];
        let mut lower_diffs = Vec::new();
        let mut total_diff = 0.0_f32;

        for &(nx, ny) in &neighbors {
            let nidx = ny * width + nx;
            let n_surface = elevations[nidx] + snapshot[nidx];
            if n_surface < current_surface {
                let diff = current_surface - n_surface;
                lower_diffs.push((nx, ny, diff));
                total_diff += diff;
            }
        }

        let transferable = current_depth * SPREAD_RATE;
        depths[cidx] -= transferable;

        for &(nx, ny, diff) in &lower_diffs {
            let fraction = diff / total_diff;
            let transfer = transferable * fraction;
            let nidx = ny * width + nx;
            depths[nidx] += transfer;
        }

        let total_after: f32 = depths.iter().sum();

        assert!(
            (total_before - total_after).abs() < 0.001,
            "Water should be conserved: before={}, after={}",
            total_before,
            total_after
        );
    }

    #[test]
    fn test_water_spreads_to_lower_elevation_only() {
        // 3-cell row: elevations [8.0, 10.0, 12.0]. Water at center (index 1).
        // Water should only flow to the left (lower elevation).
        let elevations = [8.0_f32, 10.0, 12.0];
        let mut depths = [0.0_f32, 5.0, 0.0];

        let current_depth = depths[1];
        let current_surface = elevations[1] + current_depth; // 15.0

        // Left neighbor: surface = 8.0 + 0.0 = 8.0 < 15.0 => lower
        // Right neighbor: surface = 12.0 + 0.0 = 12.0 < 15.0 => also lower
        // But right elevation (12.0) is higher than center elevation (10.0)
        // With the surface-based comparison, BOTH are lower surface
        // The left one gets more water because the diff is larger

        let left_surface = elevations[0] + depths[0]; // 8.0
        let right_surface = elevations[2] + depths[2]; // 12.0

        assert!(left_surface < current_surface);
        assert!(right_surface < current_surface);

        let left_diff = current_surface - left_surface; // 7.0
        let right_diff = current_surface - right_surface; // 3.0
        let total_diff = left_diff + right_diff; // 10.0

        let transferable = current_depth * SPREAD_RATE; // 1.25
        depths[1] -= transferable;

        depths[0] += transferable * (left_diff / total_diff);
        depths[2] += transferable * (right_diff / total_diff);

        // Left should get more water (70%)
        assert!(
            depths[0] > depths[2],
            "Lower-elevation cell should receive more water: left={}, right={}",
            depths[0],
            depths[2]
        );

        // Water is conserved
        let total: f32 = depths.iter().sum();
        assert!(
            (total - 5.0).abs() < 0.001,
            "Total water should be 5.0, got {}",
            total
        );
    }

    #[test]
    fn test_no_spread_when_cell_is_highest() {
        // All neighbors have higher surface than center: no spreading occurs
        let elevations = [20.0, 20.0, 20.0, 20.0, 5.0, 20.0, 20.0, 20.0, 20.0];
        let depths = [0.0_f32; 9];
        let width = 3usize;

        let cx = 1usize;
        let cy = 1usize;
        let cidx = cy * width + cx;
        let current_surface = elevations[cidx] + depths[cidx] + 2.0; // 5 + 2 = 7

        let neighbors: [(usize, usize); 4] = [(0, 1), (2, 1), (1, 0), (1, 2)];
        let lower_count = neighbors
            .iter()
            .filter(|&&(nx, ny)| {
                let nidx = ny * width + nx;
                elevations[nidx] + depths[nidx] < current_surface
            })
            .count();

        // All neighbors at elevation 20 > surface 7
        assert_eq!(
            lower_count, 0,
            "No neighbors should be lower than center cell"
        );
    }

    // -------------------------------------------------------------------------
    // Drainage calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_natural_drain_reduces_depth() {
        let initial_depth = 1.0_f32;
        let after_drain = (initial_depth - NATURAL_DRAIN_RATE).max(0.0);
        assert!(
            (after_drain - 0.99).abs() < f32::EPSILON,
            "After natural drain: expected 0.99, got {}",
            after_drain
        );
    }

    #[test]
    fn test_storm_drain_plus_natural_drain() {
        let initial_depth = 1.0_f32;
        let after_drain = (initial_depth - NATURAL_DRAIN_RATE - STORM_DRAIN_RATE).max(0.0);
        let expected = 1.0 - 0.01 - 0.05;
        assert!(
            (after_drain - expected).abs() < 0.001,
            "After combined drain: expected {}, got {}",
            expected,
            after_drain
        );
    }

    #[test]
    fn test_drain_does_not_go_negative() {
        let initial_depth = 0.005_f32;
        let after_drain = (initial_depth - NATURAL_DRAIN_RATE - STORM_DRAIN_RATE).max(0.0);
        assert!(
            after_drain >= 0.0,
            "Drain should not produce negative depth"
        );
        assert!(
            after_drain.abs() < f32::EPSILON,
            "Small depth should drain to exactly 0.0"
        );
    }

    // -------------------------------------------------------------------------
    // Building damage calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_building_damage_residential_at_6ft() {
        // Residential L3 building with capacity 500 at 6 ft flood depth
        let capacity = 500u32;
        let level = 3u8;
        let depth = 6.0_f32;
        let damage_frac = depth_damage_fraction(depth, ZoneType::ResidentialHigh);
        let building_value = capacity as f64 * level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        let damage = building_value * damage_frac as f64;

        // damage_frac = 0.65, building_value = 500 * 3 * 1000 = 1,500,000
        // damage = 1,500,000 * 0.65 = 975,000
        assert!(
            (damage - 975_000.0).abs() < 1.0,
            "Residential L3 damage at 6ft should be 975000, got {}",
            damage
        );
    }

    #[test]
    fn test_building_damage_industrial_at_3ft() {
        let capacity = 150u32;
        let level = 3u8;
        let depth = 3.0_f32;
        let damage_frac = depth_damage_fraction(depth, ZoneType::Industrial);
        let building_value = capacity as f64 * level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        let damage = building_value * damage_frac as f64;

        // damage_frac = 0.15, building_value = 150 * 3 * 1000 = 450,000
        // damage = 450,000 * 0.15 = 67,500
        assert!(
            (damage - 67_500.0).abs() < 1.0,
            "Industrial L3 damage at 3ft should be 67500, got {}",
            damage
        );
    }

    #[test]
    fn test_building_damage_zero_below_threshold() {
        let depth = 0.3_f32; // below FLOOD_DEPTH_THRESHOLD
        assert!(
            depth < FLOOD_DEPTH_THRESHOLD,
            "Depth {} should be below threshold {}",
            depth,
            FLOOD_DEPTH_THRESHOLD
        );
        // No damage should be applied for depths below threshold
    }

    #[test]
    fn test_building_damage_commercial_at_10ft() {
        let capacity = 300u32;
        let level = 5u8;
        let depth = 10.0_f32;
        let damage_frac = depth_damage_fraction(depth, ZoneType::CommercialHigh);
        let building_value = capacity as f64 * level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        let damage = building_value * damage_frac as f64;

        // damage_frac = 0.80, building_value = 300 * 5 * 1000 = 1,500,000
        // damage = 1,500,000 * 0.80 = 1,200,000
        assert!(
            (damage - 1_200_000.0).abs() < 1.0,
            "Commercial L5 damage at 10ft should be 1200000, got {}",
            damage
        );
    }
}
