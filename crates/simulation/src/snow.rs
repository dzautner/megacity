//! Snow accumulation, melting, and plowing system (WEATHER-014).
//!
//! During winter precipitation events when temperature < 0C (32F), snow
//! accumulates on the grid. Snow affects traffic speed, heating demand,
//! and visual rendering. A snow plowing service clears roads at a cost,
//! prioritizing highways > arterials > local roads.
//!
//! The `SnowGrid` resource tracks per-cell snow depth in inches. The
//! `SnowPlowingState` resource tracks plowing service state and costs.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::weather::{Weather, WeatherCondition};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Freezing point in Celsius. Snow accumulates when temperature is below this.
const FREEZING_POINT_C: f32 = 0.0;

/// Snow accumulation rate per slow tick during snow events (inches per tick).
/// Scales with precipitation intensity.
const BASE_SNOW_ACCUMULATION_RATE: f32 = 0.5;

/// Snow melt rate per degree Celsius above freezing per slow tick (inches per tick per degree).
const MELT_RATE_PER_DEGREE: f32 = 0.1;

/// Maximum snow depth in inches for gameplay purposes.
const MAX_SNOW_DEPTH: f32 = 24.0;

/// Speed reduction per inch of snow on roads (fraction).
/// Total reduction is clamped at MAX_SNOW_SPEED_REDUCTION.
const SPEED_REDUCTION_PER_INCH: f32 = 0.05;

/// Maximum speed reduction from snow on roads (fraction of normal speed lost).
/// At 12+ inches, roads are at maximum slowdown (80% reduction).
const MAX_SNOW_SPEED_REDUCTION: f32 = 0.80;

/// Heating demand increase per 6 inches of snow (fraction, i.e. 0.10 = +10%).
const HEATING_INCREASE_PER_6_INCHES: f32 = 0.10;

/// Cost per road cell per plowing event (dollars).
const PLOW_COST_PER_CELL: f64 = 500.0;

/// Amount of snow removed per plowing pass (inches).
const PLOW_REMOVAL_DEPTH: f32 = 6.0;

/// Threshold snow depth (inches) above which plowing is triggered on roads.
const PLOW_TRIGGER_DEPTH: f32 = 2.0;

/// Snowmelt contribution to stormwater runoff per inch melted (arbitrary units).
/// Used for spring flooding risk integration.
const SNOWMELT_RUNOFF_FACTOR: f32 = 0.5;

// =============================================================================
// Resources
// =============================================================================

/// Per-cell snow depth grid (inches). 0.0 = no snow.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnowGrid {
    pub depths: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for SnowGrid {
    fn default() -> Self {
        Self {
            depths: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl SnowGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.depths[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        self.depths[y * self.width + x] = val;
    }

    /// Average snow depth across all cells (for stats/UI).
    pub fn average_depth(&self) -> f32 {
        if self.depths.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.depths.iter().sum();
        sum / self.depths.len() as f32
    }

    /// Number of cells with snow depth > 0.
    pub fn covered_cells(&self) -> u32 {
        self.depths.iter().filter(|&&d| d > 0.0).count() as u32
    }

    /// Maximum snow depth across all cells.
    pub fn max_depth(&self) -> f32 {
        self.depths.iter().copied().fold(0.0_f32, f32::max)
    }
}

/// Aggregate snow plowing service state and statistics.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnowPlowingState {
    /// Whether the snow plowing service is enabled (player can toggle).
    pub enabled: bool,
    /// Total cost spent on plowing this season.
    pub season_cost: f64,
    /// Number of cells plowed this season.
    pub cells_plowed_season: u32,
    /// Number of cells plowed in the most recent plowing pass.
    pub cells_plowed_last: u32,
    /// Cost of the most recent plowing pass.
    pub last_plow_cost: f64,
}

impl Default for SnowPlowingState {
    fn default() -> Self {
        Self {
            enabled: true,
            season_cost: 0.0,
            cells_plowed_season: 0,
            cells_plowed_last: 0,
            last_plow_cost: 0.0,
        }
    }
}

/// Aggregate snow statistics for the UI.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnowStats {
    /// Average snow depth across all cells (inches).
    pub avg_depth: f32,
    /// Maximum snow depth on any cell (inches).
    pub max_depth: f32,
    /// Number of cells covered with snow.
    pub covered_cells: u32,
    /// Current travel speed multiplier due to snow on roads (1.0 = no effect).
    pub road_speed_multiplier: f32,
    /// Current heating demand modifier from snow (1.0 = no effect).
    pub heating_demand_modifier: f32,
    /// Total snowmelt runoff contribution this tick (for flooding).
    pub snowmelt_runoff: f32,
}

impl Default for SnowStats {
    fn default() -> Self {
        Self {
            avg_depth: 0.0,
            max_depth: 0.0,
            covered_cells: 0,
            road_speed_multiplier: 1.0,
            heating_demand_modifier: 1.0,
            snowmelt_runoff: 0.0,
        }
    }
}

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Calculate the snow accumulation amount for this tick given weather conditions.
/// Returns inches of snow to add. Only accumulates during snow events below freezing.
pub fn snow_accumulation_amount(weather: &Weather) -> f32 {
    if weather.temperature >= FREEZING_POINT_C {
        return 0.0;
    }
    match weather.current_event {
        WeatherCondition::Snow => {
            // Scale accumulation with precipitation intensity
            let intensity_factor = (weather.precipitation_intensity * 2.0).clamp(0.5, 3.0);
            BASE_SNOW_ACCUMULATION_RATE * intensity_factor
        }
        WeatherCondition::Storm if weather.temperature < FREEZING_POINT_C => {
            // Heavy snow during storms
            let intensity_factor = (weather.precipitation_intensity * 2.0).clamp(1.0, 4.0);
            BASE_SNOW_ACCUMULATION_RATE * intensity_factor
        }
        _ => 0.0,
    }
}

/// Calculate the snow melt amount for this tick given temperature.
/// Returns inches of snow to remove. Only melts when above freezing.
pub fn snow_melt_amount(temperature: f32) -> f32 {
    if temperature <= FREEZING_POINT_C {
        return 0.0;
    }
    let excess = temperature - FREEZING_POINT_C;
    excess * MELT_RATE_PER_DEGREE
}

/// Calculate the travel speed multiplier for roads based on average road snow depth.
/// Returns a multiplier in [0.2, 1.0] where 1.0 = no snow effect.
pub fn snow_speed_multiplier(avg_road_snow_depth: f32) -> f32 {
    if avg_road_snow_depth <= 0.0 {
        return 1.0;
    }
    let reduction = (avg_road_snow_depth * SPEED_REDUCTION_PER_INCH).min(MAX_SNOW_SPEED_REDUCTION);
    (1.0 - reduction).max(0.2)
}

/// Calculate the heating demand modifier from snow depth.
/// Returns a multiplier >= 1.0 where 1.0 = no snow effect.
/// Each 6 inches of snow adds 10% heating demand.
pub fn snow_heating_modifier(avg_snow_depth: f32) -> f32 {
    if avg_snow_depth <= 0.0 {
        return 1.0;
    }
    let increments = avg_snow_depth / 6.0;
    1.0 + increments * HEATING_INCREASE_PER_6_INCHES
}

/// Road plowing priority: highways first, then boulevards/avenues, then local roads.
/// Returns a priority value (lower = higher priority, plowed first).
fn plow_priority(road_type: RoadType) -> u8 {
    match road_type {
        RoadType::Highway => 0,
        RoadType::Boulevard => 1,
        RoadType::Avenue => 2,
        RoadType::OneWay => 3,
        RoadType::Local => 4,
        RoadType::Path => 5, // Paths are not plowed (pedestrian only)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Main snow accumulation and melt system. Runs every slow tick.
///
/// 1. If snowing and below freezing: accumulate snow on all non-water cells.
/// 2. If above freezing: melt snow proportional to temperature excess.
/// 3. Track snowmelt runoff for spring flooding integration.
/// 4. Update aggregate statistics.
#[allow(clippy::too_many_arguments)]
pub fn update_snow(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    grid: Res<WorldGrid>,
    mut snow_grid: ResMut<SnowGrid>,
    mut stats: ResMut<SnowStats>,
    mut stormwater: ResMut<crate::stormwater::StormwaterGrid>,
) {
    if !timer.should_run() {
        return;
    }

    let accumulation = snow_accumulation_amount(&weather);
    let melt = snow_melt_amount(weather.temperature);
    let mut total_melt_runoff = 0.0_f32;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            // Don't accumulate snow on water
            if cell.cell_type == CellType::Water {
                snow_grid.set(x, y, 0.0);
                continue;
            }

            let current = snow_grid.get(x, y);

            // Accumulate snow
            let after_accumulation = if accumulation > 0.0 {
                (current + accumulation).min(MAX_SNOW_DEPTH)
            } else {
                current
            };

            // Melt snow
            let after_melt = if melt > 0.0 && after_accumulation > 0.0 {
                let melted = melt.min(after_accumulation);
                total_melt_runoff += melted;
                (after_accumulation - melted).max(0.0)
            } else {
                after_accumulation
            };

            snow_grid.set(x, y, after_melt);
        }
    }

    // Contribute snowmelt to stormwater runoff (for spring flooding risk)
    if total_melt_runoff > 0.0 {
        let melt_per_cell =
            total_melt_runoff * SNOWMELT_RUNOFF_FACTOR / (GRID_WIDTH * GRID_HEIGHT) as f32;
        stormwater.total_runoff += total_melt_runoff * SNOWMELT_RUNOFF_FACTOR;
        // Distribute melt runoff across all non-water cells
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if grid.get(x, y).cell_type != CellType::Water {
                    let idx = y * stormwater.width + x;
                    if idx < stormwater.runoff.len() {
                        stormwater.runoff[idx] += melt_per_cell;
                    }
                }
            }
        }
    }

    // Compute average road snow depth for speed calculations
    let mut road_snow_sum = 0.0_f32;
    let mut road_count = 0u32;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                road_snow_sum += snow_grid.get(x, y);
                road_count += 1;
            }
        }
    }
    let avg_road_snow = if road_count > 0 {
        road_snow_sum / road_count as f32
    } else {
        0.0
    };

    // Update stats
    stats.avg_depth = snow_grid.average_depth();
    stats.max_depth = snow_grid.max_depth();
    stats.covered_cells = snow_grid.covered_cells();
    stats.road_speed_multiplier = snow_speed_multiplier(avg_road_snow);
    stats.heating_demand_modifier = snow_heating_modifier(stats.avg_depth);
    stats.snowmelt_runoff = total_melt_runoff * SNOWMELT_RUNOFF_FACTOR;
}

/// Snow plowing system. Runs every slow tick.
///
/// When plowing is enabled and road snow depth exceeds the trigger threshold,
/// plows roads in priority order: highways > boulevards > avenues > local roads.
/// Each plowing pass removes PLOW_REMOVAL_DEPTH inches and costs PLOW_COST_PER_CELL.
pub fn update_snow_plowing(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    mut snow_grid: ResMut<SnowGrid>,
    mut plowing: ResMut<SnowPlowingState>,
    mut budget: ResMut<CityBudget>,
) {
    if !timer.should_run() {
        return;
    }

    if !plowing.enabled {
        plowing.cells_plowed_last = 0;
        plowing.last_plow_cost = 0.0;
        return;
    }

    // Collect road cells that need plowing, sorted by priority
    let mut cells_to_plow: Vec<(usize, usize, u8)> = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                continue;
            }
            // Don't plow pedestrian paths
            if cell.road_type == RoadType::Path {
                continue;
            }
            let snow_depth = snow_grid.get(x, y);
            if snow_depth >= PLOW_TRIGGER_DEPTH {
                cells_to_plow.push((x, y, plow_priority(cell.road_type)));
            }
        }
    }

    if cells_to_plow.is_empty() {
        plowing.cells_plowed_last = 0;
        plowing.last_plow_cost = 0.0;
        return;
    }

    // Sort by priority (lower = higher priority)
    cells_to_plow.sort_by_key(|&(_, _, priority)| priority);

    let mut plowed_count = 0u32;
    let mut plow_cost = 0.0_f64;

    for (x, y, _) in &cells_to_plow {
        let current = snow_grid.get(*x, *y);
        let new_depth = (current - PLOW_REMOVAL_DEPTH).max(0.0);
        snow_grid.set(*x, *y, new_depth);
        plowed_count += 1;
        plow_cost += PLOW_COST_PER_CELL;
    }

    // Deduct cost from city treasury
    budget.treasury -= plow_cost;

    // Update plowing stats
    plowing.cells_plowed_last = plowed_count;
    plowing.last_plow_cost = plow_cost;
    plowing.cells_plowed_season += plowed_count;
    plowing.season_cost += plow_cost;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // SnowGrid tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snow_grid_default() {
        let grid = SnowGrid::default();
        assert_eq!(grid.depths.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(grid.get(0, 0), 0.0);
        assert_eq!(grid.average_depth(), 0.0);
        assert_eq!(grid.covered_cells(), 0);
        assert_eq!(grid.max_depth(), 0.0);
    }

    #[test]
    fn test_snow_grid_set_get() {
        let mut grid = SnowGrid::default();
        grid.set(10, 10, 5.0);
        assert_eq!(grid.get(10, 10), 5.0);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_snow_grid_covered_cells() {
        let mut grid = SnowGrid::default();
        grid.set(0, 0, 1.0);
        grid.set(1, 0, 2.0);
        grid.set(2, 0, 3.0);
        assert_eq!(grid.covered_cells(), 3);
    }

    #[test]
    fn test_snow_grid_max_depth() {
        let mut grid = SnowGrid::default();
        grid.set(5, 5, 8.0);
        grid.set(10, 10, 12.0);
        grid.set(15, 15, 4.0);
        assert_eq!(grid.max_depth(), 12.0);
    }

    // -------------------------------------------------------------------------
    // Snow accumulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_accumulation_above_freezing() {
        let mut weather = Weather::default();
        weather.temperature = 5.0;
        weather.current_event = WeatherCondition::Snow;
        assert_eq!(snow_accumulation_amount(&weather), 0.0);
    }

    #[test]
    fn test_accumulation_during_snow() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Snow;
        weather.precipitation_intensity = 0.2;
        let amount = snow_accumulation_amount(&weather);
        assert!(
            amount > 0.0,
            "snow should accumulate during Snow event below freezing"
        );
    }

    #[test]
    fn test_accumulation_during_storm_below_freezing() {
        let mut weather = Weather::default();
        weather.temperature = -10.0;
        weather.current_event = WeatherCondition::Storm;
        weather.precipitation_intensity = 1.0;
        let amount = snow_accumulation_amount(&weather);
        assert!(
            amount > 0.0,
            "snow should accumulate during Storm below freezing"
        );
    }

    #[test]
    fn test_accumulation_during_rain() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Rain;
        let amount = snow_accumulation_amount(&weather);
        assert_eq!(amount, 0.0, "rain should not cause snow accumulation");
    }

    #[test]
    fn test_accumulation_sunny() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Sunny;
        let amount = snow_accumulation_amount(&weather);
        assert_eq!(amount, 0.0, "sunny weather should not cause snow");
    }

    #[test]
    fn test_accumulation_scales_with_intensity() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Snow;

        weather.precipitation_intensity = 0.1;
        let low = snow_accumulation_amount(&weather);

        weather.precipitation_intensity = 1.0;
        let high = snow_accumulation_amount(&weather);

        assert!(high > low, "higher precipitation should produce more snow");
    }

    // -------------------------------------------------------------------------
    // Snow melt tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_melt_below_freezing() {
        assert_eq!(snow_melt_amount(-5.0), 0.0);
        assert_eq!(snow_melt_amount(0.0), 0.0);
    }

    #[test]
    fn test_melt_above_freezing() {
        let melt = snow_melt_amount(5.0);
        assert!(melt > 0.0, "snow should melt above freezing");
    }

    #[test]
    fn test_melt_proportional_to_temperature() {
        let melt_5 = snow_melt_amount(5.0);
        let melt_10 = snow_melt_amount(10.0);
        assert!(melt_10 > melt_5, "higher temperature should melt more snow");
        assert!(
            (melt_10 - melt_5 * 2.0).abs() < f32::EPSILON,
            "melt should be proportional to temperature excess"
        );
    }

    // -------------------------------------------------------------------------
    // Speed multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_speed_no_snow() {
        assert_eq!(snow_speed_multiplier(0.0), 1.0);
    }

    #[test]
    fn test_speed_some_snow() {
        let mult = snow_speed_multiplier(4.0);
        // 4 inches * 0.05 = 0.20 reduction => 0.80 multiplier
        assert!((mult - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_heavy_snow() {
        let mult = snow_speed_multiplier(12.0);
        // 12 inches * 0.05 = 0.60 reduction => 0.40 multiplier
        assert!((mult - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_extreme_snow_capped() {
        let mult = snow_speed_multiplier(20.0);
        // 20 inches * 0.05 = 1.0 but capped at 0.80 reduction => 0.20 multiplier
        assert!((mult - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_never_below_minimum() {
        let mult = snow_speed_multiplier(100.0);
        assert!(mult >= 0.2, "speed multiplier should never go below 0.2");
    }

    // -------------------------------------------------------------------------
    // Heating modifier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_heating_no_snow() {
        assert_eq!(snow_heating_modifier(0.0), 1.0);
    }

    #[test]
    fn test_heating_6_inches() {
        let modifier = snow_heating_modifier(6.0);
        // 6 inches / 6 = 1 increment * 0.10 + 1.0 = 1.10
        assert!((modifier - 1.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_heating_12_inches() {
        let modifier = snow_heating_modifier(12.0);
        // 12 inches / 6 = 2 increments * 0.10 + 1.0 = 1.20
        assert!((modifier - 1.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_heating_3_inches() {
        let modifier = snow_heating_modifier(3.0);
        // 3 inches / 6 = 0.5 increments * 0.10 + 1.0 = 1.05
        assert!((modifier - 1.05).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Plow priority tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_plow_priority_order() {
        assert!(plow_priority(RoadType::Highway) < plow_priority(RoadType::Boulevard));
        assert!(plow_priority(RoadType::Boulevard) < plow_priority(RoadType::Avenue));
        assert!(plow_priority(RoadType::Avenue) < plow_priority(RoadType::Local));
        assert!(plow_priority(RoadType::Local) < plow_priority(RoadType::Path));
    }

    // -------------------------------------------------------------------------
    // SnowPlowingState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_plowing_state_default() {
        let state = SnowPlowingState::default();
        assert!(state.enabled);
        assert_eq!(state.season_cost, 0.0);
        assert_eq!(state.cells_plowed_season, 0);
        assert_eq!(state.cells_plowed_last, 0);
        assert_eq!(state.last_plow_cost, 0.0);
    }

    // -------------------------------------------------------------------------
    // SnowStats tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snow_stats_default() {
        let stats = SnowStats::default();
        assert_eq!(stats.avg_depth, 0.0);
        assert_eq!(stats.max_depth, 0.0);
        assert_eq!(stats.covered_cells, 0);
        assert_eq!(stats.road_speed_multiplier, 1.0);
        assert_eq!(stats.heating_demand_modifier, 1.0);
        assert_eq!(stats.snowmelt_runoff, 0.0);
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(PLOW_COST_PER_CELL > 0.0);
        assert!(PLOW_REMOVAL_DEPTH > 0.0);
        assert!(PLOW_TRIGGER_DEPTH > 0.0);
        assert!(MAX_SNOW_DEPTH > 0.0);
        assert!(MAX_SNOW_SPEED_REDUCTION > 0.0);
        assert!(MAX_SNOW_SPEED_REDUCTION <= 1.0);
        assert!(SPEED_REDUCTION_PER_INCH > 0.0);
        assert!(HEATING_INCREASE_PER_6_INCHES > 0.0);
        assert!(BASE_SNOW_ACCUMULATION_RATE > 0.0);
        assert!(MELT_RATE_PER_DEGREE > 0.0);
    }

    #[test]
    fn test_max_speed_reduction_depth_consistent() {
        // At 12 inches depth, the reduction from SPEED_REDUCTION_PER_INCH
        // should reach a meaningful fraction of MAX_SNOW_SPEED_REDUCTION
        let reduction_at_max = 12.0_f32 * SPEED_REDUCTION_PER_INCH;
        assert!(
            reduction_at_max <= MAX_SNOW_SPEED_REDUCTION,
            "reduction at max depth ({}) should not exceed max reduction ({})",
            reduction_at_max,
            MAX_SNOW_SPEED_REDUCTION
        );
    }
}

pub struct SnowPlugin;

impl Plugin for SnowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnowGrid>()
            .init_resource::<SnowPlowingState>()
            .init_resource::<SnowStats>()
            .add_systems(
                FixedUpdate,
                (update_snow, update_snow_plowing)
                    .chain()
                    .after(crate::weather::update_weather),
            );
    }
}
