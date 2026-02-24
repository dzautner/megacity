//! Water treatment system and plugin registration.

use bevy::prelude::*;

use super::effects::{apply_treatment_grid_effects, apply_well_pump_effects};
use super::{
    calculate_disease_risk, calculate_effluent_quality, TreatmentLevel, WaterTreatmentState,
};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// System
// =============================================================================

/// System that updates water treatment plant state each slow tick.
///
/// - Discovers treatment plant service buildings and registers/removes them.
/// - Applies treatment effectiveness based on each plant's level.
/// - Distributes city demand across plants up to capacity.
/// - Calculates treatment costs and effluent quality.
/// - Computes disease risk from resulting water quality.
#[allow(clippy::too_many_arguments)]
pub fn update_water_treatment(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<WaterTreatmentState>,
    services: Query<(Entity, &ServiceBuilding)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Sync plant registry with existing service buildings ---
    // Collect all current WaterTreatmentPlant entity IDs
    let mut active_entities: Vec<Entity> = Vec::new();
    for (entity, service) in &services {
        if service.service_type != ServiceType::WaterTreatmentPlant {
            continue;
        }
        active_entities.push(entity);

        // Register new plants at Primary level
        if !state.plants.contains_key(&entity) {
            state.register_plant(entity, TreatmentLevel::Primary);
        }
    }

    // Remove plants whose entities no longer exist
    let stale_entities: Vec<Entity> = state
        .plants
        .keys()
        .filter(|e| !active_entities.contains(e))
        .copied()
        .collect();
    for entity in stale_entities {
        state.remove_plant(entity);
    }

    // --- Phase 2: Calculate demand and distribute flow ---
    let input_quality = state.avg_input_quality;
    let city_demand = state.city_demand_mgd;

    // Compute total capacity
    let total_capacity: f32 = state.plants.values().map(|p| p.capacity_mgd).sum();

    // Distribute demand proportionally across plants up to their capacity
    let mut remaining_demand = city_demand;
    let mut total_flow = 0.0_f32;
    let mut weighted_quality_sum = 0.0_f32;
    let mut total_cost = 0.0_f64;

    // Sort plant entities for deterministic iteration
    let mut plant_entities: Vec<Entity> = state.plants.keys().copied().collect();
    plant_entities.sort();

    for entity in &plant_entities {
        let plant = state.plants.get_mut(entity).unwrap();

        if remaining_demand <= 0.0 {
            plant.current_flow_mgd = 0.0;
            plant.effluent_quality = 0.0;
            plant.period_cost = 0.0;
            continue;
        }

        // Allocate flow up to this plant's capacity
        let flow = remaining_demand.min(plant.capacity_mgd);
        plant.current_flow_mgd = flow;
        remaining_demand -= flow;
        total_flow += flow;

        // Calculate effluent quality
        let effluent = calculate_effluent_quality(input_quality, plant.level);
        plant.effluent_quality = effluent;

        // Calculate treatment cost: cost_per_MG * flow_MGD
        let cost = plant.level.cost_per_million_gallons() * flow as f64;
        plant.period_cost = cost;
        total_cost += cost;

        // Weight quality by flow volume
        weighted_quality_sum += effluent * flow;
    }

    // --- Phase 3: Aggregate metrics ---
    state.total_capacity_mgd = total_capacity;
    state.total_flow_mgd = total_flow;
    state.total_period_cost = total_cost;

    // Weighted average effluent quality
    state.avg_effluent_quality = if total_flow > 0.0 {
        weighted_quality_sum / total_flow
    } else {
        0.0
    };

    // Treatment coverage: fraction of demand being treated
    state.treatment_coverage = if city_demand > 0.0 {
        (total_flow / city_demand).min(1.0)
    } else {
        1.0 // No demand = fully covered
    };

    // Disease risk: based on the blended quality of treated + untreated water
    // If not all demand is treated, the untreated portion has input_quality
    let blended_quality = if city_demand > 0.0 {
        let treated_portion = total_flow / city_demand;
        let untreated_portion = 1.0 - treated_portion.min(1.0);
        state.avg_effluent_quality * treated_portion.min(1.0) + input_quality * untreated_portion
    } else {
        1.0 // No demand = no risk
    };

    state.disease_risk = calculate_disease_risk(blended_quality);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WaterTreatmentPlugin;

impl Plugin for WaterTreatmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTreatmentState>();
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WaterTreatmentState>();
        app.add_systems(
            FixedUpdate,
            (
                update_water_treatment,
                (apply_treatment_grid_effects, apply_well_pump_effects)
                    .after(update_water_treatment),
            )
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
