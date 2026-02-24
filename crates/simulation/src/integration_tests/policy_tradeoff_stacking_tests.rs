//! Integration tests for policy tradeoff stacking and backward compat (POL-001).

use crate::policies::{Policies, Policy};
use crate::policy_tradeoffs::compute_effects;

// ====================================================================
// Stacking / interaction tests
// ====================================================================

#[test]
fn test_tradeoff_multiple_noise_policies_stack() {
    let mut policies = Policies::default();
    policies.toggle(Policy::HeavyTrafficBan);
    policies.toggle(Policy::CombustionEngineBan);
    let effects = compute_effects(&policies);
    // 0.6 * 0.7 = 0.42
    let expected = 0.6 * 0.7;
    assert!(
        (effects.noise_multiplier - expected).abs() < 0.01,
        "stacked noise should be ~{expected}, got {}",
        effects.noise_multiplier
    );
}

#[test]
fn test_tradeoff_multiple_fire_policies_stack() {
    let mut policies = Policies::default();
    policies.toggle(Policy::SmokeDetectorMandate);
    policies.toggle(Policy::SmokeDetectorDistribution);
    let effects = compute_effects(&policies);
    // 0.7 * 0.5 = 0.35
    let expected = 0.7 * 0.5;
    assert!(
        (effects.fire_hazard_multiplier - expected).abs() < 0.01,
        "stacked fire should be ~{expected}, got {}",
        effects.fire_hazard_multiplier
    );
}

#[test]
fn test_tradeoff_rent_control_plus_tax_incentive_interaction() {
    let mut policies = Policies::default();
    policies.toggle(Policy::RentControl);
    policies.toggle(Policy::TaxIncentiveZone);
    let effects = compute_effects(&policies);
    // RentControl: 0.75, TaxIncentiveZone: 1.25 -> 0.9375
    let expected = 0.75 * 1.25;
    assert!(
        (effects.construction_rate_multiplier - expected).abs() < 0.01,
        "combined construction should be ~{expected}, got {}",
        effects.construction_rate_multiplier
    );
}

#[test]
fn test_tradeoff_old_town_overrides_tax_incentive_construction() {
    let mut policies = Policies::default();
    policies.toggle(Policy::OldTownHistoric);
    policies.toggle(Policy::TaxIncentiveZone);
    let effects = compute_effects(&policies);
    assert!(
        effects.construction_rate_multiplier.abs() < f32::EPSILON,
        "old town should override tax incentive construction boost"
    );
}

#[test]
fn test_tradeoff_total_monthly_cost_tracks_active() {
    let mut policies = Policies::default();
    policies.toggle(Policy::CombustionEngineBan); // $30
    policies.toggle(Policy::MinimumWage); // $20
    policies.toggle(Policy::ParksAndRec); // $20
    let effects = compute_effects(&policies);
    let expected = 30.0 + 20.0 + 20.0;
    assert!(
        (effects.total_monthly_cost - expected).abs() < f64::EPSILON,
        "total monthly cost should be {expected}, got {}",
        effects.total_monthly_cost
    );
    assert_eq!(effects.active_policy_count, 3);
}

#[test]
fn test_tradeoff_disabling_policy_removes_effects() {
    let mut policies = Policies::default();
    policies.toggle(Policy::CombustionEngineBan);
    assert!(compute_effects(&policies).private_cars_banned);
    policies.toggle(Policy::CombustionEngineBan);
    assert!(!compute_effects(&policies).private_cars_banned);
}

// ====================================================================
// New policies cost and metadata tests
// ====================================================================

#[test]
fn test_tradeoff_new_policies_have_costs() {
    let new_policies = [
        Policy::CombustionEngineBan,
        Policy::SmallBusinessEnthusiast,
        Policy::HeavyTrafficBan,
        Policy::SmokeDetectorDistribution,
        Policy::OldTownHistoric,
        Policy::IndustrialSpacePlanning,
        Policy::RentControl,
        Policy::MinimumWage,
        Policy::PetBan,
        Policy::ParksAndRec,
    ];
    for policy in new_policies {
        let _ = policy.monthly_cost();
        assert!(!policy.name().is_empty(), "{:?} needs name", policy);
        assert!(!policy.description().is_empty(), "{:?} needs desc", policy);
    }
}

#[test]
fn test_tradeoff_tax_incentive_zone_zero_direct_cost() {
    assert!(
        Policy::TaxIncentiveZone.monthly_cost().abs() < f64::EPSILON,
        "TaxIncentiveZone has no direct cost"
    );
}

// ====================================================================
// Backward compatibility tests
// ====================================================================

#[test]
fn test_tradeoff_existing_policy_costs_unchanged() {
    assert!((Policy::FreePublicTransport.monthly_cost() - 50.0).abs() < f64::EPSILON);
    assert!((Policy::RecyclingProgram.monthly_cost() - 20.0).abs() < f64::EPSILON);
    assert!((Policy::HealthcareForAll.monthly_cost() - 45.0).abs() < f64::EPSILON);
    assert!(Policy::HighRiseBan.monthly_cost().abs() < f64::EPSILON);
}

#[test]
fn test_tradeoff_existing_happiness_unchanged() {
    let mut policies = Policies::default();
    policies.toggle(Policy::FreePublicTransport);
    policies.toggle(Policy::NightShiftBan);
    policies.toggle(Policy::HealthcareForAll);
    policies.toggle(Policy::NeighborhoodWatch);
    let expected = 3.0 + 3.0 + 2.0 + 2.0;
    assert!(
        (policies.happiness_bonus() - expected).abs() < f32::EPSILON,
        "existing happiness stacking unchanged"
    );
}

#[test]
fn test_tradeoff_existing_pollution_multiplier_unchanged() {
    let mut policies = Policies::default();
    policies.toggle(Policy::IndustrialAirFilters);
    assert!(
        (policies.pollution_multiplier() - 0.6).abs() < f32::EPSILON,
        "IndustrialAirFilters should still be 0.6"
    );
}

#[test]
fn test_tradeoff_existing_garbage_multiplier_unchanged() {
    let mut policies = Policies::default();
    policies.toggle(Policy::RecyclingProgram);
    assert!(
        (policies.garbage_multiplier() - 0.7).abs() < f32::EPSILON,
        "RecyclingProgram should still be 0.7"
    );
}

#[test]
fn test_tradeoff_existing_max_building_level_unchanged() {
    let policies = Policies::default();
    assert_eq!(policies.max_building_level(), 3);

    let mut policies2 = Policies::default();
    policies2.toggle(Policy::HighRiseBan);
    assert_eq!(policies2.max_building_level(), 2);
}

#[test]
fn test_tradeoff_existing_industrial_tax_multiplier_unchanged() {
    let mut policies = Policies::default();
    policies.toggle(Policy::HeavyIndustryTaxBreak);
    assert!(
        (policies.industrial_tax_multiplier() - 0.5).abs() < f32::EPSILON,
        "HeavyIndustryTaxBreak should still be 0.5"
    );
}

#[test]
fn test_tradeoff_existing_commercial_demand_bonus_unchanged() {
    let mut policies = Policies::default();
    policies.toggle(Policy::TourismPromotion);
    policies.toggle(Policy::SmallBusinessGrant);
    let expected = 0.15 + 0.10;
    assert!(
        (policies.commercial_demand_bonus() - expected).abs() < f32::EPSILON,
        "existing commercial bonus should be {expected}"
    );
}
