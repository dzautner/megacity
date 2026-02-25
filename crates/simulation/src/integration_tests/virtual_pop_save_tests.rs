//! Integration tests for SAVE-005 (Issue #700): VirtualPopulation serialization.
//!
//! Verifies the specific requirements from issue #700:
//! - Total population (entity + virtual) matches pre-save count after load
//! - Virtual demographic distribution (age brackets, avg_happiness, avg_age) is preserved
//! - Tax contribution and service demand survive roundtrip

use crate::test_harness::TestCity;
use crate::virtual_population::VirtualPopulation;
use crate::SaveableRegistry;

/// Save all registered saveables, reset them, then restore from the saved
/// bytes. Operates entirely through `world_mut()`.
fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);

    world.insert_resource(registry);
}

#[test]
fn test_total_population_entity_plus_virtual_matches_after_load() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        // Simulate 5000 virtual citizens across 3 districts
        for district in 0..3 {
            for i in 0..1667 {
                let age = (18 + (i % 47)) as u8;
                let employed = i % 2 == 0;
                let happiness = 50.0 + (i % 50) as f32;
                let salary = if employed { 1000.0 } else { 0.0 };
                vp.add_virtual_citizen(district, age, employed, happiness, salary, 0.12);
            }
        }
    }

    // Snapshot total population including a simulated real citizen count
    let simulated_real_count = 200u32;
    let total_before = city
        .resource::<VirtualPopulation>()
        .total_with_real(simulated_real_count);

    roundtrip(&mut city);

    let total_after = city
        .resource::<VirtualPopulation>()
        .total_with_real(simulated_real_count);

    assert_eq!(
        total_before, total_after,
        "total population (entity + virtual) must match after save/load"
    );
}

#[test]
fn test_demographic_distribution_preserved_after_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        // Add citizens of known ages to fill all brackets
        // [0-17, 18-34, 35-54, 55-64, 65+]
        vp.add_virtual_citizen(0, 10, false, 60.0, 0.0, 0.0); // child
        vp.add_virtual_citizen(0, 15, false, 65.0, 0.0, 0.0); // child
        vp.add_virtual_citizen(0, 22, true, 70.0, 900.0, 0.1); // young adult
        vp.add_virtual_citizen(0, 30, true, 75.0, 1200.0, 0.1); // young adult
        vp.add_virtual_citizen(0, 40, true, 80.0, 1500.0, 0.15); // middle-aged
        vp.add_virtual_citizen(0, 50, true, 72.0, 1800.0, 0.15); // middle-aged
        vp.add_virtual_citizen(0, 58, true, 68.0, 2000.0, 0.2); // pre-retirement
        vp.add_virtual_citizen(0, 70, false, 55.0, 0.0, 0.0); // retired
        vp.add_virtual_citizen(0, 80, false, 50.0, 0.0, 0.0); // retired
    }

    // Snapshot demographics before save
    let (brackets_before, avg_happiness_before, avg_age_before) = {
        let vp = city.resource::<VirtualPopulation>();
        let ds = &vp.district_stats[0];
        (ds.age_brackets, ds.avg_happiness, ds.avg_age)
    };

    // Expected brackets: [2, 2, 2, 1, 2]
    assert_eq!(brackets_before, [2, 2, 2, 1, 2]);

    roundtrip(&mut city);

    let vp = city.resource::<VirtualPopulation>();
    let ds = &vp.district_stats[0];

    assert_eq!(
        ds.age_brackets, brackets_before,
        "age bracket distribution must be preserved after roundtrip"
    );
    assert!(
        (ds.avg_happiness - avg_happiness_before).abs() < 0.01,
        "avg_happiness must be preserved: expected {avg_happiness_before}, got {}",
        ds.avg_happiness
    );
    assert!(
        (ds.avg_age - avg_age_before).abs() < 0.01,
        "avg_age must be preserved: expected {avg_age_before}, got {}",
        ds.avg_age
    );
}

#[test]
fn test_tax_contribution_and_service_demand_preserved() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        // Add employed citizens with known salaries and tax rates
        for i in 0..100 {
            let salary = 1000.0 + (i as f32 * 10.0);
            vp.add_virtual_citizen(0, 30, true, 70.0, salary, 0.15);
        }
        // Add some unemployed to create mixed employment stats
        for _ in 0..50 {
            vp.add_virtual_citizen(0, 25, false, 55.0, 0.0, 0.0);
        }
    }

    let (tax_before, demand_before, employed_before) = {
        let vp = city.resource::<VirtualPopulation>();
        let ds = &vp.district_stats[0];
        (ds.tax_contribution, ds.service_demand, ds.employed)
    };

    assert!(tax_before > 0.0, "tax_contribution should be positive");
    assert!(demand_before > 0.0, "service_demand should be positive");
    assert_eq!(employed_before, 100);

    roundtrip(&mut city);

    let vp = city.resource::<VirtualPopulation>();
    let ds = &vp.district_stats[0];

    assert!(
        (ds.tax_contribution - tax_before).abs() < 0.01,
        "tax_contribution must be preserved: expected {tax_before}, got {}",
        ds.tax_contribution
    );
    assert!(
        (ds.service_demand - demand_before).abs() < 0.001,
        "service_demand must be preserved: expected {demand_before}, got {}",
        ds.service_demand
    );
    assert_eq!(
        ds.employed, employed_before,
        "employed count must be preserved"
    );
}

#[test]
fn test_commuters_out_preserved_after_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        // Each employed citizen increments commuters_out
        for _ in 0..75 {
            vp.add_virtual_citizen(0, 35, true, 65.0, 1200.0, 0.1);
        }
    }

    let commuters_before = city.resource::<VirtualPopulation>().district_stats[0].commuters_out;
    assert_eq!(commuters_before, 75);

    roundtrip(&mut city);

    let vp = city.resource::<VirtualPopulation>();
    assert_eq!(
        vp.district_stats[0].commuters_out, commuters_before,
        "commuters_out must be preserved after roundtrip"
    );
}
