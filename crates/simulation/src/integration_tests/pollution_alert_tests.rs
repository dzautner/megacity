//! Integration tests for POLL-022: Pollution Alert Event System.

use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::pollution_alerts::{
    AlertSeverity, ExceedanceTracker, PollutionAlertLog, PollutionAlertType,
};
use crate::test_harness::TestCity;
use crate::water_pollution::WaterPollutionGrid;

// ---------------------------------------------------------------------------
// Air quality alerts
// ---------------------------------------------------------------------------

/// Air quality alert should fire after sustained high pollution on a residential cell.
#[test]
fn test_air_quality_alert_fires_on_sustained_pollution() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialLow);

    // Set pollution above threshold (151) at the residential cell
    {
        let mut pollution = city.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(50, 50, 200);
    }

    // Run 3+ slow cycles to trigger sustained exceedance
    city.tick_slow_cycles(4);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    assert!(
        !air_alerts.is_empty(),
        "Expected at least one air quality alert after sustained high pollution"
    );
    // Verify position
    assert!(air_alerts.iter().any(|a| a.grid_x == 50 && a.grid_y == 50));
}

/// No air quality alert should fire on non-residential cells.
#[test]
fn test_air_quality_alert_ignores_non_residential() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::Industrial);

    {
        let mut pollution = city.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(50, 50, 255);
    }

    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    // No alerts should have (50,50) since it's industrial
    assert!(
        !air_alerts.iter().any(|a| a.grid_x == 50 && a.grid_y == 50),
        "Should not alert on industrial cells"
    );
}

/// Air quality alert should NOT fire for below-threshold pollution.
#[test]
fn test_air_quality_alert_does_not_fire_below_threshold() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialLow);

    {
        let mut pollution = city.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(50, 50, 100); // Below 151 threshold
    }

    city.tick_slow_cycles(5);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    assert!(
        !air_alerts.iter().any(|a| a.grid_x == 50 && a.grid_y == 50),
        "Should not alert when pollution is below threshold"
    );
}

/// Emergency severity for hazardous pollution levels (251+).
#[test]
fn test_air_quality_emergency_severity() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialHigh);

    {
        let mut pollution = city.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(50, 50, 255); // Hazardous
    }

    city.tick_slow_cycles(4);

    let log = city.resource::<PollutionAlertLog>();
    let air_alerts = log.alerts_of_type(PollutionAlertType::AirQuality);
    let emergencies: Vec<_> = air_alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::Emergency)
        .collect();
    assert!(
        !emergencies.is_empty(),
        "Hazardous pollution should trigger Emergency severity"
    );
}

// ---------------------------------------------------------------------------
// Water quality alerts
// ---------------------------------------------------------------------------

/// Water quality alert should fire when quality drops below Clean threshold.
#[test]
fn test_water_quality_alert_fires_on_low_quality() {
    let mut city = TestCity::new();

    // Set water quality below Clean threshold (180) at a sampled cell
    // The system samples every 8th cell, so use (0,0) which is always sampled
    {
        let mut wq = city.world_mut().resource_mut::<WaterQualityGrid>();
        wq.set(0, 0, 100); // Below 180 = non-Clean
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
        wq.set(0, 0, 50); // Very low quality
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

/// Noise complaint should fire after sustained high noise on a residential cell.
#[test]
fn test_noise_complaint_fires_on_sustained_noise() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialMedium);

    // Set noise above threshold (80) at the residential cell.
    // Noise grid is recalculated each slow tick, so we need to keep setting it.
    // Instead, we directly manipulate the exceedance tracker to simulate sustained noise.
    {
        let mut noise = city.world_mut().resource_mut::<NoisePollutionGrid>();
        noise.set(50, 50, 90);
    }

    // The noise system clears the grid each tick, so we need to re-set it.
    // Run one slow cycle, then re-inject noise, repeat.
    for _ in 0..4 {
        {
            let mut noise = city.world_mut().resource_mut::<NoisePollutionGrid>();
            noise.set(50, 50, 90);
        }
        city.tick_slow_cycles(1);
    }

    let log = city.resource::<PollutionAlertLog>();
    let noise_alerts = log.alerts_of_type(PollutionAlertType::NoiseComplaint);
    assert!(
        !noise_alerts.is_empty(),
        "Expected noise complaint after sustained high noise"
    );
}

/// No noise complaint on non-residential cells.
#[test]
fn test_noise_complaint_ignores_non_residential() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::CommercialHigh);

    for _ in 0..4 {
        {
            let mut noise = city.world_mut().resource_mut::<NoisePollutionGrid>();
            noise.set(50, 50, 95);
        }
        city.tick_slow_cycles(1);
    }

    let log = city.resource::<PollutionAlertLog>();
    let noise_alerts = log.alerts_of_type(PollutionAlertType::NoiseComplaint);
    assert!(
        !noise_alerts.iter().any(|a| a.grid_x == 50 && a.grid_y == 50),
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
        .with_zone(52, 52, ZoneType::ResidentialLow); // Within radius 2 of (52,52)

    // Set high water pollution at cell (52, 52) — sampled at step_by(4), so
    // use a cell that is a multiple of 4.
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
        wp.set(52, 52, 50); // Below threshold (100)
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
    assert_eq!(restored.alerts[0].alert_type, PollutionAlertType::AirQuality);
    assert_eq!(restored.alerts[1].alert_type, PollutionAlertType::WaterQuality);
    assert_eq!(restored.alerts[1].severity, AlertSeverity::Emergency);
}

/// Exceedance tracker resets when pollution drops.
#[test]
fn test_exceedance_tracker_resets_on_clean_air() {
    let mut city = TestCity::new()
        .with_zone(50, 50, ZoneType::ResidentialLow);

    // Set high pollution for 2 slow cycles (not enough to trigger)
    {
        let mut pollution = city.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(50, 50, 200);
    }
    city.tick_slow_cycles(2);

    // Now drop pollution below threshold
    {
        let mut pollution = city.world_mut().resource_mut::<PollutionGrid>();
        pollution.set(50, 50, 50);
    }
    city.tick_slow_cycles(2);

    // Check that tracker has decremented
    let tracker = city.resource::<ExceedanceTracker>();
    let idx = 50 * 256 + 50; // y * GRID_WIDTH + x
    assert_eq!(
        tracker.air[idx], 0,
        "Exceedance counter should reset when pollution drops"
    );
}
