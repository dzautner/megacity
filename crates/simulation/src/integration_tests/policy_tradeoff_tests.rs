//! Integration tests for policy tradeoff definitions and effects (POL-001).

use crate::policies::{Policies, Policy};
use crate::policy_tradeoffs::{
    compute_effects, get_tradeoff, PolicyCategory, PolicyTradeoffEffects,
};
use crate::test_harness::TestCity;

// ====================================================================
// Tradeoff definition tests
// ====================================================================

#[test]
fn test_tradeoff_every_policy_has_definition() {
    for &policy in Policy::all() {
        let tradeoff = get_tradeoff(policy);
        assert_eq!(tradeoff.policy, policy);
        assert!(
            !tradeoff.benefits.is_empty(),
            "policy {:?} should have at least one benefit",
            policy
        );
        assert!(
            !tradeoff.drawbacks.is_empty(),
            "policy {:?} should have at least one drawback",
            policy
        );
    }
}

#[test]
fn test_tradeoff_at_least_15_policies_exist() {
    assert!(
        Policy::all().len() >= 15,
        "should have at least 15 policies, got {}",
        Policy::all().len()
    );
}

#[test]
fn test_tradeoff_policy_count_is_29() {
    assert_eq!(Policy::all().len(), 29, "should have exactly 29 policies");
}

#[test]
fn test_tradeoff_categories_cover_all_domains() {
    let categories: std::collections::HashSet<PolicyCategory> = Policy::all()
        .iter()
        .map(|&p| get_tradeoff(p).category)
        .collect();
    assert!(categories.contains(&PolicyCategory::Economy));
    assert!(categories.contains(&PolicyCategory::Environment));
    assert!(categories.contains(&PolicyCategory::Social));
    assert!(categories.contains(&PolicyCategory::Transport));
    assert!(categories.contains(&PolicyCategory::Zoning));
    assert!(categories.contains(&PolicyCategory::PublicSafety));
}

#[test]
fn test_tradeoff_category_names_are_nonempty() {
    let categories = [
        PolicyCategory::Economy,
        PolicyCategory::Environment,
        PolicyCategory::Social,
        PolicyCategory::Transport,
        PolicyCategory::Zoning,
        PolicyCategory::PublicSafety,
    ];
    for cat in categories {
        assert!(!cat.name().is_empty(), "{:?} should have a name", cat);
    }
}

// ====================================================================
// Computed effects — defaults and resource
// ====================================================================

#[test]
fn test_tradeoff_effects_default_with_no_policies() {
    let policies = Policies::default();
    let effects = compute_effects(&policies);
    assert!((effects.pollution_multiplier - 1.0).abs() < f32::EPSILON);
    assert!((effects.garbage_multiplier - 1.0).abs() < f32::EPSILON);
    assert_eq!(effects.max_building_level, 3);
    assert_eq!(effects.max_commercial_level, 3);
    assert!(!effects.building_changes_blocked);
    assert!(!effects.private_cars_banned);
    assert!(!effects.heavy_trucks_banned);
    assert_eq!(effects.active_policy_count, 0);
}

#[test]
fn test_tradeoff_effects_resource_exists_in_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<PolicyTradeoffEffects>();
}

// ====================================================================
// Individual new policy effect tests
// ====================================================================

#[test]
fn test_tradeoff_combustion_engine_ban_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::CombustionEngineBan);
    let effects = compute_effects(&policies);
    assert!(effects.pollution_multiplier < 1.0);
    assert!(effects.noise_multiplier < 1.0);
    assert!(effects.private_cars_banned);
    assert!(effects.transit_ridership_bonus > 0.0);
    assert!(effects.cycling_rate_bonus > 0.0);
    assert!(effects.car_trip_multiplier.abs() < f32::EPSILON);
}

#[test]
fn test_tradeoff_heavy_traffic_ban_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::HeavyTrafficBan);
    let effects = compute_effects(&policies);
    assert!(effects.noise_multiplier < 1.0);
    assert!(effects.industrial_output_multiplier < 1.0);
    assert!(effects.heavy_trucks_banned);
}

#[test]
fn test_tradeoff_small_business_enthusiast_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::SmallBusinessEnthusiast);
    let effects = compute_effects(&policies);
    assert_eq!(effects.max_commercial_level, 2);
    assert!(effects.commercial_demand_bonus > 0.0);
}

#[test]
fn test_tradeoff_smoke_detector_distribution_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::SmokeDetectorDistribution);
    let effects = compute_effects(&policies);
    assert!((effects.fire_hazard_multiplier - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_tradeoff_old_town_historic_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::OldTownHistoric);
    let effects = compute_effects(&policies);
    assert!(effects.building_changes_blocked);
    assert!(effects.construction_rate_multiplier.abs() < f32::EPSILON);
    assert!(effects.commercial_demand_bonus > 0.0);
}

#[test]
fn test_tradeoff_industrial_space_planning_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::IndustrialSpacePlanning);
    let effects = compute_effects(&policies);
    assert!(effects.industrial_output_multiplier > 1.0);
    assert!(effects.pollution_multiplier > 1.0);
    assert!(effects.industrial_demand_bonus > 0.0);
}

#[test]
fn test_tradeoff_rent_control_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::RentControl);
    let effects = compute_effects(&policies);
    assert!(effects.happiness_bonus > 0.0);
    assert!(effects.construction_rate_multiplier < 1.0);
}

#[test]
fn test_tradeoff_minimum_wage_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::MinimumWage);
    let effects = compute_effects(&policies);
    assert!(effects.poverty_reduction > 0.0);
    assert!(effects.business_cost_multiplier > 1.0);
    assert!(effects.happiness_bonus > 0.0);
}

#[test]
fn test_tradeoff_tax_incentive_zone_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::TaxIncentiveZone);
    let effects = compute_effects(&policies);
    assert!((effects.property_tax_multiplier - 0.5).abs() < f32::EPSILON);
    assert!(effects.construction_rate_multiplier > 1.0);
}

#[test]
fn test_tradeoff_pet_ban_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::PetBan);
    let effects = compute_effects(&policies);
    assert!(effects.garbage_multiplier < 1.0);
    assert!(effects.happiness_bonus < 0.0);
}

#[test]
fn test_tradeoff_parks_and_rec_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::ParksAndRec);
    let effects = compute_effects(&policies);
    assert!(effects.park_multiplier > 1.0);
    assert!(effects.happiness_bonus > 0.0);
}
