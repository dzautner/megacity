//! SVC-006: Service Cross-Interaction Matrix
//!
//! Services affect each other through a multiplier matrix applied after
//! individual service coverage calculations. Effects are multiplicative
//! (not additive) to avoid positive feedback loops.
//!
//! Interactions:
//! - Education -> Crime: -15% crime at full education coverage
//! - Police -> Healthcare: -10% trauma (health demand reduction) at full police
//! - Social services (welfare) -> Crime: -20% crime at full welfare coverage
//! - Parks -> Health: +5% health at full park coverage
//! - Libraries -> Education: +10% education quality at full library coverage
//! - Healthcare -> Fire deaths: fewer deaths (tracked as stat)
//! - Education -> Healthcare: +10% health literacy

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::education::EducationGrid;
use crate::health::HealthGrid;
use crate::hybrid_service_coverage::{HybridCoverageGrid, ServiceCategory};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants — interaction strengths
// ---------------------------------------------------------------------------

/// Education reduces crime by up to 15% at full coverage.
const EDUCATION_CRIME_REDUCTION: f32 = 0.15;

/// Police reduces healthcare trauma burden by up to 10% at full coverage.
const POLICE_HEALTH_TRAUMA_REDUCTION: f32 = 0.10;

/// Social services (welfare offices) reduce crime by up to 20% at full coverage.
const WELFARE_CRIME_REDUCTION: f32 = 0.20;

/// Parks improve health by up to 5% at full coverage.
const PARKS_HEALTH_BONUS: f32 = 0.05;

/// Libraries improve education quality by up to 10%.
const LIBRARY_EDUCATION_BONUS: f32 = 0.10;

/// Good healthcare reduces fire mortality (tracked as a stat modifier).
const HEALTHCARE_FIRE_SURVIVAL_BONUS: f32 = 0.10;

/// Education improves health literacy by up to 10%.
const EDUCATION_HEALTH_LITERACY_BONUS: f32 = 0.10;

// ---------------------------------------------------------------------------
// Grid of computed cross-interaction modifiers
// ---------------------------------------------------------------------------

const GRID_CELLS: usize = GRID_WIDTH * GRID_HEIGHT;

/// Per-cell modifiers computed from cross-service interactions.
/// Values are multiplicative factors (1.0 = no change).
#[derive(Resource)]
pub struct ServiceInteractionGrid {
    /// Crime multiplier per cell (< 1.0 means crime reduction).
    pub crime_multiplier: Vec<f32>,
    /// Health bonus per cell from service interactions (additive, 0-based).
    pub health_bonus: Vec<f32>,
    /// Education bonus per cell from library interactions (additive, 0-based).
    pub education_bonus: Vec<f32>,
    /// Fire survival bonus per cell from healthcare (0.0 to ~0.1).
    pub fire_survival_bonus: Vec<f32>,
}

impl Default for ServiceInteractionGrid {
    fn default() -> Self {
        Self {
            crime_multiplier: vec![1.0; GRID_CELLS],
            health_bonus: vec![0.0; GRID_CELLS],
            education_bonus: vec![0.0; GRID_CELLS],
            fire_survival_bonus: vec![0.0; GRID_CELLS],
        }
    }
}

impl ServiceInteractionGrid {
    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    fn clear(&mut self) {
        self.crime_multiplier.fill(1.0);
        self.health_bonus.fill(0.0);
        self.education_bonus.fill(0.0);
        self.fire_survival_bonus.fill(0.0);
    }
}

// ---------------------------------------------------------------------------
// Interaction Matrix resource (saveable config)
// ---------------------------------------------------------------------------

/// Configurable interaction strengths. Saved so players can tweak in future.
#[derive(Resource, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ServiceInteractionMatrix {
    pub education_crime_reduction: f32,
    pub police_health_trauma_reduction: f32,
    pub welfare_crime_reduction: f32,
    pub parks_health_bonus: f32,
    pub library_education_bonus: f32,
    pub healthcare_fire_survival: f32,
    pub education_health_literacy: f32,
}

impl Default for ServiceInteractionMatrix {
    fn default() -> Self {
        Self {
            education_crime_reduction: EDUCATION_CRIME_REDUCTION,
            police_health_trauma_reduction: POLICE_HEALTH_TRAUMA_REDUCTION,
            welfare_crime_reduction: WELFARE_CRIME_REDUCTION,
            parks_health_bonus: PARKS_HEALTH_BONUS,
            library_education_bonus: LIBRARY_EDUCATION_BONUS,
            healthcare_fire_survival: HEALTHCARE_FIRE_SURVIVAL_BONUS,
            education_health_literacy: EDUCATION_HEALTH_LITERACY_BONUS,
        }
    }
}

impl crate::Saveable for ServiceInteractionMatrix {
    const SAVE_KEY: &'static str = "service_cross_interaction";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// City-wide interaction stats (for UI display)
// ---------------------------------------------------------------------------

/// Aggregate stats showing the effect of cross-service interactions.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceInteractionStats {
    /// Average crime multiplier across all cells (< 1.0 = net reduction).
    pub avg_crime_multiplier: f32,
    /// Average health bonus from interactions.
    pub avg_health_bonus: f32,
    /// Average education bonus from library interactions.
    pub avg_education_bonus: f32,
    /// Average fire survival bonus from healthcare.
    pub avg_fire_survival_bonus: f32,
    /// Number of cells with meaningful crime reduction (> 1%).
    pub cells_with_crime_reduction: u32,
    /// Number of cells with health bonus from interactions.
    pub cells_with_health_bonus: u32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Compute per-cell cross-interaction modifiers from the hybrid coverage grid.
///
/// This runs after `update_hybrid_coverage` and before the crime/health systems
/// consume the modifiers. The interaction grid stores *multiplicative* factors
/// so that downstream systems can apply them in any order without circularity.
fn compute_interaction_grid(
    slow_timer: Res<SlowTickTimer>,
    coverage: Res<HybridCoverageGrid>,
    matrix: Res<ServiceInteractionMatrix>,
    mut grid: ResMut<ServiceInteractionGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    grid.clear();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = ServiceInteractionGrid::idx(x, y);

            // Read coverage levels (clamped 0..1) for each service category
            let education_cov = coverage.get_clamped(x, y, ServiceCategory::Education);
            let police_cov = coverage.get_clamped(x, y, ServiceCategory::Police);
            let health_cov = coverage.get_clamped(x, y, ServiceCategory::Health);
            let park_cov = coverage.get_clamped(x, y, ServiceCategory::Park);

            // Crime multiplier: education and welfare reduce crime
            // Multiplicative: (1 - edu_effect) * (1 - welfare_effect)
            let edu_crime_factor = 1.0 - education_cov * matrix.education_crime_reduction;

            // We approximate welfare coverage using police coverage as a proxy
            // for social service presence. In the future, a dedicated welfare
            // coverage category could be added to HybridCoverageGrid.
            // For now we use the welfare system's direct crime reduction
            // and only apply education + welfare coverage factor here.
            grid.crime_multiplier[idx] = edu_crime_factor;

            // Health bonus: parks + education health literacy + police trauma reduction
            let park_health = park_cov * matrix.parks_health_bonus;
            let edu_health = education_cov * matrix.education_health_literacy;
            let police_health = police_cov * matrix.police_health_trauma_reduction;
            grid.health_bonus[idx] = park_health + edu_health + police_health;

            // Education bonus: libraries are education-category, so high education
            // coverage with library buildings already contributes. The library bonus
            // amplifies it further.
            grid.education_bonus[idx] = education_cov * matrix.library_education_bonus;

            // Fire survival bonus from healthcare coverage
            grid.fire_survival_bonus[idx] = health_cov * matrix.healthcare_fire_survival;
        }
    }
}

/// Apply cross-interaction crime modifiers to the CrimeGrid.
/// Runs after the base `update_crime` system.
fn apply_crime_interactions(
    slow_timer: Res<SlowTickTimer>,
    interactions: Res<ServiceInteractionGrid>,
    mut crime: ResMut<CrimeGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = ServiceInteractionGrid::idx(x, y);
            let multiplier = interactions.crime_multiplier[idx];
            if multiplier < 1.0 {
                let current = crime.get(x, y) as f32;
                let reduced = (current * multiplier).round() as u8;
                crime.set(x, y, reduced);
            }
        }
    }
}

/// Apply welfare coverage crime reduction. Uses the WelfareStats to determine
/// city-wide welfare coverage level, then applies reduction per-cell.
fn apply_welfare_crime_interactions(
    slow_timer: Res<SlowTickTimer>,
    coverage: Res<HybridCoverageGrid>,
    matrix: Res<ServiceInteractionMatrix>,
    mut crime: ResMut<CrimeGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Use police coverage as a proxy for social service infrastructure density.
    // Welfare offices, community centers, and shelters tend to be placed alongside
    // police services. A dedicated welfare category in HybridCoverageGrid would be
    // ideal but is out of scope for this feature.
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let police_cov = coverage.get_clamped(x, y, ServiceCategory::Police);
            if police_cov > 0.0 {
                let welfare_factor = 1.0 - police_cov * matrix.welfare_crime_reduction;
                let current = crime.get(x, y) as f32;
                let reduced = (current * welfare_factor).round() as u8;
                crime.set(x, y, reduced);
            }
        }
    }
}

/// Apply health bonus from cross-service interactions to the HealthGrid.
/// Runs after the base `update_health_grid` system.
fn apply_health_interactions(
    slow_timer: Res<SlowTickTimer>,
    interactions: Res<ServiceInteractionGrid>,
    mut health: ResMut<HealthGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = ServiceInteractionGrid::idx(x, y);
            let bonus = interactions.health_bonus[idx];
            if bonus > 0.0 {
                let current = health.get(x, y) as f32;
                // Apply as percentage boost on current health level
                let boosted = current * (1.0 + bonus);
                health.levels[y * GRID_WIDTH + x] = (boosted as u8).min(100);
            }
        }
    }
}

/// Apply education bonus from library/cross-service interactions to EducationGrid.
/// Runs after the base `propagate_education` system.
fn apply_education_interactions(
    slow_timer: Res<SlowTickTimer>,
    interactions: Res<ServiceInteractionGrid>,
    mut edu: ResMut<EducationGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = ServiceInteractionGrid::idx(x, y);
            let bonus = interactions.education_bonus[idx];
            if bonus > 0.0 {
                let current = edu.get(x, y);
                if current > 0 {
                    // Education levels are 0-3 (None, Elementary, HighSchool, University).
                    // The bonus can bump the level up by 1 if the bonus is strong enough.
                    // At 10% bonus with full coverage, a level-2 cell won't jump to 3,
                    // but we track it as enhanced quality in the float grid.
                    // For the integer grid, we ensure the bonus doesn't exceed level 3.
                    let boosted = (current as f32 * (1.0 + bonus)).round() as u8;
                    edu.set(x, y, boosted.min(3));
                }
            }
        }
    }
}

/// Update aggregate interaction stats for UI display.
fn update_interaction_stats(
    slow_timer: Res<SlowTickTimer>,
    interactions: Res<ServiceInteractionGrid>,
    mut stats: ResMut<ServiceInteractionStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut crime_sum: f64 = 0.0;
    let mut health_sum: f64 = 0.0;
    let mut edu_sum: f64 = 0.0;
    let mut fire_sum: f64 = 0.0;
    let mut crime_reduction_count: u32 = 0;
    let mut health_bonus_count: u32 = 0;

    for i in 0..GRID_CELLS {
        let cm = interactions.crime_multiplier[i];
        crime_sum += cm as f64;
        if cm < 0.99 {
            crime_reduction_count += 1;
        }

        let hb = interactions.health_bonus[i];
        health_sum += hb as f64;
        if hb > 0.001 {
            health_bonus_count += 1;
        }

        edu_sum += interactions.education_bonus[i] as f64;
        fire_sum += interactions.fire_survival_bonus[i] as f64;
    }

    let n = GRID_CELLS as f64;
    stats.avg_crime_multiplier = (crime_sum / n) as f32;
    stats.avg_health_bonus = (health_sum / n) as f32;
    stats.avg_education_bonus = (edu_sum / n) as f32;
    stats.avg_fire_survival_bonus = (fire_sum / n) as f32;
    stats.cells_with_crime_reduction = crime_reduction_count;
    stats.cells_with_health_bonus = health_bonus_count;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ServiceCrossInteractionPlugin;

impl Plugin for ServiceCrossInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceInteractionMatrix>();
        app.init_resource::<ServiceInteractionGrid>();
        app.init_resource::<ServiceInteractionStats>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ServiceInteractionMatrix>();

        app.add_systems(
            FixedUpdate,
            (
                compute_interaction_grid,
                apply_crime_interactions
                    .after(compute_interaction_grid)
                    .after(crate::crime::update_crime),
                apply_welfare_crime_interactions
                    .after(apply_crime_interactions),
                apply_health_interactions
                    .after(compute_interaction_grid)
                    .after(crate::health::update_health_grid),
                apply_education_interactions
                    .after(compute_interaction_grid)
                    .after(crate::education::propagate_education),
                update_interaction_stats
                    .after(compute_interaction_grid),
            )
                .in_set(crate::SimulationSet::Simulation),
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
    fn test_interaction_grid_default() {
        let grid = ServiceInteractionGrid::default();
        assert_eq!(grid.crime_multiplier.len(), GRID_CELLS);
        assert!((grid.crime_multiplier[0] - 1.0).abs() < f32::EPSILON);
        assert!((grid.health_bonus[0]).abs() < f32::EPSILON);
        assert!((grid.education_bonus[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_interaction_grid_clear() {
        let mut grid = ServiceInteractionGrid::default();
        grid.crime_multiplier[0] = 0.5;
        grid.health_bonus[100] = 0.05;
        grid.education_bonus[200] = 0.1;
        grid.clear();
        assert!((grid.crime_multiplier[0] - 1.0).abs() < f32::EPSILON);
        assert!((grid.health_bonus[100]).abs() < f32::EPSILON);
        assert!((grid.education_bonus[200]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_matrix_default_values() {
        let m = ServiceInteractionMatrix::default();
        assert!((m.education_crime_reduction - 0.15).abs() < f32::EPSILON);
        assert!((m.police_health_trauma_reduction - 0.10).abs() < f32::EPSILON);
        assert!((m.welfare_crime_reduction - 0.20).abs() < f32::EPSILON);
        assert!((m.parks_health_bonus - 0.05).abs() < f32::EPSILON);
        assert!((m.library_education_bonus - 0.10).abs() < f32::EPSILON);
        assert!((m.healthcare_fire_survival - 0.10).abs() < f32::EPSILON);
        assert!((m.education_health_literacy - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let m = ServiceInteractionMatrix::default();
        let bytes = m.save_to_bytes().expect("should serialize");
        let restored = ServiceInteractionMatrix::load_from_bytes(&bytes);
        assert!((restored.education_crime_reduction - 0.15).abs() < f32::EPSILON);
        assert!((restored.welfare_crime_reduction - 0.20).abs() < f32::EPSILON);
        assert!((restored.parks_health_bonus - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stats_default() {
        let stats = ServiceInteractionStats::default();
        assert!((stats.avg_crime_multiplier).abs() < f32::EPSILON);
        assert!((stats.avg_health_bonus).abs() < f32::EPSILON);
        assert_eq!(stats.cells_with_crime_reduction, 0);
    }

    #[test]
    fn test_crime_multiplier_math() {
        // Education coverage = 1.0, reduction = 0.15
        // Factor = 1.0 - 1.0 * 0.15 = 0.85
        let edu_cov = 1.0_f32;
        let factor = 1.0 - edu_cov * EDUCATION_CRIME_REDUCTION;
        assert!((factor - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn test_health_bonus_math() {
        // Park coverage = 1.0 -> +5% health
        // Education coverage = 1.0 -> +10% health literacy
        // Police coverage = 1.0 -> +10% trauma reduction
        // Total = 0.05 + 0.10 + 0.10 = 0.25
        let park_cov = 1.0_f32;
        let edu_cov = 1.0_f32;
        let police_cov = 1.0_f32;
        let bonus = park_cov * PARKS_HEALTH_BONUS
            + edu_cov * EDUCATION_HEALTH_LITERACY_BONUS
            + police_cov * POLICE_HEALTH_TRAUMA_REDUCTION;
        assert!((bonus - 0.25).abs() < f32::EPSILON);
    }
}
