// ---------------------------------------------------------------------------
// Save file version constants
// ---------------------------------------------------------------------------

/// Current save file version.
/// v1 = original fields (grid, roads, clock, budget, demand, buildings, citizens, utilities, services, road_segments)
/// v2 = policies, weather, unlock_state, extended_budget, loans
/// v3 = lifecycle_timer, path_cache, velocity per citizen
/// v4 = life_sim_timer (LifeSimTimer serialization)
/// v5 = stormwater_grid (StormwaterGrid serialization)
/// v6 = water_sources (WaterSource component serialization), market-driven zone demand with vacancy rates
/// v7 = degree_days (HDD/CDD tracking for HVAC energy demand)
/// v8 = climate_zone in SaveWeather (ClimateZone resource)
/// v9 = construction_modifiers (ConstructionModifiers serialization)
/// v10 = recycling_state (RecyclingState + RecyclingEconomics serialization)
/// v11 = wind_damage_state (WindDamageState serialization)
/// v12 = uhi_grid (UhiGrid serialization for urban heat island)
/// v13 = drought_state (DroughtState serialization for drought index)
/// v14 = heat_wave_state (HeatWaveState serialization for heat wave effects)
/// v15 = composting_state (CompostingState serialization for composting facilities)
/// v16 = cold_snap_state (ColdSnapState serialization for cold snap effects)
/// v17 = water_treatment_state (WaterTreatmentState serialization for water treatment plants)
/// v18 = groundwater_depletion_state (GroundwaterDepletionState serialization)
/// v19 = wastewater_state (WastewaterState serialization)
/// v20 = hazardous_waste_state (HazardousWasteState serialization)
/// v21 = storm_drainage_state (StormDrainageState serialization for storm drainage infrastructure)
/// v22 = landfill_capacity_state (LandfillCapacityState serialization for landfill warnings)
/// v23 = flood_state (FloodState serialization for urban flooding simulation)
/// v24 = reservoir_state (ReservoirState serialization for reservoir water level tracking)
/// v25 = landfill_gas_state (LandfillGasState serialization for landfill gas collection and energy)
/// v26 = cso_state (SewerSystemState serialization for CSO events)
/// v27 = water_conservation_state (WaterConservationState serialization for water conservation)
/// v28 = fog_state (FogState serialization for fog and visibility)
/// v29 = urban_growth_boundary (UrbanGrowthBoundary serialization for UGB polygon)
/// v30 = snow_state (SnowGrid + SnowPlowingState serialization for snow accumulation and plowing)
/// v31 = agriculture_state (AgricultureState serialization for growing season and crop yield)
/// v32 = family graph (partner/children/parent Entity refs serialized as citizen indices)
// v32 = family graph (partner/children/parent relationships across save/load)
pub const CURRENT_SAVE_VERSION: u32 = 32; // v32: Family graph serialization
