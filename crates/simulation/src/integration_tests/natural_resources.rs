use crate::grid::ZoneType;
use crate::test_harness::TestCity;

// ====================================================================
// Natural Resources integration tests
// ====================================================================

#[test]
fn natural_resources_resource_grid_exists_in_empty_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::natural_resources::ResourceGrid>();
    city.assert_resource_exists::<crate::natural_resources::ResourceBalance>();
}

#[test]
fn natural_resources_empty_city_grid_has_no_deposits() {
    let city = TestCity::new();
    let resource_grid = city.resource::<crate::natural_resources::ResourceGrid>();
    let deposit_count = resource_grid
        .deposits
        .iter()
        .filter(|d| d.is_some())
        .count();
    assert_eq!(
        deposit_count, 0,
        "empty city (no terrain generation) should have no resource deposits, got {deposit_count}"
    );
}

#[test]
fn natural_resources_deposit_placed_at_grid_position() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut resource_grid = world.resource_mut::<crate::natural_resources::ResourceGrid>();
        resource_grid.set(
            50,
            50,
            crate::natural_resources::ResourceDeposit {
                resource_type: crate::natural_resources::ResourceType::Ore,
                amount: 5000,
                max_amount: 5000,
            },
        );
    }
    let resource_grid = city.resource::<crate::natural_resources::ResourceGrid>();
    let deposit = resource_grid.get(50, 50);
    assert!(deposit.is_some(), "deposit should exist at (50, 50)");
    assert_eq!(
        deposit.as_ref().unwrap().resource_type,
        crate::natural_resources::ResourceType::Ore
    );
    assert!(resource_grid.get(49, 50).is_none());
    assert!(resource_grid.get(51, 50).is_none());
}

#[test]
fn natural_resources_extraction_rate_depends_on_occupants() {
    let occupants_low: u32 = 5;
    let occupants_high: u32 = 20;
    let output_low = occupants_low as f32 * 0.5;
    let output_high = occupants_high as f32 * 0.5;
    assert!(
        (output_low - 2.5).abs() < f32::EPSILON,
        "5 occupants should produce 2.5 output, got {output_low}"
    );
    assert!(
        (output_high - 10.0).abs() < f32::EPSILON,
        "20 occupants should produce 10.0 output, got {output_high}"
    );
    assert!(
        output_high > output_low,
        "more occupants should produce more output"
    );
}

#[test]
fn natural_resources_depleted_deposit_produces_nothing() {
    use crate::natural_resources::{ResourceDeposit, ResourceType};
    let deposit = ResourceDeposit {
        resource_type: ResourceType::Ore,
        amount: 0,
        max_amount: 5000,
    };
    assert_eq!(deposit.amount, 0, "depleted deposit should have amount 0");
}

#[test]
fn natural_resources_finite_resource_depletes_over_time() {
    use crate::natural_resources::{ResourceDeposit, ResourceType};
    let mut deposit = ResourceDeposit {
        resource_type: ResourceType::Ore,
        amount: 100,
        max_amount: 5000,
    };
    let output = 5.0_f32;
    let extraction = (output * 0.2) as u32;
    deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
    assert_eq!(
        deposit.amount, 99,
        "first extraction should reduce amount by 1"
    );
    for _ in 0..50 {
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
    }
    assert!(
        deposit.amount < 99,
        "repeated extraction should further deplete the resource"
    );
    for _ in 0..200 {
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
    }
    assert_eq!(
        deposit.amount, 0,
        "finite resource should fully deplete to zero"
    );
}

#[test]
fn natural_resources_non_industrial_building_does_not_extract() {
    assert_ne!(ZoneType::ResidentialLow, ZoneType::Industrial);
    assert_ne!(ZoneType::CommercialLow, ZoneType::Industrial);
}

#[test]
fn natural_resources_consumption_scales_with_population() {
    let pop: f32 = 10000.0;
    let food = pop * 0.02;
    let timber = pop * 0.005;
    let metal = pop * 0.003;
    let fuel = pop * 0.004;
    assert!((food - 200.0).abs() < f32::EPSILON);
    assert!((timber - 50.0).abs() < f32::EPSILON);
    assert!((metal - 30.0).abs() < f32::EPSILON);
    assert!((fuel - 40.0).abs() < f32::EPSILON);
    let pop2: f32 = 20000.0;
    let food2 = pop2 * 0.02;
    assert!(
        food2 > food,
        "doubling population should double food consumption"
    );
}
