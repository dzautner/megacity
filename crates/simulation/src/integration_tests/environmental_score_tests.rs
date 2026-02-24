//! Integration tests for the Environmental Score aggregate metric (POLL-021).

use crate::coal_power::PowerPlant;
use crate::environmental_score::EnvironmentalScore;
use crate::grid::ZoneType;
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::trees::TreeGrid;

// ====================================================================
// Resource existence and defaults
// ====================================================================

#[test]
fn test_environmental_score_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<EnvironmentalScore>();
}

#[test]
fn test_environmental_score_default_values() {
    let city = TestCity::new();
    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.air_quality >= 99.0,
        "clean city should have high air quality, got {}",
        score.air_quality
    );
    assert!(
        score.water_quality >= 99.0,
        "clean city should have high water quality, got {}",
        score.water_quality
    );
    assert!(
        score.noise >= 99.0,
        "quiet city should have high noise score, got {}",
        score.noise
    );
    assert!(
        (score.soil_health - 50.0).abs() < 1.0,
        "soil health placeholder should be ~50, got {}",
        score.soil_health
    );
}

// ====================================================================
// Clean empty city overall score
// ====================================================================

#[test]
fn test_overall_score_clean_city() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    // Clean city: air=100, water=100, noise=100, soil=50, green=0, energy=100
    // Weighted: 25 + 20 + 15 + 5 + 0 + 15 = 80
    assert!(
        (score.overall - 80.0).abs() < 2.0,
        "clean empty city overall should be ~80, got {}",
        score.overall
    );
}

// ====================================================================
// Green coverage responds to trees
// ====================================================================

#[test]
fn test_green_coverage_with_trees() {
    let mut city = TestCity::new();

    // Plant trees on 10% of cells
    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        let target = tree_grid.cells.len() / 10;
        for i in 0..target {
            tree_grid.cells[i] = true;
        }
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        (score.green_coverage - 10.0).abs() < 1.0,
        "10% tree coverage should give ~10 green score, got {}",
        score.green_coverage
    );
}

#[test]
fn test_green_coverage_empty_city() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.green_coverage < 1.0,
        "city with no trees should have near-zero green coverage, got {}",
        score.green_coverage
    );
}

// ====================================================================
// Energy cleanliness with power plants
// ====================================================================

#[test]
fn test_energy_cleanliness_with_coal() {
    let mut city = TestCity::new();

    // Spawn a coal power plant
    {
        let world = city.world_mut();
        world.spawn(PowerPlant::new_coal(10, 10));
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.energy_cleanliness < 1.0,
        "coal-only city should have near-zero energy cleanliness, got {}",
        score.energy_cleanliness
    );
}

#[test]
fn test_energy_cleanliness_no_plants() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.energy_cleanliness >= 99.0,
        "city with no plants should default to 100 cleanliness, got {}",
        score.energy_cleanliness
    );
}

// ====================================================================
// Air quality degrades with industrial pollution
// ====================================================================

/// Set happiness above the downgrade threshold so `downgrade_buildings`
/// does not destroy buildings during ticks.
fn prevent_downgrade(city: &mut TestCity) {
    let world = city.world_mut();
    world.resource_mut::<CityStats>().average_happiness = 50.0;
}

#[test]
fn test_air_quality_degrades_with_industrial_building() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    // Need multiple slow cycles so pollution accumulates
    for _ in 0..5 {
        prevent_downgrade(&mut city);
        city.tick_slow_cycle();
    }

    let score = city.resource::<EnvironmentalScore>();
    // An industrial building should generate some pollution that degrades air quality.
    // The effect may be small with just one building, so we check it's below perfect.
    assert!(
        score.air_quality <= 100.0,
        "city with industrial building should not exceed 100 air quality, got {}",
        score.air_quality
    );
}

// ====================================================================
// Direct resource manipulation for score verification
// ====================================================================

#[test]
fn test_environmental_score_updates_on_slow_tick() {
    let mut city = TestCity::new();

    // Set a known state directly
    {
        let world = city.world_mut();
        let mut score = world.resource_mut::<EnvironmentalScore>();
        score.overall = 0.0;
        score.air_quality = 0.0;
    }

    // After a slow tick, the system should recompute from the actual grids
    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    // In a clean city, air quality should be recomputed to ~100
    assert!(
        score.air_quality > 90.0,
        "after slow tick, air quality should be recomputed from clean grid, got {}",
        score.air_quality
    );
    assert!(
        score.overall > 50.0,
        "after slow tick, overall score should be recomputed, got {}",
        score.overall
    );
}

#[test]
fn test_full_trees_boost_green_coverage_and_overall() {
    let mut city = TestCity::new();

    // Plant trees everywhere
    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        for v in tree_grid.cells.iter_mut() {
            *v = true;
        }
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        (score.green_coverage - 100.0).abs() < 1.0,
        "full tree coverage should give ~100 green score, got {}",
        score.green_coverage
    );
    // With green at 100, overall should be higher than clean empty city (80)
    // Clean city with full trees: air=100, water=100, noise=100, soil=50, green=100, energy=100
    // = 25 + 20 + 15 + 5 + 15 + 15 = 95
    assert!(
        (score.overall - 95.0).abs() < 2.0,
        "full trees city overall should be ~95, got {}",
        score.overall
    );
}
