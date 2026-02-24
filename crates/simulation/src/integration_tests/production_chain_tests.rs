//! Integration tests for the deep production chain system (SERV-009).

use crate::buildings::Building;
use crate::grid::{RoadType, ZoneType};
use crate::natural_resources::{ResourceDeposit, ResourceGrid, ResourceType};
use crate::production::{IndustryBuilding, IndustryType};
use crate::production_chain::{Commodity, DeepChainBuilding, DeepProductionChainState};
use crate::test_harness::TestCity;

// ====================================================================
// Resource existence and defaults
// ====================================================================

#[test]
fn test_production_chain_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<DeepProductionChainState>();
}

#[test]
fn test_production_chain_default_empty() {
    let city = TestCity::new();
    let state = city.resource::<DeepProductionChainState>();
    for &c in Commodity::all() {
        assert_eq!(state.stock(c), 0.0, "{:?} should start at 0", c);
    }
    assert_eq!(state.disrupted_count, 0);
    assert_eq!(state.commodity_trade_balance, 0.0);
}

// ====================================================================
// Forestry chain: Timber -> Lumber -> Furniture
// ====================================================================

#[test]
fn test_forestry_chain_produces_timber() {
    let mut city = TestCity::new()
        .with_road(50, 50, 50, 60, RoadType::Local)
        .with_building(49, 52, ZoneType::Industrial, 2);

    // Place forest resources near the building
    {
        let world = city.world_mut();
        let mut resource_grid = world.resource_mut::<ResourceGrid>();
        for dx in 45..55 {
            for dy in 48..56 {
                resource_grid.set(
                    dx,
                    dy,
                    ResourceDeposit {
                        resource_type: ResourceType::Forest,
                        amount: 500,
                        max_amount: 500,
                    },
                );
            }
        }
    }

    // Run a few ticks so that assign_industry_type adds IndustryBuilding
    city.tick(5);

    // Now force the IndustryBuilding to Forestry with workers
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut Building, &mut IndustryBuilding)>();
        for (mut building, mut industry) in q.iter_mut(world) {
            building.occupants = 20;
            industry.industry_type = IndustryType::Forestry;
            industry.workers = 20;
            industry.efficiency = 0.8;
        }
    }

    // Run enough production cycles for DeepChainBuilding to be assigned and produce
    // CHAIN_INTERVAL=10, so 60 ticks = 6 cycles
    city.tick(60);

    // The building should have been assigned a DeepChainBuilding
    let has_chain_building = {
        let world = city.world_mut();
        let mut q = world.query::<&DeepChainBuilding>();
        q.iter(world).count() > 0
    };

    assert!(
        has_chain_building,
        "Forestry building should be assigned a DeepChainBuilding component"
    );

    let state = city.resource::<DeepProductionChainState>();
    let timber = state.stock(Commodity::Timber);
    let prod_rate = state
        .production_rates
        .get(&Commodity::Timber)
        .copied()
        .unwrap_or(0.0);

    assert!(
        timber > 0.0 || prod_rate > 0.0,
        "Forestry chain should produce timber; stock={timber}, rate={prod_rate}"
    );
}

// ====================================================================
// Supply chain disruption
// ====================================================================

#[test]
fn test_processing_disrupted_without_inputs() {
    let mut city = TestCity::new()
        .with_road(50, 50, 50, 60, RoadType::Local)
        .with_building(49, 52, ZoneType::Industrial, 2);

    // Run ticks so assign_industry_type fires
    city.tick(5);

    // Assign as SawMill (processing stage) - needs Timber input
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut Building, &mut IndustryBuilding)>();
        for (mut building, mut industry) in q.iter_mut(world) {
            building.occupants = 15;
            industry.industry_type = IndustryType::SawMill;
            industry.workers = 15;
            industry.efficiency = 0.7;
        }
    }

    // Run without any Timber in the stockpile
    // Need enough ticks: first cycle assigns DeepChainBuilding, second+ cycles detect disruption
    city.tick(60);

    // The building should become disrupted
    let disrupted = {
        let world = city.world_mut();
        let mut q = world.query::<&DeepChainBuilding>();
        q.iter(world).any(|cb| cb.disrupted)
    };

    let state = city.resource::<DeepProductionChainState>();
    let chain_disrupted = state.chain_disrupted[1]; // Forestry chain index

    assert!(
        disrupted || chain_disrupted || state.disrupted_count > 0,
        "Processing building without inputs should be disrupted; \
         building_disrupted={disrupted}, chain_disrupted={chain_disrupted}, count={}",
        state.disrupted_count
    );
}

#[test]
fn test_downstream_stops_when_supply_cut() {
    let mut city = TestCity::new()
        .with_road(50, 50, 50, 65, RoadType::Local)
        .with_building(49, 52, ZoneType::Industrial, 2)
        .with_building(49, 58, ZoneType::Industrial, 2);

    // Place forest resources
    {
        let world = city.world_mut();
        let mut resource_grid = world.resource_mut::<ResourceGrid>();
        for dx in 45..55 {
            for dy in 48..60 {
                resource_grid.set(
                    dx,
                    dy,
                    ResourceDeposit {
                        resource_type: ResourceType::Forest,
                        amount: 500,
                        max_amount: 500,
                    },
                );
            }
        }
    }

    // Run so assign_industry_type fires
    city.tick(5);

    // Assign: first building = Forestry (extraction), second = SawMill (processing)
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut Building, &mut IndustryBuilding)>();
        let mut buildings: Vec<_> = q.iter_mut(world).collect();
        buildings.sort_by_key(|(b, _)| b.grid_y);

        if buildings.len() >= 2 {
            buildings[0].0.occupants = 20;
            buildings[0].1.industry_type = IndustryType::Forestry;
            buildings[0].1.workers = 20;
            buildings[0].1.efficiency = 0.8;

            buildings[1].0.occupants = 15;
            buildings[1].1.industry_type = IndustryType::SawMill;
            buildings[1].1.workers = 15;
            buildings[1].1.efficiency = 0.7;
        }
    }

    // Run to establish production (DeepChainBuilding assignment + production)
    city.tick(60);

    // Now remove all Timber from stockpile
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<DeepProductionChainState>();
        state.stockpile.insert(Commodity::Timber, 0.0);
    }

    // Also clear input buffers on SawMill buildings
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut DeepChainBuilding>();
        for mut chain in q.iter_mut(world) {
            if chain.stage == 1 {
                chain.input_buffer.clear();
            }
        }
    }

    // Also zero out the Forestry building's output to prevent it from refilling
    {
        let world = city.world_mut();
        let mut q = world.query::<(&mut DeepChainBuilding, &IndustryBuilding)>();
        for (mut chain, industry) in q.iter_mut(world) {
            if industry.industry_type == IndustryType::Forestry {
                chain.output_buffer.clear();
                // Also set workers to 0 to halt extraction
            }
        }
        // Halt the forestry building completely
        let mut bq = world.query::<(&mut Building, &mut IndustryBuilding)>();
        for (mut building, mut industry) in bq.iter_mut(world) {
            if industry.industry_type == IndustryType::Forestry {
                building.occupants = 0;
                industry.workers = 0;
                industry.efficiency = 0.0;
            }
        }
    }

    // Run more ticks - SawMill should become disrupted
    city.tick(60);

    let sawmill_disrupted = {
        let world = city.world_mut();
        let mut q = world.query::<&DeepChainBuilding>();
        q.iter(world).any(|cb| cb.stage == 1 && cb.disrupted)
    };

    let state = city.resource::<DeepProductionChainState>();
    assert!(
        sawmill_disrupted || state.chain_disrupted[1],
        "SawMill should be disrupted when Timber supply is cut off; \
         sawmill_disrupted={sawmill_disrupted}, chain_disrupted={}",
        state.chain_disrupted[1]
    );
}

// ====================================================================
// No disruption with no industry
// ====================================================================

#[test]
fn test_no_disruption_with_no_industry() {
    let mut city = TestCity::new();
    city.tick(20);

    let state = city.resource::<DeepProductionChainState>();
    assert_eq!(
        state.disrupted_count, 0,
        "no disruptions with no industrial buildings"
    );
    for i in 0..4 {
        assert!(
            !state.chain_disrupted[i],
            "chain {i} should not be disrupted"
        );
    }
}
