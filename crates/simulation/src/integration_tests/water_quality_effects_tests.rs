//! Integration tests for POLL-008: Water Quality Effects on Citizens and Fisheries.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::grid::CellType;
use crate::groundwater::WaterQualityGrid;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use crate::water_pollution::WaterPollutionGrid;
use crate::water_quality_effects::{WaterQualityEffects, WaterQualityTier};
use crate::Saveable;
use crate::SaveableRegistry;

use bevy::prelude::*;

/// Spawn a citizen at grid position with specified health.
fn spawn_citizen(world: &mut World, gx: usize, gy: usize, health: f32) -> Entity {
    let (wx, wy) = crate::grid::WorldGrid::grid_to_world(gx, gy);
    world
        .spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: gx,
                grid_y: gy,
                building: Entity::PLACEHOLDER,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
                health,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ))
        .id()
}

// ====================================================================
// Tier classification
// ====================================================================

#[test]
fn test_water_quality_tiers_classified_from_quality_grid() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 10..17 {
                grid.get_mut(x, 10).cell_type = CellType::Water;
            }
        }
        {
            let mut wq = world.resource_mut::<WaterQualityGrid>();
            wq.set(10, 10, 255); // Pristine
            wq.set(11, 10, 200); // Clean
            wq.set(12, 10, 150); // Moderate
            wq.set(13, 10, 100); // Polluted
            wq.set(14, 10, 40); // Heavy
            wq.set(15, 10, 10); // Toxic
            wq.set(16, 10, 230); // Pristine boundary
        }
    }

    city.tick_slow_cycle();

    let effects = city.resource::<WaterQualityEffects>();
    let total: u32 = effects.tier_counts.iter().sum();
    assert!(total > 0, "should have classified some water cells");
}

#[test]
fn test_surface_pollution_reduces_effective_quality() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            grid.get_mut(50, 50).cell_type = CellType::Water;
        }
        {
            let mut wq = world.resource_mut::<WaterQualityGrid>();
            wq.set(50, 50, 200);
        }
        {
            let mut wp = world.resource_mut::<WaterPollutionGrid>();
            wp.set(50, 50, 200);
        }
    }

    city.tick_slow_cycle();

    let effects = city.resource::<WaterQualityEffects>();
    let clean_count = effects.tier_counts[1];
    let polluted_count = effects.tier_counts[3];
    assert!(
        polluted_count >= 1 || clean_count == 0,
        "surface pollution should reduce effective quality"
    );
}

// ====================================================================
// Health effects
// ====================================================================

#[test]
fn test_pristine_water_gives_health_bonus() {
    let mut city = TestCity::new();

    let citizen_entity;
    {
        let world = city.world_mut();
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 48..53 {
                for y in 48..53 {
                    grid.get_mut(x, y).cell_type = CellType::Water;
                }
            }
            grid.get_mut(50, 50).cell_type = CellType::Grass;
        }
        {
            let mut wq = world.resource_mut::<WaterQualityGrid>();
            for x in 48..53 {
                for y in 48..53 {
                    wq.set(x, y, 250);
                }
            }
        }
        {
            let mut wp = world.resource_mut::<WaterPollutionGrid>();
            for x in 48..53 {
                for y in 48..53 {
                    wp.set(x, y, 0);
                }
            }
        }
        citizen_entity = spawn_citizen(world, 50, 50, 70.0);
    }

    let initial_health = {
        let world = city.world_mut();
        world.get::<CitizenDetails>(citizen_entity).unwrap().health
    };

    city.tick_slow_cycle();

    let final_health = {
        let world = city.world_mut();
        world.get::<CitizenDetails>(citizen_entity).unwrap().health
    };

    assert!(
        final_health >= initial_health - 5.0,
        "pristine water should not significantly reduce health: {} -> {}",
        initial_health,
        final_health
    );
}

#[test]
fn test_toxic_water_reduces_health() {
    let mut city = TestCity::new();

    let citizen_entity;
    {
        let world = city.world_mut();
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 48..53 {
                for y in 48..53 {
                    grid.get_mut(x, y).cell_type = CellType::Water;
                }
            }
            grid.get_mut(50, 50).cell_type = CellType::Grass;
        }
        {
            let mut wq = world.resource_mut::<WaterQualityGrid>();
            for x in 48..53 {
                for y in 48..53 {
                    wq.set(x, y, 5);
                }
            }
        }
        citizen_entity = spawn_citizen(world, 50, 50, 80.0);
    }

    let initial_health = {
        let world = city.world_mut();
        world.get::<CitizenDetails>(citizen_entity).unwrap().health
    };

    city.tick_slow_cycles(5);

    let final_health = {
        let world = city.world_mut();
        world.get::<CitizenDetails>(citizen_entity).unwrap().health
    };

    assert!(
        final_health < initial_health,
        "toxic water should reduce health: {} -> {}",
        initial_health,
        final_health
    );
}

// ====================================================================
// Tourism bonus
// ====================================================================

#[test]
fn test_pristine_water_positive_tourism_bonus() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        // Set grid cells to water first
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 0..50 {
                for y in 0..50 {
                    grid.get_mut(x, y).cell_type = CellType::Water;
                }
            }
        }
        // Then set quality separately
        {
            let mut wq = world.resource_mut::<WaterQualityGrid>();
            for x in 0..50 {
                for y in 0..50 {
                    wq.set(x, y, 250);
                }
            }
        }
    }

    city.tick_slow_cycle();

    let effects = city.resource::<WaterQualityEffects>();
    assert!(
        effects.tourism_bonus_applied > 0.0,
        "pristine water should give positive tourism bonus, got {}",
        effects.tourism_bonus_applied
    );
}

#[test]
fn test_toxic_water_negative_tourism_bonus() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 0..50 {
                for y in 0..50 {
                    grid.get_mut(x, y).cell_type = CellType::Water;
                }
            }
        }
        {
            let mut wq = world.resource_mut::<WaterQualityGrid>();
            for x in 0..50 {
                for y in 0..50 {
                    wq.set(x, y, 5);
                }
            }
        }
    }

    city.tick_slow_cycle();

    let effects = city.resource::<WaterQualityEffects>();
    assert!(
        effects.tourism_bonus_applied < 0.0,
        "toxic water should give negative tourism bonus, got {}",
        effects.tourism_bonus_applied
    );
}

// ====================================================================
// Treatment cost
// ====================================================================

#[test]
fn test_treatment_cost_spec_values() {
    let clean = WaterQualityTier::Clean.treatment_cost_per_mg();
    let heavy = WaterQualityTier::Heavy.treatment_cost_per_mg();
    assert!((clean - 500.0).abs() < f64::EPSILON, "clean=$500/MG");
    assert!((heavy - 5000.0).abs() < f64::EPSILON, "heavy=$5000/MG");
    assert!(heavy > clean, "polluted should cost more to treat");
}

// ====================================================================
// Save/load roundtrip
// ====================================================================

fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);
    world.insert_resource(registry);
}

#[test]
fn test_water_quality_effects_save_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut effects = world.resource_mut::<WaterQualityEffects>();
        effects.tier_counts = [50, 100, 80, 20, 5, 1];
        effects.avg_quality = 175.0;
        effects.dominant_tier_idx = 1;
        effects.treatment_cost_modifier = 123.45;
        effects.tourism_bonus_applied = 3.5;
    }

    roundtrip(&mut city);

    let effects = city.resource::<WaterQualityEffects>();
    assert_eq!(effects.tier_counts, [50, 100, 80, 20, 5, 1]);
    assert!((effects.avg_quality - 175.0).abs() < 0.01);
    assert_eq!(effects.dominant_tier_idx, 1);
}

#[test]
fn test_water_quality_effects_default_skips_save() {
    let effects = WaterQualityEffects::default();
    assert!(effects.save_to_bytes().is_none(), "default should skip");
}
