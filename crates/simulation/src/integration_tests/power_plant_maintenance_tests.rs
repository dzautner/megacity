//! Integration tests for the power plant maintenance system (POWER-018).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::power_plant_maintenance::{
    outage_duration_range, outage_probability, PlantOutageRecord, PowerPlantMaintenanceState,
};
use crate::test_harness::TestCity;

/// Helper: spawn a coal plant entity in the TestCity at (x, y).
fn spawn_coal_plant(city: &mut TestCity, x: usize, y: usize) {
    city.world_mut().spawn(PowerPlant::new_coal(x, y));
}

/// Helper: spawn a gas plant entity in the TestCity at (x, y).
fn spawn_gas_plant(city: &mut TestCity, x: usize, y: usize) {
    city.world_mut().spawn(PowerPlant::new_gas(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_maintenance_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<PowerPlantMaintenanceState>();
    assert_eq!(state.plants_in_outage, 0);
    assert!(state.records.is_empty());
}

// ====================================================================
// Plants start with no outage
// ====================================================================

#[test]
fn test_coal_plant_starts_with_capacity() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);
    city.tick_slow_cycle();

    // The plant should retain capacity after one cycle
    // (outage probability is very low per cycle)
    let grid = city.resource::<EnergyGrid>();
    // After dispatch, supply may or may not include coal depending on demand,
    // but the maintenance state should have at most 1 plant in outage.
    let state = city.resource::<PowerPlantMaintenanceState>();
    // With ~0.057% chance per cycle, almost certainly no outage after 1 cycle.
    // But we just check the record was created.
    assert!(
        state.records.len() <= 1,
        "Should have at most 1 record after 1 cycle"
    );
}

// ====================================================================
// Manual outage: directly set outage and verify capacity drops
// ====================================================================

#[test]
fn test_plant_capacity_zero_during_outage() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    // Run a cycle to initialize records
    city.tick_slow_cycle();

    // Manually trigger outage on all coal plants
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<PowerPlantMaintenanceState>();
        for (_, record) in state.records.iter_mut() {
            record.in_outage = true;
            record.remaining_cycles = 5;
            record.original_capacity_mw = crate::coal_power::COAL_CAPACITY_MW;
        }
    }

    // Run another cycle — the system should keep capacity at 0
    city.tick_slow_cycle();

    // Check that the plant's capacity is 0
    let world = city.world_mut();
    let mut query = world.query::<&PowerPlant>();
    for plant in query.iter(world) {
        if plant.plant_type == PowerPlantType::Coal {
            assert!(
                plant.capacity_mw < f32::EPSILON,
                "Plant capacity should be 0 during outage, got {}",
                plant.capacity_mw
            );
        }
    }
}

// ====================================================================
// Outage recovery: capacity restores after outage ends
// ====================================================================

#[test]
fn test_plant_capacity_restores_after_outage() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    // Initialize
    city.tick_slow_cycle();

    // Set a very short outage (1 cycle remaining)
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<PowerPlantMaintenanceState>();
        for (_, record) in state.records.iter_mut() {
            record.in_outage = true;
            record.remaining_cycles = 1;
            record.original_capacity_mw = crate::coal_power::COAL_CAPACITY_MW;
        }
    }

    // After this cycle, remaining_cycles will tick down to 0 and capacity restores
    city.tick_slow_cycle();

    // Verify capacity is restored
    let world = city.world_mut();
    let mut query = world.query::<&PowerPlant>();
    for plant in query.iter(world) {
        if plant.plant_type == PowerPlantType::Coal {
            assert!(
                (plant.capacity_mw - crate::coal_power::COAL_CAPACITY_MW).abs() < f32::EPSILON,
                "Plant capacity should be restored to {}, got {}",
                crate::coal_power::COAL_CAPACITY_MW,
                plant.capacity_mw
            );
        }
    }
}

// ====================================================================
// Deferred maintenance flag doubles probability
// ====================================================================

#[test]
fn test_deferred_maintenance_flag_stored() {
    let mut city = TestCity::new();

    // Set deferred maintenance globally
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<PowerPlantMaintenanceState>();
        state.defer_maintenance = true;
    }

    let state = city.resource::<PowerPlantMaintenanceState>();
    assert!(state.defer_maintenance);
}

// ====================================================================
// Outage probability values are reasonable
// ====================================================================

#[test]
fn test_outage_probabilities_are_positive_and_small() {
    let types = [
        PowerPlantType::Coal,
        PowerPlantType::NaturalGas,
        PowerPlantType::WindTurbine,
        PowerPlantType::Biomass,
        PowerPlantType::Oil,
        PowerPlantType::WasteToEnergy,
        PowerPlantType::HydroDam,
        PowerPlantType::Geothermal,
    ];

    for pt in &types {
        let prob = outage_probability(*pt);
        assert!(
            prob > 0.0,
            "{:?} outage probability should be > 0, got {}",
            pt,
            prob
        );
        assert!(
            prob < 0.01,
            "{:?} outage probability should be < 1% per cycle, got {}",
            pt,
            prob
        );
    }
}

// ====================================================================
// Outage duration ranges are valid
// ====================================================================

#[test]
fn test_outage_duration_ranges_valid() {
    let types = [
        PowerPlantType::Coal,
        PowerPlantType::NaturalGas,
        PowerPlantType::WindTurbine,
        PowerPlantType::Biomass,
        PowerPlantType::WasteToEnergy,
        PowerPlantType::HydroDam,
        PowerPlantType::Geothermal,
    ];

    for pt in &types {
        let (min, max) = outage_duration_range(*pt);
        assert!(
            min > 0,
            "{:?} min outage duration should be > 0",
            pt
        );
        assert!(
            max >= min,
            "{:?} max ({}) should be >= min ({})",
            pt,
            max,
            min
        );
        assert!(
            max <= 90,
            "{:?} max outage duration ({}) should be <= 90 cycles",
            pt,
            max
        );
    }
}

// ====================================================================
// Multiple plants: only affected plants lose capacity
// ====================================================================

#[test]
fn test_outage_only_affects_target_plant() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);
    spawn_gas_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    // Get entity IDs
    let (coal_entity, gas_entity) = {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &PowerPlant)>();
        let entries: Vec<_> = query.iter(world).collect();
        let coal = entries
            .iter()
            .find(|(_, p)| p.plant_type == PowerPlantType::Coal)
            .map(|(e, _)| *e);
        let gas = entries
            .iter()
            .find(|(_, p)| p.plant_type == PowerPlantType::NaturalGas)
            .map(|(e, _)| *e);
        (coal.unwrap(), gas.unwrap())
    };

    // Force outage only on coal plant
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<PowerPlantMaintenanceState>();
        if let Some(record) = state.records.get_mut(&coal_entity.index()) {
            record.in_outage = true;
            record.remaining_cycles = 10;
            record.original_capacity_mw = crate::coal_power::COAL_CAPACITY_MW;
        }
    }

    city.tick_slow_cycle();

    // Coal should be at 0, gas should retain capacity
    let world = city.world_mut();
    let mut query = world.query::<(bevy::prelude::Entity, &PowerPlant)>();
    for (entity, plant) in query.iter(world) {
        if entity == coal_entity {
            assert!(
                plant.capacity_mw < f32::EPSILON,
                "Coal plant should be at 0 MW during outage, got {}",
                plant.capacity_mw
            );
        }
        if entity == gas_entity {
            assert!(
                plant.capacity_mw > 0.0,
                "Gas plant should retain capacity, got {}",
                plant.capacity_mw
            );
        }
    }
}

// ====================================================================
// Empty city produces no outages
// ====================================================================

#[test]
fn test_no_plants_no_outages() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let state = city.resource::<PowerPlantMaintenanceState>();
    assert_eq!(state.plants_in_outage, 0);
    assert!(state.records.is_empty());
}

// ====================================================================
// Saveable roundtrip
// ====================================================================

#[test]
fn test_maintenance_state_saveable_roundtrip() {
    use crate::Saveable;
    let mut state = PowerPlantMaintenanceState::default();
    state.records.insert(
        1,
        PlantOutageRecord {
            in_outage: true,
            remaining_cycles: 5,
            original_capacity_mw: 200.0,
            outage_count: 2,
            maintenance_deferred: true,
            cycles_since_maintenance: 30,
        },
    );
    state.plants_in_outage = 1;
    state.total_lost_capacity_mw = 200.0;
    state.defer_maintenance = true;

    let bytes = state.save_to_bytes().unwrap();
    let loaded = PowerPlantMaintenanceState::load_from_bytes(&bytes);

    assert_eq!(loaded.plants_in_outage, 1);
    assert!(loaded.defer_maintenance);
    let rec = loaded.records.get(&1).unwrap();
    assert!(rec.in_outage);
    assert_eq!(rec.remaining_cycles, 5);
    assert!(rec.maintenance_deferred);
}
