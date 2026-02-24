//! POWER-010: Time-of-Use Electricity Pricing and Revenue
//!
//! Implements time-of-use electricity pricing that affects city revenue and
//! citizen costs. The system calculates kWh prices based on time of day and
//! grid scarcity, then computes revenue (from selling electricity to consumers)
//! and generation costs (fuel + O&M). Net energy income is added to the city
//! treasury periodically.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::energy_demand::EnergyGrid;
use crate::energy_dispatch::EnergyDispatchState;
use crate::time_of_day::GameClock;
use crate::{decode_or_warn, Saveable, SaveableRegistry, SimulationSet, TickCounter};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// How often (in ticks) the pricing system runs.
const PRICING_INTERVAL: u64 = 4;


// ---------------------------------------------------------------------------
// Time-of-use period
// ---------------------------------------------------------------------------

/// Time-of-use period for electricity pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum TimeOfUsePeriod {
    /// Off-peak: 22:00 - 06:00 (multiplier 0.6)
    OffPeak,
    /// Mid-peak: 06:00 - 14:00 (multiplier 1.0)
    MidPeak,
    /// On-peak: 14:00 - 22:00 (multiplier 1.5)
    OnPeak,
}

impl TimeOfUsePeriod {
    /// Determine the time-of-use period from the hour of day.
    pub fn from_hour(hour: f32) -> Self {
        let h = hour as u32;
        match h {
            22..=23 | 0..=5 => Self::OffPeak,
            6..=13 => Self::MidPeak,
            14..=21 => Self::OnPeak,
            _ => Self::MidPeak,
        }
    }
}

// ---------------------------------------------------------------------------
// EnergyPricingConfig resource
// ---------------------------------------------------------------------------

/// Configuration for time-of-use electricity pricing.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct EnergyPricingConfig {
    /// Base electricity rate in $/kWh.
    pub base_rate_per_kwh: f32,
    /// Multiplier for off-peak hours (22:00 - 06:00).
    pub off_peak_multiplier: f32,
    /// Multiplier for mid-peak hours (06:00 - 14:00).
    pub mid_peak_multiplier: f32,
    /// Multiplier for on-peak hours (14:00 - 22:00).
    pub on_peak_multiplier: f32,
    /// Generation cost per MWh (fuel + O&M baseline).
    pub generation_cost_per_mwh: f32,
}

impl Default for EnergyPricingConfig {
    fn default() -> Self {
        Self {
            base_rate_per_kwh: 0.12,
            off_peak_multiplier: 0.6,
            mid_peak_multiplier: 1.0,
            on_peak_multiplier: 1.5,
            generation_cost_per_mwh: 25.0,
        }
    }
}

impl Saveable for EnergyPricingConfig {
    const SAVE_KEY: &'static str = "energy_pricing_config";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// EnergyEconomics resource
// ---------------------------------------------------------------------------

/// Tracks energy revenue and costs for the city budget.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct EnergyEconomics {
    /// Current effective electricity price ($/kWh) after time-of-use and
    /// scarcity multipliers.
    pub current_price_per_kwh: f32,
    /// Current time-of-use period.
    pub current_period: TimeOfUsePeriod,
    /// Current time-of-use multiplier.
    pub tou_multiplier: f32,
    /// Current scarcity multiplier based on reserve margin.
    pub scarcity_multiplier: f32,
    /// Revenue from electricity sales this billing cycle ($/month).
    pub total_revenue: f64,
    /// Total generation costs this billing cycle ($/month).
    pub total_costs: f64,
    /// Net energy income: total_revenue - total_costs.
    pub net_income: f64,
    /// Revenue breakdown by consumer type.
    pub residential_revenue: f64,
    pub commercial_revenue: f64,
    pub industrial_revenue: f64,
    /// Total energy consumed this cycle (MWh).
    pub total_consumption_mwh: f32,
    /// Total energy generated this cycle (MWh).
    pub total_generation_mwh: f32,
    /// Day of last billing cycle reset.
    pub last_billing_day: u32,
    /// Cumulative energy cost impact on citizens (used by happiness system).
    /// Higher values indicate citizens are paying more for energy.
    pub citizen_cost_burden: f32,
}

impl Default for EnergyEconomics {
    fn default() -> Self {
        Self {
            current_price_per_kwh: 0.12,
            current_period: TimeOfUsePeriod::MidPeak,
            tou_multiplier: 1.0,
            scarcity_multiplier: 1.0,
            total_revenue: 0.0,
            total_costs: 0.0,
            net_income: 0.0,
            residential_revenue: 0.0,
            commercial_revenue: 0.0,
            industrial_revenue: 0.0,
            total_consumption_mwh: 0.0,
            total_generation_mwh: 0.0,
            last_billing_day: 0,
            citizen_cost_burden: 0.0,
        }
    }
}

impl Saveable for EnergyEconomics {
    const SAVE_KEY: &'static str = "energy_economics";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Scarcity multiplier calculation
// ---------------------------------------------------------------------------

/// Compute the scarcity multiplier based on reserve margin.
///
/// - Reserve margin > 20%: 1.0 (no scarcity)
/// - 10% - 20%: 1.2
/// - 5% - 10%: 1.5
/// - 0% - 5%: 2.0
/// - Deficit (< 0%): 3.0
pub fn scarcity_multiplier_from_reserve(reserve_margin: f32) -> f32 {
    if reserve_margin < 0.0 {
        3.0
    } else if reserve_margin < 0.05 {
        2.0
    } else if reserve_margin < 0.10 {
        1.5
    } else if reserve_margin < 0.20 {
        1.2
    } else {
        1.0
    }
}

/// Get the time-of-use multiplier for the given period from config.
pub fn tou_multiplier_for_period(
    config: &EnergyPricingConfig,
    period: TimeOfUsePeriod,
) -> f32 {
    match period {
        TimeOfUsePeriod::OffPeak => config.off_peak_multiplier,
        TimeOfUsePeriod::MidPeak => config.mid_peak_multiplier,
        TimeOfUsePeriod::OnPeak => config.on_peak_multiplier,
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Updates electricity pricing based on time-of-day and grid scarcity.
///
/// Runs every `PRICING_INTERVAL` ticks. Reads the `GameClock` for the current
/// hour and `EnergyGrid` for the reserve margin. Updates `EnergyEconomics`
/// with the current price and accumulates revenue/costs.
pub fn update_energy_pricing(
    tick: Res<TickCounter>,
    clock: Res<GameClock>,
    energy_grid: Res<EnergyGrid>,
    dispatch_state: Res<EnergyDispatchState>,
    config: Res<EnergyPricingConfig>,
    mut economics: ResMut<EnergyEconomics>,
) {
    if !tick.0.is_multiple_of(PRICING_INTERVAL) {
        return;
    }

    // Determine time-of-use period and multiplier.
    let period = TimeOfUsePeriod::from_hour(clock.hour);
    let tou_mult = tou_multiplier_for_period(&config, period);

    // Determine scarcity multiplier from reserve margin.
    let scarcity_mult = scarcity_multiplier_from_reserve(energy_grid.reserve_margin);

    // Calculate effective price per kWh.
    let effective_price = config.base_rate_per_kwh * tou_mult * scarcity_mult;

    // Update current pricing state.
    economics.current_price_per_kwh = effective_price;
    economics.current_period = period;
    economics.tou_multiplier = tou_mult;
    economics.scarcity_multiplier = scarcity_mult;

    // Calculate revenue from demand (demand is in MW, convert to MWh per tick).
    // Each tick represents MINUTES_PER_TICK minutes = 1/60 hour.
    // Over PRICING_INTERVAL ticks, time elapsed = PRICING_INTERVAL / 60 hours.
    let hours_per_interval = PRICING_INTERVAL as f32 / 60.0;
    let consumption_mwh = energy_grid.total_demand_mwh * hours_per_interval;
    let generation_mwh = energy_grid.total_supply_mwh * hours_per_interval;

    // Revenue = consumption * price_per_kWh * 1000 (convert MWh to kWh).
    let revenue = consumption_mwh as f64 * effective_price as f64 * 1000.0;

    // Generation cost = generation * dispatch electricity price ($/MWh)
    // or use the config generation cost if dispatch price is zero.
    let gen_cost_per_mwh = if dispatch_state.electricity_price > 0.0 {
        dispatch_state.electricity_price
    } else {
        config.generation_cost_per_mwh
    };
    let generation_cost = generation_mwh as f64 * gen_cost_per_mwh as f64;

    // Accumulate revenue and costs.
    economics.total_revenue += revenue;
    economics.total_costs += generation_cost;
    economics.net_income = economics.total_revenue - economics.total_costs;
    economics.total_consumption_mwh += consumption_mwh;
    economics.total_generation_mwh += generation_mwh;

    // Update citizen cost burden (normalized price relative to base rate).
    // A burden of 1.0 means citizens pay the base rate; higher means more.
    economics.citizen_cost_burden = effective_price / config.base_rate_per_kwh;
}

/// Monthly billing cycle reset: transfers net energy income to city treasury
/// and resets accumulators.
pub fn energy_billing_cycle(
    clock: Res<GameClock>,
    mut economics: ResMut<EnergyEconomics>,
    mut budget: ResMut<crate::economy::CityBudget>,
) {
    // Bill every 30 days.
    let billing_interval = 30;
    if clock.day <= economics.last_billing_day + billing_interval {
        return;
    }
    economics.last_billing_day = clock.day;

    // Transfer net income to treasury.
    budget.treasury += economics.net_income;

    // Reset accumulators for next billing cycle.
    economics.total_revenue = 0.0;
    economics.total_costs = 0.0;
    economics.net_income = 0.0;
    economics.residential_revenue = 0.0;
    economics.commercial_revenue = 0.0;
    economics.industrial_revenue = 0.0;
    economics.total_consumption_mwh = 0.0;
    economics.total_generation_mwh = 0.0;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnergyPricingPlugin;

impl Plugin for EnergyPricingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyPricingConfig>();
        app.init_resource::<EnergyEconomics>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(SaveableRegistry::default);
        registry.register::<EnergyPricingConfig>();
        registry.register::<EnergyEconomics>();

        app.add_systems(
            FixedUpdate,
            (
                update_energy_pricing
                    .after(crate::energy_dispatch::dispatch_energy),
                energy_billing_cycle
                    .after(update_energy_pricing),
            )
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
    fn test_time_of_use_period_off_peak() {
        assert_eq!(TimeOfUsePeriod::from_hour(0.0), TimeOfUsePeriod::OffPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(3.0), TimeOfUsePeriod::OffPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(5.9), TimeOfUsePeriod::OffPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(22.0), TimeOfUsePeriod::OffPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(23.5), TimeOfUsePeriod::OffPeak);
    }

    #[test]
    fn test_time_of_use_period_mid_peak() {
        assert_eq!(TimeOfUsePeriod::from_hour(6.0), TimeOfUsePeriod::MidPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(10.0), TimeOfUsePeriod::MidPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(13.9), TimeOfUsePeriod::MidPeak);
    }

    #[test]
    fn test_time_of_use_period_on_peak() {
        assert_eq!(TimeOfUsePeriod::from_hour(14.0), TimeOfUsePeriod::OnPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(18.0), TimeOfUsePeriod::OnPeak);
        assert_eq!(TimeOfUsePeriod::from_hour(21.9), TimeOfUsePeriod::OnPeak);
    }

    #[test]
    fn test_scarcity_multiplier_surplus() {
        assert!((scarcity_multiplier_from_reserve(0.25) - 1.0).abs() < f32::EPSILON);
        assert!((scarcity_multiplier_from_reserve(0.50) - 1.0).abs() < f32::EPSILON);
        assert!((scarcity_multiplier_from_reserve(1.00) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scarcity_multiplier_tight() {
        assert!((scarcity_multiplier_from_reserve(0.15) - 1.2).abs() < f32::EPSILON);
        assert!((scarcity_multiplier_from_reserve(0.08) - 1.5).abs() < f32::EPSILON);
        assert!((scarcity_multiplier_from_reserve(0.03) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scarcity_multiplier_deficit() {
        assert!((scarcity_multiplier_from_reserve(-0.1) - 3.0).abs() < f32::EPSILON);
        assert!((scarcity_multiplier_from_reserve(-1.0) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_config_values() {
        let config = EnergyPricingConfig::default();
        assert!((config.base_rate_per_kwh - 0.12).abs() < f32::EPSILON);
        assert!((config.off_peak_multiplier - 0.6).abs() < f32::EPSILON);
        assert!((config.mid_peak_multiplier - 1.0).abs() < f32::EPSILON);
        assert!((config.on_peak_multiplier - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_price_mid_peak_no_scarcity() {
        let config = EnergyPricingConfig::default();
        let period = TimeOfUsePeriod::MidPeak;
        let tou = tou_multiplier_for_period(&config, period);
        let scarcity = scarcity_multiplier_from_reserve(0.30);
        let price = config.base_rate_per_kwh * tou * scarcity;
        // $0.12 * 1.0 * 1.0 = $0.12
        assert!((price - 0.12).abs() < 0.001);
    }

    #[test]
    fn test_effective_price_on_peak_tight_margin() {
        let config = EnergyPricingConfig::default();
        let period = TimeOfUsePeriod::OnPeak;
        let tou = tou_multiplier_for_period(&config, period);
        let scarcity = scarcity_multiplier_from_reserve(0.03);
        let price = config.base_rate_per_kwh * tou * scarcity;
        // $0.12 * 1.5 * 2.0 = $0.36
        assert!((price - 0.36).abs() < 0.001);
    }

    #[test]
    fn test_effective_price_off_peak_surplus() {
        let config = EnergyPricingConfig::default();
        let period = TimeOfUsePeriod::OffPeak;
        let tou = tou_multiplier_for_period(&config, period);
        let scarcity = scarcity_multiplier_from_reserve(0.50);
        let price = config.base_rate_per_kwh * tou * scarcity;
        // $0.12 * 0.6 * 1.0 = $0.072
        assert!((price - 0.072).abs() < 0.001);
    }

    #[test]
    fn test_saveable_roundtrip_config() {
        let config = EnergyPricingConfig {
            base_rate_per_kwh: 0.15,
            off_peak_multiplier: 0.5,
            mid_peak_multiplier: 1.0,
            on_peak_multiplier: 2.0,
            generation_cost_per_mwh: 30.0,
        };
        let bytes = config.save_to_bytes().unwrap();
        let restored = EnergyPricingConfig::load_from_bytes(&bytes);
        assert!((restored.base_rate_per_kwh - 0.15).abs() < f32::EPSILON);
        assert!((restored.on_peak_multiplier - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_roundtrip_economics() {
        let econ = EnergyEconomics {
            current_price_per_kwh: 0.18,
            total_revenue: 5000.0,
            total_costs: 3000.0,
            net_income: 2000.0,
            citizen_cost_burden: 1.5,
            ..Default::default()
        };
        let bytes = econ.save_to_bytes().unwrap();
        let restored = EnergyEconomics::load_from_bytes(&bytes);
        assert!((restored.current_price_per_kwh - 0.18).abs() < f32::EPSILON);
        assert!((restored.total_revenue - 5000.0).abs() < f64::EPSILON);
        assert!((restored.net_income - 2000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_citizen_cost_burden_baseline() {
        let config = EnergyPricingConfig::default();
        // At mid-peak with no scarcity, burden should be 1.0
        let effective = config.base_rate_per_kwh * 1.0 * 1.0;
        let burden = effective / config.base_rate_per_kwh;
        assert!((burden - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_citizen_cost_burden_high_scarcity() {
        let config = EnergyPricingConfig::default();
        // On-peak with deficit: $0.12 * 1.5 * 3.0 = $0.54 -> burden = 4.5
        let effective = config.base_rate_per_kwh * 1.5 * 3.0;
        let burden = effective / config.base_rate_per_kwh;
        assert!((burden - 4.5).abs() < 0.001);
    }
}
