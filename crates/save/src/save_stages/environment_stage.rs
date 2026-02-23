use crate::save_codec::*;
use crate::save_types::*;

use simulation::agriculture::AgricultureState;
use simulation::degree_days::DegreeDays;
use simulation::fog::FogState;
use simulation::snow::{SnowGrid, SnowPlowingState};
use simulation::stormwater::StormwaterGrid;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};

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
