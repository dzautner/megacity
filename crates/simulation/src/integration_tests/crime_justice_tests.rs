//! SERV-005: Integration tests for Crime Types and Justice Pipeline.

use crate::crime_justice::{CrimeJusticeState, CrimeType, JusticeStage, PRISON_CAPACITY};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Crime generation
// ====================================================================

#[test]
fn test_no_crimes_in_empty_city() {
    let mut city = TestCity::new();
    tick_slow(&mut city);
    let state = city.resource::<CrimeJusticeState>();
    let total: u32 = state.district_stats.iter().map(|s| s.total_crimes()).sum();
    assert_eq!(total, 0, "Empty city should have no crimes");
}

#[test]
fn test_crimes_generated_in_tel_aviv() {
    let mut city = TestCity::with_tel_aviv();
    for _ in 0..10 {
        tick_slow(&mut city);
    }
    let state = city.resource::<CrimeJusticeState>();
    let total: u32 = state.district_stats.iter().map(|s| s.total_crimes()).sum();
    assert!(
        total > 0,
        "Tel Aviv city should generate crimes, got total={total}"
    );
}

#[test]
fn test_empty_district_has_no_crimes() {
    // Use Tel Aviv but check a district far from the populated area
    let mut city = TestCity::with_tel_aviv();
    for _ in 0..10 {
        tick_slow(&mut city);
    }
    let state = city.resource::<CrimeJusticeState>();
    // District (15,15) is at the far corner - likely empty
    let c_corner = state.get_district_stats(15, 15).total_crimes();
    assert_eq!(c_corner, 0, "Far corner district should have no crimes");
}

// ====================================================================
// 2. Police effectiveness
// ====================================================================

#[test]
fn test_police_effectiveness_zero_without_services() {
    let mut city = TestCity::new();
    tick_slow(&mut city);
    let s = city.resource::<CrimeJusticeState>();
    assert!(
        s.police_effectiveness < 0.01,
        "No police = ~0 effectiveness"
    );
}

#[test]
fn test_police_effectiveness_increases_with_stations() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PoliceStation)
        .with_service(100, 100, ServiceType::PoliceStation);
    tick_slow(&mut city);
    let s = city.resource::<CrimeJusticeState>();
    assert!(
        s.police_effectiveness > 0.2,
        "Two stations should give > 0.2, got {}",
        s.police_effectiveness
    );
}

#[test]
fn test_police_hq_more_effective_than_kiosk() {
    let mut city_hq = TestCity::new().with_service(50, 50, ServiceType::PoliceHQ);
    tick_slow(&mut city_hq);
    let eff_hq = city_hq
        .resource::<CrimeJusticeState>()
        .police_effectiveness;

    let mut city_kiosk = TestCity::new().with_service(50, 50, ServiceType::PoliceKiosk);
    tick_slow(&mut city_kiosk);
    let eff_kiosk = city_kiosk
        .resource::<CrimeJusticeState>()
        .police_effectiveness;

    assert!(eff_hq > eff_kiosk, "HQ ({eff_hq}) > kiosk ({eff_kiosk})");
}

#[test]
fn test_police_effectiveness_capped_at_one() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::PoliceHQ)
        .with_service(30, 30, ServiceType::PoliceHQ)
        .with_service(50, 50, ServiceType::PoliceHQ)
        .with_service(70, 70, ServiceType::PoliceHQ)
        .with_service(90, 90, ServiceType::PoliceHQ);
    tick_slow(&mut city);
    let s = city.resource::<CrimeJusticeState>();
    assert!(
        s.police_effectiveness <= 1.0,
        "Capped at 1.0, got {}",
        s.police_effectiveness
    );
}

// ====================================================================
// 3. Jail capacity and deterrence
// ====================================================================

#[test]
fn test_jail_capacity_from_prisons() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Prison)
        .with_service(100, 100, ServiceType::Prison);
    tick_slow(&mut city);
    let s = city.resource::<CrimeJusticeState>();
    assert_eq!(s.jail_capacity, 2 * PRISON_CAPACITY);
}

#[test]
fn test_low_deterrence_without_prison() {
    let mut city = TestCity::new();
    tick_slow(&mut city);
    let s = city.resource::<CrimeJusticeState>();
    assert!(
        s.deterrence <= 0.15,
        "No prisons = low deterrence, got {}",
        s.deterrence
    );
}

#[test]
fn test_deterrence_high_with_empty_prison() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::Prison);
    tick_slow(&mut city);
    let s = city.resource::<CrimeJusticeState>();
    assert!(
        s.deterrence > 0.8,
        "Empty prison = high deterrence, got {}",
        s.deterrence
    );
}

// ====================================================================
// 4. Justice pipeline
// ====================================================================

#[test]
fn test_justice_pipeline_stages_advance() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PoliceStation)
        .with_service(60, 60, ServiceType::Prison);
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CrimeJusticeState>();
        state.events.push(crate::crime_justice::CrimeEvent {
            crime_type: CrimeType::PettyTheft,
            district_x: 0,
            district_y: 0,
            stage: JusticeStage::Reported,
            stage_timer: 0,
        });
        state.police_effectiveness = 0.9;
        state.jail_capacity = PRISON_CAPACITY;
    }
    for _ in 0..10 {
        tick_slow(&mut city);
    }
    let state = city.resource::<CrimeJusticeState>();
    let reported = state
        .events
        .iter()
        .filter(|e| e.stage == JusticeStage::Reported)
        .count();
    assert_eq!(
        reported, 0,
        "Reported events should advance past initial stage"
    );
}

#[test]
fn test_resolved_events_are_removed() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CrimeJusticeState>();
        state.events.push(crate::crime_justice::CrimeEvent {
            crime_type: CrimeType::Assault,
            district_x: 0,
            district_y: 0,
            stage: JusticeStage::InJail,
            stage_timer: 0,
        });
        state.jail_population = 1;
    }
    tick_slow(&mut city);
    let state = city.resource::<CrimeJusticeState>();
    let jail = state
        .events
        .iter()
        .filter(|e| e.stage == JusticeStage::InJail)
        .count();
    assert_eq!(jail, 0, "Completed jail term should resolve");
}

// ====================================================================
// 5. Crime type properties
// ====================================================================

#[test]
fn test_crime_types_have_valid_factors() {
    for ct in [
        CrimeType::PettyTheft,
        CrimeType::Burglary,
        CrimeType::Assault,
        CrimeType::OrganizedCrime,
    ] {
        assert!(ct.base_weight() > 0.0);
        assert!(ct.poverty_factor() > 0.0);
        assert!(ct.unemployment_factor() > 0.0);
        assert!(ct.density_factor() > 0.0);
        assert!(ct.jail_time() > 0);
    }
}

#[test]
fn test_petty_theft_most_common() {
    assert!(CrimeType::PettyTheft.base_weight() > CrimeType::OrganizedCrime.base_weight());
    assert!(CrimeType::PettyTheft.base_weight() > CrimeType::Assault.base_weight());
}

// ====================================================================
// 6. Saveable roundtrip
// ====================================================================

#[test]
fn test_crime_justice_state_persists() {
    use crate::Saveable;
    let mut s = CrimeJusticeState::default();
    s.jail_population = 15;
    s.jail_capacity = 100;
    s.police_effectiveness = 0.65;
    s.deterrence = 0.8;
    s.get_district_stats_mut(3, 4).petty_theft_count = 42;
    let bytes = s.save_to_bytes().expect("should serialize");
    let r = CrimeJusticeState::load_from_bytes(&bytes);
    assert_eq!(r.jail_population, 15);
    assert_eq!(r.jail_capacity, 100);
    assert!((r.police_effectiveness - 0.65).abs() < 0.001);
    assert!((r.deterrence - 0.8).abs() < 0.001);
    assert_eq!(r.get_district_stats(3, 4).petty_theft_count, 42);
}
