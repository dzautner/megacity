//! Integration tests for the City Hall Administration Efficiency system (SVC-012).

use crate::city_hall::{CityHallState, CityHallTier};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::services::ServiceType;
use crate::stats::CityStats;
use crate::test_harness::TestCity;

#[test]
fn test_city_hall_state_initializes_with_defaults() {
    let city = TestCity::new();
    let state = city.resource::<CityHallState>();
    assert_eq!(state.city_hall_count, 0);
    assert!((state.admin_efficiency).abs() < f32::EPSILON);
    assert!((state.civic_pride_bonus).abs() < f32::EPSILON);
    assert!((state.corruption).abs() < f32::EPSILON);
}

#[test]
fn test_city_hall_counts_buildings() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::CityHall);

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert_eq!(
        state.city_hall_count, 1,
        "Should count 1 city hall building"
    );
}

#[test]
fn test_city_hall_efficiency_with_no_population() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::CityHall);

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    // No population but city hall exists: max efficiency
    assert!(
        state.admin_efficiency >= 1.5,
        "City hall with no population should have high efficiency, got {}",
        state.admin_efficiency
    );
}

#[test]
fn test_city_hall_understaffed_reduces_construction_speed() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::CityHall);

    // Set a large population so the single small city hall is understaffed
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 200_000;
    }

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        state.construction_speed_multiplier < 1.0,
        "Understaffed city hall should reduce construction speed, got {}",
        state.construction_speed_multiplier
    );
    assert!(
        state.construction_speed_multiplier >= 0.75,
        "Construction speed should not go below 0.75, got {}",
        state.construction_speed_multiplier
    );
}

#[test]
fn test_city_hall_understaffed_reduces_tax_revenue() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::CityHall);

    // Large population means understaffed
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 200_000;
    }

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        state.tax_revenue_multiplier < 1.0,
        "Understaffed city hall should reduce tax revenue, got {}",
        state.tax_revenue_multiplier
    );
    assert!(
        state.tax_revenue_multiplier >= 0.90,
        "Tax revenue should not go below 0.90, got {}",
        state.tax_revenue_multiplier
    );
}

#[test]
fn test_city_hall_central_location_provides_happiness() {
    // Place city hall at grid center
    let center_x = GRID_WIDTH / 2;
    let center_y = GRID_HEIGHT / 2;

    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(center_x, center_y, ServiceType::CityHall);

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        state.civic_pride_bonus > 4.0,
        "Central city hall should give near-max civic pride bonus, got {}",
        state.civic_pride_bonus
    );
}

#[test]
fn test_city_hall_edge_location_reduces_civic_pride() {
    // Place city hall at corner
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(1, 1, ServiceType::CityHall);

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        state.civic_pride_bonus < 1.0,
        "Edge city hall should have low civic pride, got {}",
        state.civic_pride_bonus
    );
}

#[test]
fn test_growing_city_without_upgrade_shows_declining_efficiency() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(128, 128, ServiceType::CityHall);

    // Small population: high efficiency
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 1_000;
    }
    city.tick_slow_cycle();

    let efficiency_small = city.resource::<CityHallState>().admin_efficiency;

    // Medium population: lower efficiency with same single city hall
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 50_000;
    }
    city.tick_slow_cycle();

    let efficiency_medium = city.resource::<CityHallState>().admin_efficiency;

    // Large population: even lower efficiency
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 200_000;
    }
    city.tick_slow_cycle();

    let efficiency_large = city.resource::<CityHallState>().admin_efficiency;

    assert!(
        efficiency_small > efficiency_medium,
        "Efficiency should decrease as population grows: small={efficiency_small} > medium={efficiency_medium}"
    );
    assert!(
        efficiency_medium > efficiency_large,
        "Efficiency should decrease further: medium={efficiency_medium} > large={efficiency_large}"
    );
}

#[test]
fn test_city_hall_tier_matches_population() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(128, 128, ServiceType::CityHall);

    // Small tier
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 10_000;
    }
    city.tick_slow_cycle();
    assert_eq!(
        city.resource::<CityHallState>().current_tier,
        CityHallTier::Small
    );

    // Medium tier
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 50_000;
    }
    city.tick_slow_cycle();
    assert_eq!(
        city.resource::<CityHallState>().current_tier,
        CityHallTier::Medium
    );

    // Large tier
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 150_000;
    }
    city.tick_slow_cycle();
    assert_eq!(
        city.resource::<CityHallState>().current_tier,
        CityHallTier::Large
    );
}

#[test]
fn test_no_city_hall_with_population_shows_corruption() {
    let mut city = TestCity::new().with_budget(100_000.0);

    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 50_000;
    }

    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        (state.corruption - 1.0).abs() < f32::EPSILON,
        "No city hall with population should have max corruption, got {}",
        state.corruption
    );
}

#[test]
fn test_well_staffed_city_hall_has_no_corruption() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(128, 128, ServiceType::CityHall);

    // Very small population relative to city hall capacity
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 100;
    }
    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        state.corruption < f32::EPSILON,
        "Well-staffed city hall should have no corruption, got {}",
        state.corruption
    );
}

#[test]
fn test_city_hall_high_efficiency_boosts_construction() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(128, 128, ServiceType::CityHall);

    // Very small population => over-staffed
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 100;
    }
    city.tick_slow_cycle();

    let state = city.resource::<CityHallState>();
    assert!(
        state.construction_speed_multiplier > 1.0,
        "Over-staffed city hall should boost construction, got {}",
        state.construction_speed_multiplier
    );
}

#[test]
fn test_city_hall_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = CityHallState::default();
    state.city_hall_count = 3;
    state.admin_efficiency = 0.75;
    state.civic_pride_bonus = 4.2;
    state.corruption = 0.13;
    state.current_tier = CityHallTier::Medium;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = CityHallState::load_from_bytes(&bytes);

    assert_eq!(restored.city_hall_count, 3);
    assert!((restored.admin_efficiency - 0.75).abs() < f32::EPSILON);
    assert!((restored.civic_pride_bonus - 4.2).abs() < f32::EPSILON);
    assert!((restored.corruption - 0.13).abs() < f32::EPSILON);
    assert_eq!(restored.current_tier, CityHallTier::Medium);
}
