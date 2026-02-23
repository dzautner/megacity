//! Property-based tests for economy invariants.
//!
//! Uses randomized city configurations to verify that fundamental economic
//! invariants hold across many different scenarios:
//! - Budget income >= 0
//! - Budget expenses >= 0
//! - Tax rates remain in valid range
//! - Citizen salary >= 0
//! - Building occupants <= capacity
//! - Total population matches citizen entity count

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::budget::ExtendedBudget;
use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails};
use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

/// Zone types used for randomized building placement.
const ZONE_TYPES: &[ZoneType] = &[
    ZoneType::ResidentialLow,
    ZoneType::ResidentialMedium,
    ZoneType::ResidentialHigh,
    ZoneType::CommercialLow,
    ZoneType::CommercialHigh,
    ZoneType::Industrial,
    ZoneType::Office,
];

/// Road types used for randomized road placement.
const ROAD_TYPES: &[RoadType] = &[
    RoadType::Local,
    RoadType::Avenue,
    RoadType::Highway,
];

/// Service types used for randomized service placement.
const SERVICE_TYPES: &[ServiceType] = &[
    ServiceType::PoliceStation,
    ServiceType::FireStation,
    ServiceType::Hospital,
];

/// Build a randomized city from a seed. Returns a TestCity with roads,
/// buildings, services, and citizens placed according to the RNG.
fn build_random_city(seed: u64) -> TestCity {
    let mut rng = StdRng::seed_from_u64(seed);

    // Start with roads forming a grid pattern
    let road_type = ROAD_TYPES[rng.gen_range(0..ROAD_TYPES.len())];
    let mut city = TestCity::new()
        .with_road(10, 10, 50, 10, road_type)
        .with_road(10, 10, 10, 50, RoadType::Local);

    // Add some random cross-roads
    let num_extra_roads = rng.gen_range(0..4);
    for i in 0..num_extra_roads {
        let y = 15 + i * 8;
        let rt = ROAD_TYPES[rng.gen_range(0..ROAD_TYPES.len())];
        city = city.with_road(10, y, 50, y, rt);
    }

    // Place random buildings along the roads
    let num_buildings = rng.gen_range(2..8);
    for i in 0..num_buildings {
        let x = 12 + (i * 4) % 36;
        let y = 11 + (i * 3) % 10;
        let zone = ZONE_TYPES[rng.gen_range(0..ZONE_TYPES.len())];
        let max_level = match zone {
            ZoneType::ResidentialLow | ZoneType::CommercialLow => 3,
            _ => 5,
        };
        let level = rng.gen_range(1..=max_level);
        city = city.with_building(x, y, zone, level);
    }

    // Place 0-2 service buildings
    let num_services = rng.gen_range(0..3);
    for i in 0..num_services {
        let x = 20 + i * 10;
        let y = 12;
        let stype = SERVICE_TYPES[rng.gen_range(0..SERVICE_TYPES.len())];
        city = city.with_service(x, y, stype);
    }

    // Place citizens with homes and work locations.
    // We need at least one residential and one commercial/industrial building.
    // Use fixed known-good positions that we placed buildings at.
    // Place a guaranteed residential + commercial pair for citizen spawning.
    city = city
        .with_building(52, 11, ZoneType::ResidentialLow, 1)
        .with_building(54, 11, ZoneType::CommercialLow, 1);

    let num_citizens = rng.gen_range(1..6);
    for _ in 0..num_citizens {
        city = city.with_citizen((52, 11), (54, 11));
    }

    // Randomize the starting budget
    let treasury = rng.gen_range(1000.0..100_000.0);
    city = city.with_budget(treasury);

    city
}

// ---------------------------------------------------------------------------
// Invariant: budget income >= 0 after simulation
// ---------------------------------------------------------------------------

#[test]
fn test_property_budget_income_non_negative_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let budget = city.budget();
        assert!(
            budget.monthly_income >= 0.0,
            "Seed {seed}: monthly_income was negative: {}",
            budget.monthly_income,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: budget expenses >= 0 after simulation
// ---------------------------------------------------------------------------

#[test]
fn test_property_budget_expenses_non_negative_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let budget = city.budget();
        assert!(
            budget.monthly_expenses >= 0.0,
            "Seed {seed}: monthly_expenses was negative: {}",
            budget.monthly_expenses,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: tax rate stays in valid range [0.0, 1.0]
// ---------------------------------------------------------------------------

#[test]
fn test_property_tax_rate_in_valid_range_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let budget = city.budget();
        assert!(
            (0.0..=1.0).contains(&budget.tax_rate),
            "Seed {seed}: tax_rate out of range: {}",
            budget.tax_rate,
        );

        // Also check per-zone tax rates
        let extended = city.resource::<ExtendedBudget>();
        let zt = &extended.zone_taxes;
        assert!(
            zt.residential >= 0.0,
            "Seed {seed}: residential tax rate negative: {}",
            zt.residential,
        );
        assert!(
            zt.commercial >= 0.0,
            "Seed {seed}: commercial tax rate negative: {}",
            zt.commercial,
        );
        assert!(
            zt.industrial >= 0.0,
            "Seed {seed}: industrial tax rate negative: {}",
            zt.industrial,
        );
        assert!(
            zt.office >= 0.0,
            "Seed {seed}: office tax rate negative: {}",
            zt.office,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: citizen salary >= 0
// ---------------------------------------------------------------------------

#[test]
fn test_property_citizen_salary_non_negative_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let world = city.world_mut();
        let mut query = world.query::<(&Citizen, &CitizenDetails)>();
        for (_citizen, details) in query.iter(world) {
            assert!(
                details.salary >= 0.0,
                "Seed {seed}: citizen salary was negative: {}",
                details.salary,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Invariant: building occupants <= capacity
// ---------------------------------------------------------------------------

#[test]
fn test_property_building_occupants_within_capacity_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        for building in query.iter(world) {
            assert!(
                building.occupants <= building.capacity,
                "Seed {seed}: building at ({},{}) has occupants {} > capacity {}",
                building.grid_x,
                building.grid_y,
                building.occupants,
                building.capacity,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Invariant: total population matches citizen entity count
// ---------------------------------------------------------------------------

#[test]
fn test_property_population_matches_citizen_count_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let citizen_count = city.citizen_count();
        // The population resource should track the citizen entity count.
        // Since citizens can immigrate/emigrate, we just verify the count
        // is internally consistent: query count matches the total.
        let world = city.world_mut();
        let entity_count = world
            .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Citizen>>()
            .iter(world)
            .count();

        assert_eq!(
            citizen_count, entity_count,
            "Seed {seed}: citizen_count() = {citizen_count} but query found {entity_count}",
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: income breakdown components are all non-negative
// ---------------------------------------------------------------------------

#[test]
fn test_property_income_breakdown_non_negative_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let extended = city.resource::<ExtendedBudget>().clone();
        let ib = &extended.income_breakdown;

        assert!(
            ib.residential_tax >= 0.0,
            "Seed {seed}: residential_tax negative: {}",
            ib.residential_tax,
        );
        assert!(
            ib.commercial_tax >= 0.0,
            "Seed {seed}: commercial_tax negative: {}",
            ib.commercial_tax,
        );
        assert!(
            ib.industrial_tax >= 0.0,
            "Seed {seed}: industrial_tax negative: {}",
            ib.industrial_tax,
        );
        assert!(
            ib.office_tax >= 0.0,
            "Seed {seed}: office_tax negative: {}",
            ib.office_tax,
        );
        assert!(
            ib.trade_income >= 0.0,
            "Seed {seed}: trade_income negative: {}",
            ib.trade_income,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: expense breakdown components are all non-negative
// ---------------------------------------------------------------------------

#[test]
fn test_property_expense_breakdown_non_negative_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let extended = city.resource::<ExtendedBudget>().clone();
        let eb = &extended.expense_breakdown;

        assert!(
            eb.road_maintenance >= 0.0,
            "Seed {seed}: road_maintenance negative: {}",
            eb.road_maintenance,
        );
        assert!(
            eb.service_costs >= 0.0,
            "Seed {seed}: service_costs negative: {}",
            eb.service_costs,
        );
        assert!(
            eb.policy_costs >= 0.0,
            "Seed {seed}: policy_costs negative: {}",
            eb.policy_costs,
        );
        assert!(
            eb.loan_payments >= 0.0,
            "Seed {seed}: loan_payments negative: {}",
            eb.loan_payments,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: income breakdown sums to monthly_income
// ---------------------------------------------------------------------------

#[test]
fn test_property_income_breakdown_sums_to_total_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let budget = city.budget().clone();
        let extended = city.resource::<ExtendedBudget>().clone();
        let ib = &extended.income_breakdown;

        let sum = ib.residential_tax
            + ib.commercial_tax
            + ib.industrial_tax
            + ib.office_tax
            + ib.trade_income;

        assert!(
            (budget.monthly_income - sum).abs() < 0.01,
            "Seed {seed}: monthly_income={} but breakdown sums to {}",
            budget.monthly_income,
            sum,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: expense breakdown sums to monthly_expenses
// ---------------------------------------------------------------------------

#[test]
fn test_property_expense_breakdown_sums_to_total_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let budget = city.budget().clone();
        let extended = city.resource::<ExtendedBudget>().clone();
        let eb = &extended.expense_breakdown;

        let sum = eb.road_maintenance + eb.service_costs + eb.policy_costs;

        assert!(
            (budget.monthly_expenses - sum).abs() < 0.01,
            "Seed {seed}: monthly_expenses={} but breakdown sums to {}",
            budget.monthly_expenses,
            sum,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: citizen health and happiness remain in [0, 100]
// ---------------------------------------------------------------------------

#[test]
fn test_property_citizen_health_happiness_bounded_across_seeds() {
    for seed in 0..20 {
        let mut city = build_random_city(seed);
        city.tick_slow_cycles(10);

        let world = city.world_mut();
        let mut query = world.query::<(&Citizen, &CitizenDetails)>();
        for (_citizen, details) in query.iter(world) {
            assert!(
                details.happiness >= 0.0 && details.happiness <= 100.0,
                "Seed {seed}: citizen happiness out of range: {}",
                details.happiness,
            );
            assert!(
                details.health >= 0.0 && details.health <= 100.0,
                "Seed {seed}: citizen health out of range: {}",
                details.health,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Invariant: property tax formula is non-negative for non-negative inputs
// ---------------------------------------------------------------------------

#[test]
fn test_property_tax_formula_non_negative_for_random_inputs() {
    use crate::economy::property_tax_for_building;

    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..100 {
        let land_value = rng.gen_range(0.0..1000.0);
        let level = rng.gen_range(1..=5);
        let rate = rng.gen_range(0.0..0.5);

        let tax = property_tax_for_building(land_value, level, rate);
        assert!(
            tax >= 0.0,
            "Property tax negative for land_value={land_value}, level={level}, rate={rate}: {tax}",
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: Tel Aviv full city still satisfies invariants
// ---------------------------------------------------------------------------

#[test]
fn test_property_tel_aviv_economy_invariants_hold() {
    let mut city = TestCity::with_tel_aviv();
    city.tick_slow_cycles(5);

    // Budget invariants
    let budget = city.budget();
    assert!(
        budget.monthly_income >= 0.0,
        "Tel Aviv: monthly_income negative: {}",
        budget.monthly_income,
    );
    assert!(
        budget.monthly_expenses >= 0.0,
        "Tel Aviv: monthly_expenses negative: {}",
        budget.monthly_expenses,
    );
    assert!(
        (0.0..=1.0).contains(&budget.tax_rate),
        "Tel Aviv: tax_rate out of range: {}",
        budget.tax_rate,
    );

    // Building occupants <= capacity
    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        assert!(
            building.occupants <= building.capacity,
            "Tel Aviv: building at ({},{}) occupants {} > capacity {}",
            building.grid_x,
            building.grid_y,
            building.occupants,
            building.capacity,
        );
    }

    // Citizen salary >= 0
    let mut cit_query = world.query::<&CitizenDetails>();
    for details in cit_query.iter(world) {
        assert!(
            details.salary >= 0.0,
            "Tel Aviv: citizen salary negative: {}",
            details.salary,
        );
    }
}
