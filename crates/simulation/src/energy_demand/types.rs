//! Types for the energy demand system: components, enums, and the EnergyGrid resource.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::Saveable;

// =============================================================================
// LoadPriority
// =============================================================================

/// Priority level for load shedding during supply shortages.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode, Default,
)]
pub enum LoadPriority {
    /// Critical infrastructure (hospitals, data centers) — shed last.
    Critical,
    /// High priority (residential, commercial) — shed after non-essential.
    High,
    /// Normal priority (industrial, offices) — shed before high.
    #[default]
    Normal,
    /// Low priority (parks, plazas) — shed first.
    Low,
}

// =============================================================================
// EnergyConsumer component
// =============================================================================

/// Component attached to buildings that consume electricity.
///
/// `base_demand_kwh` is the monthly energy consumption in kWh. The actual
/// instantaneous demand is derived by dividing by hours-per-month and
/// applying time-of-use and weather modifiers.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConsumer {
    /// Monthly base demand in kWh.
    pub base_demand_kwh: f32,
    /// Load shedding priority.
    pub priority: LoadPriority,
}

impl EnergyConsumer {
    /// Create an `EnergyConsumer` with the given base demand and priority.
    pub fn new(base_demand_kwh: f32, priority: LoadPriority) -> Self {
        Self {
            base_demand_kwh,
            priority,
        }
    }

    /// Base monthly demand (kWh) for a zoned building type.
    pub fn base_demand_for_zone(zone: ZoneType) -> f32 {
        match zone {
            ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
                1_000.0
            }
            ZoneType::CommercialLow => 3_000.0,
            ZoneType::CommercialHigh => 15_000.0,
            ZoneType::Industrial => 50_000.0,
            ZoneType::Office => 10_000.0,
            ZoneType::MixedUse => 5_000.0,
            ZoneType::None => 0.0,
        }
    }

    /// Load priority for a zoned building type.
    pub fn priority_for_zone(zone: ZoneType) -> LoadPriority {
        match zone {
            ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
                LoadPriority::High
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::MixedUse => {
                LoadPriority::High
            }
            ZoneType::Industrial | ZoneType::Office => LoadPriority::Normal,
            ZoneType::None => LoadPriority::Low,
        }
    }

    /// Base monthly demand (kWh) for a service building type.
    pub fn base_demand_for_service(service_type: ServiceType) -> f32 {
        match service_type {
            ServiceType::Hospital | ServiceType::MedicalCenter => 200_000.0,
            ServiceType::DataCenter => 500_000.0,
            ServiceType::MedicalClinic => 30_000.0,
            ServiceType::University => 80_000.0,
            ServiceType::HighSchool => 40_000.0,
            ServiceType::ElementarySchool | ServiceType::Kindergarten => 20_000.0,
            ServiceType::Library => 15_000.0,
            ServiceType::Stadium => 100_000.0,
            ServiceType::InternationalAirport => 300_000.0,
            ServiceType::RegionalAirport => 150_000.0,
            ServiceType::SmallAirstrip => 50_000.0,
            ServiceType::SubwayStation => 60_000.0,
            ServiceType::TrainStation => 40_000.0,
            ServiceType::TramDepot => 30_000.0,
            ServiceType::WaterTreatmentPlant => 80_000.0,
            ServiceType::Incinerator => 60_000.0,
            ServiceType::RecyclingCenter => 25_000.0,
            ServiceType::DistrictHeatingPlant => 100_000.0,
            ServiceType::GeothermalPlant => 120_000.0,
            ServiceType::CityHall => 30_000.0,
            ServiceType::Museum | ServiceType::Cathedral => 25_000.0,
            ServiceType::TVStation => 50_000.0,
            ServiceType::Prison => 80_000.0,
            ServiceType::CellTower => 10_000.0,
            _ => 5_000.0,
        }
    }

    /// Load priority for a service building type.
    pub fn priority_for_service(service_type: ServiceType) -> LoadPriority {
        match service_type {
            ServiceType::Hospital | ServiceType::MedicalCenter | ServiceType::MedicalClinic => {
                LoadPriority::Critical
            }
            ServiceType::DataCenter => LoadPriority::Critical,
            ServiceType::FireStation
            | ServiceType::FireHouse
            | ServiceType::FireHQ
            | ServiceType::PoliceStation
            | ServiceType::PoliceKiosk
            | ServiceType::PoliceHQ => LoadPriority::Critical,
            ServiceType::WaterTreatmentPlant | ServiceType::WellPump => LoadPriority::Critical,
            ServiceType::SmallPark
            | ServiceType::LargePark
            | ServiceType::Playground
            | ServiceType::Plaza
            | ServiceType::SportsField => LoadPriority::Low,
            _ => LoadPriority::Normal,
        }
    }
}

// =============================================================================
// EnergyGrid resource
// =============================================================================

/// City-wide energy grid state aggregated from all consumers.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct EnergyGrid {
    /// Total instantaneous demand across all consumers (MW).
    pub total_demand_mwh: f32,
    /// Total instantaneous supply from power sources (MW).
    pub total_supply_mwh: f32,
    /// Reserve margin: (supply - demand) / supply. Negative means deficit.
    pub reserve_margin: f32,
    /// Number of consumers contributing to demand.
    pub consumer_count: u32,
}

impl Default for EnergyGrid {
    fn default() -> Self {
        Self {
            total_demand_mwh: 0.0,
            total_supply_mwh: 0.0,
            reserve_margin: 1.0,
            consumer_count: 0,
        }
    }
}

impl Saveable for EnergyGrid {
    const SAVE_KEY: &'static str = "energy_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
