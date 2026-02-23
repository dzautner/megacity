//! Unit tests for transit hub types, registry, and saveable implementations.

use super::*;

// -------------------------------------------------------------------------
// TransitMode tests
// -------------------------------------------------------------------------

#[test]
fn test_transit_mode_from_service_type() {
    use crate::services::ServiceType;

    assert_eq!(
        TransitMode::from_service_type(ServiceType::BusDepot),
        Some(TransitMode::Bus)
    );
    assert_eq!(
        TransitMode::from_service_type(ServiceType::SubwayStation),
        Some(TransitMode::Metro)
    );
    assert_eq!(
        TransitMode::from_service_type(ServiceType::TrainStation),
        Some(TransitMode::Train)
    );
    assert_eq!(
        TransitMode::from_service_type(ServiceType::TramDepot),
        Some(TransitMode::Tram)
    );
    assert_eq!(
        TransitMode::from_service_type(ServiceType::FerryPier),
        Some(TransitMode::Ferry)
    );
    assert_eq!(
        TransitMode::from_service_type(ServiceType::FireStation),
        None
    );
}

// -------------------------------------------------------------------------
// TransitHubType tests
// -------------------------------------------------------------------------

#[test]
fn test_hub_type_from_modes_bus_metro() {
    let modes = vec![TransitMode::Bus, TransitMode::Metro];
    assert_eq!(
        TransitHubType::from_modes(&modes),
        Some(TransitHubType::BusMetroHub)
    );
}

#[test]
fn test_hub_type_from_modes_train_metro() {
    let modes = vec![TransitMode::Train, TransitMode::Metro];
    assert_eq!(
        TransitHubType::from_modes(&modes),
        Some(TransitHubType::TrainMetroHub)
    );
}

#[test]
fn test_hub_type_from_modes_multi_modal() {
    let modes = vec![TransitMode::Bus, TransitMode::Metro, TransitMode::Train];
    assert_eq!(
        TransitHubType::from_modes(&modes),
        Some(TransitHubType::MultiModalHub)
    );
}

#[test]
fn test_hub_type_from_modes_single_returns_none() {
    let modes = vec![TransitMode::Bus];
    assert_eq!(TransitHubType::from_modes(&modes), None);
}

#[test]
fn test_hub_type_from_modes_empty_returns_none() {
    let modes: Vec<TransitMode> = vec![];
    assert_eq!(TransitHubType::from_modes(&modes), None);
}

#[test]
fn test_hub_type_supported_modes() {
    let bm = TransitHubType::BusMetroHub.supported_modes();
    assert!(bm.contains(&TransitMode::Bus));
    assert!(bm.contains(&TransitMode::Metro));
    assert_eq!(bm.len(), 2);

    let tm = TransitHubType::TrainMetroHub.supported_modes();
    assert!(tm.contains(&TransitMode::Train));
    assert!(tm.contains(&TransitMode::Metro));
    assert_eq!(tm.len(), 2);

    let mm = TransitHubType::MultiModalHub.supported_modes();
    assert!(mm.len() >= 3);
}

// -------------------------------------------------------------------------
// TransitHub component tests
// -------------------------------------------------------------------------

#[test]
fn test_transit_hub_effective_penalty_supported_modes() {
    let hub = TransitHub::new(TransitHubType::BusMetroHub, 10, 10);
    let penalty = hub.effective_transfer_penalty(TransitMode::Bus, TransitMode::Metro);
    assert!((penalty - HUB_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);
}

#[test]
fn test_transit_hub_effective_penalty_unsupported_mode() {
    let hub = TransitHub::new(TransitHubType::BusMetroHub, 10, 10);
    let penalty = hub.effective_transfer_penalty(TransitMode::Bus, TransitMode::Train);
    assert!((penalty - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);
}

#[test]
fn test_transit_hub_penalty_reduction() {
    let hub = TransitHub::new(TransitHubType::BusMetroHub, 10, 10);
    // Reduction should be ~0.667 (from 3min to 1min)
    let expected = 1.0 - (HUB_TRANSFER_PENALTY_MINUTES / DEFAULT_TRANSFER_PENALTY_MINUTES);
    assert!((hub.transfer_penalty_reduction - expected).abs() < 0.01);
}

// -------------------------------------------------------------------------
// TransitHubs registry tests
// -------------------------------------------------------------------------

#[test]
fn test_transit_hubs_find_hub_near() {
    let mut registry = TransitHubs::default();
    registry.hubs.push(TransitHubEntry {
        grid_x: 50,
        grid_y: 50,
        hub_type: TransitHubType::BusMetroHub,
        modes: vec![TransitMode::Bus, TransitMode::Metro],
    });

    // Exact location
    assert!(registry.find_hub_near(50, 50).is_some());
    // Within detection radius
    assert!(registry.find_hub_near(51, 51).is_some());
    // Outside detection radius
    assert!(registry.find_hub_near(60, 60).is_none());
}

#[test]
fn test_transfer_penalty_at_hub() {
    let mut registry = TransitHubs::default();
    registry.hubs.push(TransitHubEntry {
        grid_x: 50,
        grid_y: 50,
        hub_type: TransitHubType::BusMetroHub,
        modes: vec![TransitMode::Bus, TransitMode::Metro],
    });

    let penalty = registry.transfer_penalty_at(50, 50, TransitMode::Bus, TransitMode::Metro);
    assert!((penalty - HUB_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);

    // Unsupported mode pair at hub
    let penalty = registry.transfer_penalty_at(50, 50, TransitMode::Bus, TransitMode::Train);
    assert!((penalty - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);

    // No hub at location
    let penalty = registry.transfer_penalty_at(100, 100, TransitMode::Bus, TransitMode::Metro);
    assert!((penalty - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Saveable trait tests
// -------------------------------------------------------------------------

#[test]
fn test_saveable_transit_hubs_skips_default() {
    use crate::Saveable;
    let hubs = TransitHubs::default();
    assert!(hubs.save_to_bytes().is_none());
}

#[test]
fn test_saveable_transit_hubs_saves_when_populated() {
    use crate::Saveable;
    let mut hubs = TransitHubs::default();
    hubs.hubs.push(TransitHubEntry {
        grid_x: 10,
        grid_y: 20,
        hub_type: TransitHubType::BusMetroHub,
        modes: vec![TransitMode::Bus, TransitMode::Metro],
    });
    assert!(hubs.save_to_bytes().is_some());
}

#[test]
fn test_saveable_transit_hubs_roundtrip() {
    use crate::Saveable;
    let mut hubs = TransitHubs::default();
    hubs.hubs.push(TransitHubEntry {
        grid_x: 10,
        grid_y: 20,
        hub_type: TransitHubType::BusMetroHub,
        modes: vec![TransitMode::Bus, TransitMode::Metro],
    });
    let bytes = hubs.save_to_bytes().expect("should serialize");
    let restored = TransitHubs::load_from_bytes(&bytes);
    assert_eq!(restored.hubs.len(), 1);
    assert_eq!(restored.hubs[0].grid_x, 10);
    assert_eq!(restored.hubs[0].grid_y, 20);
    assert_eq!(restored.hubs[0].hub_type, TransitHubType::BusMetroHub);
}

#[test]
fn test_saveable_transit_hub_stats_skips_default() {
    use crate::Saveable;
    let stats = TransitHubStats::default();
    assert!(stats.save_to_bytes().is_none());
}

#[test]
fn test_saveable_transit_hub_stats_saves_when_nonzero() {
    use crate::Saveable;
    let stats = TransitHubStats {
        total_hubs: 3,
        ..Default::default()
    };
    assert!(stats.save_to_bytes().is_some());
}

#[test]
fn test_saveable_keys() {
    use crate::Saveable;
    assert_eq!(TransitHubs::SAVE_KEY, "transit_hubs");
    assert_eq!(TransitHubStats::SAVE_KEY, "transit_hub_stats");
}

// -------------------------------------------------------------------------
// Constant verification tests
// -------------------------------------------------------------------------

#[test]
fn test_constants() {
    assert!((DEFAULT_TRANSFER_PENALTY_MINUTES - 3.0).abs() < f32::EPSILON);
    assert!((HUB_TRANSFER_PENALTY_MINUTES - 1.0).abs() < f32::EPSILON);
    assert!((HUB_LAND_VALUE_MULTIPLIER - 1.5).abs() < f32::EPSILON);
    assert!(HUB_DETECTION_RADIUS > 0);
    assert!(HUB_LAND_VALUE_RADIUS > 0);
}

// -------------------------------------------------------------------------
// Hub detection edge cases
// -------------------------------------------------------------------------

#[test]
fn test_hub_type_two_non_standard_modes() {
    // Bus + Tram: not a standard named pair, classified as MultiModalHub
    let modes = vec![TransitMode::Bus, TransitMode::Tram];
    assert_eq!(
        TransitHubType::from_modes(&modes),
        Some(TransitHubType::MultiModalHub)
    );
}

#[test]
fn test_hub_land_value_boost_exceeds_individual() {
    let hub_boost = (TRANSIT_STATION_BASE_BOOST as f32 * HUB_LAND_VALUE_MULTIPLIER) as i32;
    assert!(
        hub_boost > TRANSIT_STATION_BASE_BOOST,
        "Hub land value boost ({hub_boost}) should exceed individual station boost ({TRANSIT_STATION_BASE_BOOST})"
    );
}
