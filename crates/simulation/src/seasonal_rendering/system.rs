//! ECS system and plugin for seasonal rendering updates.
//!
//! The `update_seasonal_rendering` system runs every slow tick, reading the
//! current `Weather` resource to derive which effects should be active and
//! at what intensity.

use bevy::prelude::*;

use crate::grid::WorldGrid;
use crate::snow::SnowGrid;
use crate::trees::TreeGrid;
use crate::weather::{Season, Weather};
use crate::SlowTickTimer;

use super::compute::*;
use super::constants::LIGHTNING_FLASH_DURATION;
use super::types::{condition_to_id, season_to_id, SeasonalEffectsConfig, SeasonalRenderingState};

/// Main seasonal rendering update system. Runs every slow tick.
///
/// Reads current weather, season, snow depth, and grid state to compute
/// rendering effect intensities for the rendering layer.
#[allow(clippy::too_many_arguments)]
pub fn update_seasonal_rendering(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    world_grid: Res<WorldGrid>,
    tree_grid: Res<TreeGrid>,
    snow_grid: Res<SnowGrid>,
    config: Res<SeasonalEffectsConfig>,
    mut state: ResMut<SeasonalRenderingState>,
    tick_counter: Res<crate::TickCounter>,
) {
    if !timer.should_run() {
        return;
    }

    let season = weather.season;

    // --- Autumn: falling leaves ---
    state.leaf_intensity =
        compute_leaf_intensity(state.leaf_intensity, season, config.leaves_enabled);
    state.leaf_source_cells = if config.leaves_enabled && season == Season::Autumn {
        count_tree_cells(&tree_grid)
    } else if state.leaf_intensity > 0.0 {
        // Still decaying, keep last count
        state.leaf_source_cells
    } else {
        0
    };

    // --- Winter: snow on roofs ---
    let avg_snow = snow_grid.average_depth();
    state.snow_roof_intensity = compute_snow_roof_intensity(
        state.snow_roof_intensity,
        &weather,
        avg_snow,
        config.snow_roofs_enabled,
    );
    state.snow_roof_cells = if config.snow_roofs_enabled
        && (season == Season::Winter || state.snow_roof_intensity > 0.0)
    {
        count_building_cells(&world_grid)
    } else {
        0
    };

    // --- Winter: snowflake particles ---
    state.snowflake_intensity = compute_snowflake_intensity(&weather, config.snowflakes_enabled);

    // --- Spring: flower particles ---
    state.flower_intensity =
        compute_flower_intensity(state.flower_intensity, season, config.flowers_enabled);
    state.flower_source_cells = if config.flowers_enabled && season == Season::Spring {
        count_flower_cells(&world_grid, &tree_grid)
    } else if state.flower_intensity > 0.0 {
        state.flower_source_cells
    } else {
        0
    };

    // --- Spring: brightness boost ---
    state.spring_brightness = compute_spring_brightness(season, config.spring_brightness_enabled);

    // --- Summer: heat shimmer ---
    state.heat_shimmer_intensity =
        compute_heat_shimmer_intensity(weather.temperature, season, config.heat_shimmer_enabled);

    // --- Summer: longer shadows ---
    state.shadow_multiplier = compute_shadow_multiplier(season, config.summer_shadows_enabled);

    // --- Rain: rain streaks ---
    state.rain_streak_intensity = compute_rain_intensity(&weather, config.rain_streaks_enabled);

    // --- Storm: sky darkening ---
    state.storm_darkening = compute_storm_darkening(
        state.storm_darkening,
        &weather,
        config.storm_effects_enabled,
    );

    // --- Storm: lightning flashes ---
    if state.lightning_timer > 0 {
        state.lightning_timer -= 1;
        state.lightning_active = state.lightning_timer > 0;
    } else {
        let tick_hash = tick_counter.0 as u32;
        if should_trigger_lightning(&weather, tick_hash, config.storm_effects_enabled) {
            state.lightning_active = true;
            state.lightning_timer = LIGHTNING_FLASH_DURATION;
        } else {
            state.lightning_active = false;
        }
    }

    // --- Metadata ---
    state.current_season_id = season_to_id(season);
    state.current_condition_id = condition_to_id(weather.current_event);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SeasonalRenderingPlugin;

impl Plugin for SeasonalRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SeasonalRenderingState>()
            .init_resource::<SeasonalEffectsConfig>()
            .add_systems(
                FixedUpdate,
                update_seasonal_rendering
                    .after(crate::weather::update_weather)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        let world = app.world_mut();
        let mut registry = world.resource_mut::<crate::SaveableRegistry>();
        registry.register::<SeasonalRenderingState>();
        registry.register::<SeasonalEffectsConfig>();
    }
}
