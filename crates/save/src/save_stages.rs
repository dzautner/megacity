// ---------------------------------------------------------------------------
// Save Stages: typed intermediate structures for the staged save pipeline
// ---------------------------------------------------------------------------
//
// The save pipeline is split into focused stages, each responsible for
// collecting one domain of game state into `SaveData` fields. This avoids a
// single God-function with 40+ parameters and makes it safe to evolve each
// domain independently.
//
// ## Pipeline overview
//
// ```text
//   ECS World
//     |
//     +-- collect_grid_stage        -> GridStageOutput       (grid, roads, road_segments)
//     +-- collect_economy_stage     -> EconomyStageOutput    (clock, budget, demand, ext_budget, loans)
//     +-- collect_entity_stage      -> EntityStageOutput     (buildings, citizens, utilities, services, water_sources)
//     +-- collect_environment_stage -> EnvironmentStageOutput (weather, climate, UHI, stormwater, snow, ...)
//     +-- collect_disaster_stage    -> DisasterStageOutput   (drought, heat_wave, cold_snap, flood, ...)
//     +-- collect_policy_stage      -> PolicyStageOutput     (policies, unlocks, recycling, composting, ...)
//     |
//     +---> assemble_save_data(stages...) -> SaveData
// ```
//
// Each `collect_*` function takes only the references it needs, keeping call
// sites clean and type-safe.

use crate::save_codec::*;
use crate::save_types::*;

use bevy::prelude::Entity;
use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::{CitizenState, Gender};
use simulation::cold_snap::ColdSnapState;
use simulation::cso::SewerSystemState;
use simulation::degree_days::DegreeDays;
use simulation::drought::DroughtState;
use simulation::economy::CityBudget;
use simulation::flood_simulation::FloodState;
use simulation::fog::FogState;
use simulation::grid::WorldGrid;
use simulation::groundwater_depletion::GroundwaterDepletionState;
use simulation::hazardous_waste::HazardousWasteState;
use simulation::heat_wave::HeatWaveState;
use simulation::landfill_gas::LandfillGasState;
use simulation::landfill_warning::LandfillCapacityState;
use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::LoanBook;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
use simulation::snow::{SnowGrid, SnowPlowingState};
use simulation::storm_drainage::StormDrainageState;
use simulation::stormwater::StormwaterGrid;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::utilities::UtilitySource;
use simulation::virtual_population::VirtualPopulation;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_sources::WaterSource;
use simulation::water_treatment::WaterTreatmentState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;
use simulation::zones::ZoneDemand;

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Stage output types
// ---------------------------------------------------------------------------

/// Grid, road network, and road segment data.
pub struct GridStageOutput {
    pub grid: SaveGrid,
    pub roads: SaveRoadNetwork,
    pub road_segments: Option<SaveRoadSegmentStore>,
}

/// Economy-related: clock, budget, demand, extended budget, loans.
pub struct EconomyStageOutput {
    pub clock: SaveClock,
    pub budget: SaveBudget,
    pub demand: SaveDemand,
    pub extended_budget: Option<SaveExtendedBudget>,
    pub loan_book: Option<SaveLoanBook>,
}

/// Spawned entities: buildings, citizens, utilities, services, water sources.
pub struct EntityStageOutput {
    pub buildings: Vec<SaveBuilding>,
    pub citizens: Vec<SaveCitizen>,
    pub utility_sources: Vec<SaveUtilitySource>,
    pub service_buildings: Vec<SaveServiceBuilding>,
    pub water_sources: Option<Vec<SaveWaterSource>>,
}

/// Environment state: weather, climate, UHI, stormwater, snow, agriculture,
/// fog, degree days, construction modifiers, urban growth boundary.
pub struct EnvironmentStageOutput {
    pub weather: Option<SaveWeather>,
    pub uhi_grid: Option<SaveUhiGrid>,
    pub stormwater_grid: Option<SaveStormwaterGrid>,
    pub degree_days: Option<SaveDegreeDays>,
    pub construction_modifiers: Option<SaveConstructionModifiers>,
    pub snow_state: Option<SaveSnowState>,
    pub agriculture_state: Option<SaveAgricultureState>,
    pub fog_state: Option<SaveFogState>,
    pub urban_growth_boundary: Option<SaveUrbanGrowthBoundary>,
}

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

/// Policy / progression state: policies, unlocks, recycling, composting,
/// lifecycle timer, life sim timer, virtual population.
pub struct PolicyStageOutput {
    pub policies: Option<SavePolicies>,
    pub unlock_state: Option<SaveUnlockState>,
    pub recycling_state: Option<SaveRecyclingState>,
    pub composting_state: Option<SaveCompostingState>,
    pub lifecycle_timer: Option<SaveLifecycleTimer>,
    pub life_sim_timer: Option<SaveLifeSimTimer>,
    pub virtual_population: Option<SaveVirtualPopulation>,
}

// ---------------------------------------------------------------------------
// Stage collection functions
// ---------------------------------------------------------------------------

/// Collect grid, road network, and road segment data.
pub fn collect_grid_stage(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    segment_store: Option<&RoadSegmentStore>,
) -> GridStageOutput {
    let save_cells: Vec<SaveCell> = grid
        .cells
        .iter()
        .map(|c| SaveCell {
            elevation: c.elevation,
            cell_type: match c.cell_type {
                simulation::grid::CellType::Grass => 0,
                simulation::grid::CellType::Water => 1,
                simulation::grid::CellType::Road => 2,
            },
            zone: zone_type_to_u8(c.zone),
            road_type: road_type_to_u8(c.road_type),
            has_power: c.has_power,
            has_water: c.has_water,
        })
        .collect();

    GridStageOutput {
        grid: SaveGrid {
            cells: save_cells,
            width: grid.width,
            height: grid.height,
        },
        roads: SaveRoadNetwork {
            road_positions: roads.edges.keys().map(|n| (n.0, n.1)).collect(),
        },
        road_segments: segment_store.map(|store| SaveRoadSegmentStore {
            nodes: store
                .nodes
                .iter()
                .map(|n| SaveSegmentNode {
                    id: n.id.0,
                    x: n.position.x,
                    y: n.position.y,
                    connected_segments: n.connected_segments.iter().map(|s| s.0).collect(),
                })
                .collect(),
            segments: store
                .segments
                .iter()
                .map(|s| SaveRoadSegment {
                    id: s.id.0,
                    start_node: s.start_node.0,
                    end_node: s.end_node.0,
                    p0_x: s.p0.x,
                    p0_y: s.p0.y,
                    p1_x: s.p1.x,
                    p1_y: s.p1.y,
                    p2_x: s.p2.x,
                    p2_y: s.p2.y,
                    p3_x: s.p3.x,
                    p3_y: s.p3.y,
                    road_type: road_type_to_u8(s.road_type),
                })
                .collect(),
        }),
    }
}

/// Collect economy-related data: clock, budget, demand, extended budget, loans.
pub fn collect_economy_stage(
    clock: &GameClock,
    budget: &CityBudget,
    demand: &ZoneDemand,
    extended_budget: Option<&ExtendedBudget>,
    loan_book: Option<&LoanBook>,
) -> EconomyStageOutput {
    EconomyStageOutput {
        clock: SaveClock {
            day: clock.day,
            hour: clock.hour,
            speed: clock.speed,
        },
        budget: SaveBudget {
            treasury: budget.treasury,
            tax_rate: budget.tax_rate,
            last_collection_day: budget.last_collection_day,
        },
        demand: SaveDemand {
            residential: demand.residential,
            commercial: demand.commercial,
            industrial: demand.industrial,
            office: demand.office,
            vacancy_residential: demand.vacancy_residential,
            vacancy_commercial: demand.vacancy_commercial,
            vacancy_industrial: demand.vacancy_industrial,
            vacancy_office: demand.vacancy_office,
        },
        extended_budget: extended_budget.map(|eb| SaveExtendedBudget {
            residential_tax: eb.zone_taxes.residential,
            commercial_tax: eb.zone_taxes.commercial,
            industrial_tax: eb.zone_taxes.industrial,
            office_tax: eb.zone_taxes.office,
            fire_budget: eb.service_budgets.fire,
            police_budget: eb.service_budgets.police,
            healthcare_budget: eb.service_budgets.healthcare,
            education_budget: eb.service_budgets.education,
            sanitation_budget: eb.service_budgets.sanitation,
            transport_budget: eb.service_budgets.transport,
        }),
        loan_book: loan_book.map(|lb| SaveLoanBook {
            loans: lb
                .active_loans
                .iter()
                .map(|l| SaveLoan {
                    name: l.name.clone(),
                    amount: l.amount,
                    interest_rate: l.interest_rate,
                    monthly_payment: l.monthly_payment,
                    remaining_balance: l.remaining_balance,
                    term_months: l.term_months,
                    months_paid: l.months_paid,
                })
                .collect(),
            max_loans: lb.max_loans as u32,
            credit_rating: lb.credit_rating,
            last_payment_day: lb.last_payment_day,
            consecutive_solvent_days: lb.consecutive_solvent_days,
        }),
    }
}

/// Collect entity data: buildings, citizens, utilities, services, water sources.
pub fn collect_entity_stage(
    buildings: &[(Building, Option<MixedUseBuilding>)],
    citizens: &[CitizenSaveInput],
    utility_sources: &[UtilitySource],
    service_buildings: &[(ServiceBuilding,)],
    water_sources: Option<&[WaterSource]>,
) -> EntityStageOutput {
    // Build Entity -> citizen-array-index map for family reference serialization
    let entity_to_idx: HashMap<Entity, u32> = citizens
        .iter()
        .enumerate()
        .map(|(i, c)| (c.entity, i as u32))
        .collect();

    EntityStageOutput {
        buildings: buildings
            .iter()
            .map(|(b, mu)| SaveBuilding {
                zone_type: zone_type_to_u8(b.zone_type),
                level: b.level,
                grid_x: b.grid_x,
                grid_y: b.grid_y,
                capacity: b.capacity,
                occupants: b.occupants,
                commercial_capacity: mu.as_ref().map_or(0, |m| m.commercial_capacity),
                commercial_occupants: mu.as_ref().map_or(0, |m| m.commercial_occupants),
                residential_capacity: mu.as_ref().map_or(0, |m| m.residential_capacity),
                residential_occupants: mu.as_ref().map_or(0, |m| m.residential_occupants),
            })
            .collect(),
        citizens: citizens
            .iter()
            .map(|c| SaveCitizen {
                age: c.details.age,
                happiness: c.details.happiness,
                education: c.details.education,
                state: match c.state {
                    CitizenState::AtHome => 0,
                    CitizenState::CommutingToWork => 1,
                    CitizenState::Working => 2,
                    CitizenState::CommutingHome => 3,
                    CitizenState::CommutingToShop => 4,
                    CitizenState::Shopping => 5,
                    CitizenState::CommutingToLeisure => 6,
                    CitizenState::AtLeisure => 7,
                    CitizenState::CommutingToSchool => 8,
                    CitizenState::AtSchool => 9,
                },
                home_x: c.home_x,
                home_y: c.home_y,
                work_x: c.work_x,
                work_y: c.work_y,
                path_waypoints: c.path.waypoints.iter().map(|n| (n.0, n.1)).collect(),
                path_current_index: c.path.current_index,
                velocity_x: c.velocity.x,
                velocity_y: c.velocity.y,
                pos_x: c.position.x,
                pos_y: c.position.y,
                gender: match c.details.gender {
                    Gender::Male => 0,
                    Gender::Female => 1,
                },
                health: c.details.health,
                salary: c.details.salary,
                savings: c.details.savings,
                ambition: c.personality.ambition,
                sociability: c.personality.sociability,
                materialism: c.personality.materialism,
                resilience: c.personality.resilience,
                need_hunger: c.needs.hunger,
                need_energy: c.needs.energy,
                need_social: c.needs.social,
                need_fun: c.needs.fun,
                need_comfort: c.needs.comfort,
                activity_timer: c.activity_timer,
                family_partner: c
                    .family
                    .partner
                    .and_then(|e| entity_to_idx.get(&e).copied())
                    .unwrap_or(u32::MAX),
                family_children: c
                    .family
                    .children
                    .iter()
                    .filter_map(|e| entity_to_idx.get(e).copied())
                    .collect(),
                family_parent: c
                    .family
                    .parent
                    .and_then(|e| entity_to_idx.get(&e).copied())
                    .unwrap_or(u32::MAX),
            })
            .collect(),
        utility_sources: utility_sources
            .iter()
            .map(|u| SaveUtilitySource {
                utility_type: utility_type_to_u8(u.utility_type),
                grid_x: u.grid_x,
                grid_y: u.grid_y,
                range: u.range,
            })
            .collect(),
        service_buildings: service_buildings
            .iter()
            .map(|(sb,)| SaveServiceBuilding {
                service_type: service_type_to_u8(sb.service_type),
                grid_x: sb.grid_x,
                grid_y: sb.grid_y,
                radius_cells: (sb.radius / simulation::config::CELL_SIZE) as u32,
            })
            .collect(),
        water_sources: water_sources.map(|ws| {
            ws.iter()
                .map(|s| SaveWaterSource {
                    source_type: water_source_type_to_u8(s.source_type),
                    grid_x: s.grid_x,
                    grid_y: s.grid_y,
                    capacity_mgd: s.capacity_mgd,
                    quality: s.quality,
                    operating_cost: s.operating_cost,
                    stored_gallons: s.stored_gallons,
                    storage_capacity: s.storage_capacity,
                })
                .collect()
        }),
    }
}

/// Collect environment state: weather, UHI, stormwater, degree days,
/// construction modifiers, snow, agriculture, fog, urban growth boundary.
#[allow(clippy::too_many_arguments)]
pub fn collect_environment_stage(
    weather: Option<&Weather>,
    climate_zone: Option<&ClimateZone>,
    uhi_grid: Option<&UhiGrid>,
    stormwater_grid: Option<&StormwaterGrid>,
    degree_days: Option<&DegreeDays>,
    construction_modifiers: Option<&ConstructionModifiers>,
    snow_state: Option<(&SnowGrid, &SnowPlowingState)>,
    agriculture_state: Option<&AgricultureState>,
    fog_state: Option<&FogState>,
    urban_growth_boundary: Option<&UrbanGrowthBoundary>,
) -> EnvironmentStageOutput {
    EnvironmentStageOutput {
        weather: weather.map(|w| SaveWeather {
            season: season_to_u8(w.season),
            temperature: w.temperature,
            current_event: weather_event_to_u8(w.current_event),
            event_days_remaining: w.event_days_remaining,
            last_update_day: w.last_update_day,
            disasters_enabled: w.disasters_enabled,
            humidity: w.humidity,
            cloud_cover: w.cloud_cover,
            precipitation_intensity: w.precipitation_intensity,
            last_update_hour: w.last_update_hour,
            climate_zone: climate_zone.map(|cz| climate_zone_to_u8(*cz)).unwrap_or(0),
        }),
        uhi_grid: uhi_grid.map(|ug| SaveUhiGrid {
            cells: ug.cells.clone(),
            width: ug.width,
            height: ug.height,
        }),
        stormwater_grid: stormwater_grid.map(|sw| SaveStormwaterGrid {
            runoff: sw.runoff.clone(),
            total_runoff: sw.total_runoff,
            total_infiltration: sw.total_infiltration,
            width: sw.width,
            height: sw.height,
        }),
        degree_days: degree_days.map(|dd| SaveDegreeDays {
            daily_hdd: dd.daily_hdd,
            daily_cdd: dd.daily_cdd,
            monthly_hdd: dd.monthly_hdd,
            monthly_cdd: dd.monthly_cdd,
            annual_hdd: dd.annual_hdd,
            annual_cdd: dd.annual_cdd,
            last_update_day: dd.last_update_day,
        }),
        construction_modifiers: construction_modifiers.map(|cm| SaveConstructionModifiers {
            speed_factor: cm.speed_factor,
            cost_factor: cm.cost_factor,
        }),
        snow_state: snow_state.map(|(sg, sp)| SaveSnowState {
            depths: sg.depths.clone(),
            width: sg.width,
            height: sg.height,
            plowing_enabled: sp.enabled,
            season_cost: sp.season_cost,
            cells_plowed_season: sp.cells_plowed_season,
        }),
        agriculture_state: agriculture_state.map(|a| SaveAgricultureState {
            growing_season_active: a.growing_season_active,
            crop_yield_modifier: a.crop_yield_modifier,
            rainfall_adequacy: a.rainfall_adequacy,
            temperature_suitability: a.temperature_suitability,
            soil_quality: a.soil_quality,
            fertilizer_bonus: a.fertilizer_bonus,
            frost_risk: a.frost_risk,
            frost_events_this_year: a.frost_events_this_year,
            frost_damage_total: a.frost_damage_total,
            has_irrigation: a.has_irrigation,
            farm_count: a.farm_count,
            annual_rainfall_estimate: a.annual_rainfall_estimate,
            last_frost_check_day: a.last_frost_check_day,
            last_rainfall_day: a.last_rainfall_day,
        }),
        fog_state: fog_state.map(|s| SaveFogState {
            active: s.active,
            density: fog_density_to_u8(s.density),
            visibility_m: s.visibility_m,
            hours_active: s.hours_active,
            max_duration_hours: s.max_duration_hours,
            water_fraction: s.water_fraction,
            traffic_speed_modifier: s.traffic_speed_modifier,
            flights_suspended: s.flights_suspended,
            last_update_hour: s.last_update_hour,
        }),
        urban_growth_boundary: urban_growth_boundary.map(|u| SaveUrbanGrowthBoundary {
            enabled: u.enabled,
            vertices_x: u.vertices.iter().map(|(x, _)| *x).collect(),
            vertices_y: u.vertices.iter().map(|(_, y)| *y).collect(),
        }),
    }
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

/// Collect policy / progression state.
pub fn collect_policy_stage(
    policies: Option<&Policies>,
    unlock_state: Option<&UnlockState>,
    recycling_state: Option<(&RecyclingState, &RecyclingEconomics)>,
    composting_state: Option<&simulation::composting::CompostingState>,
    lifecycle_timer: Option<&LifecycleTimer>,
    life_sim_timer: Option<&LifeSimTimer>,
    virtual_population: Option<&VirtualPopulation>,
) -> PolicyStageOutput {
    PolicyStageOutput {
        policies: policies.map(|p| SavePolicies {
            active: p.active.iter().map(|&pol| policy_to_u8(pol)).collect(),
        }),
        unlock_state: unlock_state.map(|u| SaveUnlockState {
            development_points: u.development_points,
            spent_points: u.spent_points,
            unlocked_nodes: u
                .unlocked_nodes
                .iter()
                .map(|&n| unlock_node_to_u8(n))
                .collect(),
            last_milestone_pop: u.last_milestone_pop,
        }),
        recycling_state: recycling_state.map(|(rs, re)| SaveRecyclingState {
            tier: recycling_tier_to_u8(rs.tier),
            daily_tons_diverted: rs.daily_tons_diverted,
            daily_tons_contaminated: rs.daily_tons_contaminated,
            daily_revenue: rs.daily_revenue,
            daily_cost: rs.daily_cost,
            total_revenue: rs.total_revenue,
            total_cost: rs.total_cost,
            participating_households: rs.participating_households,
            price_paper: re.price_paper,
            price_plastic: re.price_plastic,
            price_glass: re.price_glass,
            price_metal: re.price_metal,
            price_organic: re.price_organic,
            market_cycle_position: re.market_cycle_position,
            economics_last_update_day: re.last_update_day,
        }),
        composting_state: composting_state.map(|cs| SaveCompostingState {
            facilities: cs
                .facilities
                .iter()
                .map(|f| SaveCompostFacility {
                    method: compost_method_to_u8(f.method),
                    capacity_tons_per_day: f.capacity_tons_per_day,
                    cost_per_ton: f.cost_per_ton,
                    tons_processed_today: f.tons_processed_today,
                })
                .collect(),
            participation_rate: cs.participation_rate,
            organic_fraction: cs.organic_fraction,
            total_diverted_tons: cs.total_diverted_tons,
            daily_diversion_tons: cs.daily_diversion_tons,
            compost_revenue_per_ton: cs.compost_revenue_per_ton,
            daily_revenue: cs.daily_revenue,
            biogas_mwh_per_ton: cs.biogas_mwh_per_ton,
            daily_biogas_mwh: cs.daily_biogas_mwh,
        }),
        lifecycle_timer: lifecycle_timer.map(|lt| SaveLifecycleTimer {
            last_aging_day: lt.last_aging_day,
            last_emigration_tick: lt.last_emigration_tick,
        }),
        life_sim_timer: life_sim_timer.map(|lst| SaveLifeSimTimer {
            needs_tick: lst.needs_tick,
            life_event_tick: lst.life_event_tick,
            salary_tick: lst.salary_tick,
            education_tick: lst.education_tick,
            job_seek_tick: lst.job_seek_tick,
            personality_tick: lst.personality_tick,
            health_tick: lst.health_tick,
        }),
        virtual_population: virtual_population.map(|vp| SaveVirtualPopulation {
            total_virtual: vp.total_virtual,
            virtual_employed: vp.virtual_employed,
            district_stats: vp
                .district_stats
                .iter()
                .map(|ds| SaveDistrictStats {
                    population: ds.population,
                    employed: ds.employed,
                    avg_happiness: ds.avg_happiness,
                    avg_age: ds.avg_age,
                    age_brackets: ds.age_brackets,
                    commuters_out: ds.commuters_out,
                    tax_contribution: ds.tax_contribution,
                    service_demand: ds.service_demand,
                })
                .collect(),
            max_real_citizens: vp.max_real_citizens,
        }),
    }
}

// ---------------------------------------------------------------------------
// Assembly: compose SaveData from stage outputs
// ---------------------------------------------------------------------------

/// Assemble a complete `SaveData` from the outputs of all collection stages.
///
/// Extensions are left empty -- they are populated separately by the save
/// system via `SaveableRegistry`.
pub fn assemble_save_data(
    grid_stage: GridStageOutput,
    economy_stage: EconomyStageOutput,
    entity_stage: EntityStageOutput,
    environment_stage: EnvironmentStageOutput,
    disaster_stage: DisasterStageOutput,
    policy_stage: PolicyStageOutput,
) -> SaveData {
    SaveData {
        version: CURRENT_SAVE_VERSION,
        grid: grid_stage.grid,
        roads: grid_stage.roads,
        road_segments: grid_stage.road_segments,
        clock: economy_stage.clock,
        budget: economy_stage.budget,
        demand: economy_stage.demand,
        extended_budget: economy_stage.extended_budget,
        loan_book: economy_stage.loan_book,
        buildings: entity_stage.buildings,
        citizens: entity_stage.citizens,
        utility_sources: entity_stage.utility_sources,
        service_buildings: entity_stage.service_buildings,
        water_sources: entity_stage.water_sources,
        weather: environment_stage.weather,
        uhi_grid: environment_stage.uhi_grid,
        stormwater_grid: environment_stage.stormwater_grid,
        degree_days: environment_stage.degree_days,
        construction_modifiers: environment_stage.construction_modifiers,
        snow_state: environment_stage.snow_state,
        agriculture_state: environment_stage.agriculture_state,
        fog_state: environment_stage.fog_state,
        urban_growth_boundary: environment_stage.urban_growth_boundary,
        drought_state: disaster_stage.drought_state,
        heat_wave_state: disaster_stage.heat_wave_state,
        cold_snap_state: disaster_stage.cold_snap_state,
        flood_state: disaster_stage.flood_state,
        wind_damage_state: disaster_stage.wind_damage_state,
        reservoir_state: disaster_stage.reservoir_state,
        landfill_gas_state: disaster_stage.landfill_gas_state,
        cso_state: disaster_stage.cso_state,
        hazardous_waste_state: disaster_stage.hazardous_waste_state,
        wastewater_state: disaster_stage.wastewater_state,
        storm_drainage_state: disaster_stage.storm_drainage_state,
        landfill_capacity_state: disaster_stage.landfill_capacity_state,
        groundwater_depletion_state: disaster_stage.groundwater_depletion_state,
        water_treatment_state: disaster_stage.water_treatment_state,
        water_conservation_state: disaster_stage.water_conservation_state,
        policies: policy_stage.policies,
        unlock_state: policy_stage.unlock_state,
        recycling_state: policy_stage.recycling_state,
        composting_state: policy_stage.composting_state,
        lifecycle_timer: policy_stage.lifecycle_timer,
        life_sim_timer: policy_stage.life_sim_timer,
        virtual_population: policy_stage.virtual_population,
        extensions: std::collections::BTreeMap::new(),
    }
}
