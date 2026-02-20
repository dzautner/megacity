//! Water Treatment Plant Level System (WATER-003).
//!
//! Treatment plants have upgradeable treatment levels that determine effluent
//! quality, treatment cost, and capacity. Higher treatment levels produce cleaner
//! output water but cost more to operate.
//!
//! Treatment levels:
//! - None: 0% removal (raw sewage bypass)
//! - Primary: 60% removal ($1K/MG) - physical settling
//! - Secondary: 85% removal ($2K/MG) - biological treatment
//! - Tertiary: 95% removal ($5K/MG) - nutrient removal
//! - Advanced: 99% removal ($10K/MG) - membrane filtration

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Treatment level enum
// =============================================================================

/// Treatment level for a water treatment plant, determining removal effectiveness
/// and operational cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TreatmentLevel {
    /// No treatment (bypass). 0% contaminant removal.
    #[default]
    None,
    /// Primary treatment (physical settling). 60% removal.
    Primary,
    /// Secondary treatment (biological). 85% removal.
    Secondary,
    /// Tertiary treatment (nutrient removal). 95% removal.
    Tertiary,
    /// Advanced treatment (membrane filtration). 99% removal.
    Advanced,
}

impl TreatmentLevel {
    /// Fraction of contaminants removed (0.0 = none, 1.0 = perfect).
    pub fn removal_efficiency(&self) -> f32 {
        match self {
            TreatmentLevel::None => 0.0,
            TreatmentLevel::Primary => 0.60,
            TreatmentLevel::Secondary => 0.85,
            TreatmentLevel::Tertiary => 0.95,
            TreatmentLevel::Advanced => 0.99,
        }
    }

    /// Treatment cost per million gallons processed (in dollars).
    pub fn cost_per_million_gallons(&self) -> f64 {
        match self {
            TreatmentLevel::None => 0.0,
            TreatmentLevel::Primary => 1_000.0,
            TreatmentLevel::Secondary => 2_000.0,
            TreatmentLevel::Tertiary => 5_000.0,
            TreatmentLevel::Advanced => 10_000.0,
        }
    }

    /// Cost to upgrade from the current level to the next level.
    /// Returns `None` if already at Advanced (max level).
    pub fn upgrade_cost(&self) -> Option<f64> {
        match self {
            TreatmentLevel::None => Some(25_000.0),
            TreatmentLevel::Primary => Some(50_000.0),
            TreatmentLevel::Secondary => Some(100_000.0),
            TreatmentLevel::Tertiary => Some(200_000.0),
            TreatmentLevel::Advanced => Option::None,
        }
    }

    /// Returns the next treatment level, or `None` if already at max.
    pub fn next_level(&self) -> Option<TreatmentLevel> {
        match self {
            TreatmentLevel::None => Some(TreatmentLevel::Primary),
            TreatmentLevel::Primary => Some(TreatmentLevel::Secondary),
            TreatmentLevel::Secondary => Some(TreatmentLevel::Tertiary),
            TreatmentLevel::Tertiary => Some(TreatmentLevel::Advanced),
            TreatmentLevel::Advanced => Option::None,
        }
    }

    /// Base capacity in million gallons per day (MGD) for a plant at this level.
    /// Higher-level plants tend to have lower throughput due to more processing stages.
    pub fn base_capacity_mgd(&self) -> f32 {
        match self {
            TreatmentLevel::None => 0.0,
            TreatmentLevel::Primary => 10.0,
            TreatmentLevel::Secondary => 8.0,
            TreatmentLevel::Tertiary => 5.0,
            TreatmentLevel::Advanced => 3.0,
        }
    }

    /// Display name for the treatment level.
    pub fn name(&self) -> &'static str {
        match self {
            TreatmentLevel::None => "None",
            TreatmentLevel::Primary => "Primary",
            TreatmentLevel::Secondary => "Secondary",
            TreatmentLevel::Tertiary => "Tertiary",
            TreatmentLevel::Advanced => "Advanced",
        }
    }
}

// =============================================================================
// Per-plant tracking
// =============================================================================

/// State for a single treatment plant entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlantState {
    /// Current treatment level.
    pub level: TreatmentLevel,
    /// Maximum capacity in million gallons per day.
    pub capacity_mgd: f32,
    /// Current flow being processed in MGD.
    pub current_flow_mgd: f32,
    /// Effluent quality (0.0 = fully contaminated, 1.0 = pure).
    pub effluent_quality: f32,
    /// Treatment cost incurred this period (dollars).
    pub period_cost: f64,
}

impl PlantState {
    /// Create a new plant state at the given treatment level.
    pub fn new(level: TreatmentLevel) -> Self {
        Self {
            level,
            capacity_mgd: level.base_capacity_mgd(),
            current_flow_mgd: 0.0,
            effluent_quality: 0.0,
            period_cost: 0.0,
        }
    }
}

// =============================================================================
// City-wide water treatment state resource
// =============================================================================

/// City-wide water treatment state, tracking all treatment plants and aggregate metrics.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct WaterTreatmentState {
    /// Map of treatment plant entity IDs to their state.
    pub plants: HashMap<Entity, PlantState>,
    /// Total treatment capacity across all plants (MGD).
    pub total_capacity_mgd: f32,
    /// Total flow currently being processed (MGD).
    pub total_flow_mgd: f32,
    /// Weighted average effluent quality across all plants (0.0-1.0).
    pub avg_effluent_quality: f32,
    /// Total treatment cost this period (dollars).
    pub total_period_cost: f64,
    /// City-wide water demand in MGD (derived from population).
    pub city_demand_mgd: f32,
    /// Fraction of demand being treated (0.0-1.0).
    pub treatment_coverage: f32,
    /// Average input water quality before treatment (0.0-1.0).
    pub avg_input_quality: f32,
    /// Disease risk factor from untreated/poorly treated water (0.0 = safe, 1.0 = critical).
    pub disease_risk: f32,
}

impl Default for WaterTreatmentState {
    fn default() -> Self {
        Self {
            plants: HashMap::new(),
            total_capacity_mgd: 0.0,
            total_flow_mgd: 0.0,
            avg_effluent_quality: 0.0,
            total_period_cost: 0.0,
            city_demand_mgd: 0.0,
            treatment_coverage: 0.0,
            avg_input_quality: 0.5,
            disease_risk: 0.0,
        }
    }
}

impl WaterTreatmentState {
    /// Attempt to upgrade a plant to the next treatment level.
    /// Returns the upgrade cost if successful, or `None` if already at max or entity not found.
    pub fn upgrade_plant(&mut self, entity: Entity) -> Option<f64> {
        let plant = self.plants.get_mut(&entity)?;
        let cost = plant.level.upgrade_cost()?;
        let next = plant.level.next_level()?;
        plant.level = next;
        plant.capacity_mgd = next.base_capacity_mgd();
        Some(cost)
    }

    /// Register a new treatment plant entity at the given level.
    /// If the entity already exists, its state is replaced.
    pub fn register_plant(&mut self, entity: Entity, level: TreatmentLevel) {
        self.plants.insert(entity, PlantState::new(level));
    }

    /// Remove a treatment plant entity (e.g. when demolished).
    pub fn remove_plant(&mut self, entity: Entity) {
        self.plants.remove(&entity);
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Calculate effluent quality given input quality and treatment level.
///
/// Effluent = 1.0 - (1.0 - input_quality) * (1.0 - removal_efficiency)
///
/// For example, with input quality 0.3 (30% clean) and Primary treatment (60% removal):
///   effluent = 1.0 - (0.7 * 0.4) = 1.0 - 0.28 = 0.72
pub fn calculate_effluent_quality(input_quality: f32, level: TreatmentLevel) -> f32 {
    let contamination = 1.0 - input_quality.clamp(0.0, 1.0);
    let remaining = contamination * (1.0 - level.removal_efficiency());
    (1.0 - remaining).clamp(0.0, 1.0)
}

/// Calculate disease risk from drinking water quality.
///
/// Quality 0.0 (fully contaminated) = risk 1.0
/// Quality 0.6 = moderate risk
/// Quality >= 0.85 = negligible risk (< 0.05)
/// Quality >= 0.95 = essentially zero risk
pub fn calculate_disease_risk(drinking_water_quality: f32) -> f32 {
    let q = drinking_water_quality.clamp(0.0, 1.0);
    if q >= 0.95 {
        0.0
    } else {
        // Quadratic increase as quality drops: deficit² curve
        // At 0.95: 0.0025, at 0.85: 0.0225, at 0.5: 0.25, at 0.0: 1.0
        let deficit = 1.0 - q;
        (deficit * deficit).min(1.0)
    }
}

/// Estimate city water demand in MGD from population count.
///
/// Uses 150 gallons per capita per day (GPCD), converted to MGD.
pub fn estimate_demand_mgd(population: u32) -> f32 {
    const GPCD: f32 = 150.0;
    (population as f32 * GPCD) / 1_000_000.0
}

// =============================================================================
// System
// =============================================================================

/// System that updates water treatment plant state each slow tick.
///
/// - Discovers treatment plant service buildings and registers/removes them.
/// - Applies treatment effectiveness based on each plant's level.
/// - Distributes city demand across plants up to capacity.
/// - Calculates treatment costs and effluent quality.
/// - Computes disease risk from resulting water quality.
#[allow(clippy::too_many_arguments)]
pub fn update_water_treatment(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<WaterTreatmentState>,
    services: Query<(Entity, &ServiceBuilding)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Sync plant registry with existing service buildings ---
    // Collect all current WaterTreatmentPlant entity IDs
    let mut active_entities: Vec<Entity> = Vec::new();
    for (entity, service) in &services {
        if service.service_type != ServiceType::WaterTreatmentPlant {
            continue;
        }
        active_entities.push(entity);

        // Register new plants at Primary level
        if !state.plants.contains_key(&entity) {
            state.register_plant(entity, TreatmentLevel::Primary);
        }
    }

    // Remove plants whose entities no longer exist
    let stale_entities: Vec<Entity> = state
        .plants
        .keys()
        .filter(|e| !active_entities.contains(e))
        .copied()
        .collect();
    for entity in stale_entities {
        state.remove_plant(entity);
    }

    // --- Phase 2: Calculate demand and distribute flow ---
    let input_quality = state.avg_input_quality;
    let city_demand = state.city_demand_mgd;

    // Compute total capacity
    let total_capacity: f32 = state.plants.values().map(|p| p.capacity_mgd).sum();

    // Distribute demand proportionally across plants up to their capacity
    let mut remaining_demand = city_demand;
    let mut total_flow = 0.0_f32;
    let mut weighted_quality_sum = 0.0_f32;
    let mut total_cost = 0.0_f64;

    // Sort plant entities for deterministic iteration
    let mut plant_entities: Vec<Entity> = state.plants.keys().copied().collect();
    plant_entities.sort();

    for entity in &plant_entities {
        let plant = state.plants.get_mut(entity).unwrap();

        if remaining_demand <= 0.0 {
            plant.current_flow_mgd = 0.0;
            plant.effluent_quality = 0.0;
            plant.period_cost = 0.0;
            continue;
        }

        // Allocate flow up to this plant's capacity
        let flow = remaining_demand.min(plant.capacity_mgd);
        plant.current_flow_mgd = flow;
        remaining_demand -= flow;
        total_flow += flow;

        // Calculate effluent quality
        let effluent = calculate_effluent_quality(input_quality, plant.level);
        plant.effluent_quality = effluent;

        // Calculate treatment cost: cost_per_MG * flow_MGD
        let cost = plant.level.cost_per_million_gallons() * flow as f64;
        plant.period_cost = cost;
        total_cost += cost;

        // Weight quality by flow volume
        weighted_quality_sum += effluent * flow;
    }

    // --- Phase 3: Aggregate metrics ---
    state.total_capacity_mgd = total_capacity;
    state.total_flow_mgd = total_flow;
    state.total_period_cost = total_cost;

    // Weighted average effluent quality
    state.avg_effluent_quality = if total_flow > 0.0 {
        weighted_quality_sum / total_flow
    } else {
        0.0
    };

    // Treatment coverage: fraction of demand being treated
    state.treatment_coverage = if city_demand > 0.0 {
        (total_flow / city_demand).min(1.0)
    } else {
        1.0 // No demand = fully covered
    };

    // Disease risk: based on the blended quality of treated + untreated water
    // If not all demand is treated, the untreated portion has input_quality
    let blended_quality = if city_demand > 0.0 {
        let treated_portion = total_flow / city_demand;
        let untreated_portion = 1.0 - treated_portion.min(1.0);
        state.avg_effluent_quality * treated_portion.min(1.0) + input_quality * untreated_portion
    } else {
        1.0 // No demand = no risk
    };

    state.disease_risk = calculate_disease_risk(blended_quality);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // TreatmentLevel enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_treatment_level_default_is_none() {
        assert_eq!(TreatmentLevel::default(), TreatmentLevel::None);
    }

    #[test]
    fn test_removal_efficiency_values() {
        assert!((TreatmentLevel::None.removal_efficiency() - 0.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Primary.removal_efficiency() - 0.60).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Secondary.removal_efficiency() - 0.85).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Tertiary.removal_efficiency() - 0.95).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Advanced.removal_efficiency() - 0.99).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cost_per_million_gallons_values() {
        assert!((TreatmentLevel::None.cost_per_million_gallons() - 0.0).abs() < f64::EPSILON);
        assert!(
            (TreatmentLevel::Primary.cost_per_million_gallons() - 1_000.0).abs() < f64::EPSILON
        );
        assert!(
            (TreatmentLevel::Secondary.cost_per_million_gallons() - 2_000.0).abs() < f64::EPSILON
        );
        assert!(
            (TreatmentLevel::Tertiary.cost_per_million_gallons() - 5_000.0).abs() < f64::EPSILON
        );
        assert!(
            (TreatmentLevel::Advanced.cost_per_million_gallons() - 10_000.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_cost_scales_with_level() {
        // Each successive level should cost more per MG
        let levels = [
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];
        for pair in levels.windows(2) {
            assert!(
                pair[1].cost_per_million_gallons() > pair[0].cost_per_million_gallons(),
                "{:?} should cost more than {:?}",
                pair[1],
                pair[0]
            );
        }
    }

    #[test]
    fn test_upgrade_cost_values() {
        assert_eq!(TreatmentLevel::None.upgrade_cost(), Some(25_000.0));
        assert_eq!(TreatmentLevel::Primary.upgrade_cost(), Some(50_000.0));
        assert_eq!(TreatmentLevel::Secondary.upgrade_cost(), Some(100_000.0));
        assert_eq!(TreatmentLevel::Tertiary.upgrade_cost(), Some(200_000.0));
        assert_eq!(TreatmentLevel::Advanced.upgrade_cost(), Option::None);
    }

    #[test]
    fn test_next_level_chain() {
        let mut level = TreatmentLevel::None;
        let expected = [
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];
        for expected_next in &expected {
            let next = level.next_level().expect("should have a next level");
            assert_eq!(next, *expected_next);
            level = next;
        }
        assert!(
            level.next_level().is_none(),
            "Advanced should have no next level"
        );
    }

    #[test]
    fn test_base_capacity_mgd_values() {
        assert!((TreatmentLevel::None.base_capacity_mgd() - 0.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Primary.base_capacity_mgd() - 10.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Secondary.base_capacity_mgd() - 8.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Tertiary.base_capacity_mgd() - 5.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Advanced.base_capacity_mgd() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_treatment_level_names() {
        assert_eq!(TreatmentLevel::None.name(), "None");
        assert_eq!(TreatmentLevel::Primary.name(), "Primary");
        assert_eq!(TreatmentLevel::Secondary.name(), "Secondary");
        assert_eq!(TreatmentLevel::Tertiary.name(), "Tertiary");
        assert_eq!(TreatmentLevel::Advanced.name(), "Advanced");
    }

    // -------------------------------------------------------------------------
    // PlantState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_plant_state_new() {
        let plant = PlantState::new(TreatmentLevel::Primary);
        assert_eq!(plant.level, TreatmentLevel::Primary);
        assert!((plant.capacity_mgd - 10.0).abs() < f32::EPSILON);
        assert!((plant.current_flow_mgd - 0.0).abs() < f32::EPSILON);
        assert!((plant.effluent_quality - 0.0).abs() < f32::EPSILON);
        assert!((plant.period_cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_plant_state_new_advanced() {
        let plant = PlantState::new(TreatmentLevel::Advanced);
        assert_eq!(plant.level, TreatmentLevel::Advanced);
        assert!((plant.capacity_mgd - 3.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // WaterTreatmentState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_water_treatment_state_default() {
        let state = WaterTreatmentState::default();
        assert!(state.plants.is_empty());
        assert!((state.total_capacity_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.total_flow_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.avg_effluent_quality - 0.0).abs() < f32::EPSILON);
        assert!((state.total_period_cost - 0.0).abs() < f64::EPSILON);
        assert!((state.city_demand_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.treatment_coverage - 0.0).abs() < f32::EPSILON);
        assert!((state.avg_input_quality - 0.5).abs() < f32::EPSILON);
        assert!((state.disease_risk - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_register_and_remove_plant() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(42);
        state.register_plant(entity, TreatmentLevel::Primary);
        assert!(state.plants.contains_key(&entity));
        assert_eq!(state.plants[&entity].level, TreatmentLevel::Primary);

        state.remove_plant(entity);
        assert!(!state.plants.contains_key(&entity));
    }

    #[test]
    fn test_upgrade_plant() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(1);
        state.register_plant(entity, TreatmentLevel::Primary);

        // Upgrade from Primary -> Secondary
        let cost = state.upgrade_plant(entity);
        assert_eq!(cost, Some(50_000.0));
        assert_eq!(state.plants[&entity].level, TreatmentLevel::Secondary);
        assert!((state.plants[&entity].capacity_mgd - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_upgrade_plant_at_max_level() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(2);
        state.register_plant(entity, TreatmentLevel::Advanced);

        let cost = state.upgrade_plant(entity);
        assert!(cost.is_none(), "Advanced should not be upgradeable");
        assert_eq!(state.plants[&entity].level, TreatmentLevel::Advanced);
    }

    #[test]
    fn test_upgrade_nonexistent_plant() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(999);

        let cost = state.upgrade_plant(entity);
        assert!(cost.is_none(), "Nonexistent plant should return None");
    }

    #[test]
    fn test_upgrade_chain_full() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(10);
        state.register_plant(entity, TreatmentLevel::None);

        let expected_costs = [25_000.0, 50_000.0, 100_000.0, 200_000.0];
        let expected_levels = [
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];

        for (i, (&expected_cost, &expected_level)) in expected_costs
            .iter()
            .zip(expected_levels.iter())
            .enumerate()
        {
            let cost = state.upgrade_plant(entity);
            assert_eq!(
                cost,
                Some(expected_cost),
                "Upgrade {} should cost {}",
                i,
                expected_cost
            );
            assert_eq!(state.plants[&entity].level, expected_level);
        }

        // Final upgrade should fail
        assert!(state.upgrade_plant(entity).is_none());
    }

    // -------------------------------------------------------------------------
    // Effluent quality calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effluent_quality_no_treatment() {
        // No treatment: output equals input
        let result = calculate_effluent_quality(0.3, TreatmentLevel::None);
        assert!(
            (result - 0.3).abs() < 0.001,
            "No treatment should pass through input quality, got {}",
            result
        );
    }

    #[test]
    fn test_effluent_quality_primary() {
        // Input 0.3, Primary (60% removal): 1.0 - (0.7 * 0.4) = 0.72
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Primary);
        let expected = 1.0 - (0.7 * 0.4);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_secondary() {
        // Input 0.3, Secondary (85% removal): 1.0 - (0.7 * 0.15) = 0.895
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Secondary);
        let expected = 1.0 - (0.7 * 0.15);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_tertiary() {
        // Input 0.3, Tertiary (95% removal): 1.0 - (0.7 * 0.05) = 0.965
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Tertiary);
        let expected = 1.0 - (0.7 * 0.05);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_advanced() {
        // Input 0.3, Advanced (99% removal): 1.0 - (0.7 * 0.01) = 0.993
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Advanced);
        let expected = 1.0 - (0.7 * 0.01);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_pure_input() {
        // Already pure water: output should stay pure
        let result = calculate_effluent_quality(1.0, TreatmentLevel::Primary);
        assert!(
            (result - 1.0).abs() < 0.001,
            "Pure input should produce pure output, got {}",
            result
        );
    }

    #[test]
    fn test_effluent_quality_fully_contaminated_input() {
        // Fully contaminated (0.0 quality), Primary: 1.0 - (1.0 * 0.4) = 0.6
        let result = calculate_effluent_quality(0.0, TreatmentLevel::Primary);
        assert!(
            (result - 0.60).abs() < 0.001,
            "Expected 0.60 for fully contaminated + Primary, got {}",
            result
        );
    }

    #[test]
    fn test_effluent_quality_increases_with_level() {
        let input = 0.3;
        let levels = [
            TreatmentLevel::None,
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];
        let mut prev_quality = 0.0_f32;
        for level in &levels {
            let quality = calculate_effluent_quality(input, *level);
            assert!(
                quality >= prev_quality,
                "{:?} quality {} should be >= previous {}",
                level,
                quality,
                prev_quality
            );
            prev_quality = quality;
        }
    }

    // -------------------------------------------------------------------------
    // Disease risk tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_disease_risk_pure_water() {
        let risk = calculate_disease_risk(1.0);
        assert!(
            risk.abs() < f32::EPSILON,
            "Pure water should have zero disease risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_high_quality() {
        // Quality 0.95+ = zero risk
        let risk = calculate_disease_risk(0.95);
        assert!(
            risk.abs() < f32::EPSILON,
            "Quality 0.95 should have zero risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_moderate_quality() {
        // Quality 0.85: deficit=0.15, risk=0.15^2=0.0225
        let risk = calculate_disease_risk(0.85);
        assert!(
            (risk - 0.0225).abs() < 0.01,
            "Quality 0.85 should have ~0.0225 risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_low_quality() {
        // Quality 0.5 = moderate risk
        let risk = calculate_disease_risk(0.5);
        let expected = 0.25; // (1.0 - 0.5)^2 = 0.25
        assert!(
            (risk - expected).abs() < 0.01,
            "Quality 0.5 should have ~0.25 risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_fully_contaminated() {
        // Quality 0.0 = max risk (1.0)
        let risk = calculate_disease_risk(0.0);
        assert!(
            (risk - 1.0).abs() < 0.01,
            "Fully contaminated should have risk ~1.0, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_monotonically_decreases_with_quality() {
        let mut prev_risk = calculate_disease_risk(0.0);
        for q in 1..=20 {
            let quality = q as f32 * 0.05;
            let risk = calculate_disease_risk(quality);
            assert!(
                risk <= prev_risk + f32::EPSILON,
                "Risk should decrease with quality: q={}, risk={}, prev={}",
                quality,
                risk,
                prev_risk
            );
            prev_risk = risk;
        }
    }

    // -------------------------------------------------------------------------
    // Demand estimation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_estimate_demand_mgd_zero_population() {
        let demand = estimate_demand_mgd(0);
        assert!(
            demand.abs() < f32::EPSILON,
            "Zero population should have zero demand"
        );
    }

    #[test]
    fn test_estimate_demand_mgd_small_city() {
        // 10,000 people * 150 GPCD = 1,500,000 GPD = 1.5 MGD
        let demand = estimate_demand_mgd(10_000);
        assert!(
            (demand - 1.5).abs() < 0.001,
            "10K population should need 1.5 MGD, got {}",
            demand
        );
    }

    #[test]
    fn test_estimate_demand_mgd_large_city() {
        // 1,000,000 people * 150 GPCD = 150,000,000 GPD = 150 MGD
        let demand = estimate_demand_mgd(1_000_000);
        assert!(
            (demand - 150.0).abs() < 0.1,
            "1M population should need 150 MGD, got {}",
            demand
        );
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (simulating update logic)
    // -------------------------------------------------------------------------

    #[test]
    fn test_treatment_cost_calculation() {
        // A Primary plant processing 5 MGD: 5 * $1,000 = $5,000
        let flow = 5.0_f32;
        let cost = TreatmentLevel::Primary.cost_per_million_gallons() * flow as f64;
        assert!(
            (cost - 5_000.0).abs() < 0.01,
            "Expected $5,000, got ${}",
            cost
        );
    }

    #[test]
    fn test_treatment_cost_advanced_plant() {
        // An Advanced plant processing 2 MGD: 2 * $10,000 = $20,000
        let flow = 2.0_f32;
        let cost = TreatmentLevel::Advanced.cost_per_million_gallons() * flow as f64;
        assert!(
            (cost - 20_000.0).abs() < 0.01,
            "Expected $20,000, got ${}",
            cost
        );
    }

    #[test]
    fn test_capacity_limits_flow() {
        // Plant capacity 10 MGD, city demand 15 MGD => only 10 MGD processed
        let capacity = TreatmentLevel::Primary.base_capacity_mgd();
        let demand = 15.0_f32;
        let flow = demand.min(capacity);
        assert!(
            (flow - 10.0).abs() < f32::EPSILON,
            "Flow should be capped at capacity, got {}",
            flow
        );
    }

    #[test]
    fn test_multiple_plants_aggregate_capacity() {
        let mut state = WaterTreatmentState::default();
        state.register_plant(Entity::from_raw(1), TreatmentLevel::Primary); // 10 MGD
        state.register_plant(Entity::from_raw(2), TreatmentLevel::Secondary); // 8 MGD
        state.register_plant(Entity::from_raw(3), TreatmentLevel::Tertiary); // 5 MGD

        let total: f32 = state.plants.values().map(|p| p.capacity_mgd).sum();
        assert!(
            (total - 23.0).abs() < f32::EPSILON,
            "Total capacity should be 23 MGD, got {}",
            total
        );
    }

    #[test]
    fn test_flow_distribution_under_capacity() {
        // Two plants: Primary (10 MGD) + Secondary (8 MGD) = 18 MGD capacity
        // City demand: 12 MGD
        // First plant gets 10 MGD (full), second gets 2 MGD
        let plants = vec![
            PlantState::new(TreatmentLevel::Primary),
            PlantState::new(TreatmentLevel::Secondary),
        ];

        let mut remaining = 12.0_f32;
        let mut flows = Vec::new();

        for plant in &plants {
            let flow = remaining.min(plant.capacity_mgd);
            flows.push(flow);
            remaining -= flow;
        }

        assert!((flows[0] - 10.0).abs() < f32::EPSILON);
        assert!((flows[1] - 2.0).abs() < f32::EPSILON);
        assert!(remaining.abs() < f32::EPSILON);
    }

    #[test]
    fn test_flow_distribution_over_capacity() {
        // Single Primary plant (10 MGD), demand 15 MGD => 5 MGD untreated
        let plant = PlantState::new(TreatmentLevel::Primary);
        let demand = 15.0_f32;
        let flow = demand.min(plant.capacity_mgd);
        let untreated = demand - flow;

        assert!((flow - 10.0).abs() < f32::EPSILON);
        assert!((untreated - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blended_quality_partial_treatment() {
        // 60% of water treated to quality 0.9, 40% untreated at quality 0.3
        let treated_fraction = 0.6_f32;
        let untreated_fraction = 0.4_f32;
        let treated_quality = 0.9_f32;
        let input_quality = 0.3_f32;

        let blended = treated_quality * treated_fraction + input_quality * untreated_fraction;
        // 0.9 * 0.6 + 0.3 * 0.4 = 0.54 + 0.12 = 0.66
        assert!(
            (blended - 0.66).abs() < 0.001,
            "Blended quality should be 0.66, got {}",
            blended
        );
    }

    #[test]
    fn test_no_plants_no_treatment() {
        let state = WaterTreatmentState::default();
        assert!(state.plants.is_empty());
        assert_eq!(state.total_capacity_mgd, 0.0);
        assert_eq!(state.total_flow_mgd, 0.0);
        assert_eq!(state.avg_effluent_quality, 0.0);
    }

    #[test]
    fn test_effluent_quality_clamped() {
        // Input quality beyond bounds should be clamped
        let result = calculate_effluent_quality(1.5, TreatmentLevel::Advanced);
        assert!(result <= 1.0, "Quality should be clamped to 1.0");
        assert!(result >= 0.0, "Quality should be non-negative");

        let result_neg = calculate_effluent_quality(-0.5, TreatmentLevel::Advanced);
        assert!(result_neg <= 1.0, "Quality should be clamped to 1.0");
        assert!(result_neg >= 0.0, "Quality should be non-negative");
    }
}

pub struct WaterTreatmentPlugin;

impl Plugin for WaterTreatmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTreatmentState>().add_systems(
            FixedUpdate,
            update_water_treatment.after(crate::imports_exports::process_trade),
        );
    }
}
