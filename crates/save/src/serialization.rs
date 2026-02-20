// ---------------------------------------------------------------------------
// Serialization: re-exports from sub-modules + create_save_data + tests
// ---------------------------------------------------------------------------

// Re-export everything from sub-modules so existing `use crate::serialization::*` works.
pub use crate::save_codec::*;
pub use crate::save_migrate::*;
pub use crate::save_restore::*;
pub use crate::save_types::*;

use simulation::agriculture::AgricultureState;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::CitizenState;
use simulation::cso::SewerSystemState;
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
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
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

use simulation::budget::ExtendedBudget;
use simulation::cold_snap::ColdSnapState;
use simulation::degree_days::DegreeDays;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::snow::{SnowGrid, SnowPlowingState};

#[allow(clippy::too_many_arguments)]
pub fn create_save_data(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    clock: &GameClock,
    budget: &CityBudget,
    demand: &ZoneDemand,
    buildings: &[(Building, Option<MixedUseBuilding>)],
    citizens: &[CitizenSaveInput],
    utility_sources: &[UtilitySource],
    service_buildings: &[(ServiceBuilding,)],
    segment_store: Option<&RoadSegmentStore>,
    policies: Option<&Policies>,
    weather: Option<&Weather>,
    unlock_state: Option<&UnlockState>,
    extended_budget: Option<&ExtendedBudget>,
    loan_book: Option<&LoanBook>,
    lifecycle_timer: Option<&LifecycleTimer>,
    virtual_population: Option<&VirtualPopulation>,
    life_sim_timer: Option<&LifeSimTimer>,
    stormwater_grid: Option<&StormwaterGrid>,
    water_sources: Option<&[WaterSource]>,
    degree_days: Option<&DegreeDays>,
    climate_zone: Option<&ClimateZone>,
    construction_modifiers: Option<&ConstructionModifiers>,
    recycling_state: Option<(&RecyclingState, &RecyclingEconomics)>,
    wind_damage_state: Option<&WindDamageState>,
    uhi_grid: Option<&UhiGrid>,
    drought_state: Option<&DroughtState>,
    heat_wave_state: Option<&HeatWaveState>,
    composting_state: Option<&simulation::composting::CompostingState>,
    cold_snap_state: Option<&ColdSnapState>,
    water_treatment_state: Option<&WaterTreatmentState>,
    groundwater_depletion_state: Option<&GroundwaterDepletionState>,
    wastewater_state: Option<&WastewaterState>,
    hazardous_waste_state: Option<&HazardousWasteState>,
    storm_drainage_state: Option<&StormDrainageState>,
    landfill_capacity_state: Option<&LandfillCapacityState>,
    flood_state: Option<&FloodState>,
    reservoir_state: Option<&ReservoirState>,
    landfill_gas_state: Option<&LandfillGasState>,
    cso_state: Option<&SewerSystemState>,
    water_conservation_state: Option<&WaterConservationState>,
    fog_state: Option<&FogState>,
    urban_growth_boundary: Option<&UrbanGrowthBoundary>,
    snow_state: Option<(&SnowGrid, &SnowPlowingState)>,
    agriculture_state: Option<&AgricultureState>,
) -> SaveData {
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

    SaveData {
        version: CURRENT_SAVE_VERSION,
        grid: SaveGrid {
            cells: save_cells,
            width: grid.width,
            height: grid.height,
        },
        roads: SaveRoadNetwork {
            road_positions: roads.edges.keys().map(|n| (n.0, n.1)).collect(),
        },
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
        policies: policies.map(|p| SavePolicies {
            active: p.active.iter().map(|&pol| policy_to_u8(pol)).collect(),
        }),
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
        lifecycle_timer: lifecycle_timer.map(|lt| SaveLifecycleTimer {
            last_aging_day: lt.last_aging_day,
            last_emigration_tick: lt.last_emigration_tick,
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
        life_sim_timer: life_sim_timer.map(|lst| SaveLifeSimTimer {
            needs_tick: lst.needs_tick,
            life_event_tick: lst.life_event_tick,
            salary_tick: lst.salary_tick,
            education_tick: lst.education_tick,
            job_seek_tick: lst.job_seek_tick,
            personality_tick: lst.personality_tick,
            health_tick: lst.health_tick,
        }),
        stormwater_grid: stormwater_grid.map(|sw| SaveStormwaterGrid {
            runoff: sw.runoff.clone(),
            total_runoff: sw.total_runoff,
            total_infiltration: sw.total_infiltration,
            width: sw.width,
            height: sw.height,
        }),
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
        wind_damage_state: wind_damage_state.map(|wds| SaveWindDamageState {
            current_tier: wind_damage_tier_to_u8(wds.current_tier),
            accumulated_building_damage: wds.accumulated_building_damage,
            trees_knocked_down: wds.trees_knocked_down,
            power_outage_active: wds.power_outage_active,
        }),
        uhi_grid: uhi_grid.map(|ug| SaveUhiGrid {
            cells: ug.cells.clone(),
            width: ug.width,
            height: ug.height,
        }),
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
        wastewater_state: wastewater_state.map(|ws| SaveWastewaterState {
            total_sewage_generated: ws.total_sewage_generated,
            total_treatment_capacity: ws.total_treatment_capacity,
            overflow_amount: ws.overflow_amount,
            coverage_ratio: ws.coverage_ratio,
            pollution_events: ws.pollution_events,
            health_penalty_active: ws.health_penalty_active,
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
        flood_state: flood_state.map(|fs| SaveFloodState {
            is_flooding: fs.is_flooding,
            total_flooded_cells: fs.total_flooded_cells,
            total_damage: fs.total_damage,
            max_depth: fs.max_depth,
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
        urban_growth_boundary: urban_growth_boundary.map(|u| SaveUrbanGrowthBoundary {
            enabled: u.enabled,
            vertices_x: u.vertices.iter().map(|(x, _)| *x).collect(),
            vertices_y: u.vertices.iter().map(|(_, y)| *y).collect(),
        }),
        snow_state: snow_state.map(|(sg, sp)| SaveSnowState {
            depths: sg.depths.clone(),
            width: sg.width,
            height: sg.height,
            plowing_enabled: sp.enabled,
            season_cost: sp.season_cost,
            cells_plowed_season: sp.cells_plowed_season,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simulation::budget::{ExtendedBudget, ServiceBudgets, ZoneTaxRates};
    use simulation::citizen::{CitizenDetails, PathCache, Position, Velocity};
    use simulation::degree_days::DegreeDays;
    use simulation::economy::CityBudget;
    use simulation::grid::WorldGrid;
    use simulation::life_simulation::LifeSimTimer;
    use simulation::lifecycle::LifecycleTimer;
    use simulation::loans::{self, LoanBook};
    use simulation::policies::{Policies, Policy};
    use simulation::roads::RoadNetwork;
    use simulation::stormwater::StormwaterGrid;
    use simulation::time_of_day::GameClock;
    use simulation::unlocks::{UnlockNode, UnlockState};
    use simulation::utilities::UtilityType;
    use simulation::virtual_population::VirtualPopulation;
    use simulation::water_sources::{WaterSource, WaterSourceType};
    use simulation::weather::{
        ClimateZone, ConstructionModifiers, Season, Weather, WeatherCondition,
    };
    use simulation::zones::ZoneDemand;

    #[test]
    fn test_roundtrip_serialization() {
        let mut grid = WorldGrid::new(16, 16);
        simulation::terrain::generate_terrain(&mut grid, 42);

        // Set some zones to test the new types
        grid.get_mut(5, 5).zone = simulation::grid::ZoneType::ResidentialLow;
        grid.get_mut(6, 6).zone = simulation::grid::ZoneType::ResidentialHigh;
        grid.get_mut(7, 7).zone = simulation::grid::ZoneType::CommercialLow;
        grid.get_mut(8, 8).zone = simulation::grid::ZoneType::CommercialHigh;
        grid.get_mut(9, 9).zone = simulation::grid::ZoneType::Office;

        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        assert_eq!(restored.grid.width, 16);
        assert_eq!(restored.grid.height, 16);
        assert_eq!(restored.grid.cells.len(), 256);
        assert_eq!(restored.clock.day, clock.day);
        assert!((restored.budget.treasury - budget.treasury).abs() < 0.01);

        // Verify zone roundtrip
        let idx55 = 5 * 16 + 5;
        assert_eq!(restored.grid.cells[idx55].zone, 1); // ResidentialLow
        let idx66 = 6 * 16 + 6;
        assert_eq!(restored.grid.cells[idx66].zone, 2); // ResidentialHigh
        let idx77 = 7 * 16 + 7;
        assert_eq!(restored.grid.cells[idx77].zone, 3); // CommercialLow
        let idx88 = 8 * 16 + 8;
        assert_eq!(restored.grid.cells[idx88].zone, 4); // CommercialHigh
        let idx99 = 9 * 16 + 9;
        assert_eq!(restored.grid.cells[idx99].zone, 6); // Office

        // V2 fields should be None when not provided
        assert!(restored.policies.is_none());
        assert!(restored.weather.is_none());
        assert!(restored.unlock_state.is_none());
        assert!(restored.extended_budget.is_none());
        assert!(restored.loan_book.is_none());
        assert!(restored.virtual_population.is_none());
        assert!(restored.stormwater_grid.is_none());
        assert!(restored.degree_days.is_none());
        assert!(restored.water_sources.is_none());
        assert!(restored.construction_modifiers.is_none());
        assert!(restored.recycling_state.is_none());
        assert!(restored.wind_damage_state.is_none());
        assert!(restored.uhi_grid.is_none());
        assert!(restored.drought_state.is_none());
        assert!(restored.heat_wave_state.is_none());
        assert!(restored.composting_state.is_none());
        assert!(restored.cold_snap_state.is_none());
        assert!(restored.water_treatment_state.is_none());
        assert!(restored.groundwater_depletion_state.is_none());
        assert!(restored.wastewater_state.is_none());
        assert!(restored.hazardous_waste_state.is_none());
        assert!(restored.storm_drainage_state.is_none());
        assert!(restored.landfill_capacity_state.is_none());
        assert!(restored.flood_state.is_none());
        assert!(restored.reservoir_state.is_none());
        assert!(restored.landfill_gas_state.is_none());
        assert!(restored.cso_state.is_none());
        assert!(restored.water_conservation_state.is_none());
        assert!(restored.fog_state.is_none());
        assert!(restored.urban_growth_boundary.is_none());
        assert!(restored.snow_state.is_none());
        assert!(restored.agriculture_state.is_none());
    }

    #[test]
    fn test_zone_type_roundtrip() {
        use simulation::grid::ZoneType;
        let types = [
            ZoneType::None,
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zt in &types {
            let encoded = zone_type_to_u8(*zt);
            let decoded = u8_to_zone_type(encoded);
            assert_eq!(*zt, decoded);
        }
    }

    #[test]
    fn test_utility_type_roundtrip() {
        let types = [
            UtilityType::PowerPlant,
            UtilityType::SolarFarm,
            UtilityType::WindTurbine,
            UtilityType::WaterTower,
            UtilityType::SewagePlant,
        ];
        for ut in &types {
            let encoded = utility_type_to_u8(*ut);
            let decoded = u8_to_utility_type(encoded);
            assert_eq!(*ut, decoded);
        }
    }

    #[test]
    fn test_service_type_roundtrip() {
        for i in 0..=49u8 {
            let st = u8_to_service_type(i).expect("valid service type");
            let encoded = service_type_to_u8(st);
            assert_eq!(i, encoded);
        }
        assert!(u8_to_service_type(50).is_none());
    }

    #[test]
    fn test_policy_roundtrip() {
        for &p in Policy::all() {
            let encoded = policy_to_u8(p);
            let decoded = u8_to_policy(encoded).expect("valid policy");
            assert_eq!(p, decoded);
        }
        assert!(u8_to_policy(255).is_none());
    }

    #[test]
    fn test_weather_roundtrip() {
        let weather = Weather {
            season: Season::Winter,
            temperature: -5.0,
            current_event: WeatherCondition::Snow,
            event_days_remaining: 3,
            last_update_day: 42,
            disasters_enabled: false,
            humidity: 0.8,
            cloud_cover: 0.7,
            precipitation_intensity: 0.5,
            last_update_hour: 14,
            prev_extreme: false,
            ..Default::default()
        };

        let save = SaveWeather {
            season: season_to_u8(weather.season),
            temperature: weather.temperature,
            current_event: weather_event_to_u8(weather.current_event),
            event_days_remaining: weather.event_days_remaining,
            last_update_day: weather.last_update_day,
            disasters_enabled: weather.disasters_enabled,
            humidity: weather.humidity,
            cloud_cover: weather.cloud_cover,
            precipitation_intensity: weather.precipitation_intensity,
            last_update_hour: weather.last_update_hour,
            climate_zone: 0,
        };

        let restored = restore_weather(&save);
        assert_eq!(restored.season, Season::Winter);
        assert!((restored.temperature - (-5.0)).abs() < 0.001);
        assert_eq!(restored.current_event, WeatherCondition::Snow);
        assert_eq!(restored.event_days_remaining, 3);
        assert_eq!(restored.last_update_day, 42);
        assert!(!restored.disasters_enabled);
        assert!((restored.humidity - 0.8).abs() < 0.001);
        assert!((restored.cloud_cover - 0.7).abs() < 0.001);
        assert!((restored.precipitation_intensity - 0.5).abs() < 0.001);
        assert_eq!(restored.last_update_hour, 14);
    }

    #[test]
    fn test_unlock_state_roundtrip() {
        let mut state = UnlockState::default();
        state.development_points = 10;
        state.spent_points = 3;
        state.last_milestone_pop = 2000;
        // Default already has BasicRoads, etc. Add another
        state.unlocked_nodes.push(UnlockNode::FireService);

        let save = SaveUnlockState {
            development_points: state.development_points,
            spent_points: state.spent_points,
            unlocked_nodes: state
                .unlocked_nodes
                .iter()
                .map(|&n| unlock_node_to_u8(n))
                .collect(),
            last_milestone_pop: state.last_milestone_pop,
        };

        let restored = restore_unlock_state(&save);
        assert_eq!(restored.development_points, 10);
        assert_eq!(restored.spent_points, 3);
        assert_eq!(restored.last_milestone_pop, 2000);
        assert!(restored.is_unlocked(UnlockNode::BasicRoads));
        assert!(restored.is_unlocked(UnlockNode::FireService));
        assert!(!restored.is_unlocked(UnlockNode::NuclearPower));
    }

    #[test]
    fn test_unlock_node_roundtrip() {
        for &n in UnlockNode::all() {
            let encoded = unlock_node_to_u8(n);
            let decoded = u8_to_unlock_node(encoded).expect("valid unlock node");
            assert_eq!(n, decoded);
        }
        assert!(u8_to_unlock_node(255).is_none());
    }

    #[test]
    fn test_policies_serialize_roundtrip() {
        let policies = Policies {
            active: vec![
                Policy::FreePublicTransport,
                Policy::RecyclingProgram,
                Policy::HighRiseBan,
            ],
        };

        let save = SavePolicies {
            active: policies.active.iter().map(|&p| policy_to_u8(p)).collect(),
        };

        let restored = restore_policies(&save);
        assert_eq!(restored.active.len(), 3);
        assert!(restored.is_active(Policy::FreePublicTransport));
        assert!(restored.is_active(Policy::RecyclingProgram));
        assert!(restored.is_active(Policy::HighRiseBan));
        assert!(!restored.is_active(Policy::EducationPush));
    }

    #[test]
    fn test_extended_budget_roundtrip() {
        let save = SaveExtendedBudget {
            residential_tax: 0.12,
            commercial_tax: 0.08,
            industrial_tax: 0.15,
            office_tax: 0.11,
            fire_budget: 1.2,
            police_budget: 0.8,
            healthcare_budget: 1.0,
            education_budget: 1.5,
            sanitation_budget: 0.5,
            transport_budget: 1.1,
        };

        let restored = restore_extended_budget(&save);
        assert!((restored.zone_taxes.residential - 0.12).abs() < 0.001);
        assert!((restored.zone_taxes.commercial - 0.08).abs() < 0.001);
        assert!((restored.zone_taxes.industrial - 0.15).abs() < 0.001);
        assert!((restored.zone_taxes.office - 0.11).abs() < 0.001);
        assert!((restored.service_budgets.fire - 1.2).abs() < 0.001);
        assert!((restored.service_budgets.police - 0.8).abs() < 0.001);
        assert!((restored.service_budgets.education - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_loan_book_roundtrip() {
        let save = SaveLoanBook {
            loans: vec![SaveLoan {
                name: "Small Loan".into(),
                amount: 10_000.0,
                interest_rate: 0.05,
                monthly_payment: 856.07,
                remaining_balance: 8_500.0,
                term_months: 12,
                months_paid: 2,
            }],
            max_loans: 3,
            credit_rating: 1.5,
            last_payment_day: 60,
            consecutive_solvent_days: 45,
        };

        let restored = restore_loan_book(&save);
        assert_eq!(restored.active_loans.len(), 1);
        assert_eq!(restored.active_loans[0].name, "Small Loan");
        assert!((restored.active_loans[0].amount - 10_000.0).abs() < 0.01);
        assert!((restored.active_loans[0].remaining_balance - 8_500.0).abs() < 0.01);
        assert_eq!(restored.active_loans[0].months_paid, 2);
        assert_eq!(restored.max_loans, 3);
        assert!((restored.credit_rating - 1.5).abs() < 0.001);
        assert_eq!(restored.last_payment_day, 60);
        assert_eq!(restored.consecutive_solvent_days, 45);
    }

    #[test]
    fn test_v2_full_roundtrip() {
        // Test that all V2 fields survive a full encode/decode cycle
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let policies = Policies {
            active: vec![Policy::EducationPush, Policy::WaterConservation],
        };
        let weather = Weather {
            season: Season::Summer,
            temperature: 32.0,
            current_event: WeatherCondition::Sunny,
            event_days_remaining: 4,
            last_update_day: 100,
            disasters_enabled: true,
            humidity: 0.3,
            cloud_cover: 0.05,
            precipitation_intensity: 0.0,
            last_update_hour: 12,
            prev_extreme: false,
            ..Default::default()
        };
        let mut unlock = UnlockState::default();
        unlock.development_points = 15;
        unlock.spent_points = 5;
        unlock.unlocked_nodes.push(UnlockNode::HealthCare);
        unlock.last_milestone_pop = 5000;

        let ext_budget = ExtendedBudget {
            zone_taxes: ZoneTaxRates {
                residential: 0.12,
                commercial: 0.09,
                industrial: 0.14,
                office: 0.11,
            },
            service_budgets: ServiceBudgets {
                fire: 1.3,
                police: 0.9,
                healthcare: 1.0,
                education: 1.2,
                sanitation: 0.7,
                transport: 1.1,
            },
            loans: Vec::new(),
            income_breakdown: Default::default(),
            expense_breakdown: Default::default(),
        };

        let mut loan_book = LoanBook::default();
        let mut treasury = 0.0;
        loan_book.take_loan(loans::LoanTier::Small, &mut treasury);

        let lifecycle_timer = LifecycleTimer {
            last_aging_day: 200,
            last_emigration_tick: 15,
        };

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            Some(&policies),
            Some(&weather),
            Some(&unlock),
            Some(&ext_budget),
            Some(&loan_book),
            Some(&lifecycle_timer),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode v2 should succeed");

        // Policies
        let rp = restored.policies.as_ref().expect("policies present");
        assert_eq!(rp.active.len(), 2);

        // Weather
        let rw = restored.weather.as_ref().expect("weather present");
        assert_eq!(rw.season, season_to_u8(Season::Summer));
        assert!((rw.temperature - 32.0).abs() < 0.001);
        assert_eq!(
            rw.current_event,
            weather_event_to_u8(WeatherCondition::Sunny)
        );

        // Unlock state
        let ru = restored
            .unlock_state
            .as_ref()
            .expect("unlock_state present");
        assert_eq!(ru.development_points, 15);
        assert_eq!(ru.spent_points, 5);
        assert_eq!(ru.last_milestone_pop, 5000);

        // Extended budget
        let reb = restored
            .extended_budget
            .as_ref()
            .expect("extended_budget present");
        assert!((reb.fire_budget - 1.3).abs() < 0.001);
        assert!((reb.residential_tax - 0.12).abs() < 0.001);

        // Loan book
        let rlb = restored.loan_book.as_ref().expect("loan_book present");
        assert_eq!(rlb.loans.len(), 1);
        assert_eq!(rlb.loans[0].name, "Small Loan");
    }

    #[test]
    fn test_backward_compat_v1_defaults() {
        // Simulate a V1 save that has no V2 fields: create a SaveData with
        // all V2 fields set to None, encode it, decode it, and verify defaults work.
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode v1 should succeed");

        // V2 fields should be None
        assert!(restored.policies.is_none());
        assert!(restored.weather.is_none());
        assert!(restored.unlock_state.is_none());
        assert!(restored.extended_budget.is_none());
        assert!(restored.loan_book.is_none());
        assert!(restored.lifecycle_timer.is_none());
        assert!(restored.virtual_population.is_none());
        assert!(restored.life_sim_timer.is_none());
        assert!(restored.stormwater_grid.is_none());
        assert!(restored.degree_days.is_none());
        assert!(restored.water_sources.is_none());
        assert!(restored.construction_modifiers.is_none());
        assert!(restored.recycling_state.is_none());
        assert!(restored.wind_damage_state.is_none());
        assert!(restored.uhi_grid.is_none());
        assert!(restored.drought_state.is_none());
        assert!(restored.heat_wave_state.is_none());
        assert!(restored.composting_state.is_none());
        assert!(restored.cold_snap_state.is_none());
        assert!(restored.water_treatment_state.is_none());
        assert!(restored.groundwater_depletion_state.is_none());
        assert!(restored.wastewater_state.is_none());
        assert!(restored.hazardous_waste_state.is_none());
        assert!(restored.storm_drainage_state.is_none());
        assert!(restored.landfill_capacity_state.is_none());
        assert!(restored.flood_state.is_none());
        assert!(restored.reservoir_state.is_none());
        assert!(restored.landfill_gas_state.is_none());
        assert!(restored.cso_state.is_none());
        assert!(restored.water_conservation_state.is_none());
        assert!(restored.fog_state.is_none());
        assert!(restored.urban_growth_boundary.is_none());
        assert!(restored.snow_state.is_none());
        assert!(restored.agriculture_state.is_none());
    }

    #[test]
    fn test_lifecycle_timer_roundtrip() {
        let timer = LifecycleTimer {
            last_aging_day: 730,
            last_emigration_tick: 25,
        };

        let save = SaveLifecycleTimer {
            last_aging_day: timer.last_aging_day,
            last_emigration_tick: timer.last_emigration_tick,
        };

        let restored = restore_lifecycle_timer(&save);
        assert_eq!(restored.last_aging_day, 730);
        assert_eq!(restored.last_emigration_tick, 25);
    }

    // -----------------------------------------------------------------------
    // Save versioning / migration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_save_data_sets_current_version() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_from_v0_to_current() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 0;

        let old = migrate_save(&mut save);
        assert_eq!(old, 0);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_current_version_is_noop() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(save.version, CURRENT_SAVE_VERSION);
        let old = migrate_save(&mut save);
        assert_eq!(old, CURRENT_SAVE_VERSION);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_version_roundtrips_through_encode_decode() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_from_v1() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 1;

        let old = migrate_save(&mut save);
        assert_eq!(old, 1);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_from_v2() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 2;

        let old = migrate_save(&mut save);
        assert_eq!(old, 2);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_virtual_population_roundtrip() {
        let mut vp = VirtualPopulation::default();
        vp.add_virtual_citizen(0, 25, true, 75.0, 1000.0, 0.1);
        vp.add_virtual_citizen(0, 40, false, 50.0, 0.0, 0.0);
        vp.add_virtual_citizen(1, 60, true, 80.0, 1500.0, 0.12);

        let save = SaveVirtualPopulation {
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
        };

        let restored = restore_virtual_population(&save);
        assert_eq!(restored.total_virtual, 3);
        assert_eq!(restored.virtual_employed, 2);
        assert_eq!(restored.district_stats.len(), 2);
        assert_eq!(restored.district_stats[0].population, 2);
        assert_eq!(restored.district_stats[0].employed, 1);
        assert_eq!(restored.district_stats[1].population, 1);
        assert_eq!(restored.district_stats[1].employed, 1);
        assert_eq!(restored.max_real_citizens, vp.max_real_citizens);
    }

    #[test]
    fn test_pathcache_velocity_citizen_roundtrip() {
        use simulation::roads::RoadNode;
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();
        let citizens = vec![
            CitizenSaveInput {
                details: CitizenDetails {
                    age: 30,
                    gender: simulation::citizen::Gender::Male,
                    education: 2,
                    happiness: 75.0,
                    health: 90.0,
                    salary: 3500.0,
                    savings: 7000.0,
                },
                state: CitizenState::CommutingToWork,
                home_x: 1,
                home_y: 1,
                work_x: 3,
                work_y: 3,
                path: PathCache {
                    waypoints: vec![
                        RoadNode(1, 1),
                        RoadNode(2, 1),
                        RoadNode(2, 2),
                        RoadNode(3, 3),
                    ],
                    current_index: 1,
                },
                velocity: Velocity { x: 4.5, y: -2.3 },
                position: Position { x: 100.0, y: 200.0 },
            },
            CitizenSaveInput {
                details: CitizenDetails {
                    age: 45,
                    gender: simulation::citizen::Gender::Female,
                    education: 1,
                    happiness: 60.0,
                    health: 80.0,
                    salary: 2200.0,
                    savings: 4400.0,
                },
                state: CitizenState::AtHome,
                home_x: 2,
                home_y: 2,
                work_x: 3,
                work_y: 2,
                path: PathCache {
                    waypoints: vec![],
                    current_index: 0,
                },
                velocity: Velocity { x: 0.0, y: 0.0 },
                position: Position { x: 50.0, y: 75.0 },
            },
        ];
        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &citizens,
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert_eq!(restored.citizens.len(), 2);
        // First citizen: active path with waypoints
        let c0 = &restored.citizens[0];
        assert_eq!(c0.path_waypoints, vec![(1, 1), (2, 1), (2, 2), (3, 3)]);
        assert_eq!(c0.path_current_index, 1);
        assert!((c0.velocity_x - 4.5).abs() < 0.001);
        assert!((c0.velocity_y - (-2.3)).abs() < 0.001);
        assert!((c0.pos_x - 100.0).abs() < 0.001);
        assert!((c0.pos_y - 200.0).abs() < 0.001);
        assert_eq!(c0.state, 1); // CommutingToWork
                                 // Second citizen: idle, empty path
        let c1 = &restored.citizens[1];
        assert!(c1.path_waypoints.is_empty());
        assert_eq!(c1.path_current_index, 0);
        assert!((c1.velocity_x).abs() < 0.001);
        assert!((c1.velocity_y).abs() < 0.001);
        assert!((c1.pos_x - 50.0).abs() < 0.001);
        assert!((c1.pos_y - 75.0).abs() < 0.001);
        assert_eq!(c1.state, 0); // AtHome
    }

    #[test]
    fn test_pathcache_velocity_v2_backward_compat() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();
        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        // Simulate an old save citizen with default V3 fields
        save.citizens.push(SaveCitizen {
            age: 25,
            happiness: 70.0,
            education: 1,
            state: 1, // CommutingToWork
            home_x: 1,
            home_y: 1,
            work_x: 3,
            work_y: 3,
            path_waypoints: vec![],
            path_current_index: 0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
        });
        save.version = 2;
        let old = migrate_save(&mut save);
        assert_eq!(old, 2);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
        let c = &save.citizens[0];
        assert!(c.path_waypoints.is_empty());
        assert_eq!(c.path_current_index, 0);
        assert!((c.velocity_x).abs() < 0.001);
        assert!((c.velocity_y).abs() < 0.001);
    }

    #[test]
    fn test_life_sim_timer_roundtrip() {
        let timer = LifeSimTimer {
            needs_tick: 7,
            life_event_tick: 123,
            salary_tick: 9999,
            education_tick: 500,
            job_seek_tick: 42,
            personality_tick: 1234,
            health_tick: 777,
        };

        let save = SaveLifeSimTimer {
            needs_tick: timer.needs_tick,
            life_event_tick: timer.life_event_tick,
            salary_tick: timer.salary_tick,
            education_tick: timer.education_tick,
            job_seek_tick: timer.job_seek_tick,
            personality_tick: timer.personality_tick,
            health_tick: timer.health_tick,
        };

        let restored = restore_life_sim_timer(&save);
        assert_eq!(restored.needs_tick, 7);
        assert_eq!(restored.life_event_tick, 123);
        assert_eq!(restored.salary_tick, 9999);
        assert_eq!(restored.education_tick, 500);
        assert_eq!(restored.job_seek_tick, 42);
        assert_eq!(restored.personality_tick, 1234);
        assert_eq!(restored.health_tick, 777);
    }

    #[test]
    fn test_life_sim_timer_full_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let life_sim_timer = LifeSimTimer {
            needs_tick: 5,
            life_event_tick: 300,
            salary_tick: 20000,
            education_tick: 700,
            job_seek_tick: 100,
            personality_tick: 1500,
            health_tick: 900,
        };

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&life_sim_timer),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        let rlst = restored
            .life_sim_timer
            .as_ref()
            .expect("life_sim_timer present");
        assert_eq!(rlst.needs_tick, 5);
        assert_eq!(rlst.life_event_tick, 300);
        assert_eq!(rlst.salary_tick, 20000);
        assert_eq!(rlst.education_tick, 700);
        assert_eq!(rlst.job_seek_tick, 100);
        assert_eq!(rlst.personality_tick, 1500);
        assert_eq!(rlst.health_tick, 900);
    }

    #[test]
    fn test_life_sim_timer_backward_compat() {
        // Saves without life_sim_timer should have it as None
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert!(restored.life_sim_timer.is_none());
    }

    #[test]
    fn test_migrate_from_v3() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 3;

        let old = migrate_save(&mut save);
        assert_eq!(old, 3);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_water_source_type_roundtrip() {
        let types = [
            WaterSourceType::Well,
            WaterSourceType::SurfaceIntake,
            WaterSourceType::Reservoir,
            WaterSourceType::Desalination,
        ];
        for wt in &types {
            let encoded = water_source_type_to_u8(*wt);
            let decoded = u8_to_water_source_type(encoded).expect("valid water source type");
            assert_eq!(*wt, decoded);
        }
        assert!(u8_to_water_source_type(255).is_none());
    }

    #[test]
    fn test_water_source_save_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let water_sources = vec![
            WaterSource {
                source_type: WaterSourceType::Well,
                capacity_mgd: 0.5,
                quality: 0.7,
                operating_cost: 15.0,
                grid_x: 2,
                grid_y: 2,
                stored_gallons: 0.0,
                storage_capacity: 0.0,
            },
            WaterSource {
                source_type: WaterSourceType::Reservoir,
                capacity_mgd: 20.0,
                quality: 0.8,
                operating_cost: 200.0,
                grid_x: 1,
                grid_y: 1,
                stored_gallons: 1_800_000_000.0,
                storage_capacity: 1_800_000_000.0,
            },
        ];

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&water_sources),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        let rws = restored
            .water_sources
            .as_ref()
            .expect("water_sources present");
        assert_eq!(rws.len(), 2);

        let w0 = &rws[0];
        assert_eq!(
            u8_to_water_source_type(w0.source_type),
            Some(WaterSourceType::Well)
        );
        assert!((w0.capacity_mgd - 0.5).abs() < 0.001);
        assert!((w0.quality - 0.7).abs() < 0.001);
        assert_eq!(w0.grid_x, 2);
        assert_eq!(w0.grid_y, 2);

        let w1 = &rws[1];
        assert_eq!(
            u8_to_water_source_type(w1.source_type),
            Some(WaterSourceType::Reservoir)
        );
        assert!((w1.capacity_mgd - 20.0).abs() < 0.001);
        assert!(w1.stored_gallons > 0.0);
    }

    #[test]
    fn test_water_source_restore() {
        let save = SaveWaterSource {
            source_type: water_source_type_to_u8(WaterSourceType::Desalination),
            grid_x: 5,
            grid_y: 5,
            capacity_mgd: 10.0,
            quality: 0.95,
            operating_cost: 500.0,
            stored_gallons: 0.0,
            storage_capacity: 0.0,
        };

        let ws = restore_water_source(&save).expect("valid water source");
        assert_eq!(ws.source_type, WaterSourceType::Desalination);
        assert!((ws.capacity_mgd - 10.0).abs() < 0.001);
        assert!((ws.quality - 0.95).abs() < 0.001);
        assert_eq!(ws.grid_x, 5);
        assert_eq!(ws.grid_y, 5);
    }

    #[test]
    fn test_degree_days_roundtrip() {
        let dd = DegreeDays {
            daily_hdd: 15.5,
            daily_cdd: 0.0,
            monthly_hdd: [
                10.0, 20.0, 15.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 12.0, 25.0,
            ],
            monthly_cdd: [
                0.0, 0.0, 0.0, 0.0, 5.0, 15.0, 20.0, 18.0, 10.0, 0.0, 0.0, 0.0,
            ],
            annual_hdd: 92.5,
            annual_cdd: 68.0,
            last_update_day: 150,
        };

        let save = SaveDegreeDays {
            daily_hdd: dd.daily_hdd,
            daily_cdd: dd.daily_cdd,
            monthly_hdd: dd.monthly_hdd,
            monthly_cdd: dd.monthly_cdd,
            annual_hdd: dd.annual_hdd,
            annual_cdd: dd.annual_cdd,
            last_update_day: dd.last_update_day,
        };

        let restored = restore_degree_days(&save);
        assert!((restored.daily_hdd - 15.5).abs() < 0.001);
        assert!(restored.daily_cdd.abs() < 0.001);
        assert!((restored.monthly_hdd[0] - 10.0).abs() < 0.001);
        assert!((restored.monthly_cdd[6] - 20.0).abs() < 0.001);
        assert!((restored.annual_hdd - 92.5).abs() < 0.001);
        assert!((restored.annual_cdd - 68.0).abs() < 0.001);
        assert_eq!(restored.last_update_day, 150);
    }

    #[test]
    fn test_water_source_backward_compat() {
        // Saves without water_sources should have it as None
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert!(restored.water_sources.is_none());
        assert!(restored.degree_days.is_none());
    }

    #[test]
    fn test_migrate_from_v4() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 4;

        let old = migrate_save(&mut save);
        assert_eq!(old, 4);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
        // Vacancy fields should default to 0.0 for a migrated v4 save.
        assert!((save.demand.vacancy_residential).abs() < 0.001);
        assert!((save.demand.vacancy_commercial).abs() < 0.001);
        assert!((save.demand.vacancy_industrial).abs() < 0.001);
        assert!((save.demand.vacancy_office).abs() < 0.001);
    }

    #[test]
    fn test_migrate_from_v5() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 5;

        let old = migrate_save(&mut save);
        assert_eq!(old, 5);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_stormwater_grid_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut sw = StormwaterGrid::default();
        sw.runoff[0] = 10.5;
        sw.runoff[5] = 3.2;
        sw.total_runoff = 13.7;
        sw.total_infiltration = 5.0;

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&sw),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        let rsw = restored
            .stormwater_grid
            .as_ref()
            .expect("stormwater_grid present");
        assert!((rsw.runoff[0] - 10.5).abs() < 0.001);
        assert!((rsw.runoff[5] - 3.2).abs() < 0.001);
        assert!((rsw.total_runoff - 13.7).abs() < 0.001);
        assert!((rsw.total_infiltration - 5.0).abs() < 0.001);

        let restored_sw = restore_stormwater_grid(rsw);
        assert!((restored_sw.runoff[0] - 10.5).abs() < 0.001);
        assert!((restored_sw.total_runoff - 13.7).abs() < 0.001);
    }

    #[test]
    fn test_stormwater_backward_compat() {
        // Saves without stormwater_grid should have it as None
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert!(restored.stormwater_grid.is_none());
    }

    #[test]
    fn test_climate_zone_roundtrip() {
        for &zone in ClimateZone::all() {
            let encoded = climate_zone_to_u8(zone);
            let decoded = u8_to_climate_zone(encoded);
            assert_eq!(zone, decoded, "ClimateZone roundtrip failed for {:?}", zone);
        }
        // Fallback for unknown values
        assert_eq!(u8_to_climate_zone(255), ClimateZone::Temperate);
    }

    #[test]
    fn test_construction_modifiers_roundtrip() {
        let cm = ConstructionModifiers {
            speed_factor: 0.55,
            cost_factor: 1.25,
        };

        let save = SaveConstructionModifiers {
            speed_factor: cm.speed_factor,
            cost_factor: cm.cost_factor,
        };

        let restored = restore_construction_modifiers(&save);
        assert!((restored.speed_factor - 0.55).abs() < 0.001);
        assert!((restored.cost_factor - 1.25).abs() < 0.001);
    }

    #[test]
    fn test_climate_zone_save_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let weather = Weather {
            season: Season::Summer,
            temperature: 32.0,
            current_event: WeatherCondition::Sunny,
            event_days_remaining: 4,
            last_update_day: 100,
            disasters_enabled: true,
            humidity: 0.3,
            cloud_cover: 0.05,
            precipitation_intensity: 0.0,
            last_update_hour: 12,
            prev_extreme: false,
            ..Default::default()
        };

        let climate_zone = ClimateZone::Tropical;

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            Some(&weather),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&climate_zone),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        let rw = restored.weather.as_ref().expect("weather present");
        let restored_zone = restore_climate_zone(rw);
        assert_eq!(restored_zone, ClimateZone::Tropical);
    }

    #[test]
    fn test_climate_zone_backward_compat_defaults_to_temperate() {
        // Old saves without climate_zone field should default to Temperate (0)
        let save = SaveWeather::default();
        let zone = restore_climate_zone(&save);
        assert_eq!(zone, ClimateZone::Temperate);
    }

    #[test]
    fn test_construction_modifiers_backward_compat() {
        // Saves without construction_modifiers should have it as None
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert!(restored.stormwater_grid.is_none());
        // When construction_modifiers is None, the restore uses default
        assert!(restored.construction_modifiers.is_none());
        assert!(restored.recycling_state.is_none());
        assert!(restored.wind_damage_state.is_none());
        assert!(restored.uhi_grid.is_none());
        assert!(restored.drought_state.is_none());
        assert!(restored.heat_wave_state.is_none());
        assert!(restored.composting_state.is_none());
        assert!(restored.cold_snap_state.is_none());
        assert!(restored.water_treatment_state.is_none());
        assert!(restored.groundwater_depletion_state.is_none());
        assert!(restored.wastewater_state.is_none());
        assert!(restored.hazardous_waste_state.is_none());
        assert!(restored.storm_drainage_state.is_none());
        assert!(restored.landfill_capacity_state.is_none());
        assert!(restored.flood_state.is_none());
        assert!(restored.reservoir_state.is_none());
        assert!(restored.landfill_gas_state.is_none());
        assert!(restored.cso_state.is_none());
        assert!(restored.water_conservation_state.is_none());
        assert!(restored.fog_state.is_none());
        assert!(restored.urban_growth_boundary.is_none());
        assert!(restored.snow_state.is_none());
        assert!(restored.agriculture_state.is_none());
    }

    #[test]
    fn test_vacancy_rates_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand {
            residential: 0.7,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
            vacancy_residential: 0.06,
            vacancy_commercial: 0.12,
            vacancy_industrial: 0.08,
            vacancy_office: 0.10,
        };

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        assert!((restored.demand.residential - 0.7).abs() < 0.001);
        assert!((restored.demand.commercial - 0.5).abs() < 0.001);
        assert!((restored.demand.industrial - 0.3).abs() < 0.001);
        assert!((restored.demand.office - 0.2).abs() < 0.001);
        assert!((restored.demand.vacancy_residential - 0.06).abs() < 0.001);
        assert!((restored.demand.vacancy_commercial - 0.12).abs() < 0.001);
        assert!((restored.demand.vacancy_industrial - 0.08).abs() < 0.001);
        assert!((restored.demand.vacancy_office - 0.10).abs() < 0.001);
    }
}
