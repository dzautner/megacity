//! Integration tests for city specialization save/load roundtrips (SAVE-031).

use crate::specialization::{
    CitySpecialization, CitySpecializations, SpecializationBonuses, SpecializationScore,
};
use crate::test_harness::TestCity;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

/// Save all registered saveables, reset them to defaults, then restore from
/// the saved bytes. Operates entirely through `world_mut()`.
fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);

    world.insert_resource(registry);
}

// ====================================================================
// CitySpecializations roundtrip
// ====================================================================

#[test]
fn test_specialization_scores_preserved_after_save_load() {
    let mut city = TestCity::new();

    // Set up non-default specialization scores.
    {
        let world = city.world_mut();
        let mut specs = world.resource_mut::<CitySpecializations>();
        let tourism = specs.scores.get_mut(&CitySpecialization::Tourism).unwrap();
        tourism.score = 80.0;
        tourism.level = SpecializationScore::level_from_score(80.0);

        let industry = specs.scores.get_mut(&CitySpecialization::Industry).unwrap();
        industry.score = 55.0;
        industry.level = SpecializationScore::level_from_score(55.0);

        let tech = specs
            .scores
            .get_mut(&CitySpecialization::Technology)
            .unwrap();
        tech.score = 30.0;
        tech.level = SpecializationScore::level_from_score(30.0);
    }

    roundtrip(&mut city);

    let specs = city.resource::<CitySpecializations>();
    let tourism = specs.get(CitySpecialization::Tourism);
    assert!(
        (tourism.score - 80.0).abs() < f32::EPSILON,
        "Tourism score should be preserved"
    );
    assert_eq!(tourism.level, 3, "Tourism level should be Dominant (3)");

    let industry = specs.get(CitySpecialization::Industry);
    assert!(
        (industry.score - 55.0).abs() < f32::EPSILON,
        "Industry score should be preserved"
    );
    assert_eq!(industry.level, 2, "Industry level should be Established (2)");

    let tech = specs.get(CitySpecialization::Technology);
    assert!(
        (tech.score - 30.0).abs() < f32::EPSILON,
        "Technology score should be preserved"
    );
    assert_eq!(tech.level, 1, "Technology level should be Emerging (1)");

    // Unmodified specializations should remain at default (0).
    let finance = specs.get(CitySpecialization::Finance);
    assert!(
        (finance.score - 0.0).abs() < f32::EPSILON,
        "Finance score should remain zero"
    );
    assert_eq!(finance.level, 0);
}

// ====================================================================
// SpecializationBonuses roundtrip
// ====================================================================

#[test]
fn test_specialization_bonuses_preserved_after_save_load() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut bonuses = world.resource_mut::<SpecializationBonuses>();
        bonuses.commercial_income_bonus = 0.075;
        bonuses.park_happiness_bonus = 3.0;
        bonuses.industrial_production_bonus = 0.20;
        bonuses.office_income_bonus = 0.15;
        bonuses.credit_rating_boost = 0.15;
        bonuses.culture_happiness_bonus = 4.5;
        bonuses.culture_land_value_bonus = 7.5;
    }

    roundtrip(&mut city);

    let bonuses = city.resource::<SpecializationBonuses>();
    assert!(
        (bonuses.commercial_income_bonus - 0.075).abs() < f32::EPSILON,
        "commercial_income_bonus not preserved"
    );
    assert!(
        (bonuses.park_happiness_bonus - 3.0).abs() < f32::EPSILON,
        "park_happiness_bonus not preserved"
    );
    assert!(
        (bonuses.industrial_production_bonus - 0.20).abs() < f32::EPSILON,
        "industrial_production_bonus not preserved"
    );
    assert!(
        (bonuses.office_income_bonus - 0.15).abs() < f32::EPSILON,
        "office_income_bonus not preserved"
    );
    assert!(
        (bonuses.credit_rating_boost - 0.15).abs() < f32::EPSILON,
        "credit_rating_boost not preserved"
    );
    assert!(
        (bonuses.culture_happiness_bonus - 4.5).abs() < f32::EPSILON,
        "culture_happiness_bonus not preserved"
    );
    assert!(
        (bonuses.culture_land_value_bonus - 7.5).abs() < f32::EPSILON,
        "culture_land_value_bonus not preserved"
    );
}

// ====================================================================
// Default state skips saving (returns None)
// ====================================================================

#[test]
fn test_default_specializations_skip_save() {
    use crate::Saveable;

    assert!(
        CitySpecializations::default().save_to_bytes().is_none(),
        "Default CitySpecializations should skip saving"
    );
    assert!(
        SpecializationBonuses::default().save_to_bytes().is_none(),
        "Default SpecializationBonuses should skip saving"
    );
}

// ====================================================================
// Corrupted bytes fall back to default
// ====================================================================

#[test]
fn test_specialization_corrupted_bytes_fallback_to_default() {
    use crate::Saveable;

    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB];

    let specs = CitySpecializations::load_from_bytes(&garbage);
    // Should fall back to default: all scores zero.
    for &spec in CitySpecialization::ALL {
        let s = specs.get(spec);
        assert_eq!(s.score, 0.0);
        assert_eq!(s.level, 0);
    }

    let bonuses = SpecializationBonuses::load_from_bytes(&garbage);
    assert_eq!(bonuses.commercial_income_bonus, 0.0);
    assert_eq!(bonuses.culture_happiness_bonus, 0.0);
}

// ====================================================================
// Save keys are registered
// ====================================================================

#[test]
fn test_specialization_save_keys_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    assert!(
        registered.contains("city_specializations"),
        "city_specializations key should be registered"
    );
    assert!(
        registered.contains("specialization_bonuses"),
        "specialization_bonuses key should be registered"
    );
}

// ====================================================================
// Bonuses consistent with scores after roundtrip
// ====================================================================

#[test]
fn test_specialization_bonuses_consistent_after_roundtrip() {
    let mut city = TestCity::new();

    // Set Tourism to level 2 (Established) and compute expected bonuses.
    {
        let world = city.world_mut();
        let mut specs = world.resource_mut::<CitySpecializations>();
        let tourism = specs.scores.get_mut(&CitySpecialization::Tourism).unwrap();
        tourism.score = 60.0;
        tourism.level = SpecializationScore::level_from_score(60.0); // level 2

        let mut bonuses = world.resource_mut::<SpecializationBonuses>();
        let mult = SpecializationScore::bonus_multiplier(2); // 1.5
        bonuses.commercial_income_bonus = 0.05 * mult;
        bonuses.park_happiness_bonus = 2.0 * mult;
    }

    roundtrip(&mut city);

    let specs = city.resource::<CitySpecializations>();
    let tourism = specs.get(CitySpecialization::Tourism);
    assert_eq!(tourism.level, 2);

    let bonuses = city.resource::<SpecializationBonuses>();
    let expected_mult = SpecializationScore::bonus_multiplier(tourism.level);
    assert!(
        (bonuses.commercial_income_bonus - 0.05 * expected_mult).abs() < f32::EPSILON,
        "Bonuses should be consistent with specialization level after roundtrip"
    );
    assert!(
        (bonuses.park_happiness_bonus - 2.0 * expected_mult).abs() < f32::EPSILON,
        "Park happiness bonus should be consistent after roundtrip"
    );
}
