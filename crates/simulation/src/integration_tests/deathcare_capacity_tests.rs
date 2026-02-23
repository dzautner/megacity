//! Integration tests for SVC-008: Death Care Capacity and Cemetery Fill
//!
//! Tests that cemeteries track finite capacity, crematoriums process queues,
//! and overflow bodies generate appropriate penalties.

use crate::death_care::{DeathCareGrid, DeathCareStats};
use crate::deathcare_capacity::{
    CemeteryRecord, CrematoriumRecord, DeathCareCapacityState, CEMETERY_CAPACITY,
    CREMATORIUM_BATCH_SIZE, MAX_OVERFLOW_HAPPINESS_PENALTY, OVERFLOW_HAPPINESS_PENALTY,
};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Helper
// ====================================================================

/// Tick enough slow cycles for the deathcare capacity systems to run.
fn tick_deathcare(city: &mut TestCity) {
    city.tick_slow_cycles(2);
}

// ====================================================================
// 1. Cemetery capacity tracking
// ====================================================================

#[test]
fn test_cemetery_registers_with_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Cemetery);
    tick_deathcare(&mut city);

    let state = city.world_mut().resource::<DeathCareCapacityState>();
    assert_eq!(state.cemeteries.len(), 1, "Should track one cemetery");
    let record = state.cemeteries.values().next().unwrap();
    assert_eq!(record.total_plots, CEMETERY_CAPACITY);
    assert_eq!(record.plots_used, 0);
}

#[test]
fn test_multiple_cemeteries_tracked_independently() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::Cemetery)
        .with_service(150, 150, ServiceType::Cemetery);
    tick_deathcare(&mut city);

    let state = city.world_mut().resource::<DeathCareCapacityState>();
    assert_eq!(state.cemeteries.len(), 2);
    assert_eq!(state.total_cemetery_capacity(), CEMETERY_CAPACITY * 2);
}

#[test]
fn test_crematorium_registers_on_placement() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Crematorium);
    tick_deathcare(&mut city);

    let state = city.world_mut().resource::<DeathCareCapacityState>();
    assert_eq!(state.crematoriums.len(), 1, "Should track one crematorium");
}

// ====================================================================
// 2. Cemetery interment logic
// ====================================================================

#[test]
fn test_cemetery_record_inter_reduces_remaining() {
    let mut record = CemeteryRecord::new(5);
    assert!(record.inter());
    assert!(record.inter());
    assert_eq!(record.remaining(), 3);
    assert_eq!(record.plots_used, 2);
}

#[test]
fn test_cemetery_record_full_rejects_interment() {
    let mut record = CemeteryRecord::new(2);
    assert!(record.inter());
    assert!(record.inter());
    assert!(record.is_full());
    assert!(!record.inter());
}

// ====================================================================
// 3. Crematorium queue processing
// ====================================================================

#[test]
fn test_crematorium_processes_batch() {
    let mut record = CrematoriumRecord::default();
    for _ in 0..10 {
        record.enqueue();
    }
    let processed = record.process(CREMATORIUM_BATCH_SIZE);
    assert_eq!(processed, CREMATORIUM_BATCH_SIZE);
    assert_eq!(record.queue, 10 - CREMATORIUM_BATCH_SIZE);
}

#[test]
fn test_crematorium_processes_partial_batch() {
    let mut record = CrematoriumRecord::default();
    record.enqueue();
    record.enqueue();
    let processed = record.process(CREMATORIUM_BATCH_SIZE);
    assert_eq!(processed, 2);
    assert_eq!(record.queue, 0);
}

// ====================================================================
// 4. Overflow and penalties
// ====================================================================

#[test]
fn test_no_overflow_when_capacity_available() {
    let mut state = DeathCareCapacityState::default();
    state.overflow_bodies = 0;
    assert_eq!(state.happiness_penalty(), 0.0);
    assert_eq!(state.health_penalty(), 0.0);
}

#[test]
fn test_overflow_generates_happiness_penalty() {
    let mut state = DeathCareCapacityState::default();
    state.overflow_bodies = 10;
    let expected = 10.0 * OVERFLOW_HAPPINESS_PENALTY;
    assert!(
        (state.happiness_penalty() - expected).abs() < f32::EPSILON,
        "Expected penalty {}, got {}",
        expected,
        state.happiness_penalty()
    );
}

#[test]
fn test_overflow_happiness_penalty_capped() {
    let mut state = DeathCareCapacityState::default();
    state.overflow_bodies = 9999;
    assert!(
        (state.happiness_penalty() - MAX_OVERFLOW_HAPPINESS_PENALTY).abs() < f32::EPSILON,
        "Penalty should be capped at {}",
        MAX_OVERFLOW_HAPPINESS_PENALTY
    );
}

// ====================================================================
// 5. State serialization roundtrip
// ====================================================================

#[test]
fn test_deathcare_capacity_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = DeathCareCapacityState::default();
    state
        .cemeteries
        .insert((10, 10), CemeteryRecord::new(1000));
    state.cemeteries.get_mut(&(10, 10)).unwrap().plots_used = 500;
    state
        .crematoriums
        .insert((20, 20), CrematoriumRecord::default());
    state.crematoriums.get_mut(&(20, 20)).unwrap().queue = 15;
    state.overflow_bodies = 3;
    state.total_interred = 500;
    state.total_cremated = 200;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = DeathCareCapacityState::load_from_bytes(&bytes);

    assert_eq!(restored.cemeteries.len(), 1);
    assert_eq!(
        restored.cemeteries.get(&(10, 10)).unwrap().plots_used,
        500
    );
    assert_eq!(restored.crematoriums.len(), 1);
    assert_eq!(restored.crematoriums.get(&(20, 20)).unwrap().queue, 15);
    assert_eq!(restored.overflow_bodies, 3);
    assert_eq!(restored.total_interred, 500);
    assert_eq!(restored.total_cremated, 200);
}

// ====================================================================
// 6. Integration: system runs in full simulation
// ====================================================================

#[test]
fn test_deathcare_capacity_system_initializes_state() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Cemetery)
        .with_service(130, 130, ServiceType::Crematorium);

    tick_deathcare(&mut city);

    let state = city.world_mut().resource::<DeathCareCapacityState>();
    assert_eq!(state.cemeteries.len(), 1);
    assert_eq!(state.crematoriums.len(), 1);
    assert_eq!(state.overflow_bodies, 0);
}

#[test]
fn test_deathcare_capacity_tracks_cemetery_fill_rate() {
    let mut state = DeathCareCapacityState::default();
    state
        .cemeteries
        .insert((10, 10), CemeteryRecord::new(100));

    // Fill rate is plots_used / total_plots
    let record = state.cemeteries.get(&(10, 10)).unwrap();
    let fill_rate = record.plots_used as f32 / record.total_plots as f32;
    assert_eq!(fill_rate, 0.0);

    state.cemeteries.get_mut(&(10, 10)).unwrap().plots_used = 75;
    let record = state.cemeteries.get(&(10, 10)).unwrap();
    let fill_rate = record.plots_used as f32 / record.total_plots as f32;
    assert!((fill_rate - 0.75).abs() < f32::EPSILON);
}

#[test]
fn test_deathcare_demolished_buildings_pruned() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Cemetery)
        .with_service(130, 130, ServiceType::Cemetery);

    tick_deathcare(&mut city);

    {
        let state = city.world_mut().resource::<DeathCareCapacityState>();
        assert_eq!(state.cemeteries.len(), 2);
    }

    // Demolish one cemetery by despawning its entity
    let entities: Vec<bevy::prelude::Entity> = city
        .world_mut()
        .query::<(bevy::prelude::Entity, &ServiceBuilding)>()
        .iter(city.world_mut())
        .filter(|(_, s)| s.service_type == ServiceType::Cemetery && s.grid_x == 128)
        .map(|(e, _)| e)
        .collect();

    for e in entities {
        city.world_mut().despawn(e);
    }

    tick_deathcare(&mut city);

    let state = city.world_mut().resource::<DeathCareCapacityState>();
    assert_eq!(
        state.cemeteries.len(),
        1,
        "Demolished cemetery should be pruned"
    );
}
