//! Basic type roundtrip serialization tests.

use super::*;

use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::policies::{Policies, Policy};
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::unlocks::{UnlockNode, UnlockState};
use simulation::utilities::UtilityType;
use simulation::zones::ZoneDemand;

#[test]
fn test_roundtrip_serialization() {
    let mut grid = WorldGrid::new(16, 16);
    simulation::terrain::generate_terrain(&mut grid, 42);

    // Set some zones to test the new types
    grid.get_mut(5, 5).zone = simulation::grid::ZoneType::ResidentialLow;
    grid.get_mut(6, 6).zone = simulation::grid::ZoneType::ResidentialHigh;
    grid.get_mut(7, 7).zone = simulation::grid::ZoneType::CommercialLow;
    grid.get_mut(8, 8).zone = simulation::grid::ZoneType::CommercialHigh;
    grid.get_mut(9, 9).zone = simulation::grid::ZoneType::Office;

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

    assert_eq!(restored.grid.width, 16);
    assert_eq!(restored.grid.height, 16);
    assert_eq!(restored.grid.cells.len(), 256);
    assert_eq!(restored.clock.day, clock.day);
    assert!((restored.budget.treasury - budget.treasury).abs() < 0.01);

    // Verify zone roundtrip
    let idx55 = 5 * 16 + 5;
    assert_eq!(restored.grid.cells[idx55].zone, 1); // ResidentialLow
    let idx66 = 6 * 16 + 6;
    assert_eq!(restored.grid.cells[idx66].zone, 2); // ResidentialHigh
    let idx77 = 7 * 16 + 7;
    assert_eq!(restored.grid.cells[idx77].zone, 3); // CommercialLow
    let idx88 = 8 * 16 + 8;
    assert_eq!(restored.grid.cells[idx88].zone, 4); // CommercialHigh
    let idx99 = 9 * 16 + 9;
    assert_eq!(restored.grid.cells[idx99].zone, 6); // Office

    // V2 fields should be None when not provided
    assert!(restored.policies.is_none());
    assert!(restored.weather.is_none());
    assert!(restored.unlock_state.is_none());
    assert!(restored.extended_budget.is_none());
    assert!(restored.loan_book.is_none());
    assert!(restored.virtual_population.is_none());
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
fn test_zone_type_roundtrip() {
    use simulation::grid::ZoneType;
    let types = [
        ZoneType::None,
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];
    for zt in &types {
        let encoded = zone_type_to_u8(*zt);
        let decoded = u8_to_zone_type(encoded);
        assert_eq!(*zt, decoded);
    }
}

#[test]
fn test_utility_type_roundtrip() {
    let types = [
        UtilityType::PowerPlant,
        UtilityType::SolarFarm,
        UtilityType::WindTurbine,
        UtilityType::WaterTower,
        UtilityType::SewagePlant,
    ];
    for ut in &types {
        let encoded = utility_type_to_u8(*ut);
        let decoded = u8_to_utility_type(encoded);
        assert_eq!(*ut, decoded);
    }
}

#[test]
fn test_service_type_roundtrip() {
    for i in 0..=49u8 {
        let st = u8_to_service_type(i).expect("valid service type");
        let encoded = service_type_to_u8(st);
        assert_eq!(i, encoded);
    }
    assert!(u8_to_service_type(50).is_none());
}

#[test]
fn test_policy_roundtrip() {
    for &p in Policy::all() {
        let encoded = policy_to_u8(p);
        let decoded = u8_to_policy(encoded).expect("valid policy");
        assert_eq!(p, decoded);
    }
    assert!(u8_to_policy(255).is_none());
}

#[test]
fn test_weather_roundtrip() {
    use simulation::weather::{Season, Weather, WeatherCondition};
    let weather = Weather {
        season: Season::Winter,
        temperature: -5.0,
        current_event: WeatherCondition::Snow,
        event_days_remaining: 3,
        last_update_day: 42,
        disasters_enabled: false,
        humidity: 0.8,
        cloud_cover: 0.7,
        precipitation_intensity: 0.5,
        last_update_hour: 14,
        prev_extreme: false,
        ..Default::default()
    };

    let save = SaveWeather {
        season: season_to_u8(weather.season),
        temperature: weather.temperature,
        current_event: weather_event_to_u8(weather.current_event),
        event_days_remaining: weather.event_days_remaining,
        last_update_day: weather.last_update_day,
        disasters_enabled: weather.disasters_enabled,
        humidity: weather.humidity,
        cloud_cover: weather.cloud_cover,
        precipitation_intensity: weather.precipitation_intensity,
        last_update_hour: weather.last_update_hour,
        climate_zone: 0,
    };

    let restored = restore_weather(&save);
    assert_eq!(restored.season, Season::Winter);
    assert!((restored.temperature - (-5.0)).abs() < 0.001);
    assert_eq!(restored.current_event, WeatherCondition::Snow);
    assert_eq!(restored.event_days_remaining, 3);
    assert_eq!(restored.last_update_day, 42);
    assert!(!restored.disasters_enabled);
    assert!((restored.humidity - 0.8).abs() < 0.001);
    assert!((restored.cloud_cover - 0.7).abs() < 0.001);
    assert!((restored.precipitation_intensity - 0.5).abs() < 0.001);
    assert_eq!(restored.last_update_hour, 14);
}

#[test]
fn test_unlock_state_roundtrip() {
    let mut state = UnlockState::default();
    state.development_points = 10;
    state.spent_points = 3;
    state.last_milestone_pop = 2000;
    // Default already has BasicRoads, etc. Add another
    state.unlocked_nodes.push(UnlockNode::FireService);

    let save = SaveUnlockState {
        development_points: state.development_points,
        spent_points: state.spent_points,
        unlocked_nodes: state
            .unlocked_nodes
            .iter()
            .map(|&n| unlock_node_to_u8(n))
            .collect(),
        last_milestone_pop: state.last_milestone_pop,
    };

    let restored = restore_unlock_state(&save);
    assert_eq!(restored.development_points, 10);
    assert_eq!(restored.spent_points, 3);
    assert_eq!(restored.last_milestone_pop, 2000);
    assert!(restored.is_unlocked(UnlockNode::BasicRoads));
    assert!(restored.is_unlocked(UnlockNode::FireService));
    assert!(!restored.is_unlocked(UnlockNode::NuclearPower));
}

#[test]
fn test_unlock_node_roundtrip() {
    for &n in UnlockNode::all() {
        let encoded = unlock_node_to_u8(n);
        let decoded = u8_to_unlock_node(encoded).expect("valid unlock node");
        assert_eq!(n, decoded);
    }
    assert!(u8_to_unlock_node(255).is_none());
}

#[test]
fn test_policies_serialize_roundtrip() {
    let policies = Policies {
        active: vec![
            Policy::FreePublicTransport,
            Policy::RecyclingProgram,
            Policy::HighRiseBan,
        ],
    };

    let save = SavePolicies {
        active: policies.active.iter().map(|&p| policy_to_u8(p)).collect(),
    };

    let restored = restore_policies(&save);
    assert_eq!(restored.active.len(), 3);
    assert!(restored.is_active(Policy::FreePublicTransport));
    assert!(restored.is_active(Policy::RecyclingProgram));
    assert!(restored.is_active(Policy::HighRiseBan));
    assert!(!restored.is_active(Policy::EducationPush));
}

#[test]
fn test_extended_budget_roundtrip() {
    let save = SaveExtendedBudget {
        residential_tax: 0.12,
        commercial_tax: 0.08,
        industrial_tax: 0.15,
        office_tax: 0.11,
        fire_budget: 1.2,
        police_budget: 0.8,
        healthcare_budget: 1.0,
        education_budget: 1.5,
        sanitation_budget: 0.5,
        transport_budget: 1.1,
    };

    let restored = restore_extended_budget(&save);
    assert!((restored.zone_taxes.residential - 0.12).abs() < 0.001);
    assert!((restored.zone_taxes.commercial - 0.08).abs() < 0.001);
    assert!((restored.zone_taxes.industrial - 0.15).abs() < 0.001);
    assert!((restored.zone_taxes.office - 0.11).abs() < 0.001);
    assert!((restored.service_budgets.fire - 1.2).abs() < 0.001);
    assert!((restored.service_budgets.police - 0.8).abs() < 0.001);
    assert!((restored.service_budgets.education - 1.5).abs() < 0.001);
}

#[test]
fn test_loan_book_roundtrip() {
    let save = SaveLoanBook {
        loans: vec![SaveLoan {
            name: "Small Loan".into(),
            amount: 10_000.0,
            interest_rate: 0.05,
            monthly_payment: 856.07,
            remaining_balance: 8_500.0,
            term_months: 12,
            months_paid: 2,
        }],
        max_loans: 3,
        credit_rating: 1.5,
        last_payment_day: 60,
        consecutive_solvent_days: 45,
    };

    let restored = restore_loan_book(&save);
    assert_eq!(restored.active_loans.len(), 1);
    assert_eq!(restored.active_loans[0].name, "Small Loan");
    assert!((restored.active_loans[0].amount - 10_000.0).abs() < 0.01);
    assert!((restored.active_loans[0].remaining_balance - 8_500.0).abs() < 0.01);
    assert_eq!(restored.active_loans[0].months_paid, 2);
    assert_eq!(restored.max_loans, 3);
    assert!((restored.credit_rating - 1.5).abs() < 0.001);
    assert_eq!(restored.last_payment_day, 60);
    assert_eq!(restored.consecutive_solvent_days, 45);
}
