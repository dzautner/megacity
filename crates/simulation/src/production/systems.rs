use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{CitizenDetails, WorkLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::ZoneType;
use crate::natural_resources::{ResourceGrid, ResourceType};
use crate::TickCounter;

use super::types::{chain_for, CityGoods, GoodsType, IndustryBuilding, IndustryType};

/// How often the production chains run (every N ticks). At 10Hz fixed update,
/// 10 ticks = 1 second of game time.
const PRODUCTION_INTERVAL: u64 = 10;

/// When a new industrial Building spawns without an IndustryBuilding component,
/// auto-assign an IndustryType based on nearby ResourceGrid deposits.
/// Falls back to Manufacturing if no notable deposits are found.
pub fn assign_industry_type(
    mut commands: Commands,
    new_buildings: Query<(Entity, &Building), Without<IndustryBuilding>>,
    resource_grid: Res<ResourceGrid>,
) {
    for (entity, building) in &new_buildings {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }

        let industry_type =
            pick_industry_from_nearby_resources(building.grid_x, building.grid_y, &resource_grid);

        commands
            .entity(entity)
            .insert(IndustryBuilding::new(industry_type));
    }
}

/// Scan a radius around (gx, gy) for resource deposits and pick the most
/// appropriate IndustryType. Falls back to Manufacturing.
pub(crate) fn pick_industry_from_nearby_resources(
    gx: usize,
    gy: usize,
    resource_grid: &ResourceGrid,
) -> IndustryType {
    let search_radius: isize = 8;
    // BTreeMap for deterministic tie-breaking in max_by_key (iteration order matters).
    let mut counts: BTreeMap<ResourceType, u32> = BTreeMap::new();

    for dy in -search_radius..=search_radius {
        for dx in -search_radius..=search_radius {
            let nx = gx as isize + dx;
            let ny = gy as isize + dy;
            if nx < 0 || ny < 0 || nx >= GRID_WIDTH as isize || ny >= GRID_HEIGHT as isize {
                continue;
            }
            if let Some(deposit) = resource_grid.get(nx as usize, ny as usize) {
                if deposit.amount > 0 {
                    *counts.entry(deposit.resource_type).or_insert(0) += 1;
                }
            }
        }
    }

    if counts.is_empty() {
        // No resources nearby: use a hash-based split between Manufacturing and TechAssembly
        let hash = gx.wrapping_mul(31).wrapping_add(gy.wrapping_mul(17));
        return if hash.is_multiple_of(3) {
            IndustryType::TechAssembly
        } else {
            IndustryType::Manufacturing
        };
    }

    // Pick the dominant resource type
    let Some((best_resource, _count)) = counts.iter().max_by_key(|(_, &c)| c) else {
        return IndustryType::Manufacturing;
    };

    // Decide between extraction and processing based on a position hash
    let hash = gx.wrapping_mul(13).wrapping_add(gy.wrapping_mul(7));
    let prefer_processing = hash.is_multiple_of(3); // ~33% chance of processing variant

    match best_resource {
        ResourceType::FertileLand => {
            if prefer_processing {
                IndustryType::FoodProcessing
            } else {
                IndustryType::Agriculture
            }
        }
        ResourceType::Forest => {
            if prefer_processing {
                IndustryType::SawMill
            } else {
                IndustryType::Forestry
            }
        }
        ResourceType::Ore => {
            if prefer_processing {
                IndustryType::Smelter
            } else {
                IndustryType::Mining
            }
        }
        ResourceType::Oil => {
            if prefer_processing {
                IndustryType::Refinery
            } else {
                IndustryType::OilExtraction
            }
        }
    }
}

/// Main production system. Runs every PRODUCTION_INTERVAL ticks.
///
/// 1. Updates worker counts and efficiency for each IndustryBuilding.
/// 2. Extraction industries pull raw materials from ResourceGrid deposits.
/// 3. Processing/Manufacturing consume inputs from CityGoods and produce outputs.
/// 4. Computes city-wide consumption from population.
/// 5. Surplus generates export income; deficit triggers expensive imports.
#[allow(clippy::too_many_arguments)]
pub fn update_production_chains(
    tick: Res<TickCounter>,
    mut city_goods: ResMut<CityGoods>,
    mut resource_grid: ResMut<ResourceGrid>,
    mut industry_q: Query<(Entity, &Building, &mut IndustryBuilding)>,
    workers_q: Query<(&WorkLocation, &CitizenDetails)>,
    stats: Res<crate::stats::CityStats>,
    mut budget: ResMut<CityBudget>,
) {
    if !tick.0.is_multiple_of(PRODUCTION_INTERVAL) {
        return;
    }

    // No population means no workers and no consumption — skip production and
    // trade entirely to avoid phantom import costs draining the treasury.
    if stats.population == 0 {
        return;
    }

    // -------------------------------------------------------------------------
    // 1. Reset per-cycle production/consumption rates
    // -------------------------------------------------------------------------
    for &g in GoodsType::all() {
        city_goods.production_rate.insert(g, 0.0);
        city_goods.consumption_rate.insert(g, 0.0);
    }

    // -------------------------------------------------------------------------
    // 2. Pre-compute per-building worker counts and average education
    // -------------------------------------------------------------------------
    let mut building_worker_info: HashMap<Entity, (u32, f32)> = HashMap::new(); // (count, total_edu)
    for (work_loc, details) in &workers_q {
        let entry = building_worker_info
            .entry(work_loc.building)
            .or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += details.education as f32;
    }

    // -------------------------------------------------------------------------
    // 3. Process each IndustryBuilding
    // -------------------------------------------------------------------------
    for (entity, building, mut industry) in &mut industry_q {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }

        // Update worker count
        industry.workers = building.occupants;

        if industry.workers == 0 {
            industry.efficiency = 0.0;
            continue;
        }

        // Compute efficiency: base from worker ratio, boosted by education
        let worker_ratio = (industry.workers as f32 / building.capacity.max(1) as f32).min(1.0);

        // Get average education of workers at this building (0-3 scale)
        let avg_edu = if let Some(&(count, total_edu)) = building_worker_info.get(&entity) {
            if count > 0 {
                total_edu / count as f32
            } else {
                0.0
            }
        } else {
            // Fallback: estimate from occupants
            1.0
        };
        // Education multiplier: 0.6 at edu=0, up to 1.4 at edu=3
        let edu_mult = 0.6 + avg_edu * 0.267;
        industry.efficiency = worker_ratio * edu_mult;

        let chain = chain_for(industry.industry_type);
        let production_scale = industry.efficiency * industry.workers as f32 * 0.1;

        if industry.industry_type.is_extraction() {
            // Extraction: pull from resource grid deposits around the building
            let extracted = extract_from_deposits(
                building.grid_x,
                building.grid_y,
                &mut resource_grid,
                industry.industry_type,
                production_scale,
            );

            // Add extracted goods to city stockpile
            for (goods, amount) in &chain.outputs {
                let produced = amount * extracted;
                *city_goods.available.entry(*goods).or_insert(0.0) += produced;
                *city_goods.production_rate.entry(*goods).or_insert(0.0) += produced;
                *industry.output_storage.entry(*goods).or_insert(0.0) += produced;
            }
        } else {
            // Processing/Manufacturing: consume inputs from city stockpile
            // Check if enough inputs are available
            let mut can_produce = true;
            let mut limiting_factor = 1.0f32;
            for (goods, amount_needed) in &chain.inputs {
                let needed = amount_needed * production_scale;
                let available = city_goods.available.get(goods).copied().unwrap_or(0.0);
                if available < needed {
                    if available <= 0.0 {
                        can_produce = false;
                        break;
                    }
                    // Partial production: limited by available inputs
                    limiting_factor = limiting_factor.min(available / needed);
                }
            }

            if can_produce {
                let actual_scale = production_scale * limiting_factor;

                // Consume inputs
                for (goods, amount_needed) in &chain.inputs {
                    let consumed = amount_needed * actual_scale;
                    if let Some(stock) = city_goods.available.get_mut(goods) {
                        *stock = (*stock - consumed).max(0.0);
                    }
                    *city_goods.consumption_rate.entry(*goods).or_insert(0.0) += consumed;
                    *industry.input_storage.entry(*goods).or_insert(0.0) += consumed;
                }

                // Produce outputs
                for (goods, amount) in &chain.outputs {
                    let produced = amount * actual_scale;
                    *city_goods.available.entry(*goods).or_insert(0.0) += produced;
                    *city_goods.production_rate.entry(*goods).or_insert(0.0) += produced;
                    *industry.output_storage.entry(*goods).or_insert(0.0) += produced;
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. City-wide consumption based on population
    // -------------------------------------------------------------------------
    let pop = stats.population as f32;
    let food_demand = pop * 0.005;
    let goods_demand = pop * 0.003;
    let fuel_demand = pop * 0.002;
    let electronics_demand = pop * 0.001;

    // Consume ProcessedFood first, then RawFood as fallback
    consume_goods(&mut city_goods, GoodsType::ProcessedFood, food_demand * 0.7);
    consume_goods(&mut city_goods, GoodsType::RawFood, food_demand * 0.3);
    consume_goods(&mut city_goods, GoodsType::ConsumerGoods, goods_demand);
    consume_goods(&mut city_goods, GoodsType::Fuel, fuel_demand);
    consume_goods(&mut city_goods, GoodsType::Electronics, electronics_demand);

    // -------------------------------------------------------------------------
    // 5. Trade: surplus -> export income, deficit -> import cost
    // -------------------------------------------------------------------------
    let mut trade_balance = 0.0f64;
    for &g in GoodsType::all() {
        let stock = city_goods.available.get(&g).copied().unwrap_or(0.0);
        // Surplus threshold: anything above 100 units gets exported
        if stock > 100.0 {
            let surplus = stock - 100.0;
            trade_balance += surplus as f64 * g.export_price() * 0.01; // per-tick fraction
                                                                       // Cap stockpile at 100 (export the rest)
            city_goods.available.insert(g, 100.0);
        }
        // Deficit: if stock is negative (shouldn't normally happen) or very low,
        // auto-import at higher cost. We handle this via the consumption_rate > production_rate check.
        let net = city_goods.net(g);
        if net < -0.1 {
            // City is consuming more than producing; import the deficit
            let deficit = (-net).min(10.0); // cap auto-import rate
            trade_balance -= deficit as f64 * g.import_price() * 0.01;
            // Add imported goods to stockpile
            *city_goods.available.entry(g).or_insert(0.0) += deficit * 0.5;
        }
    }

    city_goods.trade_balance = trade_balance;
    // Apply trade balance to treasury each tick (small per-tick amount)
    budget.treasury += trade_balance;
}

/// Consume `amount` of a goods type from the city stockpile.
fn consume_goods(city_goods: &mut CityGoods, goods: GoodsType, amount: f32) {
    if let Some(stock) = city_goods.available.get_mut(&goods) {
        *stock = (*stock - amount).max(0.0);
    }
    *city_goods.consumption_rate.entry(goods).or_insert(0.0) += amount;
}

/// Extract raw materials from ResourceGrid deposits near (gx, gy).
/// Returns a scaling factor (0.0..1.0+) representing how much was actually extracted
/// relative to the requested `production_scale`.
fn extract_from_deposits(
    gx: usize,
    gy: usize,
    resource_grid: &mut ResourceGrid,
    industry_type: IndustryType,
    production_scale: f32,
) -> f32 {
    let target_resource = match industry_type {
        IndustryType::Agriculture => ResourceType::FertileLand,
        IndustryType::Forestry => ResourceType::Forest,
        IndustryType::Mining => ResourceType::Ore,
        IndustryType::OilExtraction => ResourceType::Oil,
        _ => return 0.0,
    };

    let search_radius: isize = 4;
    let mut total_extracted = 0.0f32;
    let mut remaining_demand = production_scale;

    for dy in -search_radius..=search_radius {
        for dx in -search_radius..=search_radius {
            if remaining_demand <= 0.0 {
                break;
            }
            let nx = gx as isize + dx;
            let ny = gy as isize + dy;
            if nx < 0 || ny < 0 || nx >= GRID_WIDTH as isize || ny >= GRID_HEIGHT as isize {
                continue;
            }

            let deposit = resource_grid.get_mut(nx as usize, ny as usize);
            if let Some(ref mut d) = deposit {
                if d.resource_type != target_resource || d.amount == 0 {
                    continue;
                }

                // Extract up to remaining_demand, limited by deposit amount
                let extract_amount = remaining_demand.min(d.amount as f32 * 0.01);
                total_extracted += extract_amount;
                remaining_demand -= extract_amount;

                // Deplete the deposit
                if d.resource_type.is_renewable() {
                    // Renewable: slow extraction, some regen
                    let depletion = (extract_amount * 0.5) as u32;
                    d.amount = d.amount.saturating_sub(depletion);
                    d.amount = (d.amount + 1).min(d.max_amount);
                } else {
                    // Finite: permanent depletion
                    let depletion = (extract_amount * 1.0).max(1.0) as u32;
                    d.amount = d.amount.saturating_sub(depletion);
                }
            }
        }
    }

    if production_scale > 0.0 {
        total_extracted / production_scale
    } else {
        0.0
    }
}
