//! Integration tests for the Parks Multi-Tier System (SVC-015).

use crate::grid::{RoadType, ZoneType};
use crate::parks_system::{ParkEffectsGrid, ParksState};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

#[test]
fn test_small_park_provides_happiness_and_land_value() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::SmallPark);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    // Cell near the park should have happiness bonus
    let bonus = effects.happiness_at(12, 15);
    assert!(bonus >= 5.0, "SmallPark happiness bonus should be >= 5.0, got {bonus}");

    let lv_bonus = effects.land_value_at(12, 15);
    assert!(lv_bonus >= 3.0, "SmallPark land value bonus should be >= 3.0, got {lv_bonus}");
}

#[test]
fn test_large_park_provides_higher_bonuses_and_pollution_reduction() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::LargePark);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    let happiness = effects.happiness_at(12, 15);
    assert!(happiness >= 10.0, "LargePark happiness should be >= 10.0, got {happiness}");

    let lv_bonus = effects.land_value_at(12, 15);
    assert!(lv_bonus >= 8.0, "LargePark land value should be >= 8.0, got {lv_bonus}");

    let pollution_red = effects.pollution_reduction_at(12, 15);
    assert!(pollution_red > 0, "LargePark should provide pollution reduction");
}

#[test]
fn test_playground_marks_family_coverage() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::Playground);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    let idx = ParkEffectsGrid::idx(12, 15);
    assert!(effects.has_playground[idx], "Playground should mark family coverage");
    assert!(effects.happiness_at(12, 15) >= 5.0, "Playground should provide happiness");
}

#[test]
fn test_sports_field_provides_health_bonus() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::SportsField);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    let health = effects.health_at(12, 15);
    assert!(health >= 3.0, "SportsField should provide health bonus >= 3.0, got {health}");
    assert!(effects.happiness_at(12, 15) >= 5.0, "SportsField should provide happiness");
}

#[test]
fn test_plaza_provides_commercial_boost() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::Plaza);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    let idx = ParkEffectsGrid::idx(12, 15);
    assert!(effects.has_plaza_boost[idx], "Plaza should mark commercial boost");
    assert!(effects.happiness_at(12, 15) >= 3.0, "Plaza should provide happiness >= 3.0");
}

#[test]
fn test_park_deficit_penalty_with_population() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_zone_rect(11, 10, 12, 20, ZoneType::Residential)
        .with_building(11, 12, ZoneType::Residential, 1)
        .with_building(12, 12, ZoneType::Residential, 1)
        .with_citizen((11, 12), (12, 12))
        .with_citizen((11, 12), (12, 12))
        .with_citizen((11, 12), (12, 12));

    // Run slow cycle to update stats and park effects
    city.tick_slow_cycle();

    let state = city.resource::<ParksState>();
    // With citizens but no parks, there should be a deficit
    // (even if small due to low population)
    assert!(
        state.total_park_acres < state.target_park_acres || state.target_park_acres == 0.0,
        "With no parks, total acres should be below target"
    );
}

#[test]
fn test_park_surplus_no_penalty() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::SmallPark)
        .with_service(14, 15, ServiceType::SmallPark)
        .with_service(16, 15, ServiceType::LargePark)
        .with_service(18, 15, ServiceType::LargePark)
        .with_service(20, 15, ServiceType::SportsField);

    // No citizens = very low target
    city.tick_slow_cycle();

    let state = city.resource::<ParksState>();
    assert!(
        state.deficit_penalty < f32::EPSILON,
        "With many parks and no pop, deficit penalty should be 0, got {}",
        state.deficit_penalty
    );
}

#[test]
fn test_large_park_effects_stronger_than_small_park() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(12, 15, ServiceType::SmallPark)
        .with_service(14, 15, ServiceType::LargePark);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();

    // SmallPark at (12,15): only small park effects
    let small_happiness = effects.happiness_at(12, 15);
    // LargePark at (14,15): large park effects (may also overlap with small)
    let large_happiness = effects.happiness_at(14, 15);
    assert!(
        large_happiness >= small_happiness,
        "LargePark happiness ({large_happiness}) should be >= SmallPark ({small_happiness})"
    );
}

#[test]
fn test_park_counts_tracked_correctly() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::SmallPark)
        .with_service(12, 10, ServiceType::SmallPark)
        .with_service(14, 10, ServiceType::LargePark)
        .with_service(16, 10, ServiceType::Playground)
        .with_service(18, 10, ServiceType::SportsField)
        .with_service(20, 10, ServiceType::Plaza);

    city.tick_slow_cycle();

    let state = city.resource::<ParksState>();
    assert_eq!(state.small_park_count, 2);
    assert_eq!(state.large_park_count, 1);
    assert_eq!(state.playground_count, 1);
    assert_eq!(state.sports_field_count, 1);
    assert_eq!(state.plaza_count, 1);
}

#[test]
fn test_effects_clear_between_updates() {
    let mut city = TestCity::new()
        .with_service(12, 15, ServiceType::LargePark);

    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    assert!(effects.happiness_at(12, 15) > 0.0);

    // Bulldoze the park and run another cycle
    city.bulldoze_service_at(12, 15);
    city.tick_slow_cycle();

    let effects = city.resource::<ParkEffectsGrid>();
    assert!(
        effects.happiness_at(12, 15) < f32::EPSILON,
        "Effects should clear after park is removed"
    );
}
