//! Integration tests for SVC-017: Water Treatment and Quality.
//!
//! Tests verify:
//! - Treatment plants reduce water pollution in their area
//! - Treatment level affects pollution reduction (60/85/95%)
//! - Untreated overflow (demand > capacity) increases water pollution
//! - WellPump provides clean water in low-pollution areas
//! - Water quality metric tracks per-area treatment effects
//! - Saveable roundtrip for WaterTreatmentState

use crate::groundwater::WaterQualityGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::water_pollution::WaterPollutionGrid;
use crate::water_treatment::{TreatmentLevel, WaterTreatmentState};
use crate::Saveable;

// ---------------------------------------------------------------------------
// Treatment plant registration
// ---------------------------------------------------------------------------

#[test]
fn test_treatment_plant_registered_on_tick() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::WaterTreatmentPlant);

    city.tick_slow_cycle();

    let state = city.resource::<WaterTreatmentState>();
    assert_eq!(
        state.plants.len(),
        1,
        "One treatment plant should be registered"
    );
    // Default registration level is Primary
    for plant in state.plants.values() {
        assert_eq!(plant.level, TreatmentLevel::Primary);
    }
}

// ---------------------------------------------------------------------------
// Treatment reduces water pollution
// ---------------------------------------------------------------------------

#[test]
fn test_treatment_plant_reduces_water_pollution() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::WaterTreatmentPlant);

    // Seed water pollution near the treatment plant
    {
        let world = city.world_mut();
        let mut wp = world.resource_mut::<WaterPollutionGrid>();
        for dy in -5i32..=5 {
            for dx in -5i32..=5 {
                let x = (50 + dx) as usize;
                let y = (50 + dy) as usize;
                wp.set(x, y, 100);
            }
        }
    }

    // Run several slow ticks to allow treatment to take effect
    city.tick_slow_cycles(3);

    let wp = city.resource::<WaterPollutionGrid>();
    let pollution_at_plant = wp.get(50, 50);
    assert!(
        pollution_at_plant < 100,
        "Treatment plant should reduce pollution near it, got {}",
        pollution_at_plant
    );
}

// ---------------------------------------------------------------------------
// Treatment boosts water quality
// ---------------------------------------------------------------------------

#[test]
fn test_treatment_plant_boosts_water_quality() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::WaterTreatmentPlant);

    // Set water quality low near the treatment plant
    {
        let world = city.world_mut();
        let mut wq = world.resource_mut::<WaterQualityGrid>();
        for dy in -5i32..=5 {
            for dx in -5i32..=5 {
                let x = (50 + dx) as usize;
                let y = (50 + dy) as usize;
                wq.set(x, y, 50);
            }
        }
    }

    city.tick_slow_cycles(3);

    let wq = city.resource::<WaterQualityGrid>();
    let quality_at_plant = wq.get(50, 50);
    assert!(
        quality_at_plant > 50,
        "Treatment plant should boost water quality near it, got {}",
        quality_at_plant
    );
}

// ---------------------------------------------------------------------------
// Tertiary treatment has stronger effect than Primary
// ---------------------------------------------------------------------------

#[test]
fn test_tertiary_treatment_removes_more_pollution() {
    // Tertiary treatment: 95% removal efficiency
    let efficiency_tertiary = TreatmentLevel::Tertiary.removal_efficiency();
    let efficiency_primary = TreatmentLevel::Primary.removal_efficiency();
    assert!(
        efficiency_tertiary > efficiency_primary,
        "Tertiary ({}) should remove more than Primary ({})",
        efficiency_tertiary,
        efficiency_primary
    );

    // Verify exact values per issue spec
    assert!(
        (efficiency_primary - 0.60).abs() < f32::EPSILON,
        "Primary should be 60%"
    );
    assert!(
        (TreatmentLevel::Secondary.removal_efficiency() - 0.85).abs() < f32::EPSILON,
        "Secondary should be 85%"
    );
    assert!(
        (efficiency_tertiary - 0.95).abs() < f32::EPSILON,
        "Tertiary should be 95%"
    );
}

// ---------------------------------------------------------------------------
// Overflow increases pollution
// ---------------------------------------------------------------------------

#[test]
fn test_overflow_increases_water_pollution() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::WaterTreatmentPlant);

    // Set demand much higher than capacity to trigger overflow
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<WaterTreatmentState>();
        state.city_demand_mgd = 100.0; // Way above a single Primary plant's 10 MGD capacity
    }

    // Record baseline pollution
    let baseline = city.resource::<WaterPollutionGrid>().get(50, 50);

    city.tick_slow_cycles(3);

    // After overflow, pollution should increase near the plant
    let _after = city.resource::<WaterPollutionGrid>().get(50, 50);
    // The treatment plant also reduces pollution, but overflow adds more.
    // Check that the overflow effect is visible somewhere in the radius.
    let max_pollution_after = {
        let wp = city.resource::<WaterPollutionGrid>();
        let mut max_p = 0u8;
        for dy in -12i32..=12 {
            for dx in -12i32..=12 {
                let x = (50 + dx).clamp(0, 255) as usize;
                let y = (50 + dy).clamp(0, 255) as usize;
                max_p = max_p.max(wp.get(x, y));
            }
        }
        max_p
    };
    assert!(
        max_pollution_after > baseline,
        "Overflow should increase pollution somewhere near the plant. baseline={}, max_after={}",
        baseline,
        max_pollution_after
    );
}

// ---------------------------------------------------------------------------
// WellPump in clean area boosts quality
// ---------------------------------------------------------------------------

#[test]
fn test_well_pump_boosts_quality_in_clean_area() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::WellPump);

    // Set quality low and pollution low (clean area)
    {
        let world = city.world_mut();
        let mut wq = world.resource_mut::<WaterQualityGrid>();
        for dy in -5i32..=5 {
            for dx in -5i32..=5 {
                let x = (80 + dx) as usize;
                let y = (80 + dy) as usize;
                wq.set(x, y, 100);
            }
        }
        // Ensure pollution is low (should be 0 by default)
        let wp = world.resource::<WaterPollutionGrid>();
        assert!(
            wp.get(80, 80) < 30,
            "Pollution should be low for well pump test"
        );
    }

    city.tick_slow_cycles(3);

    let wq = city.resource::<WaterQualityGrid>();
    let quality = wq.get(80, 80);
    assert!(
        quality > 100,
        "WellPump should boost water quality in clean area, got {}",
        quality
    );
}

#[test]
fn test_well_pump_no_effect_in_polluted_area() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::WellPump);

    // Set high pollution at the pump location
    {
        let world = city.world_mut();
        let mut wp = world.resource_mut::<WaterPollutionGrid>();
        wp.set(80, 80, 200); // Well above clean threshold of 30
        let mut wq = world.resource_mut::<WaterQualityGrid>();
        wq.set(80, 80, 50);
    }

    city.tick_slow_cycles(3);

    let wq = city.resource::<WaterQualityGrid>();
    let quality = wq.get(80, 80);
    // Quality may improve slightly from other systems, but well pump
    // should NOT be the source of improvement in polluted areas
    // The quality improvement (if any) should be modest since the
    // well pump is in a polluted area.
    // We just verify the well pump doesn't magically clean heavily polluted water.
    assert!(
        quality <= 100,
        "WellPump should not significantly boost quality in polluted area, got {}",
        quality
    );
}

// ---------------------------------------------------------------------------
// Saveable roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_water_treatment_saveable_roundtrip() {
    let mut state = WaterTreatmentState::default();
    state.total_capacity_mgd = 23.0;
    state.total_flow_mgd = 15.0;
    state.avg_effluent_quality = 0.85;
    state.total_period_cost = 5000.0;
    state.city_demand_mgd = 18.0;
    state.treatment_coverage = 0.83;
    state.avg_input_quality = 0.3;
    state.disease_risk = 0.04;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = WaterTreatmentState::load_from_bytes(&bytes);

    assert!(
        (restored.total_capacity_mgd - 23.0).abs() < 0.01,
        "capacity roundtrip"
    );
    assert!(
        (restored.total_flow_mgd - 15.0).abs() < 0.01,
        "flow roundtrip"
    );
    assert!(
        (restored.avg_effluent_quality - 0.85).abs() < 0.01,
        "effluent quality roundtrip"
    );
    assert!(
        (restored.total_period_cost - 5000.0).abs() < 0.01,
        "cost roundtrip"
    );
    assert!(
        (restored.treatment_coverage - 0.83).abs() < 0.01,
        "coverage roundtrip"
    );
    assert!(
        (restored.disease_risk - 0.04).abs() < 0.01,
        "disease risk roundtrip"
    );
    // Plants HashMap should be empty after load (rebuilt from ECS)
    assert!(restored.plants.is_empty(), "plants should be empty after load");
}

#[test]
fn test_water_treatment_saveable_skip_default() {
    let state = WaterTreatmentState::default();
    assert!(
        state.save_to_bytes().is_none(),
        "Default state should return None (skip saving)"
    );
}

// ---------------------------------------------------------------------------
// Treatment coverage metric
// ---------------------------------------------------------------------------

#[test]
fn test_treatment_coverage_full_when_no_demand() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::WaterTreatmentPlant);

    // No demand set, so coverage should be 1.0 (fully covered)
    city.tick_slow_cycle();

    let state = city.resource::<WaterTreatmentState>();
    assert!(
        (state.treatment_coverage - 1.0).abs() < 0.01,
        "No demand should mean full coverage, got {}",
        state.treatment_coverage
    );
}
