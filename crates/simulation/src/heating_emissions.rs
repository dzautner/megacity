//! POLL-031: Residential and Commercial Heating as Air Pollution Source
//!
//! Adds area-source air pollution from residential and commercial buildings
//! when heating is active (temperature below 10°C). Emission rates scale
//! with zone density and heating demand intensity.
//!
//! | Zone Type              | Base Q | Notes                          |
//! |------------------------|--------|--------------------------------|
//! | ResidentialHigh         | 5.0    | High-density, many furnaces    |
//! | ResidentialMedium       | 3.5    | Medium-density                 |
//! | ResidentialLow          | 2.0    | Low-density, fewer units       |
//! | CommercialHigh          | 3.0    | Large HVAC systems             |
//! | CommercialLow           | 2.0    | Smaller commercial units       |
//!
//! Heating fuel modifier (city-wide default):
//! - Gas: 1.0x
//! - Oil: 1.5x
//! - Wood: 2.0x
//! - Electric: 0.0x
//!
//! Since per-building fuel type tracking doesn't exist yet, a city-wide
//! default fuel mix resource is provided (`HeatingFuelMix`).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::heating::heating_demand;
use crate::pollution::PollutionGrid;
use crate::weather::Weather;
use crate::SlowTickTimer;

// =============================================================================
// Heating fuel types and city-wide fuel mix
// =============================================================================

/// Fuel types for building heating systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeatingFuelType {
    Gas,
    Oil,
    Wood,
    Electric,
}

impl HeatingFuelType {
    /// Pollution emission multiplier for this fuel type.
    pub fn emission_multiplier(self) -> f32 {
        match self {
            HeatingFuelType::Gas => 1.0,
            HeatingFuelType::Oil => 1.5,
            HeatingFuelType::Wood => 2.0,
            HeatingFuelType::Electric => 0.0,
        }
    }
}

/// City-wide heating fuel mix. Represents the fraction of buildings using
/// each fuel type. Fractions should sum to 1.0.
///
/// The weighted emission multiplier is computed as:
///   `sum(fraction_i * multiplier_i)` for each fuel type.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct HeatingFuelMix {
    pub gas_fraction: f32,
    pub oil_fraction: f32,
    pub wood_fraction: f32,
    pub electric_fraction: f32,
}

impl Default for HeatingFuelMix {
    fn default() -> Self {
        // Default city: mostly gas with some electric
        Self {
            gas_fraction: 0.7,
            oil_fraction: 0.1,
            wood_fraction: 0.05,
            electric_fraction: 0.15,
        }
    }
}

impl HeatingFuelMix {
    /// Compute the weighted emission multiplier from the fuel mix.
    pub fn emission_multiplier(&self) -> f32 {
        self.gas_fraction * HeatingFuelType::Gas.emission_multiplier()
            + self.oil_fraction * HeatingFuelType::Oil.emission_multiplier()
            + self.wood_fraction * HeatingFuelType::Wood.emission_multiplier()
            + self.electric_fraction * HeatingFuelType::Electric.emission_multiplier()
    }
}

// =============================================================================
// Emission rates per zone type
// =============================================================================

/// Base emission Q value for heating pollution by zone type.
/// Returns `None` for zone types that don't emit heating pollution.
pub fn heating_emission_q(zone: ZoneType) -> Option<f32> {
    match zone {
        ZoneType::ResidentialHigh => Some(5.0),
        ZoneType::ResidentialMedium => Some(3.5),
        ZoneType::ResidentialLow => Some(2.0),
        ZoneType::CommercialHigh => Some(3.0),
        ZoneType::CommercialLow => Some(2.0),
        _ => None,
    }
}

// =============================================================================
// Heating emissions statistics
// =============================================================================

/// Aggregate statistics for heating-related air pollution.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeatingEmissionsStats {
    /// Number of buildings actively emitting heating pollution.
    pub emitting_buildings: u32,
    /// Total emission Q across all heating sources this tick.
    pub total_emission_q: f32,
    /// Current heating demand factor (0.0 = warm, >0 = cold).
    pub current_demand: f32,
    /// Current fuel mix emission multiplier.
    pub fuel_multiplier: f32,
}

// =============================================================================
// System: apply heating emissions to pollution grid
// =============================================================================

/// Adds heating-based pollution to the `PollutionGrid` for residential and
/// commercial buildings when heating demand is active.
///
/// Runs on slow tick, after the main wind pollution system so we add on top
/// of existing pollution values.
pub fn apply_heating_emissions(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    fuel_mix: Res<HeatingFuelMix>,
    buildings: Query<&Building>,
    mut pollution: ResMut<PollutionGrid>,
    mut stats: ResMut<HeatingEmissionsStats>,
) {
    if !timer.should_run() {
        return;
    }

    let demand = heating_demand(&weather);

    // Reset stats
    stats.current_demand = demand;
    stats.fuel_multiplier = fuel_mix.emission_multiplier();

    // No emissions when heating is not needed
    if demand <= 0.0 {
        stats.emitting_buildings = 0;
        stats.total_emission_q = 0.0;
        return;
    }

    let fuel_mult = fuel_mix.emission_multiplier();

    // No emissions if fuel mix is all-electric
    if fuel_mult <= 0.0 {
        stats.emitting_buildings = 0;
        stats.total_emission_q = 0.0;
        return;
    }

    let mut emitting = 0u32;
    let mut total_q = 0.0f32;

    for building in &buildings {
        let Some(base_q) = heating_emission_q(building.zone_type) else {
            continue;
        };

        // Effective emission = base_q * demand * fuel_multiplier
        let effective_q = base_q * demand * fuel_mult;

        if effective_q <= 0.0 {
            continue;
        }

        let x = building.grid_x;
        let y = building.grid_y;

        if x < GRID_WIDTH && y < GRID_HEIGHT {
            // Add heating pollution on top of existing pollution
            let current = pollution.get(x, y);
            let additional = effective_q.round() as u8;
            pollution.set(x, y, current.saturating_add(additional));

            emitting += 1;
            total_q += effective_q;
        }
    }

    stats.emitting_buildings = emitting;
    stats.total_emission_q = total_q;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct HeatingEmissionsPlugin;

impl Plugin for HeatingEmissionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeatingFuelMix>()
            .init_resource::<HeatingEmissionsStats>()
            .add_systems(
                FixedUpdate,
                apply_heating_emissions
                    .after(crate::wind_pollution::update_pollution_gaussian_plume)
                    .in_set(crate::SimulationSet::Simulation),
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
    fn test_heating_emission_q_residential_high() {
        assert_eq!(heating_emission_q(ZoneType::ResidentialHigh), Some(5.0));
    }

    #[test]
    fn test_heating_emission_q_residential_low() {
        assert_eq!(heating_emission_q(ZoneType::ResidentialLow), Some(2.0));
    }

    #[test]
    fn test_heating_emission_q_commercial_high() {
        assert_eq!(heating_emission_q(ZoneType::CommercialHigh), Some(3.0));
    }

    #[test]
    fn test_heating_emission_q_commercial_low() {
        assert_eq!(heating_emission_q(ZoneType::CommercialLow), Some(2.0));
    }

    #[test]
    fn test_heating_emission_q_industrial_none() {
        assert_eq!(heating_emission_q(ZoneType::Industrial), None);
    }

    #[test]
    fn test_heating_emission_q_office_none() {
        assert_eq!(heating_emission_q(ZoneType::Office), None);
    }

    #[test]
    fn test_fuel_type_multipliers() {
        assert_eq!(HeatingFuelType::Gas.emission_multiplier(), 1.0);
        assert_eq!(HeatingFuelType::Oil.emission_multiplier(), 1.5);
        assert_eq!(HeatingFuelType::Wood.emission_multiplier(), 2.0);
        assert_eq!(HeatingFuelType::Electric.emission_multiplier(), 0.0);
    }

    #[test]
    fn test_fuel_mix_all_gas() {
        let mix = HeatingFuelMix {
            gas_fraction: 1.0,
            oil_fraction: 0.0,
            wood_fraction: 0.0,
            electric_fraction: 0.0,
        };
        assert!((mix.emission_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fuel_mix_all_electric() {
        let mix = HeatingFuelMix {
            gas_fraction: 0.0,
            oil_fraction: 0.0,
            wood_fraction: 0.0,
            electric_fraction: 1.0,
        };
        assert!((mix.emission_multiplier() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fuel_mix_default() {
        let mix = HeatingFuelMix::default();
        let mult = mix.emission_multiplier();
        // 0.7*1.0 + 0.1*1.5 + 0.05*2.0 + 0.15*0.0 = 0.7 + 0.15 + 0.1 = 0.95
        assert!(
            (mult - 0.95).abs() < 0.01,
            "Default mix multiplier should be ~0.95, got {mult}"
        );
    }

    #[test]
    fn test_fuel_mix_all_wood_is_highest() {
        let wood = HeatingFuelMix {
            gas_fraction: 0.0,
            oil_fraction: 0.0,
            wood_fraction: 1.0,
            electric_fraction: 0.0,
        };
        let gas = HeatingFuelMix {
            gas_fraction: 1.0,
            oil_fraction: 0.0,
            wood_fraction: 0.0,
            electric_fraction: 0.0,
        };
        assert!(
            wood.emission_multiplier() > gas.emission_multiplier(),
            "Wood should produce more emissions than gas"
        );
    }

    #[test]
    fn test_residential_high_emits_more_than_low() {
        let q_high = heating_emission_q(ZoneType::ResidentialHigh).unwrap();
        let q_low = heating_emission_q(ZoneType::ResidentialLow).unwrap();
        assert!(
            q_high > q_low,
            "High density ({q_high}) should emit more than low ({q_low})"
        );
    }

    #[test]
    fn test_stats_default() {
        let stats = HeatingEmissionsStats::default();
        assert_eq!(stats.emitting_buildings, 0);
        assert_eq!(stats.total_emission_q, 0.0);
        assert_eq!(stats.current_demand, 0.0);
    }
}
