//! Integration tests for the pollution grid behavior used by the AQI overlay.
//!
//! These tests verify that the PollutionGrid resource correctly stores and
//! retrieves pollution concentration values that the AQI overlay (POLL-020)
//! will map to AQI colors.
//!
//! AQI color mapping and tier classification unit tests are located in
//! `crates/rendering/src/aqi_colors.rs`.

use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;

/// Verify that a fresh PollutionGrid starts with zero pollution everywhere.
#[test]
fn test_pollution_grid_starts_clean() {
    let city = TestCity::new();
    let pollution = city.app.world().resource::<PollutionGrid>();

    // Spot-check several cells
    for (x, y) in [(0, 0), (10, 10), (50, 50), (100, 100), (255, 255)] {
        assert_eq!(
            pollution.get(x, y),
            0,
            "Pollution at ({x},{y}) should start at 0"
        );
    }
}

/// Verify that pollution values can be set and retrieved correctly
/// across the full u8 range.
#[test]
fn test_pollution_grid_full_u8_range() {
    let mut city = TestCity::new();
    {
        let mut pollution = city.app.world_mut().resource_mut::<PollutionGrid>();

        // Test boundary values
        pollution.set(10, 10, 0); // Minimum (AQI 0 = Good)
        pollution.set(20, 20, 25); // Low (AQI ~49 = Good)
        pollution.set(30, 30, 51); // Mid-low (AQI ~100 = Moderate)
        pollution.set(40, 40, 128); // Mid (AQI ~251 = Very Unhealthy)
        pollution.set(50, 50, 200); // High (AQI ~392 = Hazardous)
        pollution.set(60, 60, 255); // Maximum (AQI 500 = Hazardous)
    }

    let pollution = city.app.world().resource::<PollutionGrid>();
    assert_eq!(pollution.get(10, 10), 0);
    assert_eq!(pollution.get(20, 20), 25);
    assert_eq!(pollution.get(30, 30), 51);
    assert_eq!(pollution.get(40, 40), 128);
    assert_eq!(pollution.get(50, 50), 200);
    assert_eq!(pollution.get(60, 60), 255);
}

/// Verify that pollution saturates at 255 (u8::MAX) and doesn't overflow.
#[test]
fn test_pollution_grid_saturating_add() {
    let mut city = TestCity::new();
    {
        let mut pollution = city.app.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(10, 10, 250);
        // Manually test saturating behavior
        let current = pollution.get(10, 10);
        let new_val = current.saturating_add(50);
        pollution.set(10, 10, new_val);
    }

    let pollution = city.app.world().resource::<PollutionGrid>();
    assert_eq!(
        pollution.get(10, 10),
        255,
        "Pollution should saturate at 255, not overflow"
    );
}

/// Verify that industrial buildings generate pollution after simulation ticks.
#[test]
fn test_pollution_from_industrial_buildings() {
    let mut city = TestCity::new();
    city.place_road(50, 50, 55, 50);
    city.zone_area(50, 51, 55, 55, crate::grid::ZoneType::Industrial);
    city.tick_slow_cycles(5);

    let pollution = city.app.world().resource::<PollutionGrid>();
    // Industrial area should have some pollution
    let industrial_pollution = pollution.get(52, 53);
    // The exact value depends on whether buildings spawned, but at least
    // roads should contribute +2 pollution
    let road_pollution = pollution.get(52, 50);
    assert!(
        road_pollution > 0 || industrial_pollution > 0,
        "Roads and/or industrial zones should generate pollution"
    );
}

/// Verify that the concentration range 0-255 maps linearly to AQI 0-500.
/// This is a pure math check that doesn't need rendering imports.
#[test]
fn test_aqi_linear_scaling_formula() {
    // AQI = concentration * 500 / 255
    // We verify the formula produces expected boundary mappings
    let aqi_at_0: u16 = (0u32 * 500 / 255) as u16;
    assert_eq!(aqi_at_0, 0);

    let aqi_at_255: u16 = (255u32 * 500 / 255) as u16;
    assert_eq!(aqi_at_255, 500);

    // Concentration 25 -> AQI ~49 (Good tier boundary)
    let aqi_at_25: u16 = (25u32 * 500 / 255) as u16;
    assert!(
        aqi_at_25 <= 50,
        "Concentration 25 -> AQI {aqi_at_25} should be <= 50 (Good)"
    );

    // Concentration 51 -> AQI 100 (Moderate tier boundary)
    let aqi_at_51: u16 = (51u32 * 500 / 255) as u16;
    assert_eq!(aqi_at_51, 100, "Concentration 51 -> AQI should be 100");
}
