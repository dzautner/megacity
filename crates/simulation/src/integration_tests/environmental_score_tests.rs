//! Integration tests for the Environmental Score aggregate metric (POLL-021).

use crate::coal_power::PowerPlant;
use crate::environmental_score::EnvironmentalScore;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;
use crate::trees::TreeGrid;
use crate::water_pollution::WaterPollutionGrid;

// ====================================================================
// Resource existence
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
// Air quality responds to pollution
// ====================================================================

#[test]
fn test_air_quality_degrades_with_pollution() {
    let mut city = TestCity::new();

    // Inject pollution into the grid
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<PollutionGrid>();
        for v in grid.levels.iter_mut() {
            *v = 128;
        }
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.air_quality < 60.0,
        "polluted city should have low air quality, got {}",
        score.air_quality
    );
}

// ====================================================================
// Water quality responds to contamination
// ====================================================================

#[test]
fn test_water_quality_degrades_with_contamination() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WaterPollutionGrid>();
        for v in grid.levels.iter_mut() {
            *v = 200;
        }
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.water_quality < 30.0,
        "contaminated city should have low water quality, got {}",
        score.water_quality
    );
}

// ====================================================================
// Noise score responds to noise pollution
// ====================================================================

#[test]
fn test_noise_score_degrades_with_noise() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<NoisePollutionGrid>();
        for v in grid.levels.iter_mut() {
            *v = 80;
        }
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.noise < 30.0,
        "noisy city should have low noise score, got {}",
        score.noise
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
// Overall score is weighted combination
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

#[test]
fn test_overall_score_degrades_with_all_pollution() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        for v in pollution.levels.iter_mut() {
            *v = 255;
        }
        let mut water = world.resource_mut::<WaterPollutionGrid>();
        for v in water.levels.iter_mut() {
            *v = 255;
        }
        let mut noise = world.resource_mut::<NoisePollutionGrid>();
        for v in noise.levels.iter_mut() {
            *v = 100;
        }
    }

    city.tick_slow_cycle();

    let score = city.resource::<EnvironmentalScore>();
    assert!(
        score.overall < 30.0,
        "heavily polluted city should have low overall score, got {}",
        score.overall
    );
}
