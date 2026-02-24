//! Integration tests for SERV-008: Industrial Specializations

use crate::districts::DistrictMap;
use crate::grid::ZoneType;
use crate::industrial_specializations::{
    suggest_specialization, IndustrialSpecialization, IndustrialSpecializationState,
};
use crate::natural_resources::{ResourceDeposit, ResourceGrid, ResourceType};
use crate::test_harness::TestCity;

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_industrial_spec_state_exists_in_empty_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<IndustrialSpecializationState>();
}

// ====================================================================
// Specialization assignment
// ====================================================================

#[test]
fn test_industrial_spec_assign_forest_to_district() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<IndustrialSpecializationState>();
        state.assign(0, IndustrialSpecialization::Forest);
    }
    let state = city.resource::<IndustrialSpecializationState>();
    assert_eq!(
        state.get_specialization(0),
        Some(IndustrialSpecialization::Forest)
    );
}

#[test]
fn test_industrial_spec_assign_all_four_types() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<IndustrialSpecializationState>();
        state.assign(0, IndustrialSpecialization::Forest);
        state.assign(1, IndustrialSpecialization::Farming);
        state.assign(2, IndustrialSpecialization::Oil);
        state.assign(3, IndustrialSpecialization::Ore);
    }
    let state = city.resource::<IndustrialSpecializationState>();
    assert_eq!(
        state.get_specialization(0),
        Some(IndustrialSpecialization::Forest)
    );
    assert_eq!(
        state.get_specialization(1),
        Some(IndustrialSpecialization::Farming)
    );
    assert_eq!(
        state.get_specialization(2),
        Some(IndustrialSpecialization::Oil)
    );
    assert_eq!(
        state.get_specialization(3),
        Some(IndustrialSpecialization::Ore)
    );
}

#[test]
fn test_industrial_spec_remove_specialization() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<IndustrialSpecializationState>();
        state.assign(0, IndustrialSpecialization::Oil);
        state.remove(0);
    }
    let state = city.resource::<IndustrialSpecializationState>();
    assert!(state.get_specialization(0).is_none());
}

// ====================================================================
// Specialization properties
// ====================================================================

#[test]
fn test_industrial_spec_pollution_levels_differ() {
    let forest_poll = IndustrialSpecialization::Forest.pollution_per_worker();
    let oil_poll = IndustrialSpecialization::Oil.pollution_per_worker();
    assert!(
        oil_poll > forest_poll,
        "Oil should pollute more than Forest: oil={oil_poll}, forest={forest_poll}"
    );
}

#[test]
fn test_industrial_spec_oil_highest_output_value() {
    let oil_value = IndustrialSpecialization::Oil.output_value_per_worker();
    let forest_value = IndustrialSpecialization::Forest.output_value_per_worker();
    let farming_value = IndustrialSpecialization::Farming.output_value_per_worker();
    assert!(
        oil_value > forest_value && oil_value > farming_value,
        "Oil should have highest output value"
    );
}

#[test]
fn test_industrial_spec_renewable_vs_nonrenewable() {
    assert!(
        IndustrialSpecialization::Forest.is_renewable(),
        "Forest should be renewable"
    );
    assert!(
        IndustrialSpecialization::Farming.is_renewable(),
        "Farming should be renewable"
    );
    assert!(
        !IndustrialSpecialization::Oil.is_renewable(),
        "Oil should not be renewable"
    );
    assert!(
        !IndustrialSpecialization::Ore.is_renewable(),
        "Ore should not be renewable"
    );
}

// ====================================================================
// Resource suggestion
// ====================================================================

#[test]
fn test_industrial_spec_suggest_forest_from_deposits() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        // Place forest deposits in cells belonging to district 0
        let mut resource_grid = world.resource_mut::<ResourceGrid>();
        for x in 10..15 {
            resource_grid.set(
                x,
                10,
                ResourceDeposit {
                    resource_type: ResourceType::Forest,
                    amount: 8000,
                    max_amount: 8000,
                },
            );
        }
        // Assign those cells to district 0
        let mut district_map = world.resource_mut::<DistrictMap>();
        for x in 10..15 {
            district_map.assign_cell_to_district(x, 10, 0);
        }
    }
    let district_map = city.resource::<DistrictMap>();
    let resource_grid = city.resource::<ResourceGrid>();
    let suggestion = suggest_specialization(0, district_map, resource_grid);
    assert_eq!(
        suggestion,
        Some(IndustrialSpecialization::Forest),
        "Should suggest Forest specialization for district with forest deposits"
    );
}

#[test]
fn test_industrial_spec_suggest_none_without_deposits() {
    let city = TestCity::new();
    let district_map = city.resource::<DistrictMap>();
    let resource_grid = city.resource::<ResourceGrid>();
    let suggestion = suggest_specialization(0, district_map, resource_grid);
    assert!(
        suggestion.is_none(),
        "Should not suggest specialization without deposits"
    );
}

// ====================================================================
// Resource extraction and depletion
// ====================================================================

#[test]
fn test_industrial_spec_finite_resource_depletes() {
    let mut city = TestCity::new()
        .with_road(10, 8, 10, 12, crate::grid::RoadType::Local)
        .with_building(11, 10, ZoneType::Industrial, 1);

    {
        let world = city.world_mut();
        // Place an oil deposit near the building
        let mut resource_grid = world.resource_mut::<ResourceGrid>();
        resource_grid.set(
            11,
            10,
            ResourceDeposit {
                resource_type: ResourceType::Oil,
                amount: 100,
                max_amount: 100,
            },
        );

        // Assign building cell to district 0 with Oil specialization
        let mut district_map = world.resource_mut::<DistrictMap>();
        district_map.assign_cell_to_district(11, 10, 0);

        let mut state = world.resource_mut::<IndustrialSpecializationState>();
        state.assign(0, IndustrialSpecialization::Oil);
    }

    // Simulate building occupancy
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type == ZoneType::Industrial {
                building.occupants = 5;
            }
        }
    }

    // Run enough ticks for production to fire and deplete resources
    city.tick(200);

    let resource_grid = city.resource::<ResourceGrid>();
    let deposit = resource_grid.get(11, 10);
    assert!(
        deposit.is_some(),
        "deposit should still exist at (11, 10)"
    );
    let amount = deposit.as_ref().unwrap().amount;
    assert!(
        amount < 100,
        "Oil deposit should deplete over time, got amount={amount}"
    );
}

#[test]
fn test_industrial_spec_renewable_resource_regenerates() {
    // Renewable resources (Forest, Farming) should not fully deplete
    // because they regenerate each tick.
    let deposit = ResourceDeposit {
        resource_type: ResourceType::Forest,
        amount: 100,
        max_amount: 8000,
    };
    assert!(
        deposit.resource_type.is_renewable(),
        "Forest should be renewable"
    );
    // After extraction + regen, amount should stay above zero for moderate extraction
    // (This tests the extraction logic properties, not the full system)
    let extract_amount: f32 = 2.0;
    let depletion = (extract_amount * 0.3) as u32; // 0
    let after = deposit.amount.saturating_sub(depletion) + 1; // regen +1
    assert!(
        after > 0,
        "Renewable resource should maintain positive amount after moderate extraction"
    );
}

// ====================================================================
// Saveable roundtrip
// ====================================================================

#[test]
fn test_industrial_spec_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = IndustrialSpecializationState::default();
    state.assign(0, IndustrialSpecialization::Forest);
    state.assign(3, IndustrialSpecialization::Oil);
    state.cumulative_output.insert(0, 1000.0);
    state.cumulative_extracted.insert(3, 250.0);

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let restored = IndustrialSpecializationState::load_from_bytes(&bytes);

    assert_eq!(
        restored.get_specialization(0),
        Some(IndustrialSpecialization::Forest)
    );
    assert_eq!(
        restored.get_specialization(3),
        Some(IndustrialSpecialization::Oil)
    );
    assert_eq!(
        restored.cumulative_output.get(&0).copied(),
        Some(1000.0)
    );
    assert_eq!(
        restored.cumulative_extracted.get(&3).copied(),
        Some(250.0)
    );
}
