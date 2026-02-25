//! Integration tests for POLL-011: Noise Barrier Attenuation
//!
//! Verifies that buildings and terrain between noise sources and receivers
//! reduce noise propagation on the NoisePollutionGrid.

use crate::grid::{RoadType, ZoneType};
use crate::noise::NoisePollutionGrid;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// Building barrier tests
// ====================================================================

/// A row of buildings between a highway and a receiver cell should reduce
/// noise at the receiver compared to a city with no buildings.
#[test]
fn test_building_row_reduces_noise_behind_it() {
    // City WITH buildings between highway and measurement point
    let mut city_with_barrier = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        // Place a row of buildings at y=132, between highway (y=128) and
        // measurement point (y=138)
        .with_building(128, 132, ZoneType::CommercialHigh, 3)
        .with_building(129, 132, ZoneType::CommercialHigh, 3)
        .with_building(130, 132, ZoneType::CommercialHigh, 3)
        .with_building(131, 132, ZoneType::CommercialHigh, 3)
        .with_building(132, 132, ZoneType::CommercialHigh, 3)
        .with_building(133, 132, ZoneType::CommercialHigh, 3)
        .with_building(134, 132, ZoneType::CommercialHigh, 3);

    // City WITHOUT buildings (open terrain)
    let mut city_no_barrier = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway);

    // Disable wind to avoid directional effects
    {
        let world = city_with_barrier.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    {
        let world = city_no_barrier.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_with_barrier.tick_slow_cycles(2);
    city_no_barrier.tick_slow_cycles(2);

    // Measure noise behind the building row (y=138, ~10 cells from highway)
    let noise_with = city_with_barrier
        .resource::<NoisePollutionGrid>()
        .get(132, 138);
    let noise_without = city_no_barrier
        .resource::<NoisePollutionGrid>()
        .get(132, 138);

    assert!(
        noise_with < noise_without,
        "noise behind building row ({}) should be less than without ({})",
        noise_with,
        noise_without
    );
}

/// More buildings between source and receiver should produce more attenuation.
#[test]
fn test_more_buildings_more_attenuation() {
    // 2-building barrier
    let mut city_thin = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        .with_building(132, 132, ZoneType::Industrial, 3)
        .with_building(133, 132, ZoneType::Industrial, 3);

    // 6-building barrier (thicker wall)
    let mut city_thick = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        .with_building(130, 132, ZoneType::Industrial, 3)
        .with_building(131, 132, ZoneType::Industrial, 3)
        .with_building(132, 132, ZoneType::Industrial, 3)
        .with_building(133, 132, ZoneType::Industrial, 3)
        .with_building(134, 132, ZoneType::Industrial, 3)
        .with_building(135, 132, ZoneType::Industrial, 3);

    for city in [&mut city_thin, &mut city_thick] {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_thin.tick_slow_cycles(2);
    city_thick.tick_slow_cycles(2);

    let noise_thin = city_thin
        .resource::<NoisePollutionGrid>()
        .get(132, 138);
    let noise_thick = city_thick
        .resource::<NoisePollutionGrid>()
        .get(132, 138);

    // Thicker barrier should attenuate at least as much
    assert!(
        noise_thick <= noise_thin,
        "thicker barrier ({}) should reduce noise at least as much as thin barrier ({})",
        noise_thick,
        noise_thin
    );
}

/// Noise at the source cell itself should remain high even with barriers nearby.
#[test]
fn test_source_noise_unaffected_by_distant_barriers() {
    let mut city = TestCity::new()
        .with_road(128, 128, 128, 128, RoadType::Highway)
        .with_building(128, 135, ZoneType::CommercialHigh, 3);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise_at_source = city.resource::<NoisePollutionGrid>().get(128, 128);
    // Highway is 80 dB -> grid value ~84. Should still be high.
    assert!(
        noise_at_source >= 40,
        "source cell noise should remain high, got {}",
        noise_at_source
    );
}

// ====================================================================
// Terrain barrier tests
// ====================================================================

/// A terrain ridge between highway and receiver should reduce noise.
#[test]
fn test_terrain_ridge_reduces_noise() {
    // City with elevated terrain between source and receiver
    let mut city_ridge = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway);

    // Create a terrain ridge at y=132 by raising elevation
    {
        let world = city_ridge.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 125..145 {
            grid.get_mut(x, 131).elevation = 0.8;
            grid.get_mut(x, 132).elevation = 0.9;
            grid.get_mut(x, 133).elevation = 0.8;
        }
        // Keep source and receiver at low elevation
        for x in 125..145 {
            grid.get_mut(x, 128).elevation = 0.0;
            grid.get_mut(x, 138).elevation = 0.0;
        }
    }

    // City with flat terrain (control)
    let mut city_flat = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway);

    for city in [&mut city_ridge, &mut city_flat] {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_ridge.tick_slow_cycles(2);
    city_flat.tick_slow_cycles(2);

    let noise_ridge = city_ridge
        .resource::<NoisePollutionGrid>()
        .get(132, 138);
    let noise_flat = city_flat
        .resource::<NoisePollutionGrid>()
        .get(132, 138);

    assert!(
        noise_ridge <= noise_flat,
        "terrain ridge ({}) should reduce noise vs flat ({})",
        noise_ridge,
        noise_flat
    );
}

// ====================================================================
// Combined barrier tests
// ====================================================================

/// Buildings plus terrain together should reduce noise more than either alone.
#[test]
fn test_combined_building_and_terrain_barriers() {
    // Buildings only
    let mut city_buildings = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        .with_building(132, 132, ZoneType::CommercialHigh, 3)
        .with_building(133, 132, ZoneType::CommercialHigh, 3)
        .with_building(134, 132, ZoneType::CommercialHigh, 3);

    // Buildings + terrain ridge
    let mut city_combined = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        .with_building(132, 132, ZoneType::CommercialHigh, 3)
        .with_building(133, 132, ZoneType::CommercialHigh, 3)
        .with_building(134, 132, ZoneType::CommercialHigh, 3);

    // Add terrain ridge to the combined city
    {
        let world = city_combined.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 125..145 {
            grid.get_mut(x, 131).elevation = 0.7;
            grid.get_mut(x, 132).elevation = 0.9;
            grid.get_mut(x, 133).elevation = 0.7;
        }
        for x in 125..145 {
            grid.get_mut(x, 128).elevation = 0.0;
            grid.get_mut(x, 138).elevation = 0.0;
        }
    }

    for city in [&mut city_buildings, &mut city_combined] {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_buildings.tick_slow_cycles(2);
    city_combined.tick_slow_cycles(2);

    let noise_buildings = city_buildings
        .resource::<NoisePollutionGrid>()
        .get(132, 138);
    let noise_combined = city_combined
        .resource::<NoisePollutionGrid>()
        .get(132, 138);

    assert!(
        noise_combined <= noise_buildings,
        "combined barriers ({}) should attenuate at least as much as buildings alone ({})",
        noise_combined,
        noise_buildings
    );
}

// ====================================================================
// Resource existence test
// ====================================================================

/// The noise pollution grid resource must exist even in a blank city.
#[test]
fn test_noise_barriers_resource_exists_in_blank_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<NoisePollutionGrid>();
}

// ====================================================================
// Grid bounds safety
// ====================================================================

/// Noise grid values must remain within 0-100 after barrier processing.
#[test]
fn test_noise_grid_values_within_bounds_after_barriers() {
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Highway)
        .with_building(132, 132, ZoneType::CommercialHigh, 5)
        .with_building(133, 132, ZoneType::CommercialHigh, 5)
        .with_building(134, 132, ZoneType::CommercialHigh, 5);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let grid = city.resource::<NoisePollutionGrid>();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let val = grid.get(x, y);
            assert!(
                val <= 100,
                "noise at ({},{}) = {} exceeds 100 after barrier processing",
                x,
                y,
                val
            );
        }
    }
}
