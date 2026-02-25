use bevy::prelude::*;
use crate::SaveableRegistry;
/// Exhaustive list of every `SAVE_KEY` that types implementing `Saveable` declare
/// across the codebase. The `validate_saveable_registry` startup system asserts
/// that each of these keys is present in the `SaveableRegistry`, catching the
/// class of bugs where a type implements `Saveable` but its plugin forgets to
/// call `register_saveable`.
///
/// When you add a new `Saveable` type, add its key here. The startup assertion
/// will remind you if you forget to register it.
pub const EXPECTED_SAVEABLE_KEYS: &[&str] = &[
    "achievement_tracker",
    "active_disaster",
    "advisor_panel",
    "autosave_config",
    "battery_storage",
    "biome_grid",
    "blackout_state",
    "blueprint_library",
    "active_city_effects",
    "bicycle_lanes",
    "biomass_power",
    "bus_transit",
    "chart_history",
    "city_hall",
    "climate_change",
    "city_specializations",
    "coal_power",
    "colorblind_settings",
    "crime_grid",
    "crime_justice",
    "daycare_eldercare",
    "deathcare_capacity",
    "demand_response_programs",
    "cumulative_zoning",
    "day_night_controls",
    "dismissed_advisor_tips",
    "disease_state",
    "district_policies",
    "district_map",
    "education_pipeline",
    "emergency_management",
    "energy_dispatch",
    "energy_economics",
    "energy_grid",
    "energy_pricing_config",
    "event_journal",
    "far_transfer",
    "fire_grid",
    "fire_tiers",
    "flood_protection",
    "forest_fire_grid",
    "forest_fire_stats",
    "freight_traffic",
    "form_transect",
    "game_params",
    "gas_power",
    "geothermal_power",
    "groundwater_depletion",
    "groundwater_grid",
    "heat_mitigation",
    "heating_grid",
    "heating_service",
    "historic_preservation",
    "hotel_demand",
    "inclusionary_zoning",
    "keybindings",
    "land_value",
    "lifecycle_timer",
    "landfill_state",
    "localization",
    "metro_transit",
    "milestone_tracker",
    "mode_share_stats",
    "multi_select",
    "neighborhood_quality",
    "nimby_state",
    "noise_grid",
    "oil_power",
    "nuclear_power",
    "oneway_direction_map",
    "parking_policy",
    "policy_tradeoffs",
    "parks_system",
    "police_tiers",
    "pollution_alert_log",
    "pollution_grid",
    "postal_stats",
    "population_tier_stats",
    "road_hierarchy",
    "roundabout_registry",
    "seasonal_effects_config",
    "seasonal_rendering",
    "solar_power",
    "service_budget",
    "sim_rng",
    "service_capacity",
    "social_services",
    "specialization_bonuses",
    "stormwater_grid",
    "stormwater_mgmt",
    "superblock_state",
    "superblock_policy",
    "traffic_grid",
    "traffic_los",
    "traffic_los_state",
    "telecom",
    "terrain_config",
    "tourism",
    "train_transit",
    "tram_transit",
    "tram_transit_stats",
    "transit_hub_stats",
    "transit_hubs",
    "tree_grid",
    "tree_maturity",
    "tree_canopy_stats",
    "tutorial",
    "uhi_mitigation",
    "virtual_population",
    "walkability",
    "waste_policies",
    "water_pollution_grid",
    "water_pollution_sources",
    "water_pressure",
    "water_quality_effects",
    "water_pipe_network",
    "water_quality_grid",
    "water_treatment",
    "wind_pollution_config",
    "wind_power",
    "waste_to_energy",
    "environmental_score",
    "milestone_progress",
    "play_time",
    "park_districts",
    "power_grid_balance",
    "power_lines",
    "power_plant_maintenance",
    "hybrid_coverage",
    "service_building_capacity",
    "cultural_prestige",
    "industrial_specializations",
    "save_slot_manager",
    "production_chain",
    "service_dispatch",
    "service_cross_interaction",
    "hydro_power",
    "vehicle_dispatch",
    "campus_university",
    "water_physics",
    "garbage_collection",
    "pollution_mitigation",
    "soil_contamination",
    "hope_discontent",
];
/// Startup system that validates the `SaveableRegistry` against the expected key
/// list. Panics if any expected key is missing (indicating a `Saveable` type whose
/// plugin forgot to register it) or if duplicate keys are detected.
///
/// Runs in `PostStartup` so all plugins have had a chance to register their types.
pub fn validate_saveable_registry(registry: Res<SaveableRegistry>) {
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();
    // Check for duplicate keys.
    if registered.len() != registry.entries.len() {
        let mut seen = std::collections::HashSet::new();
        for entry in &registry.entries {
            if !seen.insert(entry.key.as_str()) {
                panic!(
                    "SaveableRegistry: duplicate key '{}' detected — two types share the same SAVE_KEY",
                    entry.key
                );
            }
        }
    }
    // Check for missing registrations.
    let mut missing = Vec::new();
    for &expected in EXPECTED_SAVEABLE_KEYS {
        if !registered.contains(expected) {
            missing.push(expected);
        }
    }
    if !missing.is_empty() {
        panic!(
            "SaveableRegistry drift detected: {} expected key(s) not registered: {:?}. \
             Each type implementing `Saveable` must be registered via `register_saveable` \
             in its plugin's `build()` method.",
            missing.len(),
            missing,
        );
    }
    info!(
        "SaveableRegistry validated: {} keys registered, all {} expected keys present",
        registry.entries.len(),
        EXPECTED_SAVEABLE_KEYS.len(),
    );
}
