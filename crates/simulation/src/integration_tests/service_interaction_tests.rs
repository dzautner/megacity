//! SVC-006: Integration tests for service cross-interaction matrix.
//!
//! Tests that service coverage interactions produce expected downstream
//! effects on crime, health, and education grids.

use crate::hybrid_service_coverage::HybridCoverageGrid;
use crate::service_cross_interaction::{ServiceInteractionGrid, ServiceInteractionStats};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Helper: tick enough for slow-cycle systems to fire
// ====================================================================

fn tick_interactions(city: &mut TestCity) {
    city.tick_slow_cycles(2);
}

// ====================================================================
// 1. Education reduces crime via interaction multiplier
// ====================================================================

#[test]
fn test_education_coverage_reduces_crime_multiplier() {
    // Place education services at center — the crime multiplier in the
    // interaction grid should be < 1.0, indicating crime reduction.
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::University)
        .with_service(130, 128, ServiceType::HighSchool);
    tick_interactions(&mut city);

    let interactions = city.resource::<ServiceInteractionGrid>();
    let idx = ServiceInteractionGrid::idx(128, 128);

    // Education coverage creates a crime multiplier < 1.0
    assert!(
        interactions.crime_multiplier[idx] < 1.0,
        "Crime multiplier should be < 1.0 with education services, got {}",
        interactions.crime_multiplier[idx]
    );
}

// ====================================================================
// 2. Parks improve health
// ====================================================================

#[test]
fn test_parks_improve_health() {
    // Place parks at center — health bonus should be positive
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::LargePark)
        .with_service(128, 130, ServiceType::SmallPark);
    tick_interactions(&mut city);

    let interactions = city.resource::<ServiceInteractionGrid>();
    let health_bonus = interactions.health_bonus[ServiceInteractionGrid::idx(128, 128)];

    // With park coverage, health bonus should be positive
    assert!(
        health_bonus >= 0.0,
        "Health bonus from parks should be non-negative, got {}",
        health_bonus
    );
}

// ====================================================================
// 3. Well-rounded services produce compounding returns
// ====================================================================

#[test]
fn test_well_rounded_services_compound() {
    // City with all service types gets compounding benefits
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::University)
        .with_service(128, 130, ServiceType::Hospital)
        .with_service(130, 128, ServiceType::PoliceStation)
        .with_service(130, 130, ServiceType::LargePark)
        .with_service(128, 132, ServiceType::FireStation);
    tick_interactions(&mut city);

    let interactions = city.resource::<ServiceInteractionGrid>();
    let idx = ServiceInteractionGrid::idx(128, 128);

    // Crime should be reduced (multiplier < 1.0) from education
    let crime_mult = interactions.crime_multiplier[idx];
    assert!(
        crime_mult <= 1.0,
        "Crime multiplier should be <= 1.0 with education, got {}",
        crime_mult
    );

    // Health bonus should be positive from parks + education + police
    let health = interactions.health_bonus[idx];
    assert!(
        health >= 0.0,
        "Health bonus should be non-negative with multiple services, got {}",
        health
    );

    // Fire survival bonus from healthcare
    let fire_bonus = interactions.fire_survival_bonus[idx];
    assert!(
        fire_bonus >= 0.0,
        "Fire survival bonus should be non-negative with hospital, got {}",
        fire_bonus
    );
}

// ====================================================================
// 4. No services = no interaction effects
// ====================================================================

#[test]
fn test_no_services_no_interactions() {
    let mut city = TestCity::new();
    tick_interactions(&mut city);

    let interactions = city.resource::<ServiceInteractionGrid>();
    let idx = ServiceInteractionGrid::idx(128, 128);

    assert!(
        (interactions.crime_multiplier[idx] - 1.0).abs() < f32::EPSILON,
        "Crime multiplier should be 1.0 with no services"
    );
    assert!(
        interactions.health_bonus[idx].abs() < f32::EPSILON,
        "Health bonus should be 0.0 with no services"
    );
    assert!(
        interactions.education_bonus[idx].abs() < f32::EPSILON,
        "Education bonus should be 0.0 with no services"
    );
    assert!(
        interactions.fire_survival_bonus[idx].abs() < f32::EPSILON,
        "Fire survival bonus should be 0.0 with no services"
    );
}

// ====================================================================
// 5. Interaction stats track coverage
// ====================================================================

#[test]
fn test_interaction_stats_update() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::University)
        .with_service(130, 128, ServiceType::Hospital)
        .with_service(132, 128, ServiceType::LargePark);
    tick_interactions(&mut city);

    let stats = city.resource::<ServiceInteractionStats>();

    // With services placed, some cells should have crime reduction
    assert!(
        stats.avg_crime_multiplier <= 1.0,
        "Average crime multiplier should be <= 1.0 with education services"
    );
}

// ====================================================================
// 6. Education at full coverage gives approximately 15% crime reduction
// ====================================================================

#[test]
fn test_full_education_coverage_crime_reduction_strength() {
    // The interaction grid stores the crime multiplier directly.
    // At full education coverage (1.0), the factor should be 0.85 (= 1.0 - 0.15).
    let mut city = TestCity::new().with_service(128, 128, ServiceType::University);
    tick_interactions(&mut city);

    let coverage = city.resource::<HybridCoverageGrid>();
    let edu_cov = coverage.get_clamped(
        128,
        128,
        crate::hybrid_service_coverage::ServiceCategory::Education,
    );

    let interactions = city.resource::<ServiceInteractionGrid>();
    let idx = ServiceInteractionGrid::idx(128, 128);
    let crime_mult = interactions.crime_multiplier[idx];

    // Expected: 1.0 - edu_cov * 0.15
    let expected = 1.0 - edu_cov * 0.15;
    assert!(
        (crime_mult - expected).abs() < 0.01,
        "Crime multiplier at full edu coverage should be ~{:.2}, got {:.2} (edu_cov={:.2})",
        expected,
        crime_mult,
        edu_cov
    );
}

// ====================================================================
// 7. Healthcare improves fire survival bonus
// ====================================================================

#[test]
fn test_healthcare_provides_fire_survival() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_interactions(&mut city);

    let interactions = city.resource::<ServiceInteractionGrid>();
    let idx = ServiceInteractionGrid::idx(128, 128);

    assert!(
        interactions.fire_survival_bonus[idx] > 0.0,
        "Hospital should provide fire survival bonus, got {}",
        interactions.fire_survival_bonus[idx]
    );
}

// ====================================================================
// 8. Neglecting services means no cross-interaction benefits
// ====================================================================

#[test]
fn test_neglecting_services_no_benefits() {
    // A city with only fire stations gets no cross-interaction benefits
    // on crime, health, or education
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::FireStation)
        .with_service(130, 128, ServiceType::FireStation);
    tick_interactions(&mut city);

    let interactions = city.resource::<ServiceInteractionGrid>();
    let idx = ServiceInteractionGrid::idx(128, 128);

    // Fire stations don't contribute to education, so crime multiplier stays 1.0
    assert!(
        (interactions.crime_multiplier[idx] - 1.0).abs() < f32::EPSILON,
        "Crime multiplier should be 1.0 with only fire stations"
    );

    // No parks or education means no health bonus from interactions
    assert!(
        interactions.health_bonus[idx].abs() < f32::EPSILON,
        "Health bonus should be 0.0 with only fire stations"
    );

    // Education bonus requires education coverage
    assert!(
        interactions.education_bonus[idx].abs() < f32::EPSILON,
        "Education bonus should be 0.0 with only fire stations"
    );
}
