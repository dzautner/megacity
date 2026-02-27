//! Systems for the deep production chain simulation (SERV-009).

use bevy::prelude::*;

use crate::buildings::Building;
use crate::economy::CityBudget;
use crate::grid::ZoneType;
use crate::outside_connections::{ConnectionType, OutsideConnections};
use crate::production::IndustryBuilding;
use crate::TickCounter;

use super::types::{all_chains, Commodity, DeepChainBuilding, DeepProductionChainState};

/// How often deep production chains update (every N ticks).
const CHAIN_INTERVAL: u64 = 10;

/// Surplus threshold above which commodities are auto-exported.
const EXPORT_THRESHOLD: f32 = 80.0;

/// Maximum auto-import rate per commodity per cycle.
const MAX_IMPORT_RATE: f32 = 30.0;

// =============================================================================
// System: update_deep_production_chains
// =============================================================================

/// Main deep production chain system. For each industrial building with a
/// `DeepChainBuilding` component, run its production stage:
/// - Extraction: produce raw materials into city stockpile
/// - Processing: consume raw -> produce processed
/// - Manufacturing: consume processed -> produce final goods
///
/// Buildings without `DeepChainBuilding` get one assigned based on their
/// `IndustryBuilding.industry_type`.
#[allow(clippy::too_many_arguments)]
pub fn update_deep_production_chains(
    tick: Res<TickCounter>,
    mut commands: Commands,
    mut state: ResMut<DeepProductionChainState>,
    mut chain_q: Query<(
        Entity,
        &Building,
        &IndustryBuilding,
        Option<&mut DeepChainBuilding>,
    )>,
) {
    if !tick.0.is_multiple_of(CHAIN_INTERVAL) {
        return;
    }

    // Reset per-cycle rates
    for &c in Commodity::all() {
        state.production_rates.insert(c, 0.0);
        state.consumption_rates.insert(c, 0.0);
    }

    let chains = all_chains();

    // Collect entities that need DeepChainBuilding assigned
    let mut to_assign: Vec<(Entity, usize, usize)> = Vec::new();

    for (entity, building, industry, chain_opt) in &mut chain_q {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }
        if industry.workers == 0 {
            continue;
        }

        let Some(mut chain_building) = chain_opt else {
            // Assign a DeepChainBuilding based on industry type
            if let Some((ci, si)) = chain_and_stage_for(industry.industry_type) {
                to_assign.push((entity, ci, si));
            }
            continue;
        };
        let ci = chain_building.chain_index;
        let si = chain_building.stage;

        if ci >= chains.len() || si >= chains[ci].len() {
            continue;
        }

        let stage_def = &chains[ci][si];
        let production_scale = industry.efficiency * industry.workers as f32 * 0.1;

        // Check if inputs are available
        let mut can_produce = true;
        let mut limiting_factor = 1.0f32;

        for &(commodity, amount_needed) in stage_def.inputs {
            let needed = amount_needed * production_scale;
            let available = state.stock(commodity) + buffer_stock(&chain_building, commodity);
            if available < needed {
                if available <= 0.0 {
                    can_produce = false;
                    break;
                }
                limiting_factor = limiting_factor.min(available / needed);
            }
        }

        if !can_produce {
            // Supply chain disruption
            chain_building.disrupted = true;
            chain_building.disruption_ticks += 1;
            continue;
        }

        chain_building.disrupted = false;
        chain_building.disruption_ticks = 0;

        let actual_scale = production_scale * limiting_factor;

        // Consume inputs from city stockpile (preferring building buffer first)
        for &(commodity, amount_needed) in stage_def.inputs {
            let consumed = amount_needed * actual_scale;
            // First draw from building's input buffer
            let from_buffer = chain_building
                .input_buffer
                .get(&commodity)
                .copied()
                .unwrap_or(0.0)
                .min(consumed);
            if from_buffer > 0.0 {
                *chain_building.input_buffer.entry(commodity).or_insert(0.0) -= from_buffer;
            }
            // Draw remainder from city stockpile
            let from_city = consumed - from_buffer;
            if from_city > 0.0 {
                if let Some(stock) = state.stockpile.get_mut(&commodity) {
                    *stock = (*stock - from_city).max(0.0);
                }
            }
            *state.consumption_rates.entry(commodity).or_insert(0.0) += consumed;
        }

        // Produce outputs into building buffer, overflow to city stockpile
        for &(commodity, amount) in stage_def.outputs {
            let produced = amount * actual_scale;
            let buffer = chain_building
                .output_buffer
                .entry(commodity)
                .or_insert(0.0);
            let space = DeepChainBuilding::BUFFER_CAPACITY - *buffer;
            let to_buffer = produced.min(space);
            *buffer += to_buffer;

            // Overflow goes to city stockpile
            let overflow = produced - to_buffer;
            if overflow > 0.0 {
                *state.stockpile.entry(commodity).or_insert(0.0) += overflow;
            }
            *state.production_rates.entry(commodity).or_insert(0.0) += produced;
        }

        // Transfer output buffer to city stockpile gradually (simulates truck logistics)
        let transfer_rate = 0.3; // 30% of buffer per cycle
        let outputs: Vec<(Commodity, f32)> = chain_building
            .output_buffer
            .iter()
            .filter(|(_, &v)| v > 0.01)
            .map(|(&k, &v)| (k, v))
            .collect();
        for (commodity, amount) in outputs {
            let transfer = amount * transfer_rate;
            *chain_building.output_buffer.entry(commodity).or_insert(0.0) -= transfer;
            *state.stockpile.entry(commodity).or_insert(0.0) += transfer;
        }
    }

    // Assign DeepChainBuilding components to new buildings
    for (entity, ci, si) in to_assign {
        commands.entity(entity).insert(DeepChainBuilding::new(ci, si));
    }
}

// =============================================================================
// System: update_chain_disruptions
// =============================================================================

/// Update city-wide disruption tracking: count disrupted buildings per chain.
pub fn update_chain_disruptions(
    tick: Res<TickCounter>,
    mut state: ResMut<DeepProductionChainState>,
    chain_q: Query<&DeepChainBuilding>,
) {
    if !tick.0.is_multiple_of(CHAIN_INTERVAL) {
        return;
    }

    let mut disrupted_per_chain = [false; 4];
    let mut disrupted_count = 0u32;

    for chain_building in &chain_q {
        if chain_building.disrupted {
            disrupted_count += 1;
            let ci = chain_building.chain_index;
            if ci < 4 {
                disrupted_per_chain[ci] = true;
            }
        }
    }

    state.chain_disrupted = disrupted_per_chain;
    state.disrupted_count = disrupted_count;
}

// =============================================================================
// System: update_chain_import_export
// =============================================================================

/// Handle import/export of commodities at outside connections.
/// Surplus final goods are exported for income; missing raw materials are imported.
#[allow(clippy::too_many_arguments)]
pub fn update_chain_import_export(
    tick: Res<TickCounter>,
    mut state: ResMut<DeepProductionChainState>,
    mut budget: ResMut<CityBudget>,
    connections: Res<OutsideConnections>,
) {
    if !tick.0.is_multiple_of(CHAIN_INTERVAL) {
        return;
    }

    let has_highway = connections.has_connection(ConnectionType::Highway);
    let has_seaport = connections.has_connection(ConnectionType::SeaPort);
    let has_railway = connections.has_connection(ConnectionType::Railway);

    // No connections = no trade
    if !has_highway && !has_seaport && !has_railway {
        state.commodity_trade_balance = 0.0;
        return;
    }

    // Import cost multiplier: seaport reduces cost for raw materials
    let raw_import_discount = if has_seaport { 0.5 } else { 1.0 };
    // Railway reduces import cost for processed goods
    let processed_import_discount = if has_railway { 0.7 } else { 1.0 };

    let mut trade_balance = 0.0f64;

    for &commodity in Commodity::all() {
        let stock = state.stock(commodity);

        // Export surplus
        if stock > EXPORT_THRESHOLD {
            let surplus = stock - EXPORT_THRESHOLD;
            let price = commodity.export_price();
            let income = surplus as f64 * price * 0.01;
            trade_balance += income;
            state.total_exported += surplus as f64;
            state.stockpile.insert(commodity, EXPORT_THRESHOLD);
        }

        // Import deficit: if production chain needs inputs that aren't available
        let net = state.net(commodity);
        if net < -0.1 && stock < 10.0 {
            let import_amount = (-net).min(MAX_IMPORT_RATE);
            let base_price = commodity.import_price();
            let discount = if commodity.is_raw() {
                raw_import_discount
            } else if commodity.is_processed() {
                processed_import_discount
            } else {
                1.0
            };
            let cost = import_amount as f64 * base_price * discount * 0.01;
            trade_balance -= cost;
            state.total_imported += import_amount as f64;
            *state.stockpile.entry(commodity).or_insert(0.0) += import_amount;
        }
    }

    state.commodity_trade_balance = trade_balance;
    budget.treasury += trade_balance;
}

// =============================================================================
// Helpers
// =============================================================================

/// Map an IndustryType to (chain_index, stage_index) in the deep chain system.
fn chain_and_stage_for(
    industry: crate::production::IndustryType,
) -> Option<(usize, usize)> {
    use crate::production::IndustryType;
    match industry {
        IndustryType::Agriculture => Some((0, 0)),
        IndustryType::FoodProcessing => Some((0, 1)),
        IndustryType::Forestry => Some((1, 0)),
        IndustryType::SawMill => Some((1, 1)),
        IndustryType::OilExtraction => Some((2, 0)),
        IndustryType::Refinery => Some((2, 1)),
        IndustryType::Mining => Some((3, 0)),
        IndustryType::Smelter => Some((3, 1)),
        IndustryType::Manufacturing => None, // Manufacturing can serve multiple chains
        IndustryType::TechAssembly => Some((3, 2)),
    }
}

/// Get amount of a commodity in a building's input buffer.
fn buffer_stock(chain_building: &DeepChainBuilding, commodity: Commodity) -> f32 {
    chain_building
        .input_buffer
        .get(&commodity)
        .copied()
        .unwrap_or(0.0)
}
