//! Long-term climate change from cumulative CO2 emissions (WEATHER-016).
//!
//! Tracks cumulative CO2 emissions from fossil fuel power plants and industrial
//! buildings. As emissions accumulate past thresholds, long-term climate effects
//! are triggered: temperature increases, more extreme weather events, sea level
//! rise (permanent flooding of low-elevation coastal cells), and longer droughts.
//!
//! CO2 emission rates per MWh:
//! - Coal power plant: 1.0 ton/MWh
//! - Gas power plant:  0.4 ton/MWh
//! - Oil power plant:  0.8 ton/MWh
//! - Biomass:          0.0 ton/MWh (carbon neutral)
//!
//! Climate thresholds (cumulative tons):
//! - 1,000,000 tons: +1F average temperature increase
//! - 5,000,000 tons: +2F average temperature increase
//! - 20,000,000 tons: +3F average temperature increase
//!
//! Effects:
//! - Disaster frequency increases by +10% per 1F increase
//! - At +3F, lowest-elevation water-adjacent cells flood permanently
//! - Drought duration extends with temperature increase

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// CO2 emission rate for coal power plants (tons per MWh).
pub const CO2_COAL: f32 = 1.0;

/// CO2 emission rate for gas power plants (tons per MWh).
pub const CO2_GAS: f32 = 0.4;

/// CO2 emission rate for oil/generic power plants (tons per MWh).
pub const CO2_OIL: f32 = 0.8;

/// CO2 emission rate for biomass/renewable sources (tons per MWh, carbon neutral).
pub const CO2_BIOMASS: f32 = 0.0;

/// Base MWh production per power plant per assessment period.
/// Each utility source is assumed to generate this many MWh per yearly assessment.
const BASE_MWH_PER_PLANT: f32 = 1000.0;

/// Base CO2 from industrial buildings per assessment (tons per building).
const INDUSTRIAL_CO2_PER_BUILDING: f32 = 50.0;

/// Climate threshold: cumulative tons for +1F temperature increase.
const THRESHOLD_1F: f64 = 1_000_000.0;

/// Climate threshold: cumulative tons for +2F temperature increase.
const THRESHOLD_2F: f64 = 5_000_000.0;

/// Climate threshold: cumulative tons for +3F temperature increase.
const THRESHOLD_3F: f64 = 20_000_000.0;

/// Disaster frequency increase per 1F of warming (10% per degree F).
const DISASTER_FREQUENCY_INCREASE_PER_F: f32 = 0.10;

/// Number of game days per year (used for yearly assessments).
const DAYS_PER_YEAR: u32 = 360;

/// Elevation percentile threshold for sea level rise flooding.
/// At +3F, water-adjacent cells with elevation below this percentile flood permanently.
const SEA_LEVEL_RISE_ELEVATION_PERCENTILE: f32 = 0.15;

// =============================================================================
// Resources
// =============================================================================

/// Tracks cumulative CO2 emissions and resulting climate effects.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ClimateState {
    /// Total cumulative CO2 emissions in tons since game start.
    pub cumulative_co2: f64,
    /// CO2 emitted during the most recent yearly assessment.
    pub yearly_co2: f64,
    /// Current temperature increase in Fahrenheit due to climate change.
    pub temperature_increase_f: f32,
    /// Disaster frequency multiplier (1.0 = normal, 1.1 = +10%, etc.).
    pub disaster_frequency_multiplier: f32,
    /// Whether sea level rise flooding has been applied.
    pub sea_level_rise_applied: bool,
    /// Number of cells permanently flooded by sea level rise.
    pub flooded_cells_count: u32,
    /// Environmental score (0-100, higher = better/cleaner).
    pub environmental_score: f32,
    /// Last game day a yearly assessment was performed.
    pub last_assessment_day: u32,
    /// Drought duration multiplier (1.0 = normal, higher = longer droughts).
    pub drought_duration_multiplier: f32,
}

impl Default for ClimateState {
    fn default() -> Self {
        Self {
            cumulative_co2: 0.0,
            yearly_co2: 0.0,
            temperature_increase_f: 0.0,
            disaster_frequency_multiplier: 1.0,
            sea_level_rise_applied: false,
            flooded_cells_count: 0,
            environmental_score: 100.0,
            last_assessment_day: 0,
            drought_duration_multiplier: 1.0,
        }
    }
}

impl Saveable for ClimateState {
    const SAVE_KEY: &'static str = "climate_change";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if at default state (no emissions yet)
        if self.cumulative_co2 == 0.0 && self.last_assessment_day == 0 {
            return None;
        }
        bitcode::encode(self).ok()
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        bitcode::decode(bytes).unwrap_or_default()
    }
}

// =============================================================================
// Pure helper functions
// =============================================================================

/// Determine the temperature increase in Fahrenheit based on cumulative CO2 tons.
pub fn temperature_increase_from_co2(cumulative_tons: f64) -> f32 {
    if cumulative_tons >= THRESHOLD_3F {
        3.0
    } else if cumulative_tons >= THRESHOLD_2F {
        // Interpolate between 2F and 3F
        let progress = (cumulative_tons - THRESHOLD_2F) / (THRESHOLD_3F - THRESHOLD_2F);
        2.0 + progress as f32
    } else if cumulative_tons >= THRESHOLD_1F {
        // Interpolate between 1F and 2F
        let progress = (cumulative_tons - THRESHOLD_1F) / (THRESHOLD_2F - THRESHOLD_1F);
        1.0 + progress as f32
    } else if cumulative_tons > 0.0 {
        // Interpolate between 0F and 1F
        let progress = cumulative_tons / THRESHOLD_1F;
        progress as f32
    } else {
        0.0
    }
}

/// Calculate the disaster frequency multiplier from temperature increase.
/// +10% per 1F of warming.
pub fn disaster_multiplier_from_temp_increase(temp_increase_f: f32) -> f32 {
    1.0 + temp_increase_f * DISASTER_FREQUENCY_INCREASE_PER_F
}

/// Calculate the drought duration multiplier from temperature increase.
/// Each degree F adds 15% to drought duration.
pub fn drought_multiplier_from_temp_increase(temp_increase_f: f32) -> f32 {
    1.0 + temp_increase_f * 0.15
}

/// Calculate environmental score (0-100) based on cumulative emissions and yearly rate.
///
/// The score degrades as cumulative emissions rise, with the yearly rate affecting
/// the rate of degradation.
pub fn calculate_environmental_score(cumulative_co2: f64, yearly_co2: f64) -> f32 {
    // Base score starts at 100 and degrades with cumulative emissions.
    // Lose 10 points per million tons cumulative, capped at 0.
    let cumulative_penalty = (cumulative_co2 / 1_000_000.0 * 10.0) as f32;
    // Additional penalty for high yearly emissions (1 point per 10k tons/year).
    let yearly_penalty = (yearly_co2 / 10_000.0) as f32;
    (100.0 - cumulative_penalty - yearly_penalty).clamp(0.0, 100.0)
}

/// Get the CO2 emission rate for a utility type (tons per MWh).
pub fn co2_rate_for_utility(utility_type: UtilityType) -> f32 {
    match utility_type {
        UtilityType::PowerPlant => CO2_OIL, // Generic power plant uses oil/fossil rate
        UtilityType::SolarFarm => CO2_BIOMASS,
        UtilityType::WindTurbine => CO2_BIOMASS,
        UtilityType::NuclearPlant => CO2_BIOMASS, // Nuclear is zero-carbon
        UtilityType::Geothermal => CO2_BIOMASS,   // Geothermal is zero-carbon
        // Water utilities don't produce CO2 directly
        UtilityType::WaterTower => 0.0,
        UtilityType::SewagePlant => 0.0,
        UtilityType::PumpingStation => 0.0,
        UtilityType::WaterTreatment => 0.0,
    }
}

/// Determine if a cell is adjacent to water.
fn is_water_adjacent(grid: &WorldGrid, x: usize, y: usize) -> bool {
    let (neighbors, count) = grid.neighbors4(x, y);
    for &(nx, ny) in &neighbors[..count] {
        if grid.get(nx, ny).cell_type == CellType::Water {
            return true;
        }
    }
    false
}

/// Find the elevation threshold for sea level rise flooding.
/// Returns the elevation below which water-adjacent cells should flood.
fn sea_level_rise_threshold(grid: &WorldGrid) -> f32 {
    // Collect elevations of all non-water, water-adjacent cells
    let mut coastal_elevations: Vec<f32> = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Water && is_water_adjacent(grid, x, y) {
                coastal_elevations.push(cell.elevation);
            }
        }
    }

    if coastal_elevations.is_empty() {
        return 0.0;
    }

    coastal_elevations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Find the elevation at the given percentile
    let index = ((coastal_elevations.len() as f32 * SEA_LEVEL_RISE_ELEVATION_PERCENTILE) as usize)
        .min(coastal_elevations.len().saturating_sub(1));
    coastal_elevations[index]
}

// =============================================================================
// Systems
// =============================================================================

/// Yearly climate assessment system. Runs every slow tick but only performs the
/// assessment once per game year (every 360 days).
///
/// 1. Calculates CO2 emissions from power plants and industrial buildings.
/// 2. Updates cumulative CO2 total.
/// 3. Determines temperature increase and disaster frequency multiplier.
/// 4. At +3F, applies sea level rise (permanent coastal flooding).
/// 5. Updates environmental score.
#[allow(clippy::too_many_arguments)]
pub fn yearly_climate_assessment(
    timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut climate: ResMut<ClimateState>,
    mut grid: ResMut<WorldGrid>,
    utility_sources: Query<&UtilitySource>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() {
        return;
    }

    let current_day = clock.day;

    // Only run assessment once per year (every 360 days)
    if current_day < climate.last_assessment_day + DAYS_PER_YEAR {
        return;
    }

    // --- Calculate yearly CO2 emissions ---

    let mut yearly_co2: f64 = 0.0;

    // CO2 from power-generating utility sources
    for source in &utility_sources {
        let rate = co2_rate_for_utility(source.utility_type);
        if rate > 0.0 {
            yearly_co2 += (rate * BASE_MWH_PER_PLANT) as f64;
        }
    }

    // CO2 from industrial buildings
    let industrial_count = buildings
        .iter()
        .filter(|b| b.zone_type == ZoneType::Industrial)
        .count();
    yearly_co2 += industrial_count as f64 * INDUSTRIAL_CO2_PER_BUILDING as f64;

    // --- Update cumulative totals ---
    climate.cumulative_co2 += yearly_co2;
    climate.yearly_co2 = yearly_co2;
    climate.last_assessment_day = current_day;

    // --- Calculate climate effects ---
    climate.temperature_increase_f = temperature_increase_from_co2(climate.cumulative_co2);
    climate.disaster_frequency_multiplier =
        disaster_multiplier_from_temp_increase(climate.temperature_increase_f);
    climate.drought_duration_multiplier =
        drought_multiplier_from_temp_increase(climate.temperature_increase_f);
    climate.environmental_score =
        calculate_environmental_score(climate.cumulative_co2, climate.yearly_co2);

    // --- Sea level rise at +3F ---
    if climate.temperature_increase_f >= 3.0 && !climate.sea_level_rise_applied {
        let threshold = sea_level_rise_threshold(&grid);
        let mut flooded = 0u32;

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.cell_type != CellType::Water
                    && cell.elevation <= threshold
                    && is_water_adjacent(&grid, x, y)
                {
                    let cell_mut = grid.get_mut(x, y);
                    cell_mut.cell_type = CellType::Water;
                    cell_mut.zone = ZoneType::None;
                    cell_mut.building_id = None;
                    flooded += 1;
                }
            }
        }

        climate.sea_level_rise_applied = true;
        climate.flooded_cells_count = flooded;

        if flooded > 0 {
            info!(
                "CLIMATE CHANGE: Sea level rise has permanently flooded {} coastal cells!",
                flooded
            );
        }
    }

    // Log the assessment
    if yearly_co2 > 0.0 {
        info!(
            "CLIMATE ASSESSMENT (Day {}): +{:.0} tons CO2 this year, {:.0} total, +{:.1}F warming, env score: {:.0}",
            current_day,
            yearly_co2,
            climate.cumulative_co2,
            climate.temperature_increase_f,
            climate.environmental_score
        );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Temperature increase from CO2 tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zero_emissions_no_warming() {
        assert_eq!(temperature_increase_from_co2(0.0), 0.0);
    }

    #[test]
    fn test_threshold_1f() {
        let result = temperature_increase_from_co2(THRESHOLD_1F);
        assert!(
            (result - 1.0).abs() < f32::EPSILON,
            "At 1M tons, should be +1F, got {}",
            result
        );
    }

    #[test]
    fn test_threshold_2f() {
        let result = temperature_increase_from_co2(THRESHOLD_2F);
        assert!(
            (result - 2.0).abs() < f32::EPSILON,
            "At 5M tons, should be +2F, got {}",
            result
        );
    }

    #[test]
    fn test_threshold_3f() {
        let result = temperature_increase_from_co2(THRESHOLD_3F);
        assert!(
            (result - 3.0).abs() < f32::EPSILON,
            "At 20M tons, should be +3F, got {}",
            result
        );
    }

    #[test]
    fn test_above_threshold_3f_caps_at_3() {
        let result = temperature_increase_from_co2(50_000_000.0);
        assert!(
            (result - 3.0).abs() < f32::EPSILON,
            "Above 20M tons should cap at +3F, got {}",
            result
        );
    }

    #[test]
    fn test_interpolation_between_0_and_1f() {
        let result = temperature_increase_from_co2(500_000.0);
        assert!(
            result > 0.0 && result < 1.0,
            "At 500K tons, should be between 0F and 1F, got {}",
            result
        );
        assert!(
            (result - 0.5).abs() < f32::EPSILON,
            "At 500K (half of 1M), should be ~0.5F, got {}",
            result
        );
    }

    #[test]
    fn test_interpolation_between_1f_and_2f() {
        // Midpoint between 1M and 5M is 3M
        let result = temperature_increase_from_co2(3_000_000.0);
        assert!(
            result > 1.0 && result < 2.0,
            "At 3M tons, should be between 1F and 2F, got {}",
            result
        );
        // (3M - 1M) / (5M - 1M) = 2M / 4M = 0.5, so 1.0 + 0.5 = 1.5
        assert!(
            (result - 1.5).abs() < f32::EPSILON,
            "At 3M tons, should be ~1.5F, got {}",
            result
        );
    }

    #[test]
    fn test_interpolation_between_2f_and_3f() {
        // Midpoint between 5M and 20M is 12.5M
        let result = temperature_increase_from_co2(12_500_000.0);
        assert!(
            result > 2.0 && result < 3.0,
            "At 12.5M tons, should be between 2F and 3F, got {}",
            result
        );
        // (12.5M - 5M) / (20M - 5M) = 7.5M / 15M = 0.5, so 2.0 + 0.5 = 2.5
        assert!(
            (result - 2.5).abs() < f32::EPSILON,
            "At 12.5M tons, should be ~2.5F, got {}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Disaster multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_disaster_multiplier_no_warming() {
        assert!(
            (disaster_multiplier_from_temp_increase(0.0) - 1.0).abs() < f32::EPSILON,
            "No warming should give 1.0 multiplier"
        );
    }

    #[test]
    fn test_disaster_multiplier_1f() {
        let result = disaster_multiplier_from_temp_increase(1.0);
        assert!(
            (result - 1.1).abs() < f32::EPSILON,
            "+1F should give 1.1 multiplier, got {}",
            result
        );
    }

    #[test]
    fn test_disaster_multiplier_2f() {
        let result = disaster_multiplier_from_temp_increase(2.0);
        assert!(
            (result - 1.2).abs() < f32::EPSILON,
            "+2F should give 1.2 multiplier, got {}",
            result
        );
    }

    #[test]
    fn test_disaster_multiplier_3f() {
        let result = disaster_multiplier_from_temp_increase(3.0);
        assert!(
            (result - 1.3).abs() < f32::EPSILON,
            "+3F should give 1.3 multiplier, got {}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Drought multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drought_multiplier_no_warming() {
        assert!(
            (drought_multiplier_from_temp_increase(0.0) - 1.0).abs() < f32::EPSILON,
            "No warming should give 1.0 drought multiplier"
        );
    }

    #[test]
    fn test_drought_multiplier_1f() {
        let result = drought_multiplier_from_temp_increase(1.0);
        assert!(
            (result - 1.15).abs() < f32::EPSILON,
            "+1F should give 1.15 drought multiplier, got {}",
            result
        );
    }

    #[test]
    fn test_drought_multiplier_3f() {
        let result = drought_multiplier_from_temp_increase(3.0);
        assert!(
            (result - 1.45).abs() < f32::EPSILON,
            "+3F should give 1.45 drought multiplier, got {}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Environmental score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_environmental_score_no_emissions() {
        let score = calculate_environmental_score(0.0, 0.0);
        assert!(
            (score - 100.0).abs() < f32::EPSILON,
            "Zero emissions should give 100 score"
        );
    }

    #[test]
    fn test_environmental_score_1m_tons() {
        let score = calculate_environmental_score(1_000_000.0, 0.0);
        assert!(
            (score - 90.0).abs() < f32::EPSILON,
            "1M cumulative tons should give 90 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_5m_tons() {
        let score = calculate_environmental_score(5_000_000.0, 0.0);
        assert!(
            (score - 50.0).abs() < f32::EPSILON,
            "5M cumulative tons should give 50 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_10m_tons() {
        let score = calculate_environmental_score(10_000_000.0, 0.0);
        assert!(
            (score - 0.0).abs() < f32::EPSILON,
            "10M cumulative tons should give 0 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_with_yearly() {
        let score = calculate_environmental_score(0.0, 100_000.0);
        assert!(
            (score - 90.0).abs() < f32::EPSILON,
            "100K yearly tons should give 90 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_clamped_to_zero() {
        let score = calculate_environmental_score(50_000_000.0, 1_000_000.0);
        assert!(
            (score - 0.0).abs() < f32::EPSILON,
            "Very high emissions should clamp to 0, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_clamped_to_100() {
        // Even negative emissions shouldn't exceed 100
        let score = calculate_environmental_score(0.0, 0.0);
        assert!(score <= 100.0);
    }

    // -------------------------------------------------------------------------
    // CO2 rate tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_co2_rate_power_plant() {
        assert!(
            (co2_rate_for_utility(UtilityType::PowerPlant) - CO2_OIL).abs() < f32::EPSILON,
            "Power plant should use oil rate"
        );
    }

    #[test]
    fn test_co2_rate_solar() {
        assert!(
            (co2_rate_for_utility(UtilityType::SolarFarm) - 0.0).abs() < f32::EPSILON,
            "Solar should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_wind() {
        assert!(
            (co2_rate_for_utility(UtilityType::WindTurbine) - 0.0).abs() < f32::EPSILON,
            "Wind should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_nuclear() {
        assert!(
            (co2_rate_for_utility(UtilityType::NuclearPlant) - 0.0).abs() < f32::EPSILON,
            "Nuclear should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_geothermal() {
        assert!(
            (co2_rate_for_utility(UtilityType::Geothermal) - 0.0).abs() < f32::EPSILON,
            "Geothermal should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_water_utilities() {
        assert_eq!(co2_rate_for_utility(UtilityType::WaterTower), 0.0);
        assert_eq!(co2_rate_for_utility(UtilityType::SewagePlant), 0.0);
        assert_eq!(co2_rate_for_utility(UtilityType::PumpingStation), 0.0);
        assert_eq!(co2_rate_for_utility(UtilityType::WaterTreatment), 0.0);
    }

    // -------------------------------------------------------------------------
    // Water adjacency tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_water_adjacent_true() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water at (5, 5)
        grid.get_mut(5, 5).cell_type = CellType::Water;
        // (5, 6) should be adjacent to water
        assert!(is_water_adjacent(&grid, 5, 6));
        // (4, 5) should be adjacent to water
        assert!(is_water_adjacent(&grid, 4, 5));
    }

    #[test]
    fn test_is_water_adjacent_false() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No water placed, nothing should be water-adjacent
        assert!(!is_water_adjacent(&grid, 10, 10));
    }

    // -------------------------------------------------------------------------
    // Sea level rise threshold tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sea_level_rise_threshold_no_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No water cells, threshold should be 0
        let threshold = sea_level_rise_threshold(&grid);
        assert_eq!(threshold, 0.0);
    }

    #[test]
    fn test_sea_level_rise_threshold_with_coastal_cells() {
        let mut grid = WorldGrid::new(10, 10);
        // Create a water body on the left edge
        for y in 0..10 {
            grid.get_mut(0, y).cell_type = CellType::Water;
            grid.get_mut(0, y).elevation = 0.0;
        }
        // Set coastal cell elevations (cells at x=1 are adjacent to water)
        for y in 0..10 {
            grid.get_mut(1, y).elevation = y as f32 * 0.1;
        }
        // The threshold should be at the 15th percentile of coastal elevations
        let threshold = sea_level_rise_threshold(&grid);
        assert!(
            threshold >= 0.0,
            "Threshold should be non-negative, got {}",
            threshold
        );
    }

    // -------------------------------------------------------------------------
    // ClimateState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_climate_state_default() {
        let state = ClimateState::default();
        assert_eq!(state.cumulative_co2, 0.0);
        assert_eq!(state.yearly_co2, 0.0);
        assert_eq!(state.temperature_increase_f, 0.0);
        assert!((state.disaster_frequency_multiplier - 1.0).abs() < f32::EPSILON);
        assert!(!state.sea_level_rise_applied);
        assert_eq!(state.flooded_cells_count, 0);
        assert!((state.environmental_score - 100.0).abs() < f32::EPSILON);
        assert_eq!(state.last_assessment_day, 0);
        assert!((state.drought_duration_multiplier - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skip_default() {
        let state = ClimateState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = ClimateState::default();
        state.cumulative_co2 = 5_000_000.0;
        state.yearly_co2 = 100_000.0;
        state.temperature_increase_f = 2.0;
        state.last_assessment_day = 720;

        let bytes = state
            .save_to_bytes()
            .expect("Non-default state should save");
        let loaded = ClimateState::load_from_bytes(&bytes);

        assert!((loaded.cumulative_co2 - 5_000_000.0).abs() < f64::EPSILON);
        assert!((loaded.yearly_co2 - 100_000.0).abs() < f64::EPSILON);
        assert!((loaded.temperature_increase_f - 2.0).abs() < f32::EPSILON);
        assert_eq!(loaded.last_assessment_day, 720);
    }

    // -------------------------------------------------------------------------
    // CO2 constants tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_co2_constants() {
        assert!((CO2_COAL - 1.0).abs() < f32::EPSILON);
        assert!((CO2_GAS - 0.4).abs() < f32::EPSILON);
        assert!((CO2_OIL - 0.8).abs() < f32::EPSILON);
        assert!((CO2_BIOMASS - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_threshold_constants() {
        assert!((THRESHOLD_1F - 1_000_000.0).abs() < f64::EPSILON);
        assert!((THRESHOLD_2F - 5_000_000.0).abs() < f64::EPSILON);
        assert!((THRESHOLD_3F - 20_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_disaster_frequency_increase_per_f() {
        assert!((DISASTER_FREQUENCY_INCREASE_PER_F - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_days_per_year() {
        assert_eq!(DAYS_PER_YEAR, 360);
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ClimateChangePlugin;

impl Plugin for ClimateChangePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClimateState>().add_systems(
            FixedUpdate,
            yearly_climate_assessment.after(crate::weather::update_weather),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ClimateState>();
    }
}
