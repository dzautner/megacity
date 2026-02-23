// =============================================================================
// Pipe Burst Temperature Tiers
// =============================================================================

/// Baseline pipe burst probability per mile of water main per day (above freezing).
const PIPE_BURST_BASELINE: f32 = 0.0001;

/// Pipe burst probability at freezing (0C).
const PIPE_BURST_FREEZING: f32 = 0.001;

/// Pipe burst probability below -7C.
const PIPE_BURST_MINUS_7: f32 = 0.01;

/// Pipe burst probability below -18C.
const PIPE_BURST_MINUS_18: f32 = 0.05;

/// Pipe burst probability below -23C.
const PIPE_BURST_MINUS_23: f32 = 0.10;

// =============================================================================
// Deterministic pseudo-random (splitmix64, matching wind_damage.rs pattern)
// =============================================================================

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
fn rand_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    (hash % 1_000_000) as f32 / 1_000_000.0
}

// =============================================================================
// Pipe burst and water service functions
// =============================================================================

/// Return the pipe burst probability per mile of water main per day for a given
/// temperature in Celsius.
///
/// Tiers from the specification:
/// - Above 0C:   0.0001 (baseline)
/// - 0C to -7C:  0.001  (freezing)
/// - -7C to -18C: 0.01
/// - -18C to -23C: 0.05
/// - Below -23C:  0.10
pub fn pipe_burst_probability(temp_c: f32) -> f32 {
    if temp_c > 0.0 {
        PIPE_BURST_BASELINE
    } else if temp_c > -7.0 {
        PIPE_BURST_FREEZING
    } else if temp_c > -18.0 {
        PIPE_BURST_MINUS_7
    } else if temp_c > -23.0 {
        PIPE_BURST_MINUS_18
    } else {
        PIPE_BURST_MINUS_23
    }
}

/// Estimate water main miles from road cell count.
///
/// Approximation: each road cell represents ~0.003 miles of water main
/// (256x256 grid ~ 65k cells, typical city has ~6000 miles of water mains,
/// and roads cover roughly 30% of the grid).
const WATER_MAIN_MILES_PER_ROAD_CELL: f32 = 0.003;

/// Estimate total water main miles from road network cell count.
pub fn estimate_water_main_miles(road_cell_count: u32) -> f32 {
    road_cell_count as f32 * WATER_MAIN_MILES_PER_ROAD_CELL
}

/// Calculate the number of new pipe bursts based on temperature and water main miles.
///
/// Uses deterministic pseudo-random sampling: divides the water main network into
/// discrete segments and rolls for each segment.
pub fn calculate_pipe_bursts(temp_c: f32, water_main_miles: f32, seed: u64) -> u32 {
    let prob = pipe_burst_probability(temp_c);
    // Each "mile" is a discrete segment that can burst independently.
    let segments = water_main_miles.ceil() as u32;
    let mut bursts = 0u32;
    for i in 0..segments {
        let roll_seed = seed.wrapping_mul(0x517cc1b727220a95).wrapping_add(i as u64);
        if rand_f32(roll_seed) < prob {
            bursts += 1;
        }
    }
    bursts
}

/// Water service reduction based on pipe burst count relative to total water main miles.
///
/// Each burst reduces service proportionally. Clamped to [0.2, 1.0] (never below 20%
/// service -- some redundancy always exists).
pub fn water_service_from_bursts(pipe_burst_count: u32, water_main_miles: f32) -> f32 {
    if water_main_miles <= 0.0 {
        return 1.0;
    }
    // Each burst takes out roughly 0.5 miles of service capacity
    let affected_miles = pipe_burst_count as f32 * 0.5;
    let reduction = affected_miles / water_main_miles;
    (1.0 - reduction).clamp(0.2, 1.0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Pipe burst probability tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pipe_burst_above_freezing() {
        assert!(
            (pipe_burst_probability(10.0) - PIPE_BURST_BASELINE).abs() < f32::EPSILON,
            "Above freezing should return baseline"
        );
        assert!(
            (pipe_burst_probability(0.1) - PIPE_BURST_BASELINE).abs() < f32::EPSILON,
            "Just above freezing should return baseline"
        );
    }

    #[test]
    fn test_pipe_burst_at_freezing() {
        assert!(
            (pipe_burst_probability(0.0) - PIPE_BURST_FREEZING).abs() < f32::EPSILON,
            "At freezing should return freezing tier"
        );
        assert!(
            (pipe_burst_probability(-3.0) - PIPE_BURST_FREEZING).abs() < f32::EPSILON,
            "Between 0C and -7C should return freezing tier"
        );
        assert!(
            (pipe_burst_probability(-6.9) - PIPE_BURST_FREEZING).abs() < f32::EPSILON,
            "Just above -7C should return freezing tier"
        );
    }

    #[test]
    fn test_pipe_burst_below_minus_7() {
        assert!(
            (pipe_burst_probability(-7.0) - PIPE_BURST_MINUS_7).abs() < f32::EPSILON,
            "At -7C should return minus-7 tier"
        );
        assert!(
            (pipe_burst_probability(-12.0) - PIPE_BURST_MINUS_7).abs() < f32::EPSILON,
            "Between -7C and -18C should return minus-7 tier"
        );
        assert!(
            (pipe_burst_probability(-17.9) - PIPE_BURST_MINUS_7).abs() < f32::EPSILON,
            "Just above -18C should return minus-7 tier"
        );
    }

    #[test]
    fn test_pipe_burst_below_minus_18() {
        assert!(
            (pipe_burst_probability(-18.0) - PIPE_BURST_MINUS_18).abs() < f32::EPSILON,
            "At -18C should return minus-18 tier"
        );
        assert!(
            (pipe_burst_probability(-20.0) - PIPE_BURST_MINUS_18).abs() < f32::EPSILON,
            "Between -18C and -23C should return minus-18 tier"
        );
        assert!(
            (pipe_burst_probability(-22.9) - PIPE_BURST_MINUS_18).abs() < f32::EPSILON,
            "Just above -23C should return minus-18 tier"
        );
    }

    #[test]
    fn test_pipe_burst_below_minus_23() {
        assert!(
            (pipe_burst_probability(-23.0) - PIPE_BURST_MINUS_23).abs() < f32::EPSILON,
            "At -23C should return minus-23 tier"
        );
        assert!(
            (pipe_burst_probability(-30.0) - PIPE_BURST_MINUS_23).abs() < f32::EPSILON,
            "Well below -23C should return minus-23 tier"
        );
    }

    #[test]
    fn test_pipe_burst_monotonically_increasing() {
        let temps = [10.0, 0.0, -7.0, -18.0, -23.0, -30.0];
        let mut prev = 0.0f32;
        for &temp in &temps {
            let prob = pipe_burst_probability(temp);
            assert!(
                prob >= prev,
                "Probability should increase as temp drops: at {}C got {} < {}",
                temp,
                prob,
                prev
            );
            prev = prob;
        }
    }

    // -----------------------------------------------------------------------
    // Water main estimation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_estimate_water_main_miles() {
        let miles = estimate_water_main_miles(5000);
        assert!(
            (miles - 15.0).abs() < 0.01,
            "5000 road cells should be ~15 miles, got {}",
            miles
        );
    }

    #[test]
    fn test_estimate_water_main_miles_zero() {
        assert!(estimate_water_main_miles(0).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Water service modifier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_water_service_no_bursts() {
        let service = water_service_from_bursts(0, 15.0);
        assert!(
            (service - 1.0).abs() < f32::EPSILON,
            "No bursts should give full service, got {}",
            service
        );
    }

    #[test]
    fn test_water_service_some_bursts() {
        // 10 bursts * 0.5 miles each = 5 miles affected out of 15 total
        // 1.0 - 5/15 = 1.0 - 0.333 = 0.667
        let service = water_service_from_bursts(10, 15.0);
        assert!(
            (service - 0.667).abs() < 0.01,
            "10 bursts on 15 miles should be ~0.667, got {}",
            service
        );
    }

    #[test]
    fn test_water_service_clamped_minimum() {
        // Many bursts should still not go below 0.2
        let service = water_service_from_bursts(1000, 15.0);
        assert!(
            (service - 0.2).abs() < f32::EPSILON,
            "Water service should not go below 0.2, got {}",
            service
        );
    }

    #[test]
    fn test_water_service_zero_miles() {
        let service = water_service_from_bursts(10, 0.0);
        assert!(
            (service - 1.0).abs() < f32::EPSILON,
            "Zero water main miles should give full service, got {}",
            service
        );
    }

    // -----------------------------------------------------------------------
    // Pipe burst calculation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_calculate_pipe_bursts_deterministic() {
        let a = calculate_pipe_bursts(-20.0, 15.0, 42);
        let b = calculate_pipe_bursts(-20.0, 15.0, 42);
        assert_eq!(a, b, "Same seed should produce same result");
    }

    #[test]
    fn test_calculate_pipe_bursts_different_seeds() {
        // With different seeds, results may differ (not guaranteed, but likely for many calls)
        let mut results = std::collections::HashSet::new();
        for seed in 0..100u64 {
            results.insert(calculate_pipe_bursts(-20.0, 15.0, seed));
        }
        // With 100 different seeds at high probability (0.05), we should see variation
        assert!(
            results.len() > 1,
            "Different seeds should produce varying results"
        );
    }

    #[test]
    fn test_calculate_pipe_bursts_above_freezing() {
        // At 10C with baseline probability 0.0001, 15 miles is very unlikely to burst
        let mut total_bursts = 0u32;
        for seed in 0..100u64 {
            total_bursts += calculate_pipe_bursts(10.0, 15.0, seed);
        }
        // 100 runs * 15 segments * 0.0001 probability = ~0.15 expected total
        // Allow up to 5 for statistical variation
        assert!(
            total_bursts < 5,
            "Above freezing should have very few bursts, got {}",
            total_bursts
        );
    }

    // -----------------------------------------------------------------------
    // Deterministic PRNG tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_splitmix64_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        assert_ne!(splitmix64(42), splitmix64(43));
    }

    #[test]
    fn test_rand_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_f32(seed);
            assert!(
                (0.0..1.0).contains(&val),
                "rand_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }
}
