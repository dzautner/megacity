use crate::economy::CityBudget;
use crate::grid::{RoadType, ZoneType};
use crate::policies::{Policies, Policy};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// ====================================================================
// Policy system tests (issue #845)
// ====================================================================

#[test]
fn test_policy_resource_exists_in_new_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<Policies>();
}

#[test]
fn test_policy_default_state_has_no_active_policies() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert!(
        policies.active.is_empty(),
        "new city should have no active policies"
    );
}

#[test]
fn test_policy_toggle_enables_policy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::RecyclingProgram);
    }
    let policies = city.resource::<Policies>();
    assert!(
        policies.is_active(Policy::RecyclingProgram),
        "RecyclingProgram should be active after toggle"
    );
}

#[test]
fn test_policy_toggle_disables_policy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::RecyclingProgram);
        policies.toggle(Policy::RecyclingProgram);
    }
    let policies = city.resource::<Policies>();
    assert!(
        !policies.is_active(Policy::RecyclingProgram),
        "RecyclingProgram should be inactive after double toggle"
    );
}

#[test]
fn test_policy_multiple_active_policies_stack() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport);
        policies.toggle(Policy::NightShiftBan);
        policies.toggle(Policy::HealthcareForAll);
        policies.toggle(Policy::NeighborhoodWatch);
    }
    let policies = city.resource::<Policies>();
    // FreePublicTransport: +3, NightShiftBan: +3, HealthcareForAll: +2, NeighborhoodWatch: +2
    let expected_happiness = 3.0 + 3.0 + 2.0 + 2.0;
    let actual = policies.happiness_bonus();
    assert!(
        (actual - expected_happiness).abs() < f32::EPSILON,
        "stacked happiness bonus should be {expected_happiness}, got {actual}"
    );
}

#[test]
fn test_policy_total_monthly_cost_single_policy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::FreePublicTransport);
    }
    let policies = city.resource::<Policies>();
    let expected = 50.0;
    let actual = policies.total_monthly_cost();
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "FreePublicTransport should cost {expected}/month, got {actual}"
    );
}

#[test]
fn test_policy_total_monthly_cost_multiple_policies() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport); // 50
        policies.toggle(Policy::RecyclingProgram); // 20
        policies.toggle(Policy::EducationPush); // 40
    }
    let policies = city.resource::<Policies>();
    let expected = 50.0 + 20.0 + 40.0;
    let actual = policies.total_monthly_cost();
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "total monthly cost should be {expected}, got {actual}"
    );
}

#[test]
fn test_policy_zero_cost_policies_do_not_add_expense() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::HighRiseBan); // 0
        policies.toggle(Policy::NightShiftBan); // 0
        policies.toggle(Policy::CumulativeZoning); // 0
    }
    let policies = city.resource::<Policies>();
    let actual = policies.total_monthly_cost();
    assert!(
        actual.abs() < f64::EPSILON,
        "zero-cost policies should have 0 total cost, got {actual}"
    );
}

#[test]
fn test_policy_pollution_multiplier_with_air_filters() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::IndustrialAirFilters);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.pollution_multiplier();
    assert!(
        (mult - 0.6).abs() < f32::EPSILON,
        "pollution multiplier with IndustrialAirFilters should be 0.6, got {mult}"
    );
}

#[test]
fn test_policy_pollution_multiplier_without_air_filters() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    let mult = policies.pollution_multiplier();
    assert!(
        (mult - 1.0).abs() < f32::EPSILON,
        "pollution multiplier without IndustrialAirFilters should be 1.0, got {mult}"
    );
}

#[test]
fn test_policy_garbage_multiplier_with_recycling() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::RecyclingProgram);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.garbage_multiplier();
    assert!(
        (mult - 0.7).abs() < f32::EPSILON,
        "garbage multiplier with RecyclingProgram should be 0.7, got {mult}"
    );
}

#[test]
fn test_policy_park_multiplier_with_green_space() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::GreenSpaceInitiative);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.park_multiplier();
    assert!(
        (mult - 1.5).abs() < f32::EPSILON,
        "park multiplier with GreenSpaceInitiative should be 1.5, got {mult}"
    );
}

#[test]
fn test_policy_max_building_level_with_high_rise_ban() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<Policies>().toggle(Policy::HighRiseBan);
    }
    let policies = city.resource::<Policies>();
    assert_eq!(
        policies.max_building_level(),
        2,
        "max building level with HighRiseBan should be 2"
    );
}

#[test]
fn test_policy_max_building_level_without_high_rise_ban() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert_eq!(
        policies.max_building_level(),
        3,
        "max building level without HighRiseBan should be 3"
    );
}

#[test]
fn test_policy_industrial_tax_multiplier_with_tax_break() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::HeavyIndustryTaxBreak);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.industrial_tax_multiplier();
    assert!(
        (mult - 0.5).abs() < f32::EPSILON,
        "industrial tax multiplier with HeavyIndustryTaxBreak should be 0.5, got {mult}"
    );
}

#[test]
fn test_policy_commercial_demand_bonus_stacks() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::TourismPromotion); // +0.15
        policies.toggle(Policy::SmallBusinessGrant); // +0.10
    }
    let policies = city.resource::<Policies>();
    let expected = 0.15 + 0.10;
    let actual = policies.commercial_demand_bonus();
    assert!(
        (actual - expected).abs() < f32::EPSILON,
        "commercial demand bonus should be {expected}, got {actual}"
    );
}

#[test]
fn test_policy_education_multiplier_with_education_push() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::EducationPush);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.education_multiplier();
    assert!(
        (mult - 1.5).abs() < f32::EPSILON,
        "education multiplier with EducationPush should be 1.5, got {mult}"
    );
}

#[test]
fn test_policy_industrial_demand_bonus_with_tax_break() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::HeavyIndustryTaxBreak);
    }
    let policies = city.resource::<Policies>();
    let bonus = policies.industrial_demand_bonus();
    assert!(
        (bonus - 0.15).abs() < f32::EPSILON,
        "industrial demand bonus with HeavyIndustryTaxBreak should be 0.15, got {bonus}"
    );
}

#[test]
fn test_policy_disabling_removes_effects() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::IndustrialAirFilters);
        policies.toggle(Policy::RecyclingProgram);
    }
    // Verify effects are active
    assert!(
        (city.resource::<Policies>().pollution_multiplier() - 0.6).abs() < f32::EPSILON,
        "pollution multiplier should be 0.6 when active"
    );
    assert!(
        (city.resource::<Policies>().garbage_multiplier() - 0.7).abs() < f32::EPSILON,
        "garbage multiplier should be 0.7 when active"
    );

    // Disable them
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::IndustrialAirFilters);
        policies.toggle(Policy::RecyclingProgram);
    }
    // Verify effects are removed
    assert!(
        (city.resource::<Policies>().pollution_multiplier() - 1.0).abs() < f32::EPSILON,
        "pollution multiplier should return to 1.0 after disabling"
    );
    assert!(
        (city.resource::<Policies>().garbage_multiplier() - 1.0).abs() < f32::EPSILON,
        "garbage multiplier should return to 1.0 after disabling"
    );
}

#[test]
fn test_policy_cost_deducted_from_budget_after_tax_collection() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 50, 100, 50, RoadType::Local)
        .with_zone_rect(11, 48, 20, 49, ZoneType::ResidentialLow)
        .with_building(15, 48, ZoneType::ResidentialLow, 1);
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport); // 50/month
        policies.toggle(Policy::HealthcareForAll); // 45/month
    }
    // Advance the game clock past the 30-day tax collection interval
    // so that collect_taxes fires on the next tick.
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }
    // Run a slow cycle so collect_taxes executes
    city.tick_slow_cycle();
    // The expense breakdown should show policy costs
    let extended = city.resource::<crate::budget::ExtendedBudget>();
    let policy_costs = extended.expense_breakdown.policy_costs;
    let expected_policy_cost = 50.0 + 45.0;
    assert!(
        (policy_costs - expected_policy_cost).abs() < f64::EPSILON,
        "policy costs in expense breakdown should be {expected_policy_cost}, got {policy_costs}"
    );
    // Monthly expenses should include policy costs
    let budget = city.budget();
    assert!(
        budget.monthly_expenses >= expected_policy_cost,
        "monthly expenses ({}) should include policy costs ({expected_policy_cost})",
        budget.monthly_expenses
    );
}

#[test]
fn test_policy_no_cost_when_no_policies_active() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 50, 100, 50, RoadType::Local)
        .with_building(15, 48, ZoneType::ResidentialLow, 1);
    // Advance the game clock past the 30-day tax collection interval
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }
    city.tick_slow_cycle();
    let extended = city.resource::<crate::budget::ExtendedBudget>();
    let policy_costs = extended.expense_breakdown.policy_costs;
    assert!(
        policy_costs.abs() < f64::EPSILON,
        "policy costs should be 0 with no active policies, got {policy_costs}"
    );
}

#[test]
fn test_policy_happiness_bonus_stacks_correctly() {
    let mut city = TestCity::new();
    // Enable one happiness policy at a time and verify stacking
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::FreePublicTransport); // +3
    }
    assert!(
        (city.resource::<Policies>().happiness_bonus() - 3.0).abs() < f32::EPSILON,
        "single policy should give +3 happiness"
    );

    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::NeighborhoodWatch); // +2
    }
    assert!(
        (city.resource::<Policies>().happiness_bonus() - 5.0).abs() < f32::EPSILON,
        "two policies should give +5 happiness (3+2)"
    );
}

#[test]
fn test_policy_all_returns_all_variants() {
    let all = Policy::all();
    assert_eq!(all.len(), 18, "Policy::all() should return all 18 policies");
    // Verify a few known policies exist
    assert!(
        all.contains(&Policy::FreePublicTransport),
        "should contain FreePublicTransport"
    );
    assert!(
        all.contains(&Policy::EncourageBiking),
        "should contain EncourageBiking"
    );
    assert!(
        all.contains(&Policy::CumulativeZoning),
        "should contain CumulativeZoning"
    );
}

#[test]
fn test_policy_each_has_nonempty_name_and_description() {
    for policy in Policy::all() {
        let name = policy.name();
        let desc = policy.description();
        assert!(
            !name.is_empty(),
            "policy {:?} should have a non-empty name",
            policy
        );
        assert!(
            !desc.is_empty(),
            "policy {:?} should have a non-empty description",
            policy
        );
    }
}

#[test]
fn test_policy_disable_removes_from_cost_calculation() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport); // 50
        policies.toggle(Policy::RecyclingProgram); // 20
    }
    assert!(
        (city.resource::<Policies>().total_monthly_cost() - 70.0).abs() < f64::EPSILON,
        "total cost should be 70 with both active"
    );

    // Disable FreePublicTransport
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::FreePublicTransport);
    }
    assert!(
        (city.resource::<Policies>().total_monthly_cost() - 20.0).abs() < f64::EPSILON,
        "total cost should be 20 after disabling FreePublicTransport"
    );
}

#[test]
fn test_policy_commercial_demand_bonus_zero_without_policies() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert!(
        policies.commercial_demand_bonus().abs() < f32::EPSILON,
        "commercial demand bonus should be 0 without policies"
    );
}

#[test]
fn test_policy_industrial_demand_bonus_zero_without_tax_break() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert!(
        policies.industrial_demand_bonus().abs() < f32::EPSILON,
        "industrial demand bonus should be 0 without tax break"
    );
}
