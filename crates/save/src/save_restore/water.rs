// ---------------------------------------------------------------------------
// Restore functions for water systems: sources, reservoir, treatment,
// groundwater depletion, wastewater, water conservation
// ---------------------------------------------------------------------------

use crate::save_codec::*;
use crate::save_types::*;

use simulation::reservoir::ReservoirState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_sources::WaterSource;

/// Restore a `WaterSource` component from saved data.
pub fn restore_water_source(save: &SaveWaterSource) -> Option<WaterSource> {
    let source_type = u8_to_water_source_type(save.source_type)?;
    Some(WaterSource {
        source_type,
        capacity_mgd: save.capacity_mgd,
        quality: save.quality,
        operating_cost: save.operating_cost,
        grid_x: save.grid_x,
        grid_y: save.grid_y,
        stored_gallons: save.stored_gallons,
        storage_capacity: save.storage_capacity,
    })
}

/// Restore a `WaterTreatmentState` resource from saved data.
pub fn restore_water_treatment(
    save: &crate::save_types::SaveWaterTreatmentState,
) -> simulation::water_treatment::WaterTreatmentState {
    use std::collections::HashMap;

    simulation::water_treatment::WaterTreatmentState {
        plants: HashMap::new(), // Plants will be re-discovered from entities on next tick
        total_capacity_mgd: save.total_capacity_mgd,
        total_flow_mgd: save.total_flow_mgd,
        avg_effluent_quality: save.avg_effluent_quality,
        total_period_cost: save.total_period_cost,
        city_demand_mgd: save.city_demand_mgd,
        treatment_coverage: save.treatment_coverage,
        avg_input_quality: save.avg_input_quality,
        disease_risk: save.disease_risk,
    }
}

/// Restore a `GroundwaterDepletionState` resource from saved data.
pub fn restore_groundwater_depletion(
    save: &crate::save_types::SaveGroundwaterDepletionState,
) -> simulation::groundwater_depletion::GroundwaterDepletionState {
    simulation::groundwater_depletion::GroundwaterDepletionState {
        extraction_rate: save.extraction_rate,
        recharge_rate: save.recharge_rate,
        sustainability_ratio: save.sustainability_ratio,
        critical_depletion: save.critical_depletion,
        subsidence_cells: save.subsidence_cells,
        well_yield_modifier: save.well_yield_modifier,
        ticks_below_threshold: save.ticks_below_threshold.clone(),
        previous_levels: save.previous_levels.clone(),
        recharge_basin_count: save.recharge_basin_count,
        avg_groundwater_level: save.avg_groundwater_level,
        cells_at_risk: save.cells_at_risk,
        over_extracted_cells: save.over_extracted_cells,
    }
}

/// Restore a `WastewaterState` resource from saved data.
pub fn restore_wastewater(
    save: &crate::save_types::SaveWastewaterState,
) -> simulation::wastewater::WastewaterState {
    simulation::wastewater::WastewaterState {
        total_sewage_generated: save.total_sewage_generated,
        total_treatment_capacity: save.total_treatment_capacity,
        overflow_amount: save.overflow_amount,
        coverage_ratio: save.coverage_ratio,
        pollution_events: save.pollution_events,
        health_penalty_active: save.health_penalty_active,
    }
}

/// Restore a `ReservoirState` resource from saved data.
pub fn restore_reservoir_state(save: &SaveReservoirState) -> ReservoirState {
    ReservoirState {
        total_storage_capacity_mg: save.total_storage_capacity_mg,
        current_level_mg: save.current_level_mg,
        inflow_rate_mgd: save.inflow_rate_mgd,
        outflow_rate_mgd: save.outflow_rate_mgd,
        evaporation_rate_mgd: save.evaporation_rate_mgd,
        net_change_mgd: save.net_change_mgd,
        storage_days: save.storage_days,
        reservoir_count: save.reservoir_count,
        warning_tier: u8_to_reservoir_warning_tier(save.warning_tier),
        min_reserve_pct: save.min_reserve_pct,
    }
}

/// Restore a `WaterConservationState` resource from saved data.
pub fn restore_water_conservation(state: &SaveWaterConservationState) -> WaterConservationState {
    WaterConservationState {
        low_flow_fixtures: state.low_flow_fixtures,
        xeriscaping: state.xeriscaping,
        tiered_pricing: state.tiered_pricing,
        greywater_recycling: state.greywater_recycling,
        rainwater_harvesting: state.rainwater_harvesting,
        demand_reduction_pct: state.demand_reduction_pct,
        sewage_reduction_pct: state.sewage_reduction_pct,
        total_retrofit_cost: state.total_retrofit_cost,
        annual_savings_gallons: state.annual_savings_gallons,
        buildings_retrofitted: state.buildings_retrofitted,
    }
}
