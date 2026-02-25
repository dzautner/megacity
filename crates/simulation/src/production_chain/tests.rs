//! Unit tests for the deep production chain types.

use super::types::*;

#[test]
fn test_commodity_classification() {
    assert!(Commodity::Grain.is_raw());
    assert!(!Commodity::Grain.is_processed());
    assert!(!Commodity::Grain.is_final());

    assert!(!Commodity::Flour.is_raw());
    assert!(Commodity::Flour.is_processed());
    assert!(!Commodity::Flour.is_final());

    assert!(!Commodity::Bread.is_raw());
    assert!(!Commodity::Bread.is_processed());
    assert!(Commodity::Bread.is_final());
}

#[test]
fn test_commodity_chain_indices() {
    // Food chain = 0
    assert_eq!(Commodity::Grain.chain_index(), 0);
    assert_eq!(Commodity::Flour.chain_index(), 0);
    assert_eq!(Commodity::Bread.chain_index(), 0);

    // Forestry chain = 1
    assert_eq!(Commodity::Timber.chain_index(), 1);
    assert_eq!(Commodity::Lumber.chain_index(), 1);
    assert_eq!(Commodity::Furniture.chain_index(), 1);

    // Oil chain = 2
    assert_eq!(Commodity::CrudeOil.chain_index(), 2);
    assert_eq!(Commodity::Petroleum.chain_index(), 2);
    assert_eq!(Commodity::Plastics.chain_index(), 2);

    // Mining chain = 3
    assert_eq!(Commodity::IronOre.chain_index(), 3);
    assert_eq!(Commodity::Steel.chain_index(), 3);
    assert_eq!(Commodity::Machinery.chain_index(), 3);
}

#[test]
fn test_all_commodities_count() {
    assert_eq!(Commodity::all().len(), 12);
}

#[test]
fn test_chain_definitions() {
    let chains = all_chains();
    assert_eq!(chains.len(), 4, "should have 4 production chains");

    for chain in chains {
        assert_eq!(chain.len(), 3, "each chain should have 3 stages");
        // Stage 0 should have no inputs (extraction)
        assert!(
            chain[0].inputs.is_empty(),
            "extraction stage should have no inputs"
        );
        // Stage 0 should produce raw materials
        assert!(
            !chain[0].outputs.is_empty(),
            "extraction stage should produce outputs"
        );
        // Stages 1 and 2 should have inputs
        assert!(
            !chain[1].inputs.is_empty(),
            "processing stage should have inputs"
        );
        assert!(
            !chain[2].inputs.is_empty(),
            "manufacturing stage should have inputs"
        );
    }
}

#[test]
fn test_export_prices_increase_by_stage() {
    // Final products should be more expensive than raw materials
    assert!(Commodity::Bread.export_price() > Commodity::Grain.export_price());
    assert!(Commodity::Furniture.export_price() > Commodity::Timber.export_price());
    assert!(Commodity::Plastics.export_price() > Commodity::CrudeOil.export_price());
    assert!(Commodity::Machinery.export_price() > Commodity::IronOre.export_price());
}

#[test]
fn test_import_more_expensive_than_export() {
    for &c in Commodity::all() {
        assert!(
            c.import_price() > c.export_price(),
            "{:?} import should be more expensive than export",
            c
        );
    }
}

#[test]
fn test_default_state() {
    let state = DeepProductionChainState::default();
    for &c in Commodity::all() {
        assert_eq!(state.stock(c), 0.0);
        assert_eq!(state.net(c), 0.0);
    }
    assert_eq!(state.disrupted_count, 0);
    assert!(!state.chain_disrupted[0]);
    assert_eq!(state.commodity_trade_balance, 0.0);
}

#[test]
fn test_deep_chain_building_buffer_capacity() {
    let building = DeepChainBuilding::new(0, 0);
    assert!(!building.disrupted);
    assert_eq!(building.disruption_ticks, 0);
    assert_eq!(DeepChainBuilding::BUFFER_CAPACITY, 50.0);
}

#[test]
fn test_commodity_names() {
    for &c in Commodity::all() {
        assert!(!c.name().is_empty());
    }
}
