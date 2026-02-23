use crate::save_codec::*;
use crate::save_types::*;

use simulation::cold_snap::ColdSnapState;
use simulation::cso::SewerSystemState;
use simulation::drought::DroughtState;
use simulation::flood_simulation::FloodState;
use simulation::groundwater_depletion::GroundwaterDepletionState;
use simulation::hazardous_waste::HazardousWasteState;
use simulation::heat_wave::HeatWaveState;
use simulation::landfill_gas::LandfillGasState;
use simulation::landfill_warning::LandfillCapacityState;
use simulation::reservoir::ReservoirState;
use simulation::storm_drainage::StormDrainageState;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_treatment::WaterTreatmentState;
use simulation::wind_damage::WindDamageState;

/// Disaster / hazard state: drought, heat wave, cold snap, flood, wind
/// damage, reservoir, landfill gas, CSO, hazardous waste, wastewater, etc.
pub struct DisasterStageOutput {
    pub drought_state: Option<SaveDroughtState>,
    pub heat_wave_state: Option<SaveHeatWaveState>,
    pub cold_snap_state: Option<SaveColdSnapState>,
    pub flood_state: Option<SaveFloodState>,
    pub wind_damage_state: Option<SaveWindDamageState>,
    pub reservoir_state: Option<SaveReservoirState>,
    pub landfill_gas_state: Option<SaveLandfillGasState>,
    pub cso_state: Option<SaveCsoState>,
    pub hazardous_waste_state: Option<SaveHazardousWasteState>,
    pub wastewater_state: Option<SaveWastewaterState>,
    pub storm_drainage_state: Option<SaveStormDrainageState>,
    pub landfill_capacity_state: Option<SaveLandfillCapacityState>,
    pub groundwater_depletion_state: Option<SaveGroundwaterDepletionState>,
    pub water_treatment_state: Option<SaveWaterTreatmentState>,
    pub water_conservation_state: Option<SaveWaterConservationState>,
}

/// Collect disaster / hazard state.
#[allow(clippy::too_many_arguments)]
pub fn collect_disaster_stage(
    drought_state: Option<&DroughtState>,
    heat_wave_state: Option<&HeatWaveState>,
    cold_snap_state: Option<&ColdSnapState>,
    flood_state: Option<&FloodState>,
    wind_damage_state: Option<&WindDamageState>,
    reservoir_state: Option<&ReservoirState>,
    landfill_gas_state: Option<&LandfillGasState>,
    cso_state: Option<&SewerSystemState>,
    hazardous_waste_state: Option<&HazardousWasteState>,
    wastewater_state: Option<&WastewaterState>,
    storm_drainage_state: Option<&StormDrainageState>,
    landfill_capacity_state: Option<&LandfillCapacityState>,
    groundwater_depletion_state: Option<&GroundwaterDepletionState>,
    water_treatment_state: Option<&WaterTreatmentState>,
    water_conservation_state: Option<&WaterConservationState>,
) -> DisasterStageOutput {
    DisasterStageOutput {
        drought_state: drought_state.map(|ds| SaveDroughtState {
            rainfall_history: ds.rainfall_history.clone(),
            current_index: ds.current_index,
            current_tier: drought_tier_to_u8(ds.current_tier),
            expected_daily_rainfall: ds.expected_daily_rainfall,
            water_demand_modifier: ds.water_demand_modifier,
            agriculture_modifier: ds.agriculture_modifier,
            fire_risk_multiplier: ds.fire_risk_multiplier,
            happiness_modifier: ds.happiness_modifier,
            last_record_day: ds.last_record_day,
        }),
        heat_wave_state: heat_wave_state.map(|hw| SaveHeatWaveState {
            consecutive_hot_days: hw.consecutive_hot_days,
            severity: heat_wave_severity_to_u8(hw.severity),
            excess_mortality_per_100k: hw.excess_mortality_per_100k,
            energy_demand_multiplier: hw.energy_demand_multiplier,
            water_demand_multiplier: hw.water_demand_multiplier,
            road_damage_active: hw.road_damage_active,
            fire_risk_multiplier: hw.fire_risk_multiplier,
            blackout_risk: hw.blackout_risk,
            heat_threshold_c: hw.heat_threshold_c,
            consecutive_extreme_days: hw.consecutive_extreme_days,
            last_check_day: hw.last_check_day,
        }),
        cold_snap_state: cold_snap_state.map(|cs| SaveColdSnapState {
            consecutive_cold_days: cs.consecutive_cold_days,
            pipe_burst_count: cs.pipe_burst_count,
            is_active: cs.is_active,
            current_tier: cold_snap_tier_to_u8(cs.current_tier),
            heating_demand_modifier: cs.heating_demand_modifier,
            traffic_capacity_modifier: cs.traffic_capacity_modifier,
            schools_closed: cs.schools_closed,
            construction_halted: cs.construction_halted,
            homeless_mortality_rate: cs.homeless_mortality_rate,
            water_service_modifier: cs.water_service_modifier,
            last_check_day: cs.last_check_day,
        }),
        flood_state: flood_state.map(|fs| SaveFloodState {
            is_flooding: fs.is_flooding,
            total_flooded_cells: fs.total_flooded_cells,
            total_damage: fs.total_damage,
            max_depth: fs.max_depth,
        }),
        wind_damage_state: wind_damage_state.map(|wds| SaveWindDamageState {
            current_tier: wind_damage_tier_to_u8(wds.current_tier),
            accumulated_building_damage: wds.accumulated_building_damage,
            trees_knocked_down: wds.trees_knocked_down,
            power_outage_active: wds.power_outage_active,
        }),
        reservoir_state: reservoir_state.map(|rs| SaveReservoirState {
            total_storage_capacity_mg: rs.total_storage_capacity_mg,
            current_level_mg: rs.current_level_mg,
            inflow_rate_mgd: rs.inflow_rate_mgd,
            outflow_rate_mgd: rs.outflow_rate_mgd,
            evaporation_rate_mgd: rs.evaporation_rate_mgd,
            net_change_mgd: rs.net_change_mgd,
            storage_days: rs.storage_days,
            reservoir_count: rs.reservoir_count,
            warning_tier: reservoir_warning_tier_to_u8(rs.warning_tier),
            min_reserve_pct: rs.min_reserve_pct,
        }),
        landfill_gas_state: landfill_gas_state.map(|lgs| SaveLandfillGasState {
            total_gas_generation_cf_per_year: lgs.total_gas_generation_cf_per_year,
            methane_fraction: lgs.methane_fraction,
            co2_fraction: lgs.co2_fraction,
            collection_active: lgs.collection_active,
            collection_efficiency: lgs.collection_efficiency,
            electricity_generated_mw: lgs.electricity_generated_mw,
            uncaptured_methane_cf: lgs.uncaptured_methane_cf,
            infrastructure_cost: lgs.infrastructure_cost,
            maintenance_cost_per_year: lgs.maintenance_cost_per_year,
            fire_explosion_risk: lgs.fire_explosion_risk,
            landfills_with_collection: lgs.landfills_with_collection,
            total_landfills: lgs.total_landfills,
        }),
        cso_state: cso_state.map(|s| SaveCsoState {
            sewer_type: sewer_type_to_u8(&s.sewer_type),
            combined_capacity: s.combined_capacity,
            current_flow: s.current_flow,
            cso_active: s.cso_active,
            cso_discharge_gallons: s.cso_discharge_gallons,
            cso_events_total: s.cso_events_total,
            cso_events_this_year: s.cso_events_this_year,
            cells_with_separated_sewer: s.cells_with_separated_sewer,
            total_sewer_cells: s.total_sewer_cells,
            separation_coverage: s.separation_coverage,
            annual_cso_volume: s.annual_cso_volume,
            pollution_contribution: s.pollution_contribution,
        }),
        hazardous_waste_state: hazardous_waste_state.map(|hws| SaveHazardousWasteState {
            total_generation: hws.total_generation,
            treatment_capacity: hws.treatment_capacity,
            overflow: hws.overflow,
            illegal_dump_events: hws.illegal_dump_events,
            contamination_level: hws.contamination_level,
            federal_fines: hws.federal_fines,
            facility_count: hws.facility_count,
            daily_operating_cost: hws.daily_operating_cost,
            chemical_treated: hws.chemical_treated,
            thermal_treated: hws.thermal_treated,
            biological_treated: hws.biological_treated,
            stabilization_treated: hws.stabilization_treated,
        }),
        wastewater_state: wastewater_state.map(|ws| SaveWastewaterState {
            total_sewage_generated: ws.total_sewage_generated,
            total_treatment_capacity: ws.total_treatment_capacity,
            overflow_amount: ws.overflow_amount,
            coverage_ratio: ws.coverage_ratio,
            pollution_events: ws.pollution_events,
            health_penalty_active: ws.health_penalty_active,
        }),
        storm_drainage_state: storm_drainage_state.map(|sds| SaveStormDrainageState {
            total_drain_capacity: sds.total_drain_capacity,
            total_retention_capacity: sds.total_retention_capacity,
            current_retention_stored: sds.current_retention_stored,
            drain_count: sds.drain_count,
            retention_pond_count: sds.retention_pond_count,
            rain_garden_count: sds.rain_garden_count,
            overflow_cells: sds.overflow_cells,
            drainage_coverage: sds.drainage_coverage,
        }),
        landfill_capacity_state: landfill_capacity_state.map(|lcs| SaveLandfillCapacityState {
            total_capacity: lcs.total_capacity,
            current_fill: lcs.current_fill,
            daily_input_rate: lcs.daily_input_rate,
            days_remaining: lcs.days_remaining,
            years_remaining: lcs.years_remaining,
            remaining_pct: lcs.remaining_pct,
            current_tier: landfill_warning_tier_to_u8(lcs.current_tier),
            collection_halted: lcs.collection_halted,
            landfill_count: lcs.landfill_count,
        }),
        groundwater_depletion_state: groundwater_depletion_state.map(|gds| {
            SaveGroundwaterDepletionState {
                extraction_rate: gds.extraction_rate,
                recharge_rate: gds.recharge_rate,
                sustainability_ratio: gds.sustainability_ratio,
                critical_depletion: gds.critical_depletion,
                subsidence_cells: gds.subsidence_cells,
                well_yield_modifier: gds.well_yield_modifier,
                ticks_below_threshold: gds.ticks_below_threshold.clone(),
                previous_levels: gds.previous_levels.clone(),
                recharge_basin_count: gds.recharge_basin_count,
                avg_groundwater_level: gds.avg_groundwater_level,
                cells_at_risk: gds.cells_at_risk,
                over_extracted_cells: gds.over_extracted_cells,
            }
        }),
        water_treatment_state: water_treatment_state.map(|wts| SaveWaterTreatmentState {
            plants: wts
                .plants
                .values()
                .map(|p| SavePlantState {
                    level: treatment_level_to_u8(p.level),
                    capacity_mgd: p.capacity_mgd,
                    current_flow_mgd: p.current_flow_mgd,
                    effluent_quality: p.effluent_quality,
                    period_cost: p.period_cost,
                })
                .collect(),
            total_capacity_mgd: wts.total_capacity_mgd,
            total_flow_mgd: wts.total_flow_mgd,
            avg_effluent_quality: wts.avg_effluent_quality,
            total_period_cost: wts.total_period_cost,
            city_demand_mgd: wts.city_demand_mgd,
            treatment_coverage: wts.treatment_coverage,
            avg_input_quality: wts.avg_input_quality,
            disease_risk: wts.disease_risk,
        }),
        water_conservation_state: water_conservation_state.map(|s| SaveWaterConservationState {
            low_flow_fixtures: s.low_flow_fixtures,
            xeriscaping: s.xeriscaping,
            tiered_pricing: s.tiered_pricing,
            greywater_recycling: s.greywater_recycling,
            rainwater_harvesting: s.rainwater_harvesting,
            demand_reduction_pct: s.demand_reduction_pct,
            sewage_reduction_pct: s.sewage_reduction_pct,
            total_retrofit_cost: s.total_retrofit_cost,
            annual_savings_gallons: s.annual_savings_gallons,
            buildings_retrofitted: s.buildings_retrofitted,
        }),
    }
}
