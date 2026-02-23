//! Stormwater grid, climate zone, and construction modifiers tests.

use super::*;

use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::stormwater::StormwaterGrid;
use simulation::time_of_day::GameClock;
use simulation::weather::{ClimateZone, Season, Weather, WeatherCondition};
use simulation::zones::ZoneDemand;

#[test]
fn test_stormwater_grid_roundtrip() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let mut sw = StormwaterGrid::default();
    sw.runoff[0] = 10.5;
    sw.runoff[5] = 3.2;
    sw.total_runoff = 13.7;
    sw.total_infiltration = 5.0;

    let save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&sw),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");

    let rsw = restored
        .stormwater_grid
        .as_ref()
        .expect("stormwater_grid present");
    assert!((rsw.runoff[0] - 10.5).abs() < 0.001);
    assert!((rsw.runoff[5] - 3.2).abs() < 0.001);
    assert!((rsw.total_runoff - 13.7).abs() < 0.001);
    assert!((rsw.total_infiltration - 5.0).abs() < 0.001);

    let restored_sw = restore_stormwater_grid(rsw);
    assert!((restored_sw.runoff[0] - 10.5).abs() < 0.001);
    assert!((restored_sw.total_runoff - 13.7).abs() < 0.001);
}

#[test]
fn test_stormwater_backward_compat() {
    // Saves without stormwater_grid should have it as None
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert!(restored.stormwater_grid.is_none());
}

#[test]
fn test_climate_zone_roundtrip() {
    for &zone in ClimateZone::all() {
        let encoded = climate_zone_to_u8(zone);
        let decoded = u8_to_climate_zone(encoded);
        assert_eq!(zone, decoded, "ClimateZone roundtrip failed for {:?}", zone);
    }
    // Fallback for unknown values
    assert_eq!(u8_to_climate_zone(255), ClimateZone::Temperate);
}

#[test]
fn test_construction_modifiers_roundtrip() {
    let cm = ConstructionModifiers {
        speed_factor: 0.55,
        cost_factor: 1.25,
    };

    let save = SaveConstructionModifiers {
        speed_factor: cm.speed_factor,
        cost_factor: cm.cost_factor,
    };

    let restored = restore_construction_modifiers(&save);
    assert!((restored.speed_factor - 0.55).abs() < 0.001);
    assert!((restored.cost_factor - 1.25).abs() < 0.001);
}

#[test]
fn test_climate_zone_save_roundtrip() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let weather = Weather {
        season: Season::Summer,
        temperature: 32.0,
        current_event: WeatherCondition::Sunny,
        event_days_remaining: 4,
        last_update_day: 100,
        disasters_enabled: true,
        humidity: 0.3,
        cloud_cover: 0.05,
        precipitation_intensity: 0.0,
        last_update_hour: 12,
        prev_extreme: false,
        ..Default::default()
    };

    let climate_zone = ClimateZone::Tropical;

    let save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        Some(&weather),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&climate_zone),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");

    let rw = restored.weather.as_ref().expect("weather present");
    let restored_zone = restore_climate_zone(rw);
    assert_eq!(restored_zone, ClimateZone::Tropical);
}

#[test]
fn test_climate_zone_backward_compat_defaults_to_temperate() {
    // Old saves without climate_zone field should default to Temperate (0)
    let save = SaveWeather::default();
    let zone = restore_climate_zone(&save);
    assert_eq!(zone, ClimateZone::Temperate);
}

#[test]
fn test_construction_modifiers_backward_compat() {
    // Saves without construction_modifiers should have it as None
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert!(restored.stormwater_grid.is_none());
    // When construction_modifiers is None, the restore uses default
    assert!(restored.construction_modifiers.is_none());
    assert!(restored.recycling_state.is_none());
    assert!(restored.wind_damage_state.is_none());
    assert!(restored.uhi_grid.is_none());
    assert!(restored.drought_state.is_none());
    assert!(restored.heat_wave_state.is_none());
    assert!(restored.composting_state.is_none());
    assert!(restored.cold_snap_state.is_none());
    assert!(restored.water_treatment_state.is_none());
    assert!(restored.groundwater_depletion_state.is_none());
    assert!(restored.wastewater_state.is_none());
    assert!(restored.hazardous_waste_state.is_none());
    assert!(restored.storm_drainage_state.is_none());
    assert!(restored.landfill_capacity_state.is_none());
    assert!(restored.flood_state.is_none());
    assert!(restored.reservoir_state.is_none());
    assert!(restored.landfill_gas_state.is_none());
    assert!(restored.cso_state.is_none());
    assert!(restored.water_conservation_state.is_none());
    assert!(restored.fog_state.is_none());
    assert!(restored.urban_growth_boundary.is_none());
    assert!(restored.snow_state.is_none());
    assert!(restored.agriculture_state.is_none());
}
