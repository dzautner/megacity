//! TEST-005: Unit tests for traffic congestion calculation.
//!
//! Tests congestion_level (TrafficGrid), BPR travel time function,
//! congestion_speed_multiplier, and traffic grid operations.
//! Verifies congestion values are always clamped to [0.0, 1.0].

use crate::grid::RoadType;
use crate::road_graph_csr::{bpr_travel_time, BPR_ALPHA, BPR_BETA};
use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;
use crate::traffic_congestion::{congestion_speed_multiplier, TrafficCongestion};

// ====================================================================
// TrafficGrid::congestion_level tests
// ====================================================================

#[test]
fn test_congestion_level_zero_density_returns_zero() {
    let city = TestCity::new();
    let traffic = city.resource::<TrafficGrid>();
    // Default density is 0 everywhere
    assert!(
        (traffic.congestion_level(10, 10) - 0.0).abs() < f32::EPSILON,
        "Zero density should produce zero congestion"
    );
}

#[test]
fn test_congestion_level_max_density_returns_one() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        // Density of 20 is the saturation point (d/20 = 1.0)
        traffic.set(10, 10, 20);
    }
    let traffic = city.resource::<TrafficGrid>();
    assert!(
        (traffic.congestion_level(10, 10) - 1.0).abs() < f32::EPSILON,
        "Density 20 should produce congestion 1.0"
    );
}

#[test]
fn test_congestion_level_over_max_density_clamped_to_one() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        // Far above saturation point
        traffic.set(10, 10, 100);
    }
    let traffic = city.resource::<TrafficGrid>();
    assert!(
        (traffic.congestion_level(10, 10) - 1.0).abs() < f32::EPSILON,
        "Density above 20 should still produce congestion 1.0 (clamped)"
    );
}

#[test]
fn test_congestion_level_half_density() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, 10); // 10/20 = 0.5
    }
    let traffic = city.resource::<TrafficGrid>();
    assert!(
        (traffic.congestion_level(10, 10) - 0.5).abs() < 0.01,
        "Density 10 should produce congestion ~0.5"
    );
}

#[test]
fn test_congestion_level_quarter_density() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, 5); // 5/20 = 0.25
    }
    let traffic = city.resource::<TrafficGrid>();
    assert!(
        (traffic.congestion_level(10, 10) - 0.25).abs() < 0.01,
        "Density 5 should produce congestion ~0.25"
    );
}

#[test]
fn test_congestion_level_various_densities_in_range() {
    let mut city = TestCity::new();
    let densities: Vec<u16> = vec![0, 1, 2, 5, 10, 15, 19, 20, 25, 50, 100, 500, u16::MAX];
    for (i, &density) in densities.iter().enumerate() {
        let x = i % 256;
        let y = i / 256;
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(x, y, density);
    }
    let traffic = city.resource::<TrafficGrid>();
    for (i, &_density) in densities.iter().enumerate() {
        let x = i % 256;
        let y = i / 256;
        let level = traffic.congestion_level(x, y);
        assert!(
            (0.0..=1.0).contains(&level),
            "Congestion level must be in [0.0, 1.0], got {} for density {}",
            level,
            _density
        );
    }
}

#[test]
fn test_congestion_level_monotonically_increases_with_density() {
    let mut city = TestCity::new();
    let densities: Vec<u16> = vec![0, 1, 2, 5, 10, 15, 20];
    // Set each density in a different cell
    for (i, &density) in densities.iter().enumerate() {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(i, 0, density);
    }
    let traffic = city.resource::<TrafficGrid>();
    let levels: Vec<f32> = densities
        .iter()
        .enumerate()
        .map(|(i, _)| traffic.congestion_level(i, 0))
        .collect();
    for pair in levels.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "Congestion should be monotonically non-decreasing: {} >= {}",
            pair[1],
            pair[0]
        );
    }
}

// ====================================================================
// TrafficGrid::path_cost tests
// ====================================================================

#[test]
fn test_path_cost_empty_road_returns_base() {
    let city = TestCity::new();
    let traffic = city.resource::<TrafficGrid>();
    let cost = traffic.path_cost(10, 10);
    // base=1, congestion_penalty=0 -> cost=1
    assert_eq!(cost, 1, "Empty road should have base path cost of 1");
}

#[test]
fn test_path_cost_increases_monotonically_with_density() {
    let mut city = TestCity::new();
    let densities: Vec<u16> = vec![0, 5, 10, 15, 20];
    for (i, &density) in densities.iter().enumerate() {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(i, 0, density);
    }
    let traffic = city.resource::<TrafficGrid>();
    let costs: Vec<u32> = densities
        .iter()
        .enumerate()
        .map(|(i, _)| traffic.path_cost(i, 0))
        .collect();
    for pair in costs.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "Path cost should increase with density: {} >= {}",
            pair[1],
            pair[0]
        );
    }
}

#[test]
fn test_path_cost_with_road_type_highway_cheaper_than_local() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, 10);
    }
    let traffic = city.resource::<TrafficGrid>();
    let local_cost = traffic.path_cost_with_road(10, 10, RoadType::Local);
    let highway_cost = traffic.path_cost_with_road(10, 10, RoadType::Highway);
    assert!(
        highway_cost <= local_cost,
        "Highway should have lower or equal path cost than Local: highway={}, local={}",
        highway_cost,
        local_cost
    );
}

// ====================================================================
// BPR travel time function tests
// ====================================================================

#[test]
fn test_bpr_zero_volume_equals_free_flow() {
    let free_flow = 10.0;
    let result = bpr_travel_time(free_flow, 0.0, 100.0, BPR_ALPHA, BPR_BETA);
    assert!(
        (result - free_flow).abs() < 1e-9,
        "Zero volume should return free-flow time, got {}",
        result
    );
}

#[test]
fn test_bpr_at_capacity() {
    // At v/c = 1.0: t = t0 * (1 + 0.15 * 1^4) = t0 * 1.15
    let free_flow = 10.0;
    let result = bpr_travel_time(free_flow, 100.0, 100.0, BPR_ALPHA, BPR_BETA);
    let expected = free_flow * 1.15;
    assert!(
        (result - expected).abs() < 1e-9,
        "At capacity: expected {}, got {}",
        expected,
        result
    );
}

#[test]
fn test_bpr_over_capacity() {
    // At v/c = 2.0: t = t0 * (1 + 0.15 * 16) = t0 * 3.4
    let free_flow = 10.0;
    let result = bpr_travel_time(free_flow, 200.0, 100.0, BPR_ALPHA, BPR_BETA);
    let expected = free_flow * 3.4;
    assert!(
        (result - expected).abs() < 1e-9,
        "At 2x capacity: expected {}, got {}",
        expected,
        result
    );
}

#[test]
fn test_bpr_zero_capacity_returns_free_flow() {
    let free_flow = 10.0;
    let result = bpr_travel_time(free_flow, 50.0, 0.0, BPR_ALPHA, BPR_BETA);
    assert!(
        (result - free_flow).abs() < 1e-9,
        "Zero capacity should return free-flow time (safety), got {}",
        result
    );
}

#[test]
fn test_bpr_travel_time_always_gte_free_flow() {
    let free_flow = 10.0;
    for volume in [0.0, 10.0, 50.0, 100.0, 200.0, 500.0] {
        let result = bpr_travel_time(free_flow, volume, 100.0, BPR_ALPHA, BPR_BETA);
        assert!(
            result >= free_flow - 1e-9,
            "BPR travel time should always >= free_flow. Volume={}, result={}",
            volume,
            result
        );
    }
}

#[test]
fn test_bpr_monotonically_increasing_with_volume() {
    let free_flow = 10.0;
    let capacity = 100.0;
    let volumes = [0.0, 10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0];
    let times: Vec<f64> = volumes
        .iter()
        .map(|&v| bpr_travel_time(free_flow, v, capacity, BPR_ALPHA, BPR_BETA))
        .collect();
    for pair in times.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "BPR travel time should increase with volume: {} >= {}",
            pair[1],
            pair[0]
        );
    }
}

#[test]
fn test_bpr_nonlinear_growth_high_vc_ratios() {
    let free_flow = 10.0;
    let capacity = 100.0;
    // With beta=4, the penalty grows as (v/c)^4: strongly nonlinear
    let t_half = bpr_travel_time(free_flow, 50.0, capacity, BPR_ALPHA, BPR_BETA);
    let t_full = bpr_travel_time(free_flow, 100.0, capacity, BPR_ALPHA, BPR_BETA);
    let t_double = bpr_travel_time(free_flow, 200.0, capacity, BPR_ALPHA, BPR_BETA);

    let penalty_half_to_full = t_full - t_half;
    let penalty_full_to_double = t_double - t_full;
    assert!(
        penalty_full_to_double > penalty_half_to_full * 4.0,
        "BPR should be nonlinear: penalty from full to double ({}) should be >>4x penalty from half to full ({})",
        penalty_full_to_double,
        penalty_half_to_full
    );
}

#[test]
fn test_bpr_custom_alpha_beta() {
    // Verify the formula works with non-standard alpha/beta
    let free_flow = 10.0;
    let volume = 50.0;
    let capacity = 100.0;

    // alpha=0.5, beta=2: t = 10 * (1 + 0.5 * 0.5^2) = 10 * (1 + 0.125) = 11.25
    let result = bpr_travel_time(free_flow, volume, capacity, 0.5, 2.0);
    let expected = free_flow * (1.0 + 0.5 * (0.5_f64).powf(2.0));
    assert!(
        (result - expected).abs() < 1e-9,
        "Custom alpha/beta: expected {}, got {}",
        expected,
        result
    );
}

#[test]
fn test_bpr_different_free_flow_times() {
    // BPR should scale linearly with free-flow time
    let t1 = bpr_travel_time(5.0, 50.0, 100.0, BPR_ALPHA, BPR_BETA);
    let t2 = bpr_travel_time(10.0, 50.0, 100.0, BPR_ALPHA, BPR_BETA);
    assert!(
        (t2 - 2.0 * t1).abs() < 1e-9,
        "BPR should scale linearly with free_flow_time: 2*{} should equal {}",
        t1,
        t2
    );
}

// ====================================================================
// congestion_speed_multiplier tests
// ====================================================================

#[test]
fn test_speed_multiplier_zero_occupancy_full_speed() {
    let m = congestion_speed_multiplier(0.0);
    assert!(
        (m - 1.0).abs() < f32::EPSILON,
        "Zero occupancy should give full speed (1.0), got {}",
        m
    );
}

#[test]
fn test_speed_multiplier_half_occupancy() {
    // At 0.5: 1.0 - 0.25 = 0.75
    let m = congestion_speed_multiplier(0.5);
    assert!(
        (m - 0.75).abs() < 0.001,
        "Half occupancy should give ~0.75 speed, got {}",
        m
    );
}

#[test]
fn test_speed_multiplier_full_occupancy_hits_minimum() {
    let m = congestion_speed_multiplier(1.0);
    // At 1.0: 1.0 - 1.0 = 0.0 -> clamped to MIN_SPEED_MULTIPLIER (0.1)
    assert!(
        (m - 0.1).abs() < f32::EPSILON,
        "Full occupancy should give minimum speed (0.1), got {}",
        m
    );
}

#[test]
fn test_speed_multiplier_over_capacity_clamped() {
    let m = congestion_speed_multiplier(1.5);
    assert!(
        (m - 0.1).abs() < f32::EPSILON,
        "Over capacity should still clamp to minimum (0.1), got {}",
        m
    );
}

#[test]
fn test_speed_multiplier_always_in_valid_range() {
    let ratios = [
        0.0, 0.01, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.5, 2.0, 5.0, 10.0,
        100.0,
    ];
    for &ratio in &ratios {
        let m = congestion_speed_multiplier(ratio);
        assert!(
            (0.1..=1.0).contains(&m),
            "Speed multiplier must be in [0.1, 1.0], got {} for ratio {}",
            m,
            ratio
        );
    }
}

#[test]
fn test_speed_multiplier_monotonically_decreasing() {
    let ratios = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    for pair in ratios.windows(2) {
        let m1 = congestion_speed_multiplier(pair[0]);
        let m2 = congestion_speed_multiplier(pair[1]);
        assert!(
            m1 >= m2,
            "Speed multiplier should decrease as occupancy rises: ratio {} -> {}, but {} < {}",
            pair[0],
            pair[1],
            m1,
            m2
        );
    }
}

// ====================================================================
// TrafficCongestion resource tests
// ====================================================================

#[test]
fn test_traffic_congestion_resource_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficCongestion>();
}

#[test]
fn test_traffic_congestion_defaults_free_flow() {
    let city = TestCity::new();
    let congestion = city.resource::<TrafficCongestion>();
    // All cells should start at 1.0 (free flow)
    for x in [0, 10, 50, 128, 255] {
        for y in [0, 10, 50, 128, 255] {
            assert!(
                (congestion.get(x, y) - 1.0).abs() < f32::EPSILON,
                "Default congestion at ({},{}) should be 1.0, got {}",
                x,
                y,
                congestion.get(x, y)
            );
        }
    }
}

#[test]
fn test_traffic_congestion_out_of_bounds_returns_free_flow() {
    let city = TestCity::new();
    let congestion = city.resource::<TrafficCongestion>();
    assert!(
        (congestion.get(999, 999) - 1.0).abs() < f32::EPSILON,
        "Out of bounds should return 1.0 (free flow)"
    );
}

#[test]
fn test_traffic_congestion_set_and_get() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        congestion.set(50, 50, 0.5);
    }
    let congestion = city.resource::<TrafficCongestion>();
    assert!(
        (congestion.get(50, 50) - 0.5).abs() < f32::EPSILON,
        "Set then get should return the same value"
    );
}

// ====================================================================
// TrafficGrid operations tests
// ====================================================================

#[test]
fn test_traffic_grid_set_get_roundtrip() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(100, 100, 42);
    }
    let traffic = city.resource::<TrafficGrid>();
    assert_eq!(
        traffic.get(100, 100),
        42,
        "Set/get should roundtrip correctly"
    );
}

#[test]
fn test_traffic_grid_clear_resets_all() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, 50);
        traffic.set(20, 20, 100);
        traffic.clear();
    }
    let traffic = city.resource::<TrafficGrid>();
    assert_eq!(traffic.get(10, 10), 0, "Clear should reset to 0");
    assert_eq!(traffic.get(20, 20), 0, "Clear should reset to 0");
}

#[test]
fn test_traffic_grid_default_is_zero() {
    let city = TestCity::new();
    let traffic = city.resource::<TrafficGrid>();
    for x in [0, 50, 128, 255] {
        for y in [0, 50, 128, 255] {
            assert_eq!(
                traffic.get(x, y),
                0,
                "Default traffic density at ({},{}) should be 0",
                x,
                y
            );
        }
    }
}

#[test]
fn test_traffic_grid_saturating_add() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, u16::MAX);
        // Adding to MAX should not overflow
        let current = traffic.get(10, 10);
        traffic.set(10, 10, current.saturating_add(1));
    }
    let traffic = city.resource::<TrafficGrid>();
    assert_eq!(
        traffic.get(10, 10),
        u16::MAX,
        "Saturating add at MAX should stay at MAX"
    );
}

// ====================================================================
// Road capacity and congestion interaction tests
// ====================================================================

#[test]
fn test_road_types_have_increasing_capacity() {
    let local = RoadType::Local.capacity();
    let avenue = RoadType::Avenue.capacity();
    let boulevard = RoadType::Boulevard.capacity();
    let highway = RoadType::Highway.capacity();

    assert!(
        local < avenue,
        "Avenue capacity ({}) should exceed Local ({})",
        avenue,
        local
    );
    assert!(
        avenue < boulevard,
        "Boulevard capacity ({}) should exceed Avenue ({})",
        boulevard,
        avenue
    );
    assert!(
        boulevard < highway,
        "Highway capacity ({}) should exceed Boulevard ({})",
        highway,
        boulevard
    );
}

#[test]
fn test_higher_capacity_road_less_congested_at_same_volume() {
    // Same volume on different road types: higher capacity = less congestion
    let volume = 15.0;
    let local_ratio = volume / RoadType::Local.capacity() as f32;
    let highway_ratio = volume / RoadType::Highway.capacity() as f32;

    let local_mult = congestion_speed_multiplier(local_ratio);
    let highway_mult = congestion_speed_multiplier(highway_ratio);

    assert!(
        highway_mult > local_mult,
        "Highway ({}) should be less congested than Local ({}) at same volume",
        highway_mult,
        local_mult
    );
}

#[test]
fn test_bpr_with_road_type_capacities() {
    let free_flow = 10.0;
    let volume = 30.0;

    let t_local = bpr_travel_time(
        free_flow,
        volume,
        RoadType::Local.capacity() as f64,
        BPR_ALPHA,
        BPR_BETA,
    );
    let t_highway = bpr_travel_time(
        free_flow,
        volume,
        RoadType::Highway.capacity() as f64,
        BPR_ALPHA,
        BPR_BETA,
    );

    assert!(
        t_highway < t_local,
        "Highway BPR travel time ({}) should be less than Local ({}) at same volume",
        t_highway,
        t_local
    );
    assert!(
        t_highway >= free_flow - 1e-9,
        "Highway BPR should still be >= free_flow"
    );
}

// ====================================================================
// Integration: traffic density system with TestCity
// ====================================================================

#[test]
fn test_traffic_grid_resource_exists_in_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficGrid>();
}

#[test]
fn test_traffic_congestion_resource_exists_in_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficCongestion>();
}

#[test]
fn test_initial_city_has_zero_traffic_density() {
    let city = TestCity::new();
    let traffic = city.resource::<TrafficGrid>();
    // A freshly created city with no citizens should have zero traffic everywhere
    let total: u64 = (0..256)
        .flat_map(|y| (0..256).map(move |x| (x, y)))
        .map(|(x, y)| traffic.get(x, y) as u64)
        .sum();
    assert_eq!(
        total, 0,
        "Fresh city should have zero total traffic density"
    );
}

#[test]
fn test_initial_city_has_free_flow_everywhere() {
    let city = TestCity::new();
    let congestion = city.resource::<TrafficCongestion>();
    // Spot-check corners and center
    for (x, y) in [(0, 0), (255, 0), (0, 255), (255, 255), (128, 128)] {
        assert!(
            (congestion.get(x, y) - 1.0).abs() < f32::EPSILON,
            "Fresh city should have free flow at ({},{}), got {}",
            x,
            y,
            congestion.get(x, y)
        );
    }
}
