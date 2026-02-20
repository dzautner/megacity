// ---------------------------------------------------------------------------
// Restore functions: reconstruct simulation resources from save structs
// ---------------------------------------------------------------------------

use crate::save_codec::*;
use crate::save_types::*;

use simulation::budget::{ExtendedBudget, ServiceBudgets, ZoneTaxRates};
use simulation::degree_days::DegreeDays;
use simulation::flood_simulation::FloodState;
use simulation::landfill_gas::LandfillGasState;
use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::{self, LoanBook};
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::road_segments::{
    RoadSegment, RoadSegmentStore, SegmentId, SegmentNode, SegmentNodeId,
};
use simulation::stormwater::StormwaterGrid;
use simulation::unlocks::UnlockState;
use simulation::urban_heat_island::UhiGrid;
use simulation::virtual_population::{DistrictStats, VirtualPopulation};
use simulation::water_sources::WaterSource;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;

/// Reconstruct a `RoadSegmentStore` from saved data.
/// After calling this, call `store.rasterize_all(grid, roads)` to rebuild grid cells.
pub fn restore_road_segment_store(save: &SaveRoadSegmentStore) -> RoadSegmentStore {
    use bevy::math::Vec2;

    let nodes: Vec<SegmentNode> = save
        .nodes
        .iter()
        .map(|n| SegmentNode {
            id: SegmentNodeId(n.id),
            position: Vec2::new(n.x, n.y),
            connected_segments: n.connected_segments.iter().map(|&s| SegmentId(s)).collect(),
        })
        .collect();

    let segments: Vec<RoadSegment> = save
        .segments
        .iter()
        .map(|s| RoadSegment {
            id: SegmentId(s.id),
            start_node: SegmentNodeId(s.start_node),
            end_node: SegmentNodeId(s.end_node),
            p0: Vec2::new(s.p0_x, s.p0_y),
            p1: Vec2::new(s.p1_x, s.p1_y),
            p2: Vec2::new(s.p2_x, s.p2_y),
            p3: Vec2::new(s.p3_x, s.p3_y),
            road_type: u8_to_road_type(s.road_type),
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        })
        .collect();

    RoadSegmentStore::from_parts(nodes, segments)
}

/// Restore a `Policies` resource from saved data.
pub fn restore_policies(save: &SavePolicies) -> Policies {
    let active = save
        .active
        .iter()
        .filter_map(|&v| u8_to_policy(v))
        .collect();
    Policies { active }
}

/// Restore a `Weather` resource from saved data.
pub fn restore_weather(save: &SaveWeather) -> Weather {
    Weather {
        season: u8_to_season(save.season),
        temperature: save.temperature,
        current_event: u8_to_weather_event(save.current_event),
        event_days_remaining: save.event_days_remaining,
        last_update_day: save.last_update_day,
        disasters_enabled: save.disasters_enabled,
        humidity: save.humidity,
        cloud_cover: save.cloud_cover,
        precipitation_intensity: save.precipitation_intensity,
        last_update_hour: save.last_update_hour,
        prev_extreme: false,
        ..Default::default()
    }
}

/// Restore a `ClimateZone` resource from saved weather data.
pub fn restore_climate_zone(save: &SaveWeather) -> ClimateZone {
    u8_to_climate_zone(save.climate_zone)
}

/// Restore an `UnlockState` resource from saved data.
pub fn restore_unlock_state(save: &SaveUnlockState) -> UnlockState {
    let unlocked_nodes = save
        .unlocked_nodes
        .iter()
        .filter_map(|&v| u8_to_unlock_node(v))
        .collect();
    UnlockState {
        development_points: save.development_points,
        spent_points: save.spent_points,
        unlocked_nodes,
        last_milestone_pop: save.last_milestone_pop,
    }
}

/// Restore an `ExtendedBudget` resource from saved data.
pub fn restore_extended_budget(save: &SaveExtendedBudget) -> ExtendedBudget {
    ExtendedBudget {
        zone_taxes: ZoneTaxRates {
            residential: save.residential_tax,
            commercial: save.commercial_tax,
            industrial: save.industrial_tax,
            office: save.office_tax,
        },
        service_budgets: ServiceBudgets {
            fire: save.fire_budget,
            police: save.police_budget,
            healthcare: save.healthcare_budget,
            education: save.education_budget,
            sanitation: save.sanitation_budget,
            transport: save.transport_budget,
        },
        // Loans are stored separately in the LoanBook (budget.rs loans are legacy);
        // leave the ExtendedBudget.loans empty.
        loans: Vec::new(),
        income_breakdown: Default::default(),
        expense_breakdown: Default::default(),
    }
}

/// Restore a `LoanBook` resource from saved data.
pub fn restore_loan_book(save: &SaveLoanBook) -> LoanBook {
    let active_loans = save
        .loans
        .iter()
        .map(|sl| loans::Loan {
            name: sl.name.clone(),
            amount: sl.amount,
            interest_rate: sl.interest_rate,
            monthly_payment: sl.monthly_payment,
            remaining_balance: sl.remaining_balance,
            term_months: sl.term_months,
            months_paid: sl.months_paid,
        })
        .collect();
    LoanBook {
        active_loans,
        max_loans: save.max_loans as usize,
        credit_rating: save.credit_rating,
        last_payment_day: save.last_payment_day,
        consecutive_solvent_days: save.consecutive_solvent_days,
    }
}

/// Restore a `LifecycleTimer` resource from saved data.
pub fn restore_lifecycle_timer(save: &SaveLifecycleTimer) -> LifecycleTimer {
    LifecycleTimer {
        last_aging_day: save.last_aging_day,
        last_emigration_tick: save.last_emigration_tick,
    }
}

/// Restore a `LifeSimTimer` resource from saved data.
pub fn restore_life_sim_timer(save: &SaveLifeSimTimer) -> LifeSimTimer {
    LifeSimTimer {
        needs_tick: save.needs_tick,
        life_event_tick: save.life_event_tick,
        salary_tick: save.salary_tick,
        education_tick: save.education_tick,
        job_seek_tick: save.job_seek_tick,
        personality_tick: save.personality_tick,
        health_tick: save.health_tick,
    }
}

/// Restore a `StormwaterGrid` resource from saved data.
pub fn restore_stormwater_grid(save: &SaveStormwaterGrid) -> StormwaterGrid {
    StormwaterGrid {
        runoff: save.runoff.clone(),
        total_runoff: save.total_runoff,
        total_infiltration: save.total_infiltration,
        width: save.width,
        height: save.height,
    }
}

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

/// Restore a `DegreeDays` resource from saved data.
pub fn restore_degree_days(save: &SaveDegreeDays) -> DegreeDays {
    DegreeDays {
        daily_hdd: save.daily_hdd,
        daily_cdd: save.daily_cdd,
        monthly_hdd: save.monthly_hdd,
        monthly_cdd: save.monthly_cdd,
        annual_hdd: save.annual_hdd,
        annual_cdd: save.annual_cdd,
        last_update_day: save.last_update_day,
    }
}

/// Restore a `ConstructionModifiers` resource from saved data.
pub fn restore_construction_modifiers(save: &SaveConstructionModifiers) -> ConstructionModifiers {
    ConstructionModifiers {
        speed_factor: save.speed_factor,
        cost_factor: save.cost_factor,
    }
}

/// Restore a `VirtualPopulation` resource from saved data.
pub fn restore_virtual_population(save: &SaveVirtualPopulation) -> VirtualPopulation {
    let district_stats = save
        .district_stats
        .iter()
        .map(|ds| DistrictStats {
            population: ds.population,
            employed: ds.employed,
            avg_happiness: ds.avg_happiness,
            avg_age: ds.avg_age,
            age_brackets: ds.age_brackets,
            commuters_out: ds.commuters_out,
            tax_contribution: ds.tax_contribution,
            service_demand: ds.service_demand,
        })
        .collect();
    VirtualPopulation::from_saved(
        save.total_virtual,
        save.virtual_employed,
        district_stats,
        save.max_real_citizens,
    )
}

/// Restore a `WindDamageState` resource from saved data.
pub fn restore_wind_damage_state(save: &SaveWindDamageState) -> WindDamageState {
    WindDamageState {
        current_tier: u8_to_wind_damage_tier(save.current_tier),
        accumulated_building_damage: save.accumulated_building_damage,
        trees_knocked_down: save.trees_knocked_down,
        power_outage_active: save.power_outage_active,
    }
}

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

/// Restore a `DroughtState` resource from saved data.
pub fn restore_drought(save: &SaveDroughtState) -> simulation::drought::DroughtState {
    simulation::drought::DroughtState {
        rainfall_history: save.rainfall_history.clone(),
        current_index: save.current_index,
        current_tier: u8_to_drought_tier(save.current_tier),
        expected_daily_rainfall: save.expected_daily_rainfall,
        water_demand_modifier: save.water_demand_modifier,
        agriculture_modifier: save.agriculture_modifier,
        fire_risk_multiplier: save.fire_risk_multiplier,
        happiness_modifier: save.happiness_modifier,
        last_record_day: save.last_record_day,
    }
}

/// Restore a `HeatWaveState` resource from saved data.
pub fn restore_heat_wave(save: &SaveHeatWaveState) -> simulation::heat_wave::HeatWaveState {
    simulation::heat_wave::HeatWaveState {
        consecutive_hot_days: save.consecutive_hot_days,
        severity: u8_to_heat_wave_severity(save.severity),
        excess_mortality_per_100k: save.excess_mortality_per_100k,
        energy_demand_multiplier: save.energy_demand_multiplier,
        water_demand_multiplier: save.water_demand_multiplier,
        road_damage_active: save.road_damage_active,
        fire_risk_multiplier: save.fire_risk_multiplier,
        blackout_risk: save.blackout_risk,
        heat_threshold_c: save.heat_threshold_c,
        consecutive_extreme_days: save.consecutive_extreme_days,
        last_check_day: save.last_check_day,
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

/// Restore a `ColdSnapState` resource from saved data.
pub fn restore_cold_snap(
    save: &crate::save_types::SaveColdSnapState,
) -> simulation::cold_snap::ColdSnapState {
    simulation::cold_snap::ColdSnapState {
        consecutive_cold_days: save.consecutive_cold_days,
        pipe_burst_count: save.pipe_burst_count,
        is_active: save.is_active,
        current_tier: u8_to_cold_snap_tier(save.current_tier),
        heating_demand_modifier: save.heating_demand_modifier,
        traffic_capacity_modifier: save.traffic_capacity_modifier,
        schools_closed: save.schools_closed,
        construction_halted: save.construction_halted,
        homeless_mortality_rate: save.homeless_mortality_rate,
        water_service_modifier: save.water_service_modifier,
        last_check_day: save.last_check_day,
    }
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

/// Restore a `StormDrainageState` resource from saved data.
pub fn restore_storm_drainage(
    save: &crate::save_types::SaveStormDrainageState,
) -> simulation::storm_drainage::StormDrainageState {
    simulation::storm_drainage::StormDrainageState {
        total_drain_capacity: save.total_drain_capacity,
        total_retention_capacity: save.total_retention_capacity,
        current_retention_stored: save.current_retention_stored,
        drain_count: save.drain_count,
        retention_pond_count: save.retention_pond_count,
        rain_garden_count: save.rain_garden_count,
        overflow_cells: save.overflow_cells,
        drainage_coverage: save.drainage_coverage,
    }
}

/// Restore a `FloodState` resource from saved data.
pub fn restore_flood_state(save: &SaveFloodState) -> FloodState {
    FloodState {
        is_flooding: save.is_flooding,
        total_flooded_cells: save.total_flooded_cells,
        total_damage: save.total_damage,
        max_depth: save.max_depth,
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
