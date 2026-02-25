//! Integration tests for POLL-028: Groundwater Quality System Enhancement.
//!
//! Tests landfill leachate contamination, industrial discharge,
//! treatment plant recovery, and drinking water quality tiers.

use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::groundwater_quality::{DrinkingWaterQuality, DrinkingWaterTier};
use crate::landfill::{LandfillLinerType, LandfillState};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Landfill leachate tests
// ====================================================================

#[test]
fn test_unlined_landfill_degrades_quality_in_large_radius() {
    let mut city = TestCity::new();

    // Add an unlined landfill at (100, 100)
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(100, 100, 500_000.0, LandfillLinerType::Unlined);
    }

    // Record quality before
    let before_center = city.resource::<WaterQualityGrid>().get(100, 100);
    let before_radius8 = city.resource::<WaterQualityGrid>().get(108, 100);

    // Run slow cycles to trigger contamination
    city.tick_slow_cycles(3);

    let after_center = city.resource::<WaterQualityGrid>().get(100, 100);
    let after_radius8 = city.resource::<WaterQualityGrid>().get(108, 100);

    assert!(
        after_center < before_center,
        "Unlined landfill center should degrade: before={before_center}, after={after_center}",
    );
    assert!(
        after_radius8 < before_radius8,
        "Unlined landfill should degrade at radius 8: before={before_radius8}, after={after_radius8}",
    );
}

#[test]
fn test_lined_landfill_degrades_quality_in_smaller_radius() {
    let mut city = TestCity::new();

    // Add a lined landfill at (100, 100)
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(100, 100, 500_000.0, LandfillLinerType::Lined);
    }

    city.tick_slow_cycles(3);

    let center_after = city.resource::<WaterQualityGrid>().get(100, 100);
    // Center should still be degraded for lined
    assert!(
        center_after < 200,
        "Lined landfill center should be degraded, got {center_after}",
    );

    // At distance 5 (beyond lined radius of 3), quality should be better than center
    let radius5_after = city.resource::<WaterQualityGrid>().get(105, 100);
    assert!(
        radius5_after >= center_after,
        "Quality at radius 5 ({radius5_after}) should be >= center ({center_after}) for lined landfill",
    );
}

#[test]
fn test_unlined_degrades_more_than_lined() {
    let mut city_unlined = TestCity::new();
    let mut city_lined = TestCity::new();

    // Add unlined landfill
    {
        let world = city_unlined.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(100, 100, 500_000.0, LandfillLinerType::Unlined);
    }

    // Add lined landfill at same location in separate city
    {
        let world = city_lined.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(100, 100, 500_000.0, LandfillLinerType::Lined);
    }

    city_unlined.tick_slow_cycles(3);
    city_lined.tick_slow_cycles(3);

    let q_unlined = city_unlined.resource::<WaterQualityGrid>().get(100, 100);
    let q_lined = city_lined.resource::<WaterQualityGrid>().get(100, 100);

    assert!(
        q_unlined < q_lined,
        "Unlined landfill ({q_unlined}) should degrade more than lined ({q_lined})",
    );
}

// ====================================================================
// Industrial discharge tests
// ====================================================================

#[test]
fn test_industrial_building_degrades_nearby_quality() {
    let city = TestCity::new().with_building(80, 80, ZoneType::Industrial, 1);

    let before = city.resource::<WaterQualityGrid>().get(80, 80);

    let mut city = city;
    city.tick_slow_cycles(3);

    let after = city.resource::<WaterQualityGrid>().get(80, 80);

    assert!(
        after < before,
        "Industrial building should degrade quality: before={before}, after={after}",
    );
}

#[test]
fn test_industrial_discharge_limited_radius() {
    let city = TestCity::new().with_building(80, 80, ZoneType::Industrial, 1);

    let mut city = city;
    city.tick_slow_cycles(1);

    // At distance 10, industrial discharge (radius 5) should not reach.
    // Quality at center should be worse than far away.
    let center_after = city.resource::<WaterQualityGrid>().get(80, 80);
    let far_after = city.resource::<WaterQualityGrid>().get(90, 80);

    assert!(
        far_after >= center_after,
        "Quality far from industry ({far_after}) should be >= center ({center_after})",
    );
}

// ====================================================================
// Treatment plant recovery tests
// ====================================================================

#[test]
fn test_treatment_plant_improves_quality_over_time() {
    let mut city =
        TestCity::new().with_service(60, 60, ServiceType::WaterTreatmentPlant);

    // Manually degrade quality around the plant
    {
        let world = city.world_mut();
        let mut quality = world.resource_mut::<WaterQualityGrid>();
        for dy in -5i32..=5 {
            for dx in -5i32..=5 {
                let nx = 60 + dx;
                let ny = 60 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < 256 && (ny as usize) < 256 {
                    quality.set(nx as usize, ny as usize, 50);
                }
            }
        }
    }

    let before = city.resource::<WaterQualityGrid>().get(60, 60);

    city.tick_slow_cycles(3);

    let after = city.resource::<WaterQualityGrid>().get(60, 60);

    assert!(
        after > before,
        "Treatment plant should improve quality: before={before}, after={after}",
    );
}

// ====================================================================
// Drinking water quality tier tests
// ====================================================================

#[test]
fn test_drinking_water_quality_with_wells() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::WellPump)
        .with_service(60, 60, ServiceType::WellPump);

    city.tick_slow_cycles(1);

    let drinking = city.resource::<DrinkingWaterQuality>();
    assert_eq!(drinking.well_count, 2, "Should detect 2 well pumps");
    assert!(
        drinking.avg_well_quality > 0.0,
        "Average well quality should be positive",
    );
}

#[test]
fn test_drinking_water_quality_no_wells_defaults() {
    let mut city = TestCity::new();

    city.tick_slow_cycles(1);

    let drinking = city.resource::<DrinkingWaterQuality>();
    assert_eq!(drinking.well_count, 0);
    assert_eq!(drinking.tier, DrinkingWaterTier::Good);
}

#[test]
fn test_drinking_water_degrades_near_unlined_landfill() {
    // Place a well far from any landfill and one near an unlined landfill.
    // After contamination cycles, the near-well should have worse quality.
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::WellPump)
        .with_service(200, 200, ServiceType::WellPump);

    // Add unlined landfill right at well (100, 100)
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        state.add_site_with_options(100, 100, 500_000.0, LandfillLinerType::Unlined);
    }

    city.tick_slow_cycles(5);

    // The contaminated well should have lower quality than the clean one
    let q_near = city.resource::<WaterQualityGrid>().get(100, 100);
    let q_far = city.resource::<WaterQualityGrid>().get(200, 200);

    assert!(
        q_near < q_far,
        "Well near unlined landfill ({q_near}) should have worse quality than far well ({q_far})",
    );
}

// ====================================================================
// Converted-to-park landfill should not contaminate
// ====================================================================

#[test]
fn test_converted_to_park_landfill_no_contamination() {
    let mut city = TestCity::new();

    // Add a landfill and immediately convert to park
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<LandfillState>();
        let id = state.add_site_with_options(
            100,
            100,
            500_000.0,
            LandfillLinerType::Unlined,
        );
        if let Some(site) = state.get_site_mut(id) {
            site.status = crate::landfill::LandfillStatus::ConvertedToPark;
        }
    }

    let before = city.resource::<WaterQualityGrid>().get(100, 100);

    city.tick_slow_cycles(3);

    let after = city.resource::<WaterQualityGrid>().get(100, 100);

    // Quality should not decrease from our system (natural recovery may increase it).
    assert!(
        after >= before,
        "Converted-to-park landfill should not contaminate: before={before}, after={after}",
    );
}
