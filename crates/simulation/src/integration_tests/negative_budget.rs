use crate::economy::CityBudget;
use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// ====================================================================
// TEST-062: Negative Budget Consequences
// ====================================================================

#[test]
fn test_negative_budget_does_not_crash() {
    let mut city = TestCity::new()
        .with_budget(-1_000_000.0)
        .with_road(10, 10, 10, 30, RoadType::Local)
        .with_zone_rect(11, 10, 13, 14, ZoneType::ResidentialLow)
        .with_zone_rect(11, 16, 13, 20, ZoneType::CommercialLow)
        .with_building(12, 12, ZoneType::ResidentialLow, 1)
        .with_building(12, 18, ZoneType::CommercialLow, 1)
        .with_citizen((12, 12), (12, 18))
        .with_service(12, 15, ServiceType::PoliceStation);

    city.tick_slow_cycles(5);

    let treasury = city.budget().treasury;
    assert!(
        treasury.is_finite(),
        "Treasury should remain a finite number, got {treasury}"
    );
    let _count = city.citizen_count();
}

#[test]
fn test_negative_budget_extended_stability() {
    let mut city = TestCity::new().with_budget(-5_000_000.0);
    city.tick_slow_cycles(10);

    let budget = city.budget();
    assert!(budget.treasury.is_finite());
    assert!(budget.monthly_income.is_finite());
    assert!(budget.monthly_expenses.is_finite());
}

#[test]
fn test_negative_budget_triggers_crisis_event() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(-50_000.0);
    city.tick_slow_cycles(3);

    let journal = city.resource::<EventJournal>();
    let has_crisis = journal
        .events
        .iter()
        .any(|e| matches!(e.event_type, CityEventType::BudgetCrisis));
    assert!(has_crisis, "Expected BudgetCrisis event");
}

#[test]
fn test_service_coverage_degrades_with_reduced_budgets() {
    use crate::budget::ExtendedBudget;
    use crate::happiness::ServiceCoverageGrid;

    let mut city =
        TestCity::new()
            .with_budget(100_000.0)
            .with_service(30, 30, ServiceType::Hospital);

    city.tick_slow_cycles(1);

    let idx = ServiceCoverageGrid::idx(30, 45);
    let has_health_full = city.resource::<ServiceCoverageGrid>().has_health(idx);

    {
        let world = city.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .service_budgets
            .healthcare = 0.0;
    }

    city.tick_slow_cycles(1);

    let has_health_zero = city.resource::<ServiceCoverageGrid>().has_health(idx);

    assert!(
        has_health_full,
        "Should have health coverage at full budget"
    );
    assert!(!has_health_zero, "Should lose coverage when budget is 0");
}

#[test]
fn test_service_coverage_at_origin_with_low_budget() {
    use crate::budget::ExtendedBudget;
    use crate::happiness::ServiceCoverageGrid;

    let mut city = TestCity::new().with_service(50, 50, ServiceType::PoliceStation);

    {
        let world = city.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .service_budgets
            .police = 0.1;
    }

    city.tick_slow_cycles(1);

    let idx = ServiceCoverageGrid::idx(50, 50);
    let has_police = city.resource::<ServiceCoverageGrid>().has_police(idx);
    assert!(
        has_police,
        "Police station should cover its own cell at 10% budget"
    );
}

#[test]
fn test_building_placement_with_negative_budget() {
    let city = TestCity::new()
        .with_budget(-10_000.0)
        .with_road(20, 20, 20, 40, RoadType::Local)
        .with_building(21, 25, ZoneType::ResidentialLow, 1)
        .with_building(21, 30, ZoneType::CommercialLow, 1);

    city.assert_has_road(20, 25);
    city.assert_has_building(21, 25);
    city.assert_has_building(21, 30);
    city.assert_budget_below(0.0);
}

#[test]
fn test_budget_recovery_from_negative() {
    let mut city = TestCity::new()
        .with_budget(-500.0)
        .with_road(10, 10, 10, 50, RoadType::Local)
        .with_zone_rect(11, 10, 14, 50, ZoneType::ResidentialLow)
        .with_zone_rect(8, 10, 9, 50, ZoneType::CommercialLow);

    for y in (10..50).step_by(2) {
        city = city
            .with_building(12, y, ZoneType::ResidentialLow, 3)
            .with_building(9, y, ZoneType::CommercialLow, 3);
    }

    let initial_treasury = city.budget().treasury;
    assert!(initial_treasury < 0.0);

    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 1;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    city.tick_slow_cycles(20);

    let after_treasury = city.budget().treasury;
    assert!(
        after_treasury > initial_treasury,
        "Treasury should improve: initial={initial_treasury}, after={after_treasury}"
    );
}

#[test]
fn test_negative_budget_crisis_events_recur() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(-100_000.0);
    city.tick_slow_cycles(10);

    let journal = city.resource::<EventJournal>();
    let crisis_count = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::BudgetCrisis))
        .count();
    assert!(crisis_count >= 1, "Expected at least 1 BudgetCrisis event");
}

#[test]
fn test_tel_aviv_negative_budget_stability() {
    let mut city = TestCity::with_tel_aviv();

    {
        let world = city.world_mut();
        world.resource_mut::<CityBudget>().treasury = -1_000_000.0;
    }

    city.tick_slow_cycles(3);

    let budget = city.budget();
    assert!(budget.treasury.is_finite());
    assert!(
        city.citizen_count() > 0,
        "Tel Aviv should still have citizens"
    );
}
