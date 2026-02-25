//! SVC-023: Power Grid Demand/Supply Balance
//!
//! Unified power grid balance dashboard tracking demand by sector, supply by
//! source, reserve margin with alert thresholds, time-of-day demand curves,
//! and power shortage notifications.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::blackout::BlackoutState;
use crate::buildings::Building;
use crate::coal_power::{CoalPowerState, PowerPlant, PowerPlantType};
use crate::energy_demand::{EnergyConsumer, EnergyGrid};
use crate::energy_dispatch::EnergyDispatchState;
use crate::gas_power::GasPowerState;
use crate::grid::ZoneType;
use crate::notifications::{NotificationEvent, NotificationPriority};
use crate::services::ServiceBuilding;
use crate::solar_power::SolarPowerState;
use crate::time_of_day::GameClock;
use crate::wind_power::WindPowerState;
use crate::{decode_or_warn, Saveable, SaveableRegistry, SimulationSet, TickCounter};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const BALANCE_INTERVAL: u64 = 8;
const RESERVE_MARGIN_HEALTHY: f32 = 0.20;
const RESERVE_MARGIN_TIGHT: f32 = 0.10;
const RESERVE_MARGIN_WARNING: f32 = 0.05;
const RESERVE_MARGIN_CRITICAL: f32 = 0.0;
const NOTIFICATION_COOLDOWN_TICKS: u64 = 600;

// ---------------------------------------------------------------------------
// Alert level
// ---------------------------------------------------------------------------

/// Grid stress level derived from reserve margin.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode, Default,
)]
pub enum GridAlertLevel {
    #[default]
    Healthy,
    Tight,
    Warning,
    Critical,
    Deficit,
}

impl GridAlertLevel {
    pub fn from_reserve_margin(margin: f32) -> Self {
        if margin < RESERVE_MARGIN_CRITICAL {
            Self::Deficit
        } else if margin < RESERVE_MARGIN_WARNING {
            Self::Critical
        } else if margin < RESERVE_MARGIN_TIGHT {
            Self::Warning
        } else if margin < RESERVE_MARGIN_HEALTHY {
            Self::Tight
        } else {
            Self::Healthy
        }
    }
}

// ---------------------------------------------------------------------------
// Demand/supply breakdowns
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct SectorDemand {
    pub residential_mw: f32,
    pub commercial_mw: f32,
    pub industrial_mw: f32,
    pub services_mw: f32,
}

impl SectorDemand {
    pub fn total(&self) -> f32 {
        self.residential_mw + self.commercial_mw + self.industrial_mw + self.services_mw
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct SourceSupply {
    pub coal_mw: f32,
    pub gas_mw: f32,
    pub solar_mw: f32,
    pub wind_mw: f32,
    pub wte_mw: f32,
    pub biomass_mw: f32,
    pub battery_mw: f32,
}

impl SourceSupply {
    pub fn total(&self) -> f32 {
        self.coal_mw + self.gas_mw + self.solar_mw + self.wind_mw + self.wte_mw + self.biomass_mw + self.battery_mw
    }
}

// ---------------------------------------------------------------------------
// Per-sector time-of-day demand curves
// ---------------------------------------------------------------------------

/// Residential: peaks evening (17-21h) when people are home.
pub fn residential_demand_curve(hour: f32) -> f32 {
    match hour as u32 {
        0..=5 => 0.5,
        6..=8 => 0.9,
        9..=16 => 0.6,
        17..=21 => 1.5,
        22..=23 => 0.8,
        _ => 0.6,
    }
}

/// Commercial: peaks business hours (9-17h).
pub fn commercial_demand_curve(hour: f32) -> f32 {
    match hour as u32 {
        0..=5 => 0.3,
        6..=8 => 0.7,
        9..=17 => 1.4,
        18..=20 => 0.9,
        21..=23 => 0.4,
        _ => 0.5,
    }
}

/// Industrial: relatively flat (24/7 operations).
pub fn industrial_demand_curve(hour: f32) -> f32 {
    match hour as u32 {
        0..=5 => 0.85,
        6..=21 => 1.0,
        22..=23 => 0.9,
        _ => 0.9,
    }
}

// ---------------------------------------------------------------------------
// PowerGridBalance resource
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PowerGridBalance {
    pub demand: SectorDemand,
    pub supply: SourceSupply,
    pub total_demand_mw: f32,
    pub total_supply_mw: f32,
    pub total_capacity_mw: f32,
    pub reserve_margin: f32,
    pub alert_level: GridAlertLevel,
    pub brownout_active: bool,
    pub affected_cells: u32,
    pub renewable_fraction: f32,
    pub residential_curve: f32,
    pub commercial_curve: f32,
    pub industrial_curve: f32,
    /// Tick of last shortage notification (cooldown).
    #[serde(default)]
    pub last_notification_tick: u64,
}

impl Default for PowerGridBalance {
    fn default() -> Self {
        Self {
            demand: SectorDemand::default(),
            supply: SourceSupply::default(),
            total_demand_mw: 0.0,
            total_supply_mw: 0.0,
            total_capacity_mw: 0.0,
            reserve_margin: 1.0,
            alert_level: GridAlertLevel::Healthy,
            brownout_active: false,
            affected_cells: 0,
            renewable_fraction: 0.0,
            residential_curve: 1.0,
            commercial_curve: 1.0,
            industrial_curve: 1.0,
            last_notification_tick: 0,
        }
    }
}

impl Saveable for PowerGridBalance {
    const SAVE_KEY: &'static str = "power_grid_balance";

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

/// Aggregates demand by sector from EnergyConsumer + Building/ServiceBuilding.
#[allow(clippy::too_many_arguments)]
pub fn aggregate_sector_demand(
    tick: Res<TickCounter>,
    clock: Res<GameClock>,
    energy_grid: Res<EnergyGrid>,
    zoned_consumers: Query<(&EnergyConsumer, &Building), Without<ServiceBuilding>>,
    service_consumers: Query<(&EnergyConsumer, &ServiceBuilding)>,
    mut balance: ResMut<PowerGridBalance>,
) {
    if !tick.0.is_multiple_of(BALANCE_INTERVAL) {
        return;
    }

    let hour = clock.hour;
    let res_curve = residential_demand_curve(hour);
    let com_curve = commercial_demand_curve(hour);
    let ind_curve = industrial_demand_curve(hour);

    balance.residential_curve = res_curve;
    balance.commercial_curve = com_curve;
    balance.industrial_curve = ind_curve;

    let mut residential = 0.0_f32;
    let mut commercial = 0.0_f32;
    let mut industrial = 0.0_f32;

    for (consumer, building) in &zoned_consumers {
        let base_mw = consumer.base_demand_kwh / 720_000.0;
        match building.zone_type {
            ZoneType::ResidentialLow
            | ZoneType::ResidentialMedium
            | ZoneType::ResidentialHigh => residential += base_mw * res_curve,
            ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::MixedUse => {
                commercial += base_mw * com_curve;
            }
            ZoneType::Industrial | ZoneType::Office => industrial += base_mw * ind_curve,
            ZoneType::None => {}
        }
    }

    let mut services = 0.0_f32;
    for (consumer, _) in &service_consumers {
        services += consumer.base_demand_kwh / 720_000.0;
    }

    balance.demand = SectorDemand {
        residential_mw: residential,
        commercial_mw: commercial,
        industrial_mw: industrial,
        services_mw: services,
    };
    balance.total_demand_mw = energy_grid.total_demand_mwh;
}

/// Aggregates supply by source from individual power state resources.
#[allow(clippy::too_many_arguments)]
pub fn aggregate_source_supply(
    tick: Res<TickCounter>,
    coal_state: Res<CoalPowerState>,
    gas_state: Res<GasPowerState>,
    solar_state: Res<SolarPowerState>,
    wind_state: Res<WindPowerState>,
    dispatch_state: Res<EnergyDispatchState>,
    energy_grid: Res<EnergyGrid>,
    plants: Query<&PowerPlant>,
    mut balance: ResMut<PowerGridBalance>,
) {
    if !tick.0.is_multiple_of(BALANCE_INTERVAL) {
        return;
    }

    let wte_output: f32 = plants
        .iter()
        .filter(|p| p.plant_type == PowerPlantType::WasteToEnergy)
        .map(|p| p.current_output_mw)
        .sum();

    let biomass_output: f32 = plants
        .iter()
        .filter(|p| p.plant_type == PowerPlantType::Biomass)
        .map(|p| p.current_output_mw)
        .sum();

    let supply = SourceSupply {
        coal_mw: coal_state.total_output_mw,
        gas_mw: gas_state.total_output_mw,
        solar_mw: solar_state.total_output_mw,
        wind_mw: wind_state.total_output_mw,
        wte_mw: wte_output,
        biomass_mw: biomass_output,
        battery_mw: 0.0,
    };

    let total_supply = energy_grid.total_supply_mwh;
    let renewable = supply.solar_mw + supply.wind_mw;
    let renewable_fraction = if total_supply > 0.0 {
        (renewable / total_supply).clamp(0.0, 1.0)
    } else {
        0.0
    };

    balance.supply = supply;
    balance.total_supply_mw = total_supply;
    balance.total_capacity_mw = dispatch_state.total_capacity_mw;
    balance.renewable_fraction = renewable_fraction;
}

/// Computes reserve margin, alert level, and brownout status.
pub fn compute_grid_balance(
    tick: Res<TickCounter>,
    blackout_state: Res<BlackoutState>,
    mut balance: ResMut<PowerGridBalance>,
) {
    if !tick.0.is_multiple_of(BALANCE_INTERVAL) {
        return;
    }

    let demand = balance.total_demand_mw;
    let capacity = balance.total_capacity_mw;

    let margin = if capacity > 0.0 {
        (capacity - demand) / capacity
    } else if demand > 0.0 {
        -1.0
    } else {
        1.0
    };

    balance.reserve_margin = margin;
    balance.alert_level = GridAlertLevel::from_reserve_margin(margin);
    balance.brownout_active = blackout_state.active;
    balance.affected_cells = blackout_state.affected_cell_count;
}

/// Sends power shortage notifications when alert level is Critical or Deficit.
pub fn notify_power_shortage(
    tick: Res<TickCounter>,
    mut balance: ResMut<PowerGridBalance>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    if !tick.0.is_multiple_of(BALANCE_INTERVAL) {
        return;
    }

    let should_notify = matches!(
        balance.alert_level,
        GridAlertLevel::Critical | GridAlertLevel::Deficit
    );
    if !should_notify {
        return;
    }
    if tick.0.saturating_sub(balance.last_notification_tick) < NOTIFICATION_COOLDOWN_TICKS {
        return;
    }

    let (priority, message) = match balance.alert_level {
        GridAlertLevel::Deficit => (
            NotificationPriority::Emergency,
            format!(
                "Power grid deficit! Demand {:.0} MW exceeds supply {:.0} MW. {} cells blacked out.",
                balance.total_demand_mw, balance.total_supply_mw, balance.affected_cells
            ),
        ),
        GridAlertLevel::Critical => (
            NotificationPriority::Warning,
            format!(
                "Power grid critically low! Reserve margin {:.1}%. Build more power plants.",
                balance.reserve_margin * 100.0
            ),
        ),
        _ => return,
    };

    notifications.send(NotificationEvent {
        text: message,
        priority,
        location: None,
    });
    balance.last_notification_tick = tick.0;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PowerGridBalancePlugin;

impl Plugin for PowerGridBalancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerGridBalance>();

        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(SaveableRegistry::default);
        registry.register::<PowerGridBalance>();

        app.add_systems(
            FixedUpdate,
            (
                aggregate_sector_demand,
                aggregate_source_supply,
                compute_grid_balance
                    .after(aggregate_sector_demand)
                    .after(aggregate_source_supply),
                notify_power_shortage.after(compute_grid_balance),
            )
                .after(crate::energy_dispatch::dispatch_energy)
                .after(crate::blackout::evaluate_blackout)
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
    fn test_alert_levels() {
        assert_eq!(GridAlertLevel::from_reserve_margin(0.25), GridAlertLevel::Healthy);
        assert_eq!(GridAlertLevel::from_reserve_margin(0.15), GridAlertLevel::Tight);
        assert_eq!(GridAlertLevel::from_reserve_margin(0.07), GridAlertLevel::Warning);
        assert_eq!(GridAlertLevel::from_reserve_margin(0.02), GridAlertLevel::Critical);
        assert_eq!(GridAlertLevel::from_reserve_margin(-0.1), GridAlertLevel::Deficit);
    }

    #[test]
    fn test_residential_peaks_evening() {
        assert!(residential_demand_curve(19.0) > residential_demand_curve(12.0));
    }

    #[test]
    fn test_commercial_peaks_daytime() {
        assert!(commercial_demand_curve(12.0) > commercial_demand_curve(2.0));
    }

    #[test]
    fn test_industrial_flat() {
        let ratio = industrial_demand_curve(12.0) / industrial_demand_curve(2.0);
        assert!(ratio < 1.25);
    }

    #[test]
    fn test_sector_demand_total() {
        let d = SectorDemand { residential_mw: 100.0, commercial_mw: 50.0, industrial_mw: 200.0, services_mw: 30.0 };
        assert!((d.total() - 380.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_source_supply_total() {
        let s = SourceSupply { coal_mw: 200.0, gas_mw: 100.0, solar_mw: 14.0, wind_mw: 25.0, wte_mw: 10.0, biomass_mw: 20.0, battery_mw: 5.0 };
        assert!((s.total() - 374.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let b = PowerGridBalance {
            total_demand_mw: 300.0, total_supply_mw: 350.0, total_capacity_mw: 500.0,
            reserve_margin: 0.40, alert_level: GridAlertLevel::Healthy,
            renewable_fraction: 0.15, ..Default::default()
        };
        let bytes = b.save_to_bytes().unwrap();
        let r = PowerGridBalance::load_from_bytes(&bytes);
        assert!((r.total_demand_mw - 300.0).abs() < f32::EPSILON);
        assert!((r.reserve_margin - 0.40).abs() < f32::EPSILON);
        assert_eq!(r.alert_level, GridAlertLevel::Healthy);
    }

    #[test]
    fn test_default_healthy() {
        let b = PowerGridBalance::default();
        assert_eq!(b.alert_level, GridAlertLevel::Healthy);
        assert!(!b.brownout_active);
    }
}
