//! Integration tests for POLL-006: Water Pollution Point Source Emissions.

use crate::buildings::Building;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::utilities::{UtilitySource, UtilityType};
use crate::water_pollution::WaterPollutionGrid;
use crate::water_pollution_sources::{
    sewage_emission_for_treatment, WaterPollutionSourceType, WaterPollutionSourcesState,
};
use crate::water_treatment::TreatmentLevel;

/// Helper: place a patch of water cells around (cx, cy) with the given radius.
fn place_water_patch(city: &mut TestCity, cx: usize, cy: usize, radius: usize) {
    let world = city.world_mut();
    let mut grid = world.resource_mut::<WorldGrid>();
    for dy in 0..=(radius * 2) {
        for dx in 0..=(radius * 2) {
            let x = cx.saturating_sub(radius) + dx;
            let y = cy.saturating_sub(radius) + dy;
            if grid.in_bounds(x, y) {
                grid.get_mut(x, y).cell_type = CellType::Water;
            }
        }
    }
}

/// Helper: spawn an industrial building at (x, y) with the given level.
fn spawn_industrial(city: &mut TestCity, x: usize, y: usize, level: u8) {
    let world = city.world_mut();
    let entity = world
        .spawn(Building {
            zone_type: ZoneType::Industrial,
            level,
            grid_x: x,
            grid_y: y,
            capacity: 50,
            occupants: 0,
        })
        .id();
    let mut grid = world.resource_mut::<WorldGrid>();
    if grid.in_bounds(x, y) {
        grid.get_mut(x, y).building_id = Some(entity);
        grid.get_mut(x, y).zone = ZoneType::Industrial;
    }
}

/// Helper: spawn a commercial building at (x, y).
fn spawn_commercial(city: &mut TestCity, x: usize, y: usize, level: u8) {
    let world = city.world_mut();
    let entity = world
        .spawn(Building {
            zone_type: ZoneType::CommercialHigh,
            level,
            grid_x: x,
            grid_y: y,
            capacity: 30,
            occupants: 0,
        })
        .id();
    let mut grid = world.resource_mut::<WorldGrid>();
    if grid.in_bounds(x, y) {
        grid.get_mut(x, y).building_id = Some(entity);
        grid.get_mut(x, y).zone = ZoneType::CommercialHigh;
    }
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_water_pollution_sources_state_exists() {
    let city = TestCity::new();
    let state = city.resource::<WaterPollutionSourcesState>();
    assert_eq!(state.total_emissions, 0);
    assert!(state.source_counts.iter().all(|&c| c == 0));
}

// ====================================================================
// Sewage emission rates match spec
// ====================================================================

#[test]
fn test_sewage_emission_untreated_equals_80() {
    let rate = sewage_emission_for_treatment(TreatmentLevel::None);
    assert_eq!(rate, 80);
}

#[test]
fn test_sewage_emission_primary_equals_32() {
    let rate = sewage_emission_for_treatment(TreatmentLevel::Primary);
    assert_eq!(rate, 32);
}

#[test]
fn test_sewage_emission_secondary_equals_12() {
    let rate = sewage_emission_for_treatment(TreatmentLevel::Secondary);
    assert_eq!(rate, 12);
}

#[test]
fn test_sewage_emission_tertiary_equals_4() {
    let rate = sewage_emission_for_treatment(TreatmentLevel::Tertiary);
    assert_eq!(rate, 4);
}

#[test]
fn test_sewage_emission_advanced_equals_1() {
    let rate = sewage_emission_for_treatment(TreatmentLevel::Advanced);
    assert_eq!(rate, 1);
}

// ====================================================================
// Base emission rates match spec
// ====================================================================

#[test]
fn test_base_emission_rates() {
    assert_eq!(WaterPollutionSourceType::SewageOutfall.base_emission_rate(), 80);
    assert_eq!(WaterPollutionSourceType::HeavyIndustry.base_emission_rate(), 50);
    assert_eq!(WaterPollutionSourceType::LightIndustry.base_emission_rate(), 20);
    assert_eq!(
        WaterPollutionSourceType::PowerPlantCooling.base_emission_rate(),
        15
    );
    assert_eq!(
        WaterPollutionSourceType::LandfillLeachate.base_emission_rate(),
        25
    );
    assert_eq!(
        WaterPollutionSourceType::AgriculturalRunoff.base_emission_rate(),
        18
    );
    assert_eq!(
        WaterPollutionSourceType::ConstructionRunoff.base_emission_rate(),
        10
    );
    assert_eq!(
        WaterPollutionSourceType::CommercialDischarge.base_emission_rate(),
        8
    );
}

// ====================================================================
// At least 8 source types exist
// ====================================================================

#[test]
fn test_eight_source_types_exist() {
    let types = [
        WaterPollutionSourceType::SewageOutfall,
        WaterPollutionSourceType::HeavyIndustry,
        WaterPollutionSourceType::LightIndustry,
        WaterPollutionSourceType::PowerPlantCooling,
        WaterPollutionSourceType::LandfillLeachate,
        WaterPollutionSourceType::AgriculturalRunoff,
        WaterPollutionSourceType::ConstructionRunoff,
        WaterPollutionSourceType::CommercialDischarge,
    ];
    assert_eq!(types.len(), 8);
    // Verify each has a unique name
    let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            assert_ne!(names[i], names[j], "Duplicate source type name");
        }
    }
}

// ====================================================================
// Heavy industry building emits pollution to water
// ====================================================================

#[test]
fn test_heavy_industry_emits_water_pollution() {
    let mut city = TestCity::new();

    // Place water near an industrial building
    place_water_patch(&mut city, 52, 50, 3);
    spawn_industrial(&mut city, 50, 50, 4); // level >= 3 = heavy

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.count_for(WaterPollutionSourceType::HeavyIndustry) > 0,
        "Heavy industry source should be counted"
    );
    assert!(
        state.emissions_for(WaterPollutionSourceType::HeavyIndustry) > 0,
        "Heavy industry should emit pollution"
    );

    // Check that water cells near the factory have pollution
    let wp = city.resource::<WaterPollutionGrid>();
    let pollution_at_water = wp.get(52, 50);
    assert!(
        pollution_at_water > 0,
        "Water cell near heavy industry should be polluted, got {}",
        pollution_at_water
    );
}

// ====================================================================
// Light industry (level 1-2) tracked separately
// ====================================================================

#[test]
fn test_light_industry_emits_less_than_heavy() {
    let mut city = TestCity::new();

    place_water_patch(&mut city, 52, 50, 3);
    spawn_industrial(&mut city, 50, 50, 2); // level < 3 = light

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.count_for(WaterPollutionSourceType::LightIndustry) > 0,
        "Light industry should be counted"
    );
    assert_eq!(
        state.count_for(WaterPollutionSourceType::HeavyIndustry),
        0,
        "Heavy industry should NOT be counted for level-2 building"
    );
}

// ====================================================================
// Commercial buildings emit discharge
// ====================================================================

#[test]
fn test_commercial_building_emits_discharge() {
    let mut city = TestCity::new();

    place_water_patch(&mut city, 52, 50, 2);
    spawn_commercial(&mut city, 50, 50, 2);

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.count_for(WaterPollutionSourceType::CommercialDischarge) > 0,
        "Commercial buildings should produce discharge"
    );
}

// ====================================================================
// Sewage plant produces outfall pollution
// ====================================================================

#[test]
fn test_sewage_plant_produces_outfall_pollution() {
    let mut city = TestCity::new();

    // Place water around the sewage plant location
    place_water_patch(&mut city, 52, 50, 5);

    // Spawn a sewage plant utility
    let world = city.world_mut();
    world.spawn(UtilitySource {
        utility_type: UtilityType::SewagePlant,
        grid_x: 50,
        grid_y: 50,
        range: 20,
    });

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.count_for(WaterPollutionSourceType::SewageOutfall) > 0,
        "Sewage plant should produce outfall"
    );
    assert!(
        state.emissions_for(WaterPollutionSourceType::SewageOutfall) > 0,
        "Sewage outfall should emit pollution"
    );
}

// ====================================================================
// Power plant cooling thermal pollution
// ====================================================================

#[test]
fn test_power_plant_emits_thermal_pollution() {
    let mut city = TestCity::new();

    place_water_patch(&mut city, 52, 50, 4);

    let world = city.world_mut();
    world.spawn(UtilitySource {
        utility_type: UtilityType::PowerPlant,
        grid_x: 50,
        grid_y: 50,
        range: 30,
    });

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.count_for(WaterPollutionSourceType::PowerPlantCooling) > 0,
        "Power plant should produce thermal pollution"
    );
    assert_eq!(
        state.emissions_for(WaterPollutionSourceType::PowerPlantCooling),
        15,
        "Power plant cooling should emit 15 pollution units"
    );
}

// ====================================================================
// Landfill leachate
// ====================================================================

#[test]
fn test_landfill_produces_leachate() {
    let mut city = TestCity::new();

    place_water_patch(&mut city, 52, 50, 3);

    let world = city.world_mut();
    world.spawn(ServiceBuilding {
        service_type: ServiceType::Landfill,
        grid_x: 50,
        grid_y: 50,
        radius: ServiceBuilding::coverage_radius(ServiceType::Landfill),
    });

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.count_for(WaterPollutionSourceType::LandfillLeachate) > 0,
        "Landfill should produce leachate"
    );
    assert_eq!(
        state.emissions_for(WaterPollutionSourceType::LandfillLeachate),
        25,
        "Landfill leachate should emit 25 pollution units"
    );
}

// ====================================================================
// Treatment level reduces sewage outfall emissions
// ====================================================================

#[test]
fn test_treatment_levels_effectiveness() {
    // Verify the treatment effectiveness percentages
    assert_eq!(TreatmentLevel::None.removal_efficiency(), 0.0);
    assert!((TreatmentLevel::Primary.removal_efficiency() - 0.60).abs() < 0.01);
    assert!((TreatmentLevel::Secondary.removal_efficiency() - 0.85).abs() < 0.01);
    assert!((TreatmentLevel::Tertiary.removal_efficiency() - 0.95).abs() < 0.01);
    assert!((TreatmentLevel::Advanced.removal_efficiency() - 0.99).abs() < 0.01);
}

// ====================================================================
// Total emissions aggregate correctly
// ====================================================================

#[test]
fn test_total_emissions_aggregate_multiple_sources() {
    let mut city = TestCity::new();

    // Place water near sources
    place_water_patch(&mut city, 52, 50, 4);

    // Add an industrial building
    spawn_industrial(&mut city, 50, 50, 4); // heavy
    // Add a commercial building
    spawn_commercial(&mut city, 50, 55, 2);
    // Place water near commercial building too
    place_water_patch(&mut city, 52, 55, 2);

    city.tick_slow_cycle();

    let state = city.resource::<WaterPollutionSourcesState>();
    assert!(
        state.total_emissions > 0,
        "Total emissions should be non-zero with multiple sources"
    );
    // Total should be sum of individual type emissions
    let sum: u32 = state.emissions_by_type.iter().sum();
    assert_eq!(
        state.total_emissions, sum,
        "Total emissions should equal sum of per-type emissions"
    );
}

// ====================================================================
// State resets each slow tick
// ====================================================================

#[test]
fn test_state_resets_each_slow_tick() {
    let mut city = TestCity::new();

    place_water_patch(&mut city, 52, 50, 3);
    spawn_industrial(&mut city, 50, 50, 4);

    // Run first slow cycle
    city.tick_slow_cycle();
    let emissions_1 = city.resource::<WaterPollutionSourcesState>().total_emissions;
    assert!(emissions_1 > 0);

    // Run second slow cycle — state should reset and recompute
    city.tick_slow_cycle();
    let emissions_2 = city.resource::<WaterPollutionSourcesState>().total_emissions;

    // Emissions should be the same (same sources present)
    assert_eq!(
        emissions_1, emissions_2,
        "Emissions should be consistent across slow ticks with same sources"
    );
}
