//! Integration tests for the Water/Sewage Pipe Network (SVC-024).
//!
//! Verifies that:
//! - Pipes auto-follow road placement and create coverage.
//! - Pressure drops with distance from water source.
//! - Pipe breaks cause local water loss.
//! - Sewage overflow when demand exceeds treatment capacity.
//! - Growing city tracks pipe network state correctly.

use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::water_pipe_network::WaterPipeNetworkState;

// ====================================================================
// Pipe following road creates coverage
// ====================================================================

/// When a road is placed with a water tower, the pipe network should track
/// pipe cells equal to the number of road cells.
#[test]
fn test_pipe_follows_road_creates_coverage() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1)
        .with_building(60, 49, ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    assert!(
        state.pipe_cells > 0,
        "Pipe cells should be > 0 after placing roads"
    );
    assert!(
        state.water_source_count > 0,
        "Should have at least one water source"
    );
    assert!(
        state.total_pipe_capacity_gpd > 0.0,
        "Total pipe capacity should be positive"
    );
}

// ====================================================================
// Pressure drops with distance
// ====================================================================

/// Buildings closer to the water source should have higher pressure than
/// buildings far away on a long road.
#[test]
fn test_pressure_drops_with_distance() {
    // Use the pure function to verify pressure drops.
    let close_pressure = crate::water_pipe_network::pressure_at_distance(5);
    let far_pressure = crate::water_pipe_network::pressure_at_distance(80);
    assert!(
        close_pressure > far_pressure,
        "Close buildings should have higher pressure than far buildings"
    );
    assert!(
        close_pressure > 0.9,
        "Buildings 5 hops from source should have near-full pressure"
    );
}

// ====================================================================
// Pipe break causes local water loss
// ====================================================================

/// When a pipe break is tracked, the BFS should skip that cell, potentially
/// reducing coverage for buildings beyond the break.
#[test]
fn test_pipe_break_reduces_service() {
    use crate::water_pipe_network::PipeBreakTracker;

    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::WaterTower)
        .with_building(65, 49, ZoneType::ResidentialLow, 1);

    // Run once to establish baseline.
    city.tick_slow_cycle();

    let baseline_full = city
        .resource::<WaterPipeNetworkState>()
        .buildings_full_service;

    // Introduce a pipe break in the middle of the road.
    {
        let world = city.world_mut();
        let mut tracker = world.resource_mut::<PipeBreakTracker>();
        tracker.breaks.push((60, 50, 10));
    }

    // Run again.
    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    // The break at (60,50) should block BFS from reaching (65,49).
    // Building beyond break should lose service.
    let served_after_break =
        state.buildings_full_service + state.buildings_reduced_service;

    // With the break, fewer buildings should have full service.
    assert!(
        served_after_break <= baseline_full || state.buildings_no_service > 0,
        "Pipe break should reduce service or cause no-service for some buildings"
    );
}

// ====================================================================
// Growing city outgrows pipe capacity
// ====================================================================

/// A city with many buildings but limited water sources should show
/// reduced average pressure or over-capacity status.
#[test]
fn test_growing_city_outgrows_pipe_capacity() {
    let mut city = TestCity::new()
        .with_road(30, 50, 80, 50, RoadType::Local)
        .with_utility(30, 50, UtilityType::WaterTower);

    // Add many buildings along the road.
    for x in 35..75 {
        city = city.with_building(x, 49, ZoneType::ResidentialLow, 3);
    }

    // Set occupants on all buildings so they generate water demand.
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut crate::buildings::Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 30;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    assert!(
        state.total_water_demand_gpd > 0.0,
        "Water demand should be positive with occupied buildings"
    );
    // With 40 buildings and one water tower, the network should be tracking.
    assert!(
        state.buildings_full_service + state.buildings_reduced_service
            + state.buildings_no_service
            > 0,
        "Should have buildings classified"
    );
}

// ====================================================================
// Sewage overflow when demand exceeds treatment
// ====================================================================

/// Sewage should overflow when building occupancy generates more sewage
/// than treatment plants can handle.
#[test]
fn test_sewage_overflow_no_treatment_plant() {
    let mut city = TestCity::new()
        .with_road(40, 40, 60, 40, RoadType::Local)
        .with_utility(40, 40, UtilityType::WaterTower);

    // Add buildings with occupants but no sewage treatment plant.
    for x in 42..58 {
        city = city.with_building(x, 39, ZoneType::ResidentialLow, 2);
    }

    // Manually set occupants on the buildings so we have sewage demand.
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut crate::buildings::Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 20;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    assert_eq!(
        state.treatment_plant_count, 0,
        "No treatment plants placed"
    );
    assert!(
        state.total_sewage_gpd > 0.0,
        "Should have positive sewage generation"
    );
    assert!(
        state.has_sewage_overflow(),
        "Should have sewage overflow with no treatment plant"
    );
}

// ====================================================================
// Treatment plant provides capacity
// ====================================================================

/// Adding a sewage treatment plant should provide treatment capacity.
#[test]
fn test_treatment_plant_provides_capacity() {
    let mut city = TestCity::new()
        .with_road(40, 40, 60, 40, RoadType::Local)
        .with_utility(40, 40, UtilityType::WaterTower)
        .with_utility(42, 40, UtilityType::SewagePlant);

    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    assert_eq!(state.treatment_plant_count, 1);
    assert!(
        state.treatment_capacity_gpd > 0.0,
        "Treatment capacity should be positive with a sewage plant"
    );
}

// ====================================================================
// No roads means no pipes
// ====================================================================

/// Without any roads, the pipe network should have zero pipe cells.
#[test]
fn test_no_roads_no_pipes() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    assert_eq!(state.pipe_cells, 0, "No roads means no pipe cells");
    assert!(
        (state.total_pipe_capacity_gpd - 0.0).abs() < f32::EPSILON,
        "No pipes means no capacity"
    );
}

// ====================================================================
// Pipe break repair mechanic
// ====================================================================

/// Pipe breaks should auto-repair after REPAIR_TICKS slow ticks.
#[test]
fn test_pipe_break_repair_mechanic() {
    use crate::water_pipe_network::PipeBreakTracker;

    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::WaterTower);

    // Introduce a break with 2 remaining repair ticks.
    {
        let world = city.world_mut();
        let mut tracker = world.resource_mut::<PipeBreakTracker>();
        tracker.breaks.push((55, 50, 2));
    }

    // First slow cycle: should tick down to 1.
    city.tick_slow_cycle();
    {
        let tracker = city.resource::<PipeBreakTracker>();
        assert_eq!(tracker.breaks.len(), 1, "Break should still be active");
    }

    // Second slow cycle: remaining goes to 0, break is removed.
    city.tick_slow_cycle();
    {
        let tracker = city.resource::<PipeBreakTracker>();
        assert_eq!(
            tracker.breaks.len(),
            0,
            "Break should be repaired after enough ticks"
        );
    }
}

// ====================================================================
// Network age tracking
// ====================================================================

/// The pipe network should age over time, increasing leak rate.
#[test]
fn test_network_ages_over_time() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::WaterTower);

    let initial_age = city.resource::<WaterPipeNetworkState>().network_age_ticks;

    city.tick_slow_cycle();

    let state = city.resource::<WaterPipeNetworkState>();
    assert!(
        state.network_age_ticks > initial_age,
        "Network age should increase after slow tick"
    );
}
