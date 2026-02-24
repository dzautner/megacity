//! POLL-021: City Environmental Score Aggregate Metric
//!
//! Computes a single aggregate "Environmental Score" (0–100) summarizing
//! city environmental health as a weighted average of six sub-scores:
//!
//! | Sub-score          | Weight |
//! |--------------------|--------|
//! | Air quality        |  25%   |
//! | Water quality      |  20%   |
//! | Noise              |  15%   |
//! | Soil health        |  10%   |
//! | Green coverage     |  15%   |
//! | Energy cleanliness |  15%   |
//!
//! Achievement triggers:
//! - "Green City" at score > 80
//! - "Eco Champion" at score > 95

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::PowerPlant;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::trees::TreeGrid;
use crate::water_pollution::WaterPollutionGrid;
use crate::{decode_or_warn, Saveable, SimulationSet, SlowTickTimer};

// ---------------------------------------------------------------------------
// Weights
// ---------------------------------------------------------------------------

const WEIGHT_AIR_QUALITY: f32 = 0.25;
const WEIGHT_WATER_QUALITY: f32 = 0.20;
const WEIGHT_NOISE: f32 = 0.15;
const WEIGHT_SOIL_HEALTH: f32 = 0.10;
const WEIGHT_GREEN_COVERAGE: f32 = 0.15;
const WEIGHT_ENERGY_CLEANLINESS: f32 = 0.15;

/// Maximum AQI value for normalization (500 AQI → score 0).
const MAX_AQI: f32 = 500.0;

/// Placeholder soil health score until a proper soil system exists.
const PLACEHOLDER_SOIL_HEALTH: f32 = 50.0;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// City-wide environmental score and its component sub-scores.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct EnvironmentalScore {
    /// Overall environmental score (0–100).
    pub overall: f32,
    /// Air quality sub-score (0–100). 0 AQI → 100, 500 AQI → 0.
    pub air_quality: f32,
    /// Water quality sub-score (0–100). Lower contamination → higher score.
    pub water_quality: f32,
    /// Noise sub-score (0–100). Lower noise → higher score.
    pub noise: f32,
    /// Soil health sub-score (0–100). Placeholder until soil system exists.
    pub soil_health: f32,
    /// Green coverage sub-score (0–100). Percentage of cells with trees or parks.
    pub green_coverage: f32,
    /// Energy cleanliness sub-score (0–100). Fraction of power from renewables.
    pub energy_cleanliness: f32,
}

impl Default for EnvironmentalScore {
    fn default() -> Self {
        Self {
            overall: 50.0,
            air_quality: 100.0,
            water_quality: 100.0,
            noise: 100.0,
            soil_health: PLACEHOLDER_SOIL_HEALTH,
            green_coverage: 0.0,
            energy_cleanliness: 100.0,
        }
    }
}

impl Saveable for EnvironmentalScore {
    const SAVE_KEY: &'static str = "environmental_score";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Sub-score computation helpers
// ---------------------------------------------------------------------------

/// Compute air quality sub-score from the pollution grid.
///
/// We treat average grid pollution as a proxy for AQI (0–255 grid → 0–500 AQI).
/// Score = 100 × (1 − avg_aqi / 500), clamped to [0, 100].
fn compute_air_quality(pollution: &PollutionGrid) -> f32 {
    let total: u64 = pollution.levels.iter().map(|&v| v as u64).sum();
    let cell_count = pollution.levels.len() as f64;
    if cell_count == 0.0 {
        return 100.0;
    }
    // Map 0-255 grid values to 0-500 AQI scale.
    let avg_grid = total as f64 / cell_count;
    let avg_aqi = avg_grid * (MAX_AQI as f64 / 255.0);
    let score = 100.0 * (1.0 - avg_aqi / MAX_AQI as f64);
    (score as f32).clamp(0.0, 100.0)
}

/// Compute water quality sub-score from the water pollution grid.
///
/// Score = 100 × (1 − avg_contamination / 255), clamped to [0, 100].
fn compute_water_quality(water_pollution: &WaterPollutionGrid) -> f32 {
    let total: u64 = water_pollution.levels.iter().map(|&v| v as u64).sum();
    let cell_count = water_pollution.levels.len() as f64;
    if cell_count == 0.0 {
        return 100.0;
    }
    let avg = total as f64 / cell_count;
    let score = 100.0 * (1.0 - avg / 255.0);
    (score as f32).clamp(0.0, 100.0)
}

/// Compute noise sub-score from the noise pollution grid.
///
/// Noise grid values are 0-100. Score = 100 − average_noise.
fn compute_noise_score(noise: &NoisePollutionGrid) -> f32 {
    let total: u64 = noise.levels.iter().map(|&v| v as u64).sum();
    let cell_count = noise.levels.len() as f64;
    if cell_count == 0.0 {
        return 100.0;
    }
    let avg = total as f64 / cell_count;
    let score = 100.0 - avg;
    (score as f32).clamp(0.0, 100.0)
}

/// Compute green coverage sub-score from the tree grid.
///
/// Green coverage = (cells with trees / total cells) × 100.
/// We cap at 100% so any tree density > 100% of cells is still 100.
fn compute_green_coverage(tree_grid: &TreeGrid) -> f32 {
    let tree_count: usize = tree_grid.cells.iter().filter(|&&v| v).count();
    let total_cells = (GRID_WIDTH * GRID_HEIGHT) as f32;
    if total_cells == 0.0 {
        return 0.0;
    }
    let percentage = (tree_count as f32 / total_cells) * 100.0;
    percentage.clamp(0.0, 100.0)
}

/// Compute energy cleanliness sub-score from power plants.
///
/// Renewable = fuel_cost == 0.0. Score = (renewable_output / total_output) × 100.
/// If no power plants exist, default to 100 (no dirty energy).
fn compute_energy_cleanliness(plants: &[(f32, f32)]) -> f32 {
    let mut total_output: f32 = 0.0;
    let mut renewable_output: f32 = 0.0;
    for &(output, fuel_cost) in plants {
        total_output += output;
        if fuel_cost == 0.0 {
            renewable_output += output;
        }
    }
    if total_output <= 0.0 {
        return 100.0;
    }
    let fraction = renewable_output / total_output;
    (fraction * 100.0).clamp(0.0, 100.0)
}

/// Compute overall weighted score from sub-scores.
fn compute_overall(score: &EnvironmentalScore) -> f32 {
    let weighted = score.air_quality * WEIGHT_AIR_QUALITY
        + score.water_quality * WEIGHT_WATER_QUALITY
        + score.noise * WEIGHT_NOISE
        + score.soil_health * WEIGHT_SOIL_HEALTH
        + score.green_coverage * WEIGHT_GREEN_COVERAGE
        + score.energy_cleanliness * WEIGHT_ENERGY_CLEANLINESS;
    weighted.clamp(0.0, 100.0)
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Periodic system that recomputes the environmental score every slow tick.
#[allow(clippy::too_many_arguments)]
pub fn update_environmental_score(
    slow_timer: Res<SlowTickTimer>,
    pollution: Res<PollutionGrid>,
    water_pollution: Res<WaterPollutionGrid>,
    noise: Res<NoisePollutionGrid>,
    tree_grid: Res<TreeGrid>,
    power_plants: Query<&PowerPlant>,
    mut env_score: ResMut<EnvironmentalScore>,
) {
    if !slow_timer.should_run() {
        return;
    }

    env_score.air_quality = compute_air_quality(&pollution);
    env_score.water_quality = compute_water_quality(&water_pollution);
    env_score.noise = compute_noise_score(&noise);
    env_score.soil_health = PLACEHOLDER_SOIL_HEALTH;

    env_score.green_coverage = compute_green_coverage(&tree_grid);

    // Collect power plant data for energy cleanliness.
    let plant_data: Vec<(f32, f32)> = power_plants
        .iter()
        .map(|p| (p.current_output_mw, p.fuel_cost))
        .collect();
    env_score.energy_cleanliness = compute_energy_cleanliness(&plant_data);

    env_score.overall = compute_overall(&env_score);
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for the aggregate Environmental Score metric.
pub struct EnvironmentalScorePlugin;

impl Plugin for EnvironmentalScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnvironmentalScore>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<EnvironmentalScore>();

        app.add_systems(
            FixedUpdate,
            update_environmental_score.in_set(SimulationSet::PostSim),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_quality_clean() {
        let grid = PollutionGrid::default();
        assert!((compute_air_quality(&grid) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_air_quality_max_pollution() {
        let mut grid = PollutionGrid::default();
        for v in grid.levels.iter_mut() {
            *v = 255;
        }
        assert!(compute_air_quality(&grid) < 0.01);
    }

    #[test]
    fn test_water_quality_clean() {
        let grid = WaterPollutionGrid::default();
        assert!((compute_water_quality(&grid) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_noise_score_silent() {
        let grid = NoisePollutionGrid::default();
        assert!((compute_noise_score(&grid) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_green_coverage_empty() {
        let grid = TreeGrid::default();
        assert!(compute_green_coverage(&grid) < 0.01);
    }

    #[test]
    fn test_green_coverage_all_trees() {
        let mut grid = TreeGrid::default();
        for v in grid.cells.iter_mut() {
            *v = true;
        }
        assert!((compute_green_coverage(&grid) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_energy_cleanliness_all_renewable() {
        let plants = vec![(10.0, 0.0), (20.0, 0.0)];
        assert!((compute_energy_cleanliness(&plants) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_energy_cleanliness_no_renewable() {
        let plants = vec![(10.0, 30.0), (20.0, 40.0)];
        assert!(compute_energy_cleanliness(&plants) < 0.01);
    }

    #[test]
    fn test_energy_cleanliness_mixed() {
        // 10 MW renewable, 10 MW fossil → 50%
        let plants = vec![(10.0, 0.0), (10.0, 30.0)];
        assert!((compute_energy_cleanliness(&plants) - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_energy_cleanliness_no_plants() {
        let plants: Vec<(f32, f32)> = vec![];
        assert!((compute_energy_cleanliness(&plants) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_overall_weights_sum_to_one() {
        let sum = WEIGHT_AIR_QUALITY
            + WEIGHT_WATER_QUALITY
            + WEIGHT_NOISE
            + WEIGHT_SOIL_HEALTH
            + WEIGHT_GREEN_COVERAGE
            + WEIGHT_ENERGY_CLEANLINESS;
        assert!(
            (sum - 1.0).abs() < 0.001,
            "weights must sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn test_overall_perfect_score() {
        let score = EnvironmentalScore {
            overall: 0.0,
            air_quality: 100.0,
            water_quality: 100.0,
            noise: 100.0,
            soil_health: 100.0,
            green_coverage: 100.0,
            energy_cleanliness: 100.0,
        };
        assert!((compute_overall(&score) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_overall_zero_score() {
        let score = EnvironmentalScore {
            overall: 0.0,
            air_quality: 0.0,
            water_quality: 0.0,
            noise: 0.0,
            soil_health: 0.0,
            green_coverage: 0.0,
            energy_cleanliness: 0.0,
        };
        assert!(compute_overall(&score) < 0.01);
    }

    #[test]
    fn test_default_overall() {
        let score = EnvironmentalScore::default();
        let overall = compute_overall(&score);
        // Default: air=100, water=100, noise=100, soil=50, green=0, energy=100
        // = 100*0.25 + 100*0.20 + 100*0.15 + 50*0.10 + 0*0.15 + 100*0.15
        // = 25 + 20 + 15 + 5 + 0 + 15 = 80
        assert!(
            (overall - 80.0).abs() < 0.01,
            "default overall should be 80.0, got {}",
            overall
        );
    }
}
