//! Core types, constants, and helper functions for FAR bonuses and TDR.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::districts::{DISTRICTS_X, DISTRICTS_Y};
use crate::grid::ZoneType;
use crate::services::ServiceType;

// =============================================================================
// Constants
// =============================================================================

/// FAR bonus multiplier for including affordable housing.
pub const AFFORDABLE_HOUSING_BONUS: f32 = 0.20;

/// FAR bonus multiplier for providing a public plaza.
pub const PUBLIC_PLAZA_BONUS: f32 = 0.10;

/// FAR bonus multiplier for transit contribution.
pub const TRANSIT_CONTRIBUTION_BONUS: f32 = 0.15;

/// Maximum total FAR bonus from all sources (caps stacking).
pub const MAX_BONUS_MULTIPLIER: f32 = 0.45;

/// Default unused FAR generated per historic district cell.
/// Historic districts typically have low-rise buildings, so much of
/// the zoned FAR capacity goes unused.
pub const HISTORIC_UNUSED_FAR_PER_CELL: f32 = 1.0;

/// Default unused FAR generated per park cell.
/// Parks use zero floor area on their lot, so all zoned FAR is available.
pub const PARK_UNUSED_FAR_PER_CELL: f32 = 0.5;

/// Maximum transfer radius in districts (same or adjacent).
/// A value of 1 means the receiving cell's district and all neighboring
/// districts (8-connected) are eligible sources.
pub const TRANSFER_DISTRICT_RADIUS: usize = 1;

/// Maximum FAR that can be transferred to a single cell.
pub const MAX_TRANSFER_FAR_PER_CELL: f32 = 2.0;

// =============================================================================
// FAR Bonus Type
// =============================================================================

/// Types of FAR bonuses that can be applied to a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum FarBonusType {
    /// Developer includes affordable housing units.
    AffordableHousing,
    /// Developer provides a publicly accessible plaza.
    PublicPlaza,
    /// Developer contributes to transit infrastructure.
    TransitContribution,
}

impl FarBonusType {
    /// Returns the FAR multiplier bonus for this type.
    pub fn multiplier(self) -> f32 {
        match self {
            FarBonusType::AffordableHousing => AFFORDABLE_HOUSING_BONUS,
            FarBonusType::PublicPlaza => PUBLIC_PLAZA_BONUS,
            FarBonusType::TransitContribution => TRANSIT_CONTRIBUTION_BONUS,
        }
    }
}

// =============================================================================
// FAR Transfer State Resource
// =============================================================================

/// Resource tracking FAR bonuses and TDR state across the city.
#[derive(Resource, Debug, Clone, Encode, Decode)]
pub struct FarTransferState {
    /// Per-cell FAR bonus (from developer incentives), indexed `y * GRID_WIDTH + x`.
    pub bonus_far: Vec<f32>,
    /// Per-cell transferred FAR (from TDR), indexed `y * GRID_WIDTH + x`.
    pub transferred_far: Vec<f32>,
    /// Per-cell active bonus types (bitflags-style: bit 0 = affordable, 1 = plaza, 2 = transit).
    pub bonus_flags: Vec<u8>,
    /// Total unused FAR available in TDR source parcels (per district index).
    pub district_available_far: Vec<f32>,
    /// Total FAR already transferred out of each district.
    pub district_transferred_far: Vec<f32>,
    /// City-wide total bonus FAR granted.
    pub total_bonus_far: f32,
    /// City-wide total transferred FAR.
    pub total_transferred_far: f32,
}

impl Default for FarTransferState {
    fn default() -> Self {
        let num_districts = DISTRICTS_X * DISTRICTS_Y;
        Self {
            bonus_far: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            transferred_far: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            bonus_flags: vec![0; GRID_WIDTH * GRID_HEIGHT],
            district_available_far: vec![0.0; num_districts],
            district_transferred_far: vec![0.0; num_districts],
            total_bonus_far: 0.0,
            total_transferred_far: 0.0,
        }
    }
}

impl FarTransferState {
    /// Get the FAR bonus at a given cell.
    #[inline]
    pub fn bonus_at(&self, x: usize, y: usize) -> f32 {
        self.bonus_far[y * GRID_WIDTH + x]
    }

    /// Get the transferred FAR at a given cell.
    #[inline]
    pub fn transferred_at(&self, x: usize, y: usize) -> f32 {
        self.transferred_far[y * GRID_WIDTH + x]
    }

    /// Get the total effective FAR adjustment (bonus + transferred) at a cell.
    #[inline]
    pub fn effective_far_adjustment(&self, x: usize, y: usize) -> f32 {
        self.bonus_at(x, y) + self.transferred_at(x, y)
    }

    /// Check if a specific bonus type is active at a cell.
    pub fn has_bonus(&self, x: usize, y: usize, bonus_type: FarBonusType) -> bool {
        let idx = y * GRID_WIDTH + x;
        let bit = bonus_type_to_bit(bonus_type);
        self.bonus_flags[idx] & bit != 0
    }

    /// Get the available (untransferred) FAR for a district.
    pub fn available_far_for_district(&self, district_idx: usize) -> f32 {
        if district_idx >= self.district_available_far.len() {
            return 0.0;
        }
        let available = self.district_available_far[district_idx];
        let transferred = self.district_transferred_far[district_idx];
        (available - transferred).max(0.0)
    }
}

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Convert a FarBonusType to a bitmask bit.
pub fn bonus_type_to_bit(bonus_type: FarBonusType) -> u8 {
    match bonus_type {
        FarBonusType::AffordableHousing => 1,
        FarBonusType::PublicPlaza => 2,
        FarBonusType::TransitContribution => 4,
    }
}

/// Calculate the total FAR bonus multiplier from active bonus flags.
/// The result is capped at `MAX_BONUS_MULTIPLIER`.
pub fn calculate_bonus_multiplier(flags: u8) -> f32 {
    let mut total = 0.0;
    if flags & bonus_type_to_bit(FarBonusType::AffordableHousing) != 0 {
        total += AFFORDABLE_HOUSING_BONUS;
    }
    if flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0 {
        total += PUBLIC_PLAZA_BONUS;
    }
    if flags & bonus_type_to_bit(FarBonusType::TransitContribution) != 0 {
        total += TRANSIT_CONTRIBUTION_BONUS;
    }
    total.min(MAX_BONUS_MULTIPLIER)
}

/// Calculate the FAR bonus for a cell given its base FAR and bonus flags.
pub fn calculate_far_bonus(base_far: f32, flags: u8) -> f32 {
    base_far * calculate_bonus_multiplier(flags)
}

/// Determine which bonus types a building qualifies for based on its
/// zone type and level. Higher-level buildings are more likely to include
/// public benefits.
///
/// Rules:
/// - Level 3+ residential/mixed-use: affordable housing bonus
/// - Level 2+ commercial/mixed-use/office: public plaza bonus
/// - Level 4+ any buildable zone: transit contribution bonus
pub fn eligible_bonuses(zone_type: ZoneType, level: u8) -> u8 {
    let mut flags: u8 = 0;

    // Affordable housing: level 3+ residential or mixed-use
    if level >= 3 && (zone_type.is_residential() || zone_type.is_mixed_use()) {
        flags |= bonus_type_to_bit(FarBonusType::AffordableHousing);
    }

    // Public plaza: level 2+ commercial, mixed-use, or office
    if level >= 2
        && (zone_type.is_commercial() || zone_type.is_mixed_use() || zone_type == ZoneType::Office)
    {
        flags |= bonus_type_to_bit(FarBonusType::PublicPlaza);
    }

    // Transit contribution: level 4+ any buildable zone
    if level >= 4 && zone_type != ZoneType::None {
        flags |= bonus_type_to_bit(FarBonusType::TransitContribution);
    }

    flags
}

/// Check whether two districts are within transfer radius.
/// Returns true if the districts are the same or adjacent (8-connected)
/// within `TRANSFER_DISTRICT_RADIUS`.
pub fn districts_within_transfer_radius(
    src_dx: usize,
    src_dy: usize,
    dst_dx: usize,
    dst_dy: usize,
) -> bool {
    let diff_x = (src_dx as isize - dst_dx as isize).unsigned_abs();
    let diff_y = (src_dy as isize - dst_dy as isize).unsigned_abs();
    diff_x <= TRANSFER_DISTRICT_RADIUS && diff_y <= TRANSFER_DISTRICT_RADIUS
}

/// Calculate the effective FAR limit for a cell, including bonuses and TDR.
pub fn effective_far(base_far: f32, bonus: f32, transferred: f32) -> f32 {
    base_far + bonus + transferred
}

/// Check whether a service type qualifies as a TDR source (park).
pub fn is_park_service(service_type: ServiceType) -> bool {
    matches!(
        service_type,
        ServiceType::SmallPark
            | ServiceType::LargePark
            | ServiceType::Playground
            | ServiceType::Plaza
            | ServiceType::SportsField
    )
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for FarTransferState {
    const SAVE_KEY: &'static str = "far_transfer";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no bonuses or transfers are active
        if self.total_bonus_far == 0.0 && self.total_transferred_far == 0.0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
