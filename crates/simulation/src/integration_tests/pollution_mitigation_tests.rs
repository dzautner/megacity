//! Integration tests for POLL-004: Air Pollution Mitigation Policies.

use crate::coal_power::PowerPlant;
use crate::grid::ZoneType;
use crate::pollution::PollutionGrid;
use crate::pollution_mitigation::PollutionMitigationPolicies;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::wind::WindState;

// ====================================================================
// Scrubbers on Power Plants
// ====================================================================

#[test]
fn test_scrubbers_reduce_power_plant_pollution() {
    // City with a coal power plant, no scrubbers
    let mut city_no_scrub = TestCity::new();
    {
        let world = city_no_scrub.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        // Spawn a coal power plant
        world.spawn(PowerPlant::new_coal(50, 50));
    }
    city_no_scrub.tick_slow_cycle();
    let p_no_scrub = city_no_scrub.resource::<PollutionGrid>().get(50, 50);

    // City with a coal power plant, scrubbers enabled
    let mut city_scrub = TestCity::new();
    {
        let world = city_scrub.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world.spawn(PowerPlant::new_coal(50, 50));
        world
            .resource_mut::<PollutionMitigationPolicies>()
            .scrubbers_on_power_plants = true;
    }
    city_scrub.tick_slow_cycle();
    let p_scrub = city_scrub.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_no_scrub > 0,
        "Power plant should emit pollution without scrubbers, got {p_no_scrub}"
    );
    assert!(
        p_scrub < p_no_scrub,
        "Scrubbers should reduce pollution: with={p_scrub}, without={p_no_scrub}"
    );
}

// ====================================================================
// Catalytic Converters
// ====================================================================

#[test]
fn test_catalytic_converters_reduce_road_pollution() {
    use crate::grid::{CellType, RoadType};

    // City with roads, no catalytic converters
    let mut city_no_cat = TestCity::new();
    {
        let world = city_no_cat.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        // Place a strip of roads
        for x in 45..55 {
            let cell = grid.get_mut(x, 50);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::TwoLane;
        }
    }
    city_no_cat.tick_slow_cycle();
    let p_no_cat = city_no_cat.resource::<PollutionGrid>().get(50, 50);

    // City with roads, catalytic converters enabled
    let mut city_cat = TestCity::new();
    {
        let world = city_cat.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 45..55 {
            let cell = grid.get_mut(x, 50);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::TwoLane;
        }
        world
            .resource_mut::<PollutionMitigationPolicies>()
            .catalytic_converters = true;
    }
    city_cat.tick_slow_cycle();
    let p_cat = city_cat.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_no_cat > 0,
        "Roads should emit pollution without converters, got {p_no_cat}"
    );
    assert!(
        p_cat < p_no_cat,
        "Catalytic converters should reduce road pollution: with={p_cat}, without={p_no_cat}"
    );
}

// ====================================================================
// Electric Vehicle Mandate (phased rollout)
// ====================================================================

#[test]
fn test_ev_mandate_reduces_road_pollution_progressively() {
    use crate::grid::{CellType, RoadType};

    // City with EV mandate at half phase-in
    let mut city_half = TestCity::new();
    {
        let world = city_half.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 45..55 {
            let cell = grid.get_mut(x, 50);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::TwoLane;
        }
        let mut mit = world.resource_mut::<PollutionMitigationPolicies>();
        mit.ev_mandate = true;
        mit.ev_mandate_activation_day = Some(0);
        // Set clock to halfway through phase-in
        world.resource_mut::<GameClock>().day = 5 * 360 / 2;
    }
    city_half.tick_slow_cycle();
    let p_half = city_half.resource::<PollutionGrid>().get(50, 50);

    // City with EV mandate at full phase-in
    let mut city_full = TestCity::new();
    {
        let world = city_full.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 45..55 {
            let cell = grid.get_mut(x, 50);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::TwoLane;
        }
        let mut mit = world.resource_mut::<PollutionMitigationPolicies>();
        mit.ev_mandate = true;
        mit.ev_mandate_activation_day = Some(0);
        // Set clock to full phase-in
        world.resource_mut::<GameClock>().day = 5 * 360;
    }
    city_full.tick_slow_cycle();
    let p_full = city_full.resource::<PollutionGrid>().get(50, 50);

    // Full phase-in should produce less or equal pollution than half
    assert!(
        p_full <= p_half,
        "Full EV mandate ({p_full}) should produce <= pollution than half phase ({p_half})"
    );
}

// ====================================================================
// Emissions Cap
// ====================================================================

#[test]
fn test_emissions_cap_reduces_industrial_pollution() {
    // City with industrial buildings, no cap
    let mut city_no_cap = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3);
    {
        let world = city_no_cap.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_no_cap.tick_slow_cycle();
    let p_no_cap = city_no_cap.resource::<PollutionGrid>().get(50, 50);

    // City with industrial buildings, emissions cap
    let mut city_cap = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3);
    {
        let world = city_cap.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world
            .resource_mut::<PollutionMitigationPolicies>()
            .emissions_cap = true;
    }
    city_cap.tick_slow_cycle();
    let p_cap = city_cap.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_no_cap > 0,
        "Industrial should emit pollution without cap, got {p_no_cap}"
    );
    assert!(
        p_cap < p_no_cap,
        "Emissions cap should reduce pollution: with={p_cap}, without={p_no_cap}"
    );
}

// ====================================================================
// Emissions Cap industrial profit penalty
// ====================================================================

#[test]
fn test_emissions_cap_profit_penalty() {
    let mut policies = PollutionMitigationPolicies::default();
    policies.emissions_cap = true;

    let mult = policies.industrial_profit_multiplier();
    assert!(
        (mult - 0.9).abs() < f32::EPSILON,
        "Emissions cap should reduce industrial profit by 10%, got multiplier {mult}"
    );
}

// ====================================================================
// Default policies have no effect on pollution
// ====================================================================

#[test]
fn test_default_mitigation_no_pollution_change() {
    // Two identical cities: one with default mitigation, one without
    let mut city_a = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 2);
    {
        let world = city_a.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_a.tick_slow_cycle();
    let p_a = city_a.resource::<PollutionGrid>().get(50, 50);

    let mut city_b = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 2);
    {
        let world = city_b.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_b.tick_slow_cycle();
    let p_b = city_b.resource::<PollutionGrid>().get(50, 50);

    assert_eq!(
        p_a, p_b,
        "Default mitigation policies should not change pollution: a={p_a}, b={p_b}"
    );
}

// ====================================================================
// Multiple policies stack
// ====================================================================

#[test]
fn test_multiple_policies_stack_for_greater_reduction() {
    use crate::grid::{CellType, RoadType};

    // City with only catalytic converters
    let mut city_cat_only = TestCity::new();
    {
        let world = city_cat_only.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 45..55 {
            let cell = grid.get_mut(x, 50);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::TwoLane;
        }
        world
            .resource_mut::<PollutionMitigationPolicies>()
            .catalytic_converters = true;
    }
    city_cat_only.tick_slow_cycle();
    let p_cat = city_cat_only.resource::<PollutionGrid>().get(50, 50);

    // City with both catalytic converters AND fully phased EV mandate
    let mut city_both = TestCity::new();
    {
        let world = city_both.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 45..55 {
            let cell = grid.get_mut(x, 50);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::TwoLane;
        }
        let mut mit = world.resource_mut::<PollutionMitigationPolicies>();
        mit.catalytic_converters = true;
        mit.ev_mandate = true;
        mit.ev_mandate_activation_day = Some(0);
        world.resource_mut::<GameClock>().day = 5 * 360;
    }
    city_both.tick_slow_cycle();
    let p_both = city_both.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_both <= p_cat,
        "Both policies ({p_both}) should reduce at least as much as cat alone ({p_cat})"
    );
}

// ====================================================================
// Saveable round-trip
// ====================================================================

#[test]
fn test_pollution_mitigation_saveable_roundtrip() {
    use crate::Saveable;

    let mut original = PollutionMitigationPolicies::default();
    original.scrubbers_on_power_plants = true;
    original.ev_mandate = true;
    original.ev_mandate_activation_day = Some(42);
    original.emissions_cap = true;

    let bytes = original.save_to_bytes().expect("should produce bytes");
    let restored = PollutionMitigationPolicies::load_from_bytes(&bytes);

    assert_eq!(
        restored.scrubbers_on_power_plants,
        original.scrubbers_on_power_plants
    );
    assert_eq!(restored.ev_mandate, original.ev_mandate);
    assert_eq!(
        restored.ev_mandate_activation_day,
        original.ev_mandate_activation_day
    );
    assert_eq!(restored.emissions_cap, original.emissions_cap);
    assert_eq!(
        restored.catalytic_converters,
        original.catalytic_converters
    );
}

#[test]
fn test_pollution_mitigation_saveable_skips_default() {
    use crate::Saveable;

    let default = PollutionMitigationPolicies::default();
    assert!(
        default.save_to_bytes().is_none(),
        "Default state should skip save"
    );
}
