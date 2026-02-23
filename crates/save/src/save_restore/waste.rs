// ---------------------------------------------------------------------------
// Restore functions for waste management: recycling, composting, hazardous
// waste, landfill gas, landfill capacity, UHI grid
// ---------------------------------------------------------------------------

use crate::save_codec::*;
use crate::save_types::*;

use simulation::landfill_gas::LandfillGasState;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::urban_heat_island::UhiGrid;

/// Restore `RecyclingState` and `RecyclingEconomics` from saved data.
pub fn restore_recycling(save: &SaveRecyclingState) -> (RecyclingState, RecyclingEconomics) {
    let tier = u8_to_recycling_tier(save.tier);
    let state = RecyclingState {
        tier,
        daily_tons_diverted: save.daily_tons_diverted,
        daily_tons_contaminated: save.daily_tons_contaminated,
        daily_revenue: save.daily_revenue,
        daily_cost: save.daily_cost,
        total_revenue: save.total_revenue,
        total_cost: save.total_cost,
        participating_households: save.participating_households,
    };
    let economics = RecyclingEconomics {
        price_paper: save.price_paper,
        price_plastic: save.price_plastic,
        price_glass: save.price_glass,
        price_metal: save.price_metal,
        price_organic: save.price_organic,
        market_cycle_position: save.market_cycle_position,
        last_update_day: save.economics_last_update_day,
    };
    (state, economics)
}

/// Restore a `UhiGrid` resource from saved data.
pub fn restore_uhi_grid(save: &SaveUhiGrid) -> UhiGrid {
    UhiGrid {
        cells: save.cells.clone(),
        width: save.width,
        height: save.height,
    }
}

/// Restore a `CompostingState` resource from saved data.
pub fn restore_composting(
    save: &crate::save_types::SaveCompostingState,
) -> simulation::composting::CompostingState {
    use simulation::composting::{CompostFacility, CompostingState};
    CompostingState {
        facilities: save
            .facilities
            .iter()
            .map(|f| CompostFacility {
                method: u8_to_compost_method(f.method),
                capacity_tons_per_day: f.capacity_tons_per_day,
                cost_per_ton: f.cost_per_ton,
                tons_processed_today: f.tons_processed_today,
            })
            .collect(),
        participation_rate: save.participation_rate,
        organic_fraction: save.organic_fraction,
        total_diverted_tons: save.total_diverted_tons,
        daily_diversion_tons: save.daily_diversion_tons,
        compost_revenue_per_ton: save.compost_revenue_per_ton,
        daily_revenue: save.daily_revenue,
        biogas_mwh_per_ton: save.biogas_mwh_per_ton,
        daily_biogas_mwh: save.daily_biogas_mwh,
    }
}

/// Restore a `HazardousWasteState` resource from saved data.
pub fn restore_hazardous_waste(
    save: &crate::save_types::SaveHazardousWasteState,
) -> simulation::hazardous_waste::HazardousWasteState {
    simulation::hazardous_waste::HazardousWasteState {
        total_generation: save.total_generation,
        treatment_capacity: save.treatment_capacity,
        overflow: save.overflow,
        illegal_dump_events: save.illegal_dump_events,
        contamination_level: save.contamination_level,
        federal_fines: save.federal_fines,
        facility_count: save.facility_count,
        daily_operating_cost: save.daily_operating_cost,
        chemical_treated: save.chemical_treated,
        thermal_treated: save.thermal_treated,
        biological_treated: save.biological_treated,
        stabilization_treated: save.stabilization_treated,
    }
}

/// Restore a `LandfillGasState` resource from saved data.
pub fn restore_landfill_gas(save: &SaveLandfillGasState) -> LandfillGasState {
    LandfillGasState {
        total_gas_generation_cf_per_year: save.total_gas_generation_cf_per_year,
        methane_fraction: save.methane_fraction,
        co2_fraction: save.co2_fraction,
        collection_active: save.collection_active,
        collection_efficiency: save.collection_efficiency,
        electricity_generated_mw: save.electricity_generated_mw,
        uncaptured_methane_cf: save.uncaptured_methane_cf,
        infrastructure_cost: save.infrastructure_cost,
        maintenance_cost_per_year: save.maintenance_cost_per_year,
        fire_explosion_risk: save.fire_explosion_risk,
        landfills_with_collection: save.landfills_with_collection,
        total_landfills: save.total_landfills,
    }
}

/// Restore a `LandfillCapacityState` resource from saved data.
pub fn restore_landfill_capacity(
    save: &crate::save_types::SaveLandfillCapacityState,
) -> simulation::landfill_warning::LandfillCapacityState {
    simulation::landfill_warning::LandfillCapacityState {
        total_capacity: save.total_capacity,
        current_fill: save.current_fill,
        daily_input_rate: save.daily_input_rate,
        days_remaining: save.days_remaining,
        years_remaining: save.years_remaining,
        remaining_pct: save.remaining_pct,
        current_tier: u8_to_landfill_warning_tier(save.current_tier),
        collection_halted: save.collection_halted,
        landfill_count: save.landfill_count,
    }
}
