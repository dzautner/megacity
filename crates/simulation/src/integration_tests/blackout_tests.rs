//! Integration tests for POWER-016: Blackout and Rolling Blackout System.

use crate::blackout::BlackoutState;
use crate::Saveable;
use crate::coal_power::PowerPlant;
use crate::energy_demand::{EnergyConsumer, LoadPriority};
use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

/// Helper: create a PowerPlant component at the given position.
fn make_plant(capacity_mw: f32, fuel_cost: f32) -> PowerPlant {
    PowerPlant {
        plant_type: crate::coal_power::PowerPlantType::Coal,
        capacity_mw,
        current_output_mw: 0.0,
        fuel_cost,
        grid_x: 0,
        grid_y: 0,
    }
}

/// Create a TestCity with baseline weather (18.3C) for predictable demand.
fn new_baseline_city() -> TestCity {
    TestCity::new().with_weather(18.3)
}

/// Spawn a standalone EnergyConsumer that produces `target_mw` of demand.
fn spawn_demand(city: &mut TestCity, target_mw: f32, priority: LoadPriority) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, priority));
}

/// Tick enough for demand aggregation, dispatch, and blackout evaluation.
fn tick_blackout(city: &mut TestCity) {
    city.tick(8);
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[test]
fn test_blackout_state_initialized() {
    let city = new_baseline_city();
    let state = city.resource::<BlackoutState>();
    assert!(!state.active, "Blackout should not be active by default");
    assert_eq!(state.affected_cell_count, 0);
    assert_eq!(state.duration_days, 0);
}

#[test]
fn test_no_blackout_with_surplus_supply() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Supply far exceeds demand.
    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert!(!state.active, "No blackout when supply > demand");
    assert_eq!(state.affected_cell_count, 0);
    assert!((state.load_shed_fraction - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_blackout_activates_on_deficit() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Set up powered cells first.
    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        for x in 10..20 {
            for y in 10..20 {
                grid.get_mut(x, y).has_power = true;
                grid.get_mut(x, y).zone = ZoneType::ResidentialLow;
            }
        }
    }

    // Supply less than demand: 50 MW supply, 100 MW demand.
    city.world_mut().spawn(make_plant(50.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert!(state.active, "Blackout should be active when demand > supply");
    assert!(
        state.load_shed_fraction > 0.0,
        "Load shed fraction should be positive during deficit, got {}",
        state.load_shed_fraction
    );
}

#[test]
fn test_blackout_clears_when_supply_restored() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Set up powered cells.
    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        for x in 10..15 {
            grid.get_mut(x, 10).has_power = true;
            grid.get_mut(x, 10).zone = ZoneType::Industrial;
        }
    }

    // Start with deficit.
    city.world_mut().spawn(make_plant(50.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert!(state.active, "Blackout should be active");

    // Add more supply to clear deficit.
    city.world_mut().spawn(make_plant(200.0, 30.0));
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert!(
        !state.active,
        "Blackout should clear when supply is restored"
    );
    assert_eq!(state.duration_days, 0);
}

#[test]
fn test_low_priority_shed_first() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Set up cells with different zone types (which map to different priorities).
    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        // Low priority (ZoneType::None => Low)
        for x in 10..20 {
            grid.get_mut(x, 10).has_power = true;
            grid.get_mut(x, 10).zone = ZoneType::None;
        }
        // High priority (Residential => High)
        for x in 10..20 {
            grid.get_mut(x, 11).has_power = true;
            grid.get_mut(x, 11).zone = ZoneType::ResidentialLow;
        }
    }

    // Small deficit: only need to shed a few cells.
    // 20 total powered cells, need to shed ~25% = 5 cells.
    city.world_mut().spawn(make_plant(75.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert!(state.active);
    // Low priority tier (index 0) should have shed cells.
    assert!(
        state.shed_by_tier[0] > 0,
        "Low priority cells should be shed first, got {:?}",
        state.shed_by_tier
    );
}

#[test]
fn test_rolling_blackout_rotation_advances() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        for x in 10..30 {
            grid.get_mut(x, 10).has_power = true;
            grid.get_mut(x, 10).zone = ZoneType::Industrial; // Normal priority
        }
    }

    city.world_mut().spawn(make_plant(50.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);

    // First tick to activate blackout.
    tick_blackout(&mut city);
    let rotation_1 = city.resource::<BlackoutState>().rotation_offset;

    // Second tick to advance rotation.
    tick_blackout(&mut city);
    let rotation_2 = city.resource::<BlackoutState>().rotation_offset;

    assert!(
        rotation_2 > rotation_1 || rotation_2 == 0, // wrapping is fine
        "Rotation should advance, got {} -> {}",
        rotation_1,
        rotation_2
    );
}

#[test]
fn test_blackout_duration_tracks_days() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;
    city.world_mut().resource_mut::<GameClock>().day = 10;

    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        for x in 10..15 {
            grid.get_mut(x, 10).has_power = true;
            grid.get_mut(x, 10).zone = ZoneType::Industrial;
        }
    }

    city.world_mut().spawn(make_plant(50.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert_eq!(state.start_day, 10);

    // Advance to day 14.
    city.world_mut().resource_mut::<GameClock>().day = 14;
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert_eq!(
        state.duration_days, 4,
        "Duration should be 4 days, got {}",
        state.duration_days
    );
}

#[test]
fn test_critical_services_shed_last() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Place a hospital (critical priority) at (15, 15).
    let hospital_entity = city
        .world_mut()
        .spawn(crate::services::ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 15,
            grid_y: 15,
            radius: 400.0,
        })
        .id();

    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        // Hospital cell.
        grid.get_mut(15, 15).has_power = true;
        grid.get_mut(15, 15).building_id = Some(hospital_entity);

        // Many low-priority cells.
        for x in 20..40 {
            grid.get_mut(x, 10).has_power = true;
            grid.get_mut(x, 10).zone = ZoneType::None; // Low priority
        }
    }

    // Moderate deficit that should only shed low priority.
    city.world_mut().spawn(make_plant(70.0, 25.0));
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    let state = city.resource::<BlackoutState>();
    assert!(state.active);
    // Critical tier (index 3) should not be shed with moderate deficit.
    assert_eq!(
        state.shed_by_tier[3], 0,
        "Critical cells should not be shed with moderate deficit"
    );
}

#[test]
fn test_blackout_sets_has_power_false() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Set up powered cells.
    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        for x in 10..20 {
            grid.get_mut(x, 10).has_power = true;
            grid.get_mut(x, 10).zone = ZoneType::None; // Low priority
        }
    }

    // Full deficit: 0 supply.
    spawn_demand(&mut city, 100.0, LoadPriority::Normal);
    tick_blackout(&mut city);

    // Check that some cells lost power.
    let grid = city.resource::<crate::grid::WorldGrid>();
    let unpowered_count = (10..20)
        .filter(|&x| !grid.get(x, 10).has_power)
        .count();

    assert!(
        unpowered_count > 0,
        "At least some cells should have lost power during blackout"
    );
}

#[test]
fn test_saveable_roundtrip() {
    let state = BlackoutState {
        active: true,
        affected_cell_count: 42,
        rotation_offset: 7,
        duration_days: 2,
        start_day: 5,
        load_shed_fraction: 0.35,
        shed_by_tier: [10, 20, 12, 0],
        hospital_casualties: 3,
        blackout_grid: vec![],
    };

    let bytes = state.save_to_bytes().unwrap();
    let restored = BlackoutState::load_from_bytes(&bytes);

    assert!(restored.active);
    assert_eq!(restored.affected_cell_count, 42);
    assert_eq!(restored.rotation_offset, 7);
    assert_eq!(restored.duration_days, 2);
    assert_eq!(restored.hospital_casualties, 3);
}
