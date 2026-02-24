//! Integration tests for POLL-022: Pollution Alert Event System.
//!
//! Positive tests for air quality and noise alerts use actual pollution sources
//! (industrial buildings, coal power plants, roads) because the simulation
//! recalculates grids each slow tick from real sources. Direct injection of
//! grid values gets overwritten before the alert system reads them.

use crate::coal_power::PowerPlant;
use crate::grid::{RoadType, ZoneType};

use crate::groundwater::WaterQualityGrid;
use crate::pollution_alerts::{
    AlertSeverity, ExceedanceTracker, PollutionAlertLog, PollutionAlertType,
};
use crate::test_harness::TestCity;
use crate::water_pollution::WaterPollutionGrid;

// ---------------------------------------------------------------------------
// Air quality alerts
// ---------------------------------------------------------------------------

/// Air quality alert fires when coal power plants generate sustained high
/// pollution near residential zones.
#[test]
fn test_air_quality_alert_fires_from_coal_plants() {
    let mut city = TestCity::new()
        // Place residential zone near the coal plants
        .with_zone_rect(48, 48, 52, 52, ZoneType::ResidentialLow)
        // Place roads to generate some baseline traffic pollution
        .with_road(40, 50, 60, 50, RoadType::Highway);

    // Spawn 3 coal power plants adjacent to the residential area to generate
    // very high pollution (Q=100 each, ~100 concentration at source cell).
    let world = city.world_mut();
    world.spawn(PowerPlant::new_coal(46, 50));
    world.spawn(PowerPlant::new_coal(47, 50));
    world.spawn(PowerPlant::new_coal(45, 50));

    // Run enough slow cycles for sustained exceedance to trigger (3+)
    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    assert!(
        !air_alerts.is_empty(),
        "Expected air quality alerts from coal plant pollution near residential"
    );
}

/// No air quality alert on non-residential cells even with high pollution.
#[test]
fn test_air_quality_alert_ignores_non_residential() {
    let mut city = TestCity::new()
        .with_zone_rect(48, 48, 52, 52, ZoneType::Industrial);

    // Spawn coal plants next to industrial zone
    let world = city.world_mut();
    world.spawn(PowerPlant::new_coal(46, 50));
    world.spawn(PowerPlant::new_coal(47, 50));

    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    // Industrial cells should not generate air quality alerts
    assert!(
        air_alerts.is_empty(),
        "Should not alert on industrial cells even with high pollution"
    );
}

/// Air quality alert should NOT fire with clean air (no pollution sources).
#[test]
fn test_air_quality_alert_does_not_fire_without_sources() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialLow);

    // No pollution sources, just residential zone
    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    assert!(
        air_alerts.is_empty(),
        "Should not alert when there are no pollution sources"
    );
}

// ---------------------------------------------------------------------------
// Water quality alerts
// ---------------------------------------------------------------------------

/// Water quality alert should fire when quality drops below Clean threshold.
#[test]
fn test_water_quality_alert_fires_on_low_quality() {
    let mut city = TestCity::new();

    // Set water quality below Clean threshold (180) at a sampled cell.
    // WaterQualityGrid is not fully recalculated from sources each tick,
    // so direct injection works.
    {
        let mut wq = city.world_mut().resource_mut::<WaterQualityGrid>();
        wq.set(0, 0, 100);
    }

    city.tick_slow_cycles(1);

    let log = city.resource::<PollutionAlertLog>();
    let water_alerts = log.alerts_of_type(PollutionAlertType::WaterQuality);
    assert!(
        !water_alerts.is_empty(),
        "Expected water quality alert when quality drops below Clean"
    );
}

/// Water quality alert should have Emergency severity for very low quality.
#[test]
fn test_water_quality_emergency_at_very_low_quality() {
    let mut city = TestCity::new();

    {
        let mut wq = city.world_mut().resource_mut::<WaterQualityGrid>();
        wq.set(0, 0, 50);
    }

    city.tick_slow_cycles(1);

    let log = city.resource::<PollutionAlertLog>();
    let water_alerts = log.alerts_of_type(PollutionAlertType::WaterQuality);
    let emergencies: Vec<_> = water_alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::Emergency)
        .collect();
    assert!(
        !emergencies.is_empty(),
        "Very low water quality should trigger Emergency"
    );
}

/// No water quality alert when quality is above Clean threshold.
#[test]
fn test_water_quality_no_alert_above_clean() {
    let mut city = TestCity::new();

    // Default WaterQualityGrid starts at 200 which is above 180 threshold
    city.tick_slow_cycles(2);

    let log = city.resource::<PollutionAlertLog>();
    let water_alerts = log.alerts_of_type(PollutionAlertType::WaterQuality);
    assert!(
        water_alerts.is_empty(),
        "No alerts expected when water quality is above Clean threshold"
    );
}

// ---------------------------------------------------------------------------
// Noise complaints
// ---------------------------------------------------------------------------

/// Noise complaint fires from sustained airport noise near residential.
/// Two InternationalAirports (intensity=45 each) on opposite sides of a
/// residential area generate combined noise that exceeds the 60 threshold.
/// Note: roads are NOT placed through the residential area because road
/// placement clears the zone to None, preventing alert detection.
#[test]
fn test_noise_complaint_fires_from_airports() {
    let mut city = TestCity::new()
        .with_zone_rect(50, 48, 55, 52, ZoneType::ResidentialMedium)
        // Place airports on both sides of the residential area
        .with_service(50, 46, crate::services::ServiceType::InternationalAirport)
        .with_service(50, 54, crate::services::ServiceType::InternationalAirport);

    // Also add industrial buildings adjacent to residential for extra noise
    let world = city.world_mut();
    for x in 48..=55 {
        world.spawn(crate::buildings::Building {
            zone_type: ZoneType::Industrial,
            level: 5,
            grid_x: x,
            grid_y: 47,
            capacity: 10,
            occupants: 0,
        });
    }

    // Run enough slow cycles for sustained exceedance (3+ slow ticks)
    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let noise_alerts = log.alerts_of_type(PollutionAlertType::NoiseComplaint);
    assert!(
        !noise_alerts.is_empty(),
        "Expected noise complaints from dual airports near residential"
    );
}

/// No noise complaint on non-residential cells even with high noise.
#[test]
fn test_noise_complaint_ignores_non_residential() {
    let mut city = TestCity::new()
        .with_zone_rect(50, 50, 55, 55, ZoneType::CommercialHigh)
        .with_road(48, 52, 58, 52, RoadType::Highway)
        .with_service(50, 53, crate::services::ServiceType::InternationalAirport);

    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let noise_alerts = log.alerts_of_type(PollutionAlertType::NoiseComplaint);
    assert!(
        noise_alerts.is_empty(),
        "Should not generate noise complaints for commercial zones"
    );
}

// ---------------------------------------------------------------------------
// Soil contamination alerts
// ---------------------------------------------------------------------------

/// Soil contamination alert fires when high water pollution is near residential.
#[test]
fn test_soil_contamination_alert_near_residential() {
    let mut city = TestCity::new()
        .with_zone(52, 52, ZoneType::ResidentialLow);

    {
        let mut wp = city.world_mut().resource_mut::<WaterPollutionGrid>();
        wp.set(52, 52, 150);
    }

    city.tick_slow_cycles(1);

    let log = city.resource::<PollutionAlertLog>();
    let soil_alerts = log.alerts_of_type(PollutionAlertType::SoilContamination);
    assert!(
        !soil_alerts.is_empty(),
        "Expected soil contamination alert when water pollution is high near residential"
    );
}

/// No soil contamination alert when water pollution is below threshold.
#[test]
fn test_soil_contamination_no_alert_below_threshold() {
    let mut city = TestCity::new()
        .with_zone(52, 52, ZoneType::ResidentialLow);

    {
        let mut wp = city.world_mut().resource_mut::<WaterPollutionGrid>();
        wp.set(52, 52, 50);
    }

    city.tick_slow_cycles(2);

    let log = city.resource::<PollutionAlertLog>();
    let soil_alerts = log.alerts_of_type(PollutionAlertType::SoilContamination);
    assert!(
        soil_alerts.is_empty(),
        "No soil alert expected below contamination threshold"
    );
}

// ---------------------------------------------------------------------------
// Alert log persistence
// ---------------------------------------------------------------------------

/// PollutionAlertLog save/load roundtrip via Saveable trait.
#[test]
fn test_alert_log_saveable_roundtrip() {
    use crate::Saveable;

    let mut log = PollutionAlertLog::default();
    log.push(crate::pollution_alerts::PollutionAlert {
        alert_type: PollutionAlertType::AirQuality,
        severity: AlertSeverity::Warning,
        grid_x: 10,
        grid_y: 20,
        tick: 5000,
    });
    log.push(crate::pollution_alerts::PollutionAlert {
        alert_type: PollutionAlertType::WaterQuality,
        severity: AlertSeverity::Emergency,
        grid_x: 30,
        grid_y: 40,
        tick: 6000,
    });

    let bytes = log.save_to_bytes().expect("non-empty log should serialize");
    let restored = PollutionAlertLog::load_from_bytes(&bytes);
    assert_eq!(restored.alerts.len(), 2);
    assert_eq!(
        restored.alerts[0].alert_type,
        PollutionAlertType::AirQuality
    );
    assert_eq!(
        restored.alerts[1].alert_type,
        PollutionAlertType::WaterQuality
    );
    assert_eq!(restored.alerts[1].severity, AlertSeverity::Emergency);
}

/// Exceedance tracker resets when pollution drops.
#[test]
fn test_exceedance_tracker_resets_on_clean_air() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialLow);

    // Pre-set tracker to a high value
    {
        let world = city.world_mut();
        let mut tracker = world.resource_mut::<ExceedanceTracker>();
        let idx = 50 * 256 + 50;
        tracker.air[idx] = 2;
    }

    // Run with no pollution sources — grid stays at 0
    // The tracker should decrement back to 0 over slow cycles
    city.tick_slow_cycles(3);

    let tracker = city.resource::<ExceedanceTracker>();
    let idx = 50 * 256 + 50;
    assert_eq!(
        tracker.air[idx], 0,
        "Exceedance counter should reset when pollution drops"
    );
}
