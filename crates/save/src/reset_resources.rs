use bevy::prelude::*;

use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::cold_snap::ColdSnapState;
use simulation::composting::CompostingState;
use simulation::cso::SewerSystemState;
use simulation::degree_days::DegreeDays;
use simulation::drought::DroughtState;
use simulation::economy::CityBudget;
use simulation::flood_simulation::{FloodGrid, FloodState};
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
use simulation::snow::{SnowGrid, SnowPlowingState, SnowStats};
use simulation::storm_drainage::StormDrainageState;
use simulation::stormwater::StormwaterGrid;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::virtual_population::VirtualPopulation;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_treatment::WaterTreatmentState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;
use simulation::zones::ZoneDemand;

pub(crate) fn reset_all_resources(world: &mut World) {
    {
        let (width, height) = {
            let grid = world.resource::<WorldGrid>();
            (grid.width, grid.height)
        };
        *world.resource_mut::<WorldGrid>() = WorldGrid::new(width, height);
    }
    *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();
    *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();

    // Reset clock
    {
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = 1;
        clock.hour = 8.0;
        clock.speed = 1.0;
        clock.paused = false;
    }

    // Reset budget
    {
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = 50_000.0;
        budget.tax_rate = 0.10;
        budget.last_collection_day = 0;
    }

    // Reset demand
    *world.resource_mut::<ZoneDemand>() = ZoneDemand::default();

    // Reset V2 resources
    *world.resource_mut::<Policies>() = Policies::default();
    *world.resource_mut::<Weather>() = Weather::default();
    *world.resource_mut::<ClimateZone>() = ClimateZone::default();
    *world.resource_mut::<UnlockState>() = UnlockState::default();
    *world.resource_mut::<ExtendedBudget>() = ExtendedBudget::default();
    *world.resource_mut::<LoanBook>() = LoanBook::default();
    *world.resource_mut::<VirtualPopulation>() = VirtualPopulation::default();
    *world.resource_mut::<LifecycleTimer>() = LifecycleTimer::default();
    *world.resource_mut::<LifeSimTimer>() = LifeSimTimer::default();
    *world.resource_mut::<StormwaterGrid>() = StormwaterGrid::default();
    *world.resource_mut::<DegreeDays>() = DegreeDays::default();
    *world.resource_mut::<ConstructionModifiers>() = ConstructionModifiers::default();
    *world.resource_mut::<RecyclingState>() = RecyclingState::default();
    *world.resource_mut::<RecyclingEconomics>() = RecyclingEconomics::default();
    *world.resource_mut::<WindDamageState>() = WindDamageState::default();
    *world.resource_mut::<UhiGrid>() = UhiGrid::default();
    *world.resource_mut::<DroughtState>() = DroughtState::default();
    *world.resource_mut::<HeatWaveState>() = HeatWaveState::default();
    *world.resource_mut::<CompostingState>() = CompostingState::default();
    *world.resource_mut::<ColdSnapState>() = ColdSnapState::default();
    *world.resource_mut::<WaterTreatmentState>() = WaterTreatmentState::default();
    *world.resource_mut::<GroundwaterDepletionState>() = GroundwaterDepletionState::default();
    *world.resource_mut::<WastewaterState>() = WastewaterState::default();
    *world.resource_mut::<HazardousWasteState>() = HazardousWasteState::default();
    *world.resource_mut::<StormDrainageState>() = StormDrainageState::default();
    *world.resource_mut::<LandfillCapacityState>() = LandfillCapacityState::default();
    *world.resource_mut::<FloodState>() = FloodState::default();
    *world.resource_mut::<FloodGrid>() = FloodGrid::default();
    *world.resource_mut::<ReservoirState>() = ReservoirState::default();
    *world.resource_mut::<LandfillGasState>() = LandfillGasState::default();
    *world.resource_mut::<SewerSystemState>() = SewerSystemState::default();
    *world.resource_mut::<WaterConservationState>() = WaterConservationState::default();
    *world.resource_mut::<FogState>() = FogState::default();
    *world.resource_mut::<UrbanGrowthBoundary>() = UrbanGrowthBoundary::default();
    *world.resource_mut::<SnowGrid>() = SnowGrid::default();
    *world.resource_mut::<SnowPlowingState>() = SnowPlowingState::default();
    *world.resource_mut::<SnowStats>() = SnowStats::default();
    *world.resource_mut::<AgricultureState>() = AgricultureState::default();
}
