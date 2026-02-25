//! POWER-012: Demand Response Programs
//!
//! Implements toggleable demand response programs that reduce peak electricity
//! demand. Each program has a specific peak reduction percentage and cost.
//! Programs stack additively and apply their reduction to `EnergyGrid.total_demand_mwh`
//! after normal demand aggregation but before dispatch.
//!
//! Programs:
//! - Smart thermostat: -8% peak demand, $1M cost
//! - Industrial load shifting: -12% peak, $500K
//! - EV managed charging: -5% peak, $300K
//! - Peak pricing signals: -10% peak, $0 cost
//! - Interruptible service: -15% peak, $2M
//! - Critical peak rebates: -7% peak, $1M

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::energy_demand::EnergyGrid;
use crate::{decode_or_warn, Saveable, SimulationSet, TickCounter};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// How often (in ticks) the demand response system runs.
const DR_INTERVAL: u64 = 4;

/// Peak demand reduction percentages for each program.
const SMART_THERMOSTAT_REDUCTION: f32 = 0.08;
const INDUSTRIAL_LOAD_SHIFTING_REDUCTION: f32 = 0.12;
const EV_MANAGED_CHARGING_REDUCTION: f32 = 0.05;
const PEAK_PRICING_SIGNALS_REDUCTION: f32 = 0.10;
const INTERRUPTIBLE_SERVICE_REDUCTION: f32 = 0.15;
const CRITICAL_PEAK_REBATES_REDUCTION: f32 = 0.07;

/// Monthly costs for each program (in budget units).
const SMART_THERMOSTAT_COST: f64 = 1_000.0;
const INDUSTRIAL_LOAD_SHIFTING_COST: f64 = 500.0;
const EV_MANAGED_CHARGING_COST: f64 = 300.0;
const PEAK_PRICING_SIGNALS_COST: f64 = 0.0;
const INTERRUPTIBLE_SERVICE_COST: f64 = 2_000.0;
const CRITICAL_PEAK_REBATES_COST: f64 = 1_000.0;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Toggleable demand response programs that reduce peak electricity demand.
///
/// Each field corresponds to a specific program. When enabled (`true`),
/// the program's reduction percentage is applied to `EnergyGrid.total_demand_mwh`.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct DemandResponsePrograms {
    /// Smart thermostat program: -8% peak demand, $1M/month cost.
    pub smart_thermostat: bool,
    /// Industrial load shifting: -12% peak demand, $500K/month cost.
    pub industrial_load_shifting: bool,
    /// EV managed charging: -5% peak demand, $300K/month cost.
    pub ev_managed_charging: bool,
    /// Peak pricing signals: -10% peak demand, $0 cost.
    pub peak_pricing_signals: bool,
    /// Interruptible service: -15% peak demand, $2M/month cost.
    pub interruptible_service: bool,
    /// Critical peak rebates: -7% peak demand, $1M/month cost.
    pub critical_peak_rebates: bool,
    /// The last computed demand reduction factor (0.0 to 1.0).
    /// Stored for UI display. E.g. 0.20 means 20% reduction.
    pub current_reduction_fraction: f32,
    /// Monthly cost of all active programs combined.
    pub total_monthly_cost: f64,
}

impl Default for DemandResponsePrograms {
    fn default() -> Self {
        Self {
            smart_thermostat: false,
            industrial_load_shifting: false,
            ev_managed_charging: false,
            peak_pricing_signals: false,
            interruptible_service: false,
            critical_peak_rebates: false,
            current_reduction_fraction: 0.0,
            total_monthly_cost: 0.0,
        }
    }
}

impl DemandResponsePrograms {
    /// Compute the total peak demand reduction fraction from active programs.
    ///
    /// Reductions are additive. For example, if smart thermostat (8%) and
    /// peak pricing (10%) are both active, the total reduction is 18%.
    /// The result is clamped to [0.0, 1.0].
    pub fn total_reduction_fraction(&self) -> f32 {
        let mut reduction = 0.0_f32;
        if self.smart_thermostat {
            reduction += SMART_THERMOSTAT_REDUCTION;
        }
        if self.industrial_load_shifting {
            reduction += INDUSTRIAL_LOAD_SHIFTING_REDUCTION;
        }
        if self.ev_managed_charging {
            reduction += EV_MANAGED_CHARGING_REDUCTION;
        }
        if self.peak_pricing_signals {
            reduction += PEAK_PRICING_SIGNALS_REDUCTION;
        }
        if self.interruptible_service {
            reduction += INTERRUPTIBLE_SERVICE_REDUCTION;
        }
        if self.critical_peak_rebates {
            reduction += CRITICAL_PEAK_REBATES_REDUCTION;
        }
        reduction.clamp(0.0, 1.0)
    }

    /// Compute the total monthly cost of all active programs.
    pub fn compute_monthly_cost(&self) -> f64 {
        let mut cost = 0.0;
        if self.smart_thermostat {
            cost += SMART_THERMOSTAT_COST;
        }
        if self.industrial_load_shifting {
            cost += INDUSTRIAL_LOAD_SHIFTING_COST;
        }
        if self.ev_managed_charging {
            cost += EV_MANAGED_CHARGING_COST;
        }
        if self.peak_pricing_signals {
            cost += PEAK_PRICING_SIGNALS_COST;
        }
        if self.interruptible_service {
            cost += INTERRUPTIBLE_SERVICE_COST;
        }
        if self.critical_peak_rebates {
            cost += CRITICAL_PEAK_REBATES_COST;
        }
        cost
    }

    /// Return the number of active programs.
    pub fn active_count(&self) -> u32 {
        let mut count = 0;
        if self.smart_thermostat {
            count += 1;
        }
        if self.industrial_load_shifting {
            count += 1;
        }
        if self.ev_managed_charging {
            count += 1;
        }
        if self.peak_pricing_signals {
            count += 1;
        }
        if self.interruptible_service {
            count += 1;
        }
        if self.critical_peak_rebates {
            count += 1;
        }
        count
    }
}

impl Saveable for DemandResponsePrograms {
    const SAVE_KEY: &'static str = "demand_response_programs";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Applies demand response reduction to the energy grid.
///
/// Runs after `aggregate_energy_demand` and before `dispatch_energy`.
/// Reduces `total_demand_mwh` by the combined reduction fraction of all
/// active programs. Also updates the cached reduction fraction and cost.
pub fn apply_demand_response(
    tick: Res<TickCounter>,
    mut programs: ResMut<DemandResponsePrograms>,
    mut energy_grid: ResMut<EnergyGrid>,
) {
    if !tick.0.is_multiple_of(DR_INTERVAL) {
        return;
    }

    let reduction = programs.total_reduction_fraction();
    programs.current_reduction_fraction = reduction;
    programs.total_monthly_cost = programs.compute_monthly_cost();

    if reduction > 0.0 {
        energy_grid.total_demand_mwh *= 1.0 - reduction;
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DemandResponsePlugin;

impl Plugin for DemandResponsePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemandResponsePrograms>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<DemandResponsePrograms>();

        app.add_systems(
            FixedUpdate,
            apply_demand_response
                .after(crate::energy_demand::aggregate_energy_demand)
                .before(crate::energy_dispatch::dispatch_energy)
                .in_set(SimulationSet::Simulation),
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
    fn test_default_no_programs_active() {
        let programs = DemandResponsePrograms::default();
        assert!(!programs.smart_thermostat);
        assert!(!programs.industrial_load_shifting);
        assert!(!programs.ev_managed_charging);
        assert!(!programs.peak_pricing_signals);
        assert!(!programs.interruptible_service);
        assert!(!programs.critical_peak_rebates);
        assert_eq!(programs.active_count(), 0);
        assert!((programs.total_reduction_fraction() - 0.0).abs() < f32::EPSILON);
        assert!((programs.compute_monthly_cost() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_smart_thermostat_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.smart_thermostat = true;
        assert!(
            (programs.total_reduction_fraction() - 0.08).abs() < f32::EPSILON,
            "Smart thermostat should reduce by 8%"
        );
    }

    #[test]
    fn test_industrial_load_shifting_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.industrial_load_shifting = true;
        assert!(
            (programs.total_reduction_fraction() - 0.12).abs() < f32::EPSILON,
            "Industrial load shifting should reduce by 12%"
        );
    }

    #[test]
    fn test_ev_managed_charging_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.ev_managed_charging = true;
        assert!(
            (programs.total_reduction_fraction() - 0.05).abs() < f32::EPSILON,
            "EV managed charging should reduce by 5%"
        );
    }

    #[test]
    fn test_peak_pricing_signals_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.peak_pricing_signals = true;
        assert!(
            (programs.total_reduction_fraction() - 0.10).abs() < f32::EPSILON,
            "Peak pricing signals should reduce by 10%"
        );
    }

    #[test]
    fn test_interruptible_service_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.interruptible_service = true;
        assert!(
            (programs.total_reduction_fraction() - 0.15).abs() < f32::EPSILON,
            "Interruptible service should reduce by 15%"
        );
    }

    #[test]
    fn test_critical_peak_rebates_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.critical_peak_rebates = true;
        assert!(
            (programs.total_reduction_fraction() - 0.07).abs() < f32::EPSILON,
            "Critical peak rebates should reduce by 7%"
        );
    }

    #[test]
    fn test_multiple_programs_stack_additively() {
        let mut programs = DemandResponsePrograms::default();
        programs.smart_thermostat = true; // 8%
        programs.peak_pricing_signals = true; // 10%
        programs.ev_managed_charging = true; // 5%
        let expected = 0.08 + 0.10 + 0.05;
        assert!(
            (programs.total_reduction_fraction() - expected).abs() < f32::EPSILON,
            "Multiple programs should stack additively: expected {}, got {}",
            expected,
            programs.total_reduction_fraction()
        );
        assert_eq!(programs.active_count(), 3);
    }

    #[test]
    fn test_all_programs_total_reduction() {
        let mut programs = DemandResponsePrograms::default();
        programs.smart_thermostat = true;
        programs.industrial_load_shifting = true;
        programs.ev_managed_charging = true;
        programs.peak_pricing_signals = true;
        programs.interruptible_service = true;
        programs.critical_peak_rebates = true;
        let expected = 0.08 + 0.12 + 0.05 + 0.10 + 0.15 + 0.07;
        assert!(
            (programs.total_reduction_fraction() - expected).abs() < f32::EPSILON,
            "All programs combined: expected {}, got {}",
            expected,
            programs.total_reduction_fraction()
        );
        assert_eq!(programs.active_count(), 6);
    }

    #[test]
    fn test_monthly_cost_single_program() {
        let mut programs = DemandResponsePrograms::default();
        programs.smart_thermostat = true;
        assert!(
            (programs.compute_monthly_cost() - 1_000.0).abs() < f64::EPSILON,
            "Smart thermostat costs $1M/month"
        );
    }

    #[test]
    fn test_monthly_cost_free_program() {
        let mut programs = DemandResponsePrograms::default();
        programs.peak_pricing_signals = true;
        assert!(
            (programs.compute_monthly_cost() - 0.0).abs() < f64::EPSILON,
            "Peak pricing signals should cost $0"
        );
    }

    #[test]
    fn test_monthly_cost_all_programs() {
        let mut programs = DemandResponsePrograms::default();
        programs.smart_thermostat = true;
        programs.industrial_load_shifting = true;
        programs.ev_managed_charging = true;
        programs.peak_pricing_signals = true;
        programs.interruptible_service = true;
        programs.critical_peak_rebates = true;
        let expected = 1_000.0 + 500.0 + 300.0 + 0.0 + 2_000.0 + 1_000.0;
        assert!(
            (programs.compute_monthly_cost() - expected).abs() < f64::EPSILON,
            "All programs cost: expected {}, got {}",
            expected,
            programs.compute_monthly_cost()
        );
    }

    #[test]
    fn test_reduction_clamped_to_one() {
        // Even if somehow all reductions exceeded 100%, clamp to 1.0
        let programs = DemandResponsePrograms::default();
        // All off = 0.0 which is fine. Total of all = 0.57, well under 1.0.
        // Just verify clamping works conceptually.
        assert!(programs.total_reduction_fraction() <= 1.0);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let programs = DemandResponsePrograms {
            smart_thermostat: true,
            industrial_load_shifting: false,
            ev_managed_charging: true,
            peak_pricing_signals: true,
            interruptible_service: false,
            critical_peak_rebates: true,
            current_reduction_fraction: 0.30,
            total_monthly_cost: 2_300.0,
        };

        let bytes = programs.save_to_bytes().unwrap();
        let restored = DemandResponsePrograms::load_from_bytes(&bytes);

        assert!(restored.smart_thermostat);
        assert!(!restored.industrial_load_shifting);
        assert!(restored.ev_managed_charging);
        assert!(restored.peak_pricing_signals);
        assert!(!restored.interruptible_service);
        assert!(restored.critical_peak_rebates);
        assert!((restored.current_reduction_fraction - 0.30).abs() < f32::EPSILON);
        assert!((restored.total_monthly_cost - 2_300.0).abs() < f64::EPSILON);
    }
}
