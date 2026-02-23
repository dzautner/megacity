// ---------------------------------------------------------------------------
// Restore functions for weather and climate: weather, climate zone, degree days,
// construction modifiers, wind damage, drought, heat wave, cold snap, fog, snow
// ---------------------------------------------------------------------------

use crate::save_codec::*;
use crate::save_types::*;

use simulation::degree_days::DegreeDays;
use simulation::fog::FogState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;

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

/// Restore a `WindDamageState` resource from saved data.
pub fn restore_wind_damage_state(save: &SaveWindDamageState) -> WindDamageState {
    WindDamageState {
        current_tier: u8_to_wind_damage_tier(save.current_tier),
        accumulated_building_damage: save.accumulated_building_damage,
        trees_knocked_down: save.trees_knocked_down,
        power_outage_active: save.power_outage_active,
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

/// Restore a `FogState` resource from saved data.
pub fn restore_fog_state(save: &SaveFogState) -> FogState {
    FogState {
        active: save.active,
        density: u8_to_fog_density(save.density),
        visibility_m: save.visibility_m,
        hours_active: save.hours_active,
        max_duration_hours: save.max_duration_hours,
        water_fraction: save.water_fraction,
        traffic_speed_modifier: save.traffic_speed_modifier,
        flights_suspended: save.flights_suspended,
        last_update_hour: save.last_update_hour,
        water_fraction_last_day: 0, // Will be recomputed on next day
    }
}

/// Restore `SnowGrid` and `SnowPlowingState` from saved data.
pub fn restore_snow(
    state: &SaveSnowState,
) -> (
    simulation::snow::SnowGrid,
    simulation::snow::SnowPlowingState,
) {
    let grid = simulation::snow::SnowGrid {
        depths: state.depths.clone(),
        width: state.width,
        height: state.height,
    };
    let plowing = simulation::snow::SnowPlowingState {
        enabled: state.plowing_enabled,
        season_cost: state.season_cost,
        cells_plowed_season: state.cells_plowed_season,
        cells_plowed_last: 0,
        last_plow_cost: 0.0,
    };
    (grid, plowing)
}
