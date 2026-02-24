use bevy::prelude::*;

use crate::*;

/// Register all simulation feature plugins.
///
/// Each plugin is registered on its own line for conflict-free parallel additions.
/// When adding a new feature plugin, just append a new `app.add_plugins(...)` line
/// at the end of the appropriate section.
pub(crate) fn register_feature_plugins(app: &mut App) {
    // Core simulation chain
    app.add_plugins(sim_rng::SimRngPlugin);
    app.add_plugins(game_params::GameParamsPlugin);
    app.add_plugins(time_of_day::TimeOfDayPlugin);
    app.add_plugins(zones::ZonesPlugin);
    app.add_plugins(buildings::BuildingsPlugin);
    app.add_plugins(education_jobs::EducationJobsPlugin);
    app.add_plugins(citizen_spawner::CitizenSpawnerPlugin);
    app.add_plugins(movement::MovementPlugin);
    app.add_plugins(traffic::TrafficPlugin);
    app.add_plugins(traffic_grid_save::TrafficGridSavePlugin);
    app.add_plugins(bicycle_lanes::BicycleLanesPlugin);

    // Happiness and services
    app.add_plugins(postal::PostalPlugin);
    app.add_plugins(telecom::TelecomPlugin);
    app.add_plugins(happiness::HappinessPlugin);
    app.add_plugins(service_capacity::ServiceCapacityPlugin);
    app.add_plugins(parks_system::ParksSystemPlugin);
    app.add_plugins(economy::EconomyPlugin);
    app.add_plugins(service_budget::ServiceBudgetPlugin);
    app.add_plugins(stats::StatsPlugin);
    app.add_plugins(chart_data::ChartDataPlugin);
    app.add_plugins(utilities::UtilitiesPlugin);
    app.add_plugins(network_viz::NetworkVizPlugin);
    app.add_plugins(education::EducationPlugin);
    app.add_plugins(education_pipeline::EducationPipelinePlugin);

    // Pollution, land value, garbage, districts
    app.add_plugins(pollution::PollutionPlugin);
    app.add_plugins(building_emissions::BuildingEmissionsPlugin);
    app.add_plugins(pollution_health::PollutionHealthPlugin);
    app.add_plugins(pollution_alerts::PollutionAlertPlugin);
    app.add_plugins(land_value::LandValuePlugin);
    app.add_plugins(garbage::GarbagePlugin);
    app.add_plugins(districts::DistrictsPlugin);
    app.add_plugins(district_policies::DistrictPoliciesPlugin);
    app.add_plugins(policy_effects::PolicyTradeoffsPlugin);
    app.add_plugins(districts_save::DistrictSavePlugin);
    app.add_plugins(superblock::SuperblockPlugin);
    app.add_plugins(superblock_policy::SuperblockPolicyPlugin);
    app.add_plugins(neighborhood_quality::NeighborhoodQualityPlugin);
    app.add_plugins(lifecycle::LifecyclePlugin);
    app.add_plugins(building_upgrade::BuildingUpgradePlugin);
    app.add_plugins(imports_exports::ImportsExportsPlugin);
    app.add_plugins(historic_preservation::HistoricPreservationPlugin);
    app.add_plugins(inclusionary_zoning::InclusionaryZoningPlugin);
    app.add_plugins(far_transfer::FarTransferPlugin);

    // Waste and recycling
    app.add_plugins(waste_effects::WasteEffectsPlugin);
    app.add_plugins(recycling::RecyclingPlugin);
    app.add_plugins(road_maintenance::RoadMaintenancePlugin);
    app.add_plugins(road_upgrade::RoadUpgradePlugin);
    app.add_plugins(curve_road_drawing::CurveRoadDrawingPlugin);
    app.add_plugins(oneway::OneWayPlugin);
    app.add_plugins(traffic_accidents::TrafficAccidentsPlugin);
    app.add_plugins(traffic_congestion::TrafficCongestionPlugin);
    app.add_plugins(traffic_los::TrafficLosPlugin);
    app.add_plugins(road_hierarchy::RoadHierarchyPlugin);
    app.add_plugins(bus_transit::BusTransitPlugin);
    app.add_plugins(transit_hub::TransitHubPlugin);
    app.add_plugins(loans::LoansPlugin);
    app.add_plugins(bulldoze_refund::BulldozeRefundPlugin);
    app.add_plugins(roundabout::RoundaboutPlugin);

    // Day/night visual controls
    app.add_plugins(day_night_controls::DayNightControlsPlugin);

    // Weather and environment
    app.add_plugins(weather::WeatherPlugin);
    app.add_plugins(fog::FogPlugin);
    app.add_plugins(degree_days::DegreeDaysPlugin);
    app.add_plugins(energy_demand::EnergyDemandPlugin);
    app.add_plugins(coal_power::CoalPowerPlugin);
    app.add_plugins(gas_power::GasPowerPlugin);
    app.add_plugins(energy_dispatch::EnergyDispatchPlugin);
    app.add_plugins(blackout::BlackoutPlugin);
    app.add_plugins(battery_storage::BatteryStoragePlugin);
    app.add_plugins(heating::HeatingPlugin);
    app.add_plugins(heating_service::HeatingServicePlugin);
    app.add_plugins(wind::WindPlugin);
    app.add_plugins(wind_damage::WindDamagePlugin);
    app.add_plugins(wind_power::WindPowerPlugin);
    app.add_plugins(urban_heat_island::UrbanHeatIslandPlugin);
    app.add_plugins(uhi_mitigation::UhiMitigationPlugin);
    app.add_plugins(drought::DroughtPlugin);
    app.add_plugins(noise::NoisePlugin);
    app.add_plugins(noise_effects::NoiseEffectsPlugin);
    app.add_plugins(crime::CrimePlugin);
    app.add_plugins(crime_justice::CrimeJusticePlugin);
    app.add_plugins(police_tiers::PoliceTiersPlugin);
    app.add_plugins(health::HealthPlugin);
    app.add_plugins(disease_model::DiseaseModelPlugin);
    app.add_plugins(death_care::DeathCarePlugin);
    app.add_plugins(deathcare_capacity::DeathCareCapacityPlugin);
    app.add_plugins(climate_change::ClimateChangePlugin);
    app.add_plugins(seasonal_rendering::SeasonalRenderingPlugin);

    // Water systems
    app.add_plugins(water_pollution::WaterPollutionPlugin);
    app.add_plugins(water_pollution_sources::WaterPollutionSourcesPlugin);
    app.add_plugins(groundwater::GroundwaterPlugin);
    app.add_plugins(stormwater::StormwaterPlugin);
    app.add_plugins(water_demand::WaterDemandPlugin);
    app.add_plugins(heat_wave::HeatWavePlugin);
    app.add_plugins(heat_mitigation::HeatMitigationPlugin);
    app.add_plugins(composting::CompostingPlugin);
    app.add_plugins(cold_snap::ColdSnapPlugin);
    app.add_plugins(cso::CsoPlugin);
    app.add_plugins(water_treatment::WaterTreatmentPlugin);
    app.add_plugins(water_conservation::WaterConservationPlugin);
    app.add_plugins(water_pressure::WaterPressurePlugin);
    app.add_plugins(groundwater_depletion::GroundwaterDepletionPlugin);
    app.add_plugins(wastewater::WastewaterPlugin);
    app.add_plugins(water_quality_effects::WaterQualityEffectsPlugin);

    // Waste management
    app.add_plugins(hazardous_waste::HazardousWastePlugin);
    app.add_plugins(landfill::LandfillPlugin);
    app.add_plugins(landfill_gas::LandfillGasPlugin);
    app.add_plugins(landfill_warning::LandfillWarningPlugin);
    app.add_plugins(waste_policies::WastePoliciesPlugin);

    // Infrastructure and resources
    app.add_plugins(storm_drainage::StormDrainagePlugin);
    app.add_plugins(water_sources::WaterSourcesPlugin);
    app.add_plugins(natural_resources::NaturalResourcesPlugin);
    app.add_plugins(wealth::WealthPlugin);
    app.add_plugins(tourism::TourismPlugin);
    app.add_plugins(hotel_demand::HotelDemandPlugin);
    app.add_plugins(unlocks::UnlocksPlugin);
    app.add_plugins(milestones::MilestonesPlugin);
    app.add_plugins(reservoir::ReservoirPlugin);
    app.add_plugins(flood_simulation::FloodSimulationPlugin);
    app.add_plugins(flood_protection::FloodProtectionPlugin);
    app.add_plugins(trees::TreesPlugin);
    app.add_plugins(airport::AirportPlugin);
    app.add_plugins(metro_transit::MetroTransitPlugin);
    app.add_plugins(train_transit::TrainTransitPlugin);
    app.add_plugins(snow::SnowPlugin);
    app.add_plugins(solar_power::SolarPowerPlugin);

    // Transit and connections
    app.add_plugins(tram_transit::TramTransitPlugin);
    app.add_plugins(outside_connections::OutsideConnectionsPlugin);

    // Production and economy
    app.add_plugins(agriculture::AgriculturePlugin);
    app.add_plugins(production::ProductionPlugin);
    app.add_plugins(market::MarketPlugin);
    app.add_plugins(events::EventsPlugin);
    app.add_plugins(event_journal_save::EventJournalSavePlugin);
    app.add_plugins(notifications::NotificationsPlugin);
    app.add_plugins(specialization::SpecializationPlugin);
    app.add_plugins(specialization_save::SpecializationSavePlugin);
    app.add_plugins(advisors::AdvisorsPlugin);
    app.add_plugins(achievements::AchievementsPlugin);
    app.add_plugins(freight_traffic::FreightTrafficPlugin);

    // Building lifecycle and disasters
    app.add_plugins(abandonment::AbandonmentPlugin);
    app.add_plugins(fire::FirePlugin);
    app.add_plugins(fire_tiers::FireTiersPlugin);
    app.add_plugins(forest_fire::ForestFirePlugin);
    app.add_plugins(disasters::DisastersPlugin);
    app.add_plugins(disaster_save::DisasterSavePlugin);
    app.add_plugins(emergency_management::EmergencyManagementPlugin);

    // Citizens and population
    app.add_plugins(life_simulation::LifeSimulationPlugin);
    app.add_plugins(homelessness::HomelessnessPlugin);
    app.add_plugins(welfare::WelfarePlugin);
    app.add_plugins(daycare_eldercare::DaycareEldercarePlugin);
    app.add_plugins(immigration::ImmigrationPlugin);
    app.add_plugins(population_tiers::PopulationTiersPlugin);
    app.add_plugins(lod::LodPlugin);
    app.add_plugins(virtual_population::VirtualPopulationPlugin);
    app.add_plugins(virtual_population_save::VirtualPopulationSavePlugin);
    app.add_plugins(urban_growth_boundary::UrbanGrowthBoundaryPlugin);
    app.add_plugins(nimby::NimbyPlugin);
    app.add_plugins(walkability::WalkabilityPlugin);
    app.add_plugins(form_transect::FormTransectPlugin);
    app.add_plugins(cumulative_zoning::CumulativeZoningPlugin);
    app.add_plugins(parking::ParkingPlugin);
    app.add_plugins(tutorial::TutorialPlugin);
    app.add_plugins(multi_select::MultiSelectPlugin);
    app.add_plugins(blueprints::BlueprintPlugin);
    app.add_plugins(simulation_invariants::SimulationInvariantsPlugin);

    // Mode choice (TRAF-007)
    app.add_plugins(mode_choice::ModeChoicePlugin);

    // Localization infrastructure
    app.add_plugins(localization::LocalizationPlugin);

    // Accessibility
    app.add_plugins(colorblind::ColorblindPlugin);

    // Customizable keybindings
    app.add_plugins(keybindings::KeyBindingsPlugin);

    // Freehand road drawing (UX-020)
    app.add_plugins(freehand_road::FreehandRoadPlugin);

    // Auto-grid road placement (TRAF-010)
    app.add_plugins(auto_grid_road::AutoGridRoadPlugin);

    // Undo/redo system
    app.add_plugins(undo_redo::UndoRedoPlugin);

    // Environmental grid save/load (POLL-033)
    app.add_plugins(env_grid_save::EnvGridSavePlugin);

    // Social services building types (SVC-013)
    app.add_plugins(social_services::SocialServicesPlugin);
    // Wind-aware Gaussian plume pollution dispersion (SVC-021)
    app.add_plugins(wind_pollution::WindPollutionPlugin);

    // Heating grid save/load (SAVE-040)
    app.add_plugins(heating_save::HeatingSavePlugin);

    // Post-load commuting citizen reset (SAVE-008)
    app.add_plugins(reset_commuting_on_load::ResetCommutingOnLoadPlugin);

    // Fire grid save/load (SAVE-037)
    app.add_plugins(fire_grid_save::FireGridSavePlugin);

    // Autosave with configurable interval (SAVE-002)
    app.add_plugins(autosave::AutosavePlugin);

    // Environmental Score aggregate metric (POLL-021)
    app.add_plugins(environmental_score::EnvironmentalScorePlugin);

    // Time-of-Use Electricity Pricing (POWER-010)
    app.add_plugins(energy_pricing::EnergyPricingPlugin);

    // Power line transmission and service radius (POWER-011)
    app.add_plugins(power_lines::PowerLinePlugin);

    // Waste-to-Energy power plant (POWER-014)
    app.add_plugins(waste_to_energy::WtePlugin);
    // Coverage metrics precomputed for UI (PERF-001)
    app.add_plugins(coverage_metrics::CoverageMetricsPlugin);

    // Play time tracking for save metadata (SAVE-020)
    app.add_plugins(play_time::PlayTimePlugin);

    // Bevy diagnostics and trace spans (TEST-031)
    app.add_plugins(diagnostics::DiagnosticsPlugin);

    // Post-load derived state rebuild (SAVE-026)
    app.add_plugins(post_load_rebuild::PostLoadRebuildPlugin);
}
