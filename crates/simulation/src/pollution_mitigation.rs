//! POLL-004: Air Pollution Mitigation Policies and Technology Upgrades
//!
//! Provides player-activated pollution mitigation policies that reduce emission
//! rates from specific source categories. Each policy has distinct tradeoffs:
//!
//! | Policy                  | Effect                              | Tradeoff            |
//! |-------------------------|-------------------------------------|---------------------|
//! | Scrubbers on Power Plants | -50% power plant emissions        | +50% O&M cost       |
//! | Catalytic Converters    | -30% road emissions                 | —                   |
//! | Electric Vehicle Mandate| -60% road emissions (phased 5yr)    | Phased rollout      |
//! | Emissions Cap           | -20% all industrial emissions       | -10% industrial profit |
//!
//! The system computes per-source-category emission weights from the live world
//! state and applies a weighted composite reduction to the pollution grid after
//! the Gaussian plume system runs.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::building_emissions::{self, SourceCategory};
use crate::buildings::Building;
use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::pollution::PollutionGrid;
use crate::services::ServiceBuilding;
use crate::time_of_day::GameClock;
use crate::traffic::TrafficGrid;
use crate::SimulationSet;
use crate::SlowTickTimer;
use crate::{decode_or_warn, Saveable};

// =============================================================================
// Constants
// =============================================================================

/// Number of game days over which the EV mandate phases in (5 years * 360 days).
const EV_MANDATE_PHASE_DAYS: u32 = 5 * 360;

/// Scrubber power plant emission reduction (50%).
const SCRUBBER_REDUCTION: f32 = 0.50;

/// Catalytic converter road emission reduction (30%).
const CATALYTIC_REDUCTION: f32 = 0.30;

/// EV mandate road emission reduction at full phase-in (60%).
const EV_MANDATE_FULL_REDUCTION: f32 = 0.60;

/// Emissions cap industrial emission reduction (20%).
const EMISSIONS_CAP_REDUCTION: f32 = 0.20;

/// Emissions cap industrial profit penalty (10%).
pub const EMISSIONS_CAP_PROFIT_PENALTY: f32 = 0.10;

/// Coal/gas/WTE/biomass/oil power plants base Q values (matching wind_pollution).
const COAL_Q: f32 = 100.0;
const GAS_Q: f32 = 35.0;
const WTE_Q: f32 = 20.0;
const BIOMASS_Q: f32 = 25.0;
const OIL_Q: f32 = 75.0;

// =============================================================================
// Resource
// =============================================================================

/// Tracks which pollution mitigation policies the player has activated.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
pub struct PollutionMitigationPolicies {
    /// Scrubbers on Power Plants: reduces power plant emissions by 50%.
    pub scrubbers_on_power_plants: bool,
    /// Catalytic Converters: reduces road emissions by 30%.
    pub catalytic_converters: bool,
    /// Electric Vehicle Mandate: reduces road emissions by up to 60%,
    /// phased in over 5 game-years from activation.
    pub ev_mandate: bool,
    /// The game day when the EV mandate was activated (for phase-in tracking).
    pub ev_mandate_activation_day: Option<u32>,
    /// Emissions Cap: reduces all industrial emissions by 20%,
    /// but also reduces industrial profit by 10%.
    pub emissions_cap: bool,
}


impl PollutionMitigationPolicies {
    /// Returns true if no policies are active (used for save skip optimization).
    pub fn is_default(&self) -> bool {
        !self.scrubbers_on_power_plants
            && !self.catalytic_converters
            && !self.ev_mandate
            && !self.emissions_cap
    }

    /// Power plant emission multiplier (1.0 = no reduction).
    pub fn power_plant_multiplier(&self) -> f32 {
        if self.scrubbers_on_power_plants {
            1.0 - SCRUBBER_REDUCTION
        } else {
            1.0
        }
    }

    /// Road emission multiplier (1.0 = no reduction).
    /// Catalytic converters and EV mandate stack multiplicatively.
    pub fn road_multiplier(&self, current_day: u32) -> f32 {
        let mut mult = 1.0_f32;
        if self.catalytic_converters {
            mult *= 1.0 - CATALYTIC_REDUCTION;
        }
        if self.ev_mandate {
            let ev_reduction = self.ev_mandate_phase_fraction(current_day) * EV_MANDATE_FULL_REDUCTION;
            mult *= 1.0 - ev_reduction;
        }
        mult
    }

    /// Industrial emission multiplier (1.0 = no reduction).
    pub fn industrial_multiplier(&self) -> f32 {
        if self.emissions_cap {
            1.0 - EMISSIONS_CAP_REDUCTION
        } else {
            1.0
        }
    }

    /// Industrial profit multiplier (1.0 = no penalty).
    pub fn industrial_profit_multiplier(&self) -> f32 {
        if self.emissions_cap {
            1.0 - EMISSIONS_CAP_PROFIT_PENALTY
        } else {
            1.0
        }
    }

    /// Fraction of EV mandate phase-in complete (0.0 to 1.0).
    pub fn ev_mandate_phase_fraction(&self, current_day: u32) -> f32 {
        match self.ev_mandate_activation_day {
            Some(activation_day) => {
                let elapsed = current_day.saturating_sub(activation_day);
                (elapsed as f32 / EV_MANDATE_PHASE_DAYS as f32).clamp(0.0, 1.0)
            }
            None => 0.0,
        }
    }
}

// =============================================================================
// Saveable
// =============================================================================

impl Saveable for PollutionMitigationPolicies {
    const SAVE_KEY: &'static str = "pollution_mitigation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.is_default() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn::<Self>(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Tracks the EV mandate activation day when the policy is toggled on.
fn track_ev_mandate_activation(
    mut policies: ResMut<PollutionMitigationPolicies>,
    clock: Res<GameClock>,
) {
    if policies.ev_mandate && policies.ev_mandate_activation_day.is_none() {
        policies.ev_mandate_activation_day = Some(clock.day);
    }
    if !policies.ev_mandate {
        policies.ev_mandate_activation_day = None;
    }
}

/// Applies pollution mitigation reductions to the pollution grid after the
/// Gaussian plume system has computed base pollution values.
///
/// Computes per-source-category total emission Q from the live world state,
/// calculates the fractional contribution of each category, and applies
/// policy-specific reductions as a composite multiplier on the grid.
#[allow(clippy::too_many_arguments)]
fn apply_pollution_mitigation(
    slow_tick: Res<SlowTickTimer>,
    mitigation: Res<PollutionMitigationPolicies>,
    mut pollution: ResMut<PollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    power_plants: Query<&PowerPlant>,
    services: Query<&ServiceBuilding>,
    traffic: Res<TrafficGrid>,
    clock: Res<GameClock>,
) {
    if !slow_tick.should_run() {
        return;
    }
    if mitigation.is_default() {
        return;
    }

    // Compute total Q contribution by source category
    let mut total_q = 0.0_f32;
    let mut power_plant_q = 0.0_f32;
    let mut road_q = 0.0_f32;
    let mut industrial_q = 0.0_f32;

    // Roads
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                let congestion = traffic.congestion_level(x, y);
                let q = building_emissions::road_emission_q(congestion);
                road_q += q;
                total_q += q;
            }
        }
    }

    // Power plants
    for plant in &power_plants {
        let base_q = match plant.plant_type {
            PowerPlantType::Coal => COAL_Q,
            PowerPlantType::NaturalGas => GAS_Q,
            PowerPlantType::WasteToEnergy => WTE_Q,
            PowerPlantType::Biomass => BIOMASS_Q,
            PowerPlantType::Oil => OIL_Q,
            _ => 0.0,
        };
        power_plant_q += base_q;
        total_q += base_q;
    }

    // Zoned buildings
    for building in &buildings {
        if let Some(profile) =
            building_emissions::building_emission_profile(building.zone_type, building.level)
        {
            total_q += profile.base_q;
            if profile.category == SourceCategory::Industrial {
                industrial_q += profile.base_q;
            }
        }
    }

    // Service buildings (combustion)
    for service in &services {
        if let Some(profile) =
            building_emissions::service_emission_profile(service.service_type)
        {
            total_q += profile.base_q;
        }
    }

    if total_q <= 0.0 {
        return;
    }

    // Compute composite reduction: the fraction of total Q that is removed
    let pp_mult = mitigation.power_plant_multiplier();
    let road_mult = mitigation.road_multiplier(clock.day);
    let ind_mult = mitigation.industrial_multiplier();

    let pp_reduction = (power_plant_q / total_q) * (1.0 - pp_mult);
    let road_reduction = (road_q / total_q) * (1.0 - road_mult);
    let ind_reduction = (industrial_q / total_q) * (1.0 - ind_mult);

    let composite_multiplier = (1.0 - pp_reduction - road_reduction - ind_reduction).clamp(0.0, 1.0);

    // Apply to the entire pollution grid
    if (composite_multiplier - 1.0).abs() < 0.001 {
        return; // No meaningful reduction
    }

    for level in pollution.levels.iter_mut() {
        *level = (*level as f32 * composite_multiplier).clamp(0.0, 255.0) as u8;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct PollutionMitigationPlugin;

impl Plugin for PollutionMitigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PollutionMitigationPolicies>();

        // Register for save/load
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<PollutionMitigationPolicies>();

        app.add_systems(
            FixedUpdate,
            (
                track_ev_mandate_activation.in_set(SimulationSet::PreSim),
                apply_pollution_mitigation
                    .after(crate::wind_pollution::update_pollution_gaussian_plume)
                    .in_set(SimulationSet::Simulation),
            ),
        );
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policies_have_no_effect() {
        let policies = PollutionMitigationPolicies::default();
        assert!((policies.power_plant_multiplier() - 1.0).abs() < f32::EPSILON);
        assert!((policies.road_multiplier(100) - 1.0).abs() < f32::EPSILON);
        assert!((policies.industrial_multiplier() - 1.0).abs() < f32::EPSILON);
        assert!((policies.industrial_profit_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scrubbers_halve_power_plant_emissions() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.scrubbers_on_power_plants = true;
        assert!((policies.power_plant_multiplier() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_catalytic_converters_reduce_road_emissions() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.catalytic_converters = true;
        assert!((policies.road_multiplier(100) - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ev_mandate_phases_in_over_five_years() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.ev_mandate = true;
        policies.ev_mandate_activation_day = Some(0);

        // At activation: 0% reduction
        assert!((policies.ev_mandate_phase_fraction(0) - 0.0).abs() < f32::EPSILON);

        // At 50% through phase-in
        let half_way = EV_MANDATE_PHASE_DAYS / 2;
        let frac = policies.ev_mandate_phase_fraction(half_way);
        assert!(
            (frac - 0.5).abs() < 0.01,
            "Expected ~0.5, got {frac}"
        );

        // At full phase-in
        let full = policies.ev_mandate_phase_fraction(EV_MANDATE_PHASE_DAYS);
        assert!((full - 1.0).abs() < f32::EPSILON);

        // Beyond full phase-in (clamped to 1.0)
        let beyond = policies.ev_mandate_phase_fraction(EV_MANDATE_PHASE_DAYS + 1000);
        assert!((beyond - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ev_mandate_road_multiplier_at_full_phase() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.ev_mandate = true;
        policies.ev_mandate_activation_day = Some(0);

        let mult = policies.road_multiplier(EV_MANDATE_PHASE_DAYS);
        // 1.0 - 0.60 = 0.40
        assert!(
            (mult - 0.4).abs() < 0.01,
            "Expected ~0.4, got {mult}"
        );
    }

    #[test]
    fn test_catalytic_and_ev_stack_multiplicatively() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.catalytic_converters = true;
        policies.ev_mandate = true;
        policies.ev_mandate_activation_day = Some(0);

        let mult = policies.road_multiplier(EV_MANDATE_PHASE_DAYS);
        // 0.7 * 0.4 = 0.28
        assert!(
            (mult - 0.28).abs() < 0.01,
            "Expected ~0.28, got {mult}"
        );
    }

    #[test]
    fn test_emissions_cap_reduces_industrial() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.emissions_cap = true;
        assert!((policies.industrial_multiplier() - 0.8).abs() < f32::EPSILON);
        assert!((policies.industrial_profit_multiplier() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_default_detects_no_policies() {
        let policies = PollutionMitigationPolicies::default();
        assert!(policies.is_default());
    }

    #[test]
    fn test_is_default_false_when_policy_active() {
        let mut policies = PollutionMitigationPolicies::default();
        policies.catalytic_converters = true;
        assert!(!policies.is_default());
    }
}

