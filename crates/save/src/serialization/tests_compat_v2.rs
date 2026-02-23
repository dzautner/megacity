//! V2 full roundtrip, backward compatibility, and lifecycle timer tests.

use super::*;

use simulation::budget::{ExtendedBudget, ServiceBudgets, ZoneTaxRates};
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::{self, LoanBook};
use simulation::policies::{Policies, Policy};
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::unlocks::{UnlockNode, UnlockState};
use simulation::weather::{Season, Weather, WeatherCondition};
use simulation::zones::ZoneDemand;

#[test]
fn test_v2_full_roundtrip() {
    // Test that all V2 fields survive a full encode/decode cycle
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let policies = Policies {
        active: vec![Policy::EducationPush, Policy::WaterConservation],
    };
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
    let mut unlock = UnlockState::default();
    unlock.development_points = 15;
    unlock.spent_points = 5;
    unlock.unlocked_nodes.push(UnlockNode::HealthCare);
    unlock.last_milestone_pop = 5000;

    let ext_budget = ExtendedBudget {
        zone_taxes: ZoneTaxRates {
            residential: 0.12,
            commercial: 0.09,
            industrial: 0.14,
            office: 0.11,
        },
        service_budgets: ServiceBudgets {
            fire: 1.3,
            police: 0.9,
            healthcare: 1.0,
            education: 1.2,
            sanitation: 0.7,
            transport: 1.1,
        },
        loans: Vec::new(),
        income_breakdown: Default::default(),
        expense_breakdown: Default::default(),
    };

    let mut loan_book = LoanBook::default();
    let mut treasury = 0.0;
    loan_book.take_loan(loans::LoanTier::Small, &mut treasury);

    let lifecycle_timer = LifecycleTimer {
        last_aging_day: 200,
        last_emigration_tick: 15,
    };

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
        Some(&policies),
        Some(&weather),
        Some(&unlock),
        Some(&ext_budget),
        Some(&loan_book),
        Some(&lifecycle_timer),
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
    let restored = SaveData::decode(&bytes).expect("decode v2 should succeed");

    // Policies
    let rp = restored.policies.as_ref().expect("policies present");
    assert_eq!(rp.active.len(), 2);

    // Weather
    let rw = restored.weather.as_ref().expect("weather present");
    assert_eq!(rw.season, season_to_u8(Season::Summer));
    assert!((rw.temperature - 32.0).abs() < 0.001);
    assert_eq!(
        rw.current_event,
        weather_event_to_u8(WeatherCondition::Sunny)
    );

    // Unlock state
    let ru = restored
        .unlock_state
        .as_ref()
        .expect("unlock_state present");
    assert_eq!(ru.development_points, 15);
    assert_eq!(ru.spent_points, 5);
    assert_eq!(ru.last_milestone_pop, 5000);

    // Extended budget
    let reb = restored
        .extended_budget
        .as_ref()
        .expect("extended_budget present");
    assert!((reb.fire_budget - 1.3).abs() < 0.001);
    assert!((reb.residential_tax - 0.12).abs() < 0.001);

    // Loan book
    let rlb = restored.loan_book.as_ref().expect("loan_book present");
    assert_eq!(rlb.loans.len(), 1);
    assert_eq!(rlb.loans[0].name, "Small Loan");
}

#[test]
fn test_backward_compat_v1_defaults() {
    // Simulate a V1 save that has no V2 fields: create a SaveData with
    // all V2 fields set to None, encode it, decode it, and verify defaults work.
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
    let restored = SaveData::decode(&bytes).expect("decode v1 should succeed");

    // V2 fields should be None
    assert!(restored.policies.is_none());
    assert!(restored.weather.is_none());
    assert!(restored.unlock_state.is_none());
    assert!(restored.extended_budget.is_none());
    assert!(restored.loan_book.is_none());
    assert!(restored.lifecycle_timer.is_none());
    assert!(restored.virtual_population.is_none());
    assert!(restored.life_sim_timer.is_none());
    assert!(restored.stormwater_grid.is_none());
    assert!(restored.degree_days.is_none());
    assert!(restored.water_sources.is_none());
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

#[test]
fn test_lifecycle_timer_roundtrip() {
    let timer = LifecycleTimer {
        last_aging_day: 730,
        last_emigration_tick: 25,
    };

    let save = SaveLifecycleTimer {
        last_aging_day: timer.last_aging_day,
        last_emigration_tick: timer.last_emigration_tick,
    };

    let restored = restore_lifecycle_timer(&save);
    assert_eq!(restored.last_aging_day, 730);
    assert_eq!(restored.last_emigration_tick, 25);
}
