use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::ZoneType;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub zone_type: ZoneType,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
}

/// Component for mixed-use buildings that have both commercial ground floors
/// and residential upper floors. Attached alongside [`Building`] when the
/// zone is `ZoneType::MixedUse`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MixedUseBuilding {
    pub commercial_capacity: u32,
    pub commercial_occupants: u32,
    pub residential_capacity: u32,
    pub residential_occupants: u32,
}

impl MixedUseBuilding {
    /// Returns (commercial_capacity, residential_capacity) for a given building level.
    /// L1=(5,8), L2=(15,30), L3=(20+20 office=40, 80), L4=(40+80=120, 200), L5=(80+200=280, 400)
    pub fn capacities_for_level(level: u8) -> (u32, u32) {
        match level {
            1 => (5, 8),
            2 => (15, 30),
            3 => (40, 80),
            4 => (120, 200),
            5 => (280, 400),
            _ => (0, 0),
        }
    }

    /// Total capacity (commercial + residential) for a given level.
    pub fn total_capacity_for_level(level: u8) -> u32 {
        let (c, r) = Self::capacities_for_level(level);
        c + r
    }
}

/// Marker component for buildings that are still under construction.
/// While present, the building cannot accept occupants.
/// Approximately 10 seconds at 10Hz fixed timestep (100 ticks).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct UnderConstruction {
    pub ticks_remaining: u32,
    pub total_ticks: u32,
}

impl Building {
    pub fn capacity_for_level(zone: ZoneType, level: u8) -> u32 {
        match (zone, level) {
            // Low-density residential: houses and small apartments
            (ZoneType::ResidentialLow, 1) => 10,
            (ZoneType::ResidentialLow, 2) => 30,
            (ZoneType::ResidentialLow, 3) => 80,
            // Medium-density residential: townhouses, duplexes, small apartments
            (ZoneType::ResidentialMedium, 1) => 15,
            (ZoneType::ResidentialMedium, 2) => 50,
            (ZoneType::ResidentialMedium, 3) => 120,
            (ZoneType::ResidentialMedium, 4) => 250,
            // High-density residential: apartment blocks and towers
            (ZoneType::ResidentialHigh, 1) => 50,
            (ZoneType::ResidentialHigh, 2) => 200,
            (ZoneType::ResidentialHigh, 3) => 500,
            (ZoneType::ResidentialHigh, 4) => 1000,
            (ZoneType::ResidentialHigh, 5) => 2000,
            // Low-density commercial: shops and small stores
            (ZoneType::CommercialLow, 1) => 8,
            (ZoneType::CommercialLow, 2) => 25,
            (ZoneType::CommercialLow, 3) => 60,
            // High-density commercial: malls and department stores
            (ZoneType::CommercialHigh, 1) => 30,
            (ZoneType::CommercialHigh, 2) => 100,
            (ZoneType::CommercialHigh, 3) => 300,
            (ZoneType::CommercialHigh, 4) => 600,
            (ZoneType::CommercialHigh, 5) => 1200,
            // Industrial: factories and warehouses
            (ZoneType::Industrial, 1) => 20,
            (ZoneType::Industrial, 2) => 60,
            (ZoneType::Industrial, 3) => 150,
            (ZoneType::Industrial, 4) => 300,
            (ZoneType::Industrial, 5) => 600,
            // Office: office towers
            (ZoneType::Office, 1) => 30,
            (ZoneType::Office, 2) => 100,
            (ZoneType::Office, 3) => 300,
            (ZoneType::Office, 4) => 700,
            (ZoneType::Office, 5) => 1500,
            // Mixed-use: total capacity (commercial + residential)
            (ZoneType::MixedUse, l) => MixedUseBuilding::total_capacity_for_level(l),
            _ => 0,
        }
    }
}

/// Returns the maximum building level allowed by the Floor Area Ratio (FAR)
/// constraint for the given zone type.
///
/// For each candidate level 1..=5, the implied FAR is computed as:
///   implied_far = (capacity_for_level(level) * 20.0) / 256.0
///
/// The highest level where implied_far <= zone.default_far() is returned.
/// Always returns at least 1 (minimum building level).
pub fn max_level_for_far(zone: ZoneType) -> u32 {
    let far_limit = zone.default_far();
    let mut best = 1u32;
    for level in 1..=5u8 {
        let capacity = Building::capacity_for_level(zone, level);
        if capacity == 0 {
            break;
        }
        let implied_far = (capacity as f32 * 20.0) / 256.0;
        if implied_far <= far_limit {
            best = level as u32;
        }
    }
    best
}
