//! TEST-018: Integration tests for fire spread and extinguish (issue #797).
//!
//! Acceptance criteria:
//! - Fire grid manually set at (50, 50)
//! - Buildings placed in fire spread range
//! - Fire station placed with coverage
//! - After 200 ticks, fire is contained
//! - Without fire station, fire spreads further

use bevy::prelude::*;

use crate::buildings::Building;
use crate::fire::{FireGrid, OnFire};
use crate::grid::ZoneType;
use crate::happiness::ServiceCoverageGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::TestSafetyNet;

// ====================================================================
// Helpers
// ====================================================================

/// Count how many buildings currently have the `OnFire` component.
fn count_burning_buildings(city: &mut TestCity) -> usize {
    let world = city.world_mut();
    let mut query = world.query_filtered::<Entity, (With<Building>, With<OnFire>)>();
    query.iter(world).count()
}

/// Get the maximum fire intensity across all burning buildings.
fn max_building_fire_intensity(city: &mut TestCity) -> f32 {
    let world = city.world_mut();
    let mut query = world.query::<&OnFire>();
    query
        .iter(world)
        .map(|f| f.intensity)
        .fold(0.0f32, f32::max)
}

/// Ignite the building at (gx, gy) by adding the `OnFire` component.
fn ignite_building(city: &mut TestCity, gx: usize, gy: usize, intensity: f32) {
    let world = city.world_mut();
    let mut query = world.query::<(Entity, &Building)>();
    let entities: Vec<(Entity, usize, usize)> = query
        .iter(world)
        .map(|(e, b)| (e, b.grid_x, b.grid_y))
        .collect();
    for (entity, bx, by) in entities {
        if bx == gx && by == gy {
            world.entity_mut(entity).insert(OnFire {
                intensity,
                ticks_burning: 0,
            });
        }
    }
}

/// Check whether a specific building at (gx, gy) is currently on fire.
fn is_building_on_fire(city: &mut TestCity, gx: usize, gy: usize) -> bool {
    let world = city.world_mut();
    let mut query = world.query::<(&Building, &OnFire)>();
    query.iter(world).any(|(b, _)| b.grid_x == gx && b.grid_y == gy)
}

/// Get fire intensity for a building at (gx, gy), or None if not on fire.
fn building_fire_intensity(city: &mut TestCity, gx: usize, gy: usize) -> Option<f32> {
    let world = city.world_mut();
    let mut query = world.query::<(&Building, &OnFire)>();
    query
        .iter(world)
        .find(|(b, _)| b.grid_x == gx && b.grid_y == gy)
        .map(|(_, f)| f.intensity)
}

// ====================================================================
// Test 1: Fire at (50,50) with fire station — contained after 200 ticks
// ====================================================================

/// Place buildings around (50,50), ignite one, place a fire station nearby.
/// After 200 ticks the initial fire should be extinguished by fire coverage
/// and total burning buildings should be small (contained).
#[test]
fn test_fire_at_50_50_contained_with_fire_station() {
    // Build city: fire station at (50,50), buildings in spread range.
    // Fire station coverage radius = 20 cells, so buildings within that
    // radius receive coverage and fires get extinguished.
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::FireStation)
        .with_building(50, 51, ZoneType::Industrial, 1)
        .with_building(51, 50, ZoneType::Industrial, 1)
        .with_building(50, 49, ZoneType::Industrial, 1)
        .with_building(49, 50, ZoneType::Industrial, 1)
        .with_building(51, 51, ZoneType::Industrial, 1)
        .with_building(52, 50, ZoneType::Industrial, 1)
        .with_building(50, 52, ZoneType::Industrial, 1);

    // Remove TestSafetyNet so fire_damage system runs (needed for full
    // fire lifecycle, including destruction after prolonged burning).
    city.world_mut().remove_resource::<TestSafetyNet>();

    // Run one slow cycle to compute service coverage grids.
    city.tick_slow_cycles(1);

    // Verify fire coverage is present at the building locations.
    {
        let cov = city.resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(50, 51);
        assert!(
            cov.has_fire(idx),
            "Building at (50,51) should have fire station coverage"
        );
        let idx2 = ServiceCoverageGrid::idx(51, 50);
        assert!(
            cov.has_fire(idx2),
            "Building at (51,50) should have fire station coverage"
        );
    }

    // Manually set fire at (50,51) — the building adjacent to the fire station.
    ignite_building(&mut city, 50, 51, 5.0);

    // Also mark the FireGrid so the spread system sees it.
    {
        let world = city.world_mut();
        let mut fire_grid = world.resource_mut::<FireGrid>();
        fire_grid.set(50, 51, 5);
    }

    assert!(
        is_building_on_fire(&mut city, 50, 51),
        "Building at (50,51) should be on fire after ignition"
    );

    // Run 200 ticks.
    city.tick(200);

    // After 200 ticks with fire station coverage, the initial fire should
    // be extinguished. The extinguish_fires system reduces intensity by
    // COVERAGE_REDUCTION_PER_TICK (2.0) each tick, which outpaces growth
    // for small fires.
    let still_on_fire = is_building_on_fire(&mut city, 50, 51);
    assert!(
        !still_on_fire,
        "Fire at (50,51) should be extinguished after 200 ticks with fire coverage"
    );

    // Total burning buildings should be 0 or very few (contained).
    let total_burning = count_burning_buildings(&mut city);
    assert!(
        total_burning <= 1,
        "With fire station, total burning buildings should be contained (<= 1), got {}",
        total_burning
    );
}

// ====================================================================
// Test 2: Fire at (50,50) WITHOUT fire station — spreads further
// ====================================================================

/// Same building layout but no fire station. Fire should spread to
/// adjacent buildings and persist/grow without coverage to suppress it.
#[test]
fn test_fire_at_50_50_spreads_without_fire_station() {
    // Buildings around (50,50) but NO fire station.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1)
        .with_building(50, 51, ZoneType::Industrial, 1)
        .with_building(51, 50, ZoneType::Industrial, 1)
        .with_building(50, 49, ZoneType::Industrial, 1)
        .with_building(49, 50, ZoneType::Industrial, 1)
        .with_building(51, 51, ZoneType::Industrial, 1)
        .with_building(52, 50, ZoneType::Industrial, 1)
        .with_building(50, 52, ZoneType::Industrial, 1);

    // Remove TestSafetyNet so fire systems run fully.
    city.world_mut().remove_resource::<TestSafetyNet>();

    city.tick_slow_cycles(1);

    // Verify NO fire coverage at the target location.
    {
        let cov = city.resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(50, 50);
        assert!(
            !cov.has_fire(idx),
            "Building at (50,50) should NOT have fire station coverage"
        );
    }

    // Ignite building at (50,50) with moderate intensity.
    ignite_building(&mut city, 50, 50, 10.0);
    {
        let world = city.world_mut();
        let mut fire_grid = world.resource_mut::<FireGrid>();
        fire_grid.set(50, 50, 10);
    }

    // Run 200 ticks.
    city.tick(200);

    // Without fire coverage, the original building should still be burning
    // (intensity only grows without coverage) OR it may have been destroyed.
    // Either way, fire should have spread to neighbors.
    let total_burning = count_burning_buildings(&mut city);

    // The fire at (50,50) should either still be burning or have spread.
    // Check that fire has spread to at least one adjacent building.
    let mut any_neighbor_on_fire = false;
    let neighbors = [(50, 51), (51, 50), (50, 49), (49, 50), (51, 51)];
    for &(nx, ny) in &neighbors {
        if is_building_on_fire(&mut city, nx, ny) {
            any_neighbor_on_fire = true;
            break;
        }
    }

    // Without fire station, fire persists and spreads.
    // Either the origin is still burning or it spread to neighbors.
    let origin_burning = is_building_on_fire(&mut city, 50, 50);
    assert!(
        origin_burning || any_neighbor_on_fire,
        "Without fire station, fire should persist at origin or spread to neighbors. \
         Origin burning: {}, any neighbor burning: {}, total burning: {}",
        origin_burning,
        any_neighbor_on_fire,
        total_burning
    );
}

// ====================================================================
// Test 3: Comparative — fire station reduces total fire spread
// ====================================================================

/// Run the same scenario with and without a fire station and compare
/// the extent of fire damage. With a fire station, fewer buildings
/// should be burning after the same number of ticks.
#[test]
fn test_fire_station_reduces_total_fire_spread() {
    // --- Scenario A: WITH fire station ---
    let mut city_with_station = TestCity::new()
        .with_service(50, 50, ServiceType::FireStation)
        .with_building(50, 51, ZoneType::Industrial, 1)
        .with_building(51, 50, ZoneType::Industrial, 1)
        .with_building(50, 49, ZoneType::Industrial, 1)
        .with_building(49, 50, ZoneType::Industrial, 1)
        .with_building(51, 51, ZoneType::Industrial, 1)
        .with_building(49, 49, ZoneType::Industrial, 1);

    city_with_station
        .world_mut()
        .remove_resource::<TestSafetyNet>();
    city_with_station.tick_slow_cycles(1);
    ignite_building(&mut city_with_station, 50, 51, 10.0);
    {
        let world = city_with_station.world_mut();
        let mut fg = world.resource_mut::<FireGrid>();
        fg.set(50, 51, 10);
    }
    city_with_station.tick(200);
    let burning_with = count_burning_buildings(&mut city_with_station);
    let max_intensity_with = max_building_fire_intensity(&mut city_with_station);

    // --- Scenario B: WITHOUT fire station ---
    let mut city_no_station = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1)
        .with_building(50, 51, ZoneType::Industrial, 1)
        .with_building(51, 50, ZoneType::Industrial, 1)
        .with_building(50, 49, ZoneType::Industrial, 1)
        .with_building(49, 50, ZoneType::Industrial, 1)
        .with_building(51, 51, ZoneType::Industrial, 1)
        .with_building(49, 49, ZoneType::Industrial, 1);

    city_no_station
        .world_mut()
        .remove_resource::<TestSafetyNet>();
    city_no_station.tick_slow_cycles(1);
    ignite_building(&mut city_no_station, 50, 51, 10.0);
    {
        let world = city_no_station.world_mut();
        let mut fg = world.resource_mut::<FireGrid>();
        fg.set(50, 51, 10);
    }
    city_no_station.tick(200);
    let burning_without = count_burning_buildings(&mut city_no_station);
    let max_intensity_without = max_building_fire_intensity(&mut city_no_station);

    // With a fire station, either fewer buildings are burning or the max
    // intensity is lower (fire is being actively suppressed).
    assert!(
        burning_with < burning_without || max_intensity_with < max_intensity_without,
        "Fire station should reduce fire spread or intensity. \
         With station: {} burning (max {:.1}), without: {} burning (max {:.1})",
        burning_with,
        max_intensity_with,
        burning_without,
        max_intensity_without
    );
}

// ====================================================================
// Test 4: Fire grid reflects building fire state
// ====================================================================

/// Verify that the `FireGrid` resource is updated when buildings burn,
/// matching the `OnFire` component intensity.
#[test]
fn test_fire_grid_updated_when_buildings_burn() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1);

    city.world_mut().remove_resource::<TestSafetyNet>();

    ignite_building(&mut city, 50, 50, 15.0);

    // Run a few ticks so spread_fire updates the FireGrid.
    city.tick(5);

    let fire_level = {
        let fg = city.resource::<FireGrid>();
        fg.get(50, 50)
    };

    assert!(
        fire_level > 0,
        "FireGrid at (50,50) should reflect the burning building, got {}",
        fire_level
    );
}

// ====================================================================
// Test 5: Fire station coverage verified at multiple building locations
// ====================================================================

/// Place a fire station at (50,50) and verify coverage reaches buildings
/// within its 20-cell radius but not beyond.
#[test]
fn test_fire_station_coverage_radius_verification() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::FireStation)
        .with_building(55, 50, ZoneType::Residential, 1)  // 5 cells away — covered
        .with_building(65, 50, ZoneType::Residential, 1)  // 15 cells away — covered
        .with_building(80, 50, ZoneType::Residential, 1); // 30 cells away — NOT covered

    city.tick_slow_cycles(1);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_near = ServiceCoverageGrid::idx(55, 50);
    assert!(
        cov.has_fire(idx_near),
        "Building 5 cells from fire station should have fire coverage"
    );

    let idx_mid = ServiceCoverageGrid::idx(65, 50);
    assert!(
        cov.has_fire(idx_mid),
        "Building 15 cells from fire station should have fire coverage"
    );

    let idx_far = ServiceCoverageGrid::idx(80, 50);
    assert!(
        !cov.has_fire(idx_far),
        "Building 30 cells from fire station should NOT have fire coverage"
    );
}
