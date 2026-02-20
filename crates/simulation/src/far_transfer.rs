//! FAR Bonuses and Transfer of Development Rights (ZONE-012).
//!
//! Implements two complementary FAR enhancement mechanics:
//!
//! **FAR Bonuses**: Developers can exceed the base FAR limit in exchange for
//! public benefits:
//! - Affordable housing inclusion: +20% FAR bonus
//! - Public plaza provision: +10% FAR bonus
//! - Transit contribution: +15% FAR bonus
//!
//! **Transfer of Development Rights (TDR)**: Unused FAR capacity from
//! historic preservation districts and park parcels can be transferred to
//! nearby development sites:
//! - Source parcels: historic districts and park service buildings
//! - Transfer radius: within the same district or adjacent districts
//! - Transferred FAR is removed from the source (prevents double-counting)
//! - Creates a gameplay market for development rights
//!
//! The effective FAR for a cell is:
//!   `base_far + bonus_far + transferred_far`
//!
//! Computed on the slow tick (every ~100 ticks).

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::districts::{DistrictMap, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};
use crate::grid::ZoneType;
use crate::historic_preservation::HistoricPreservationState;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

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
// Systems
// =============================================================================

/// Update FAR bonuses for all cells with buildings.
/// Determines which bonus types each building qualifies for based on
/// zone type and level, then calculates the FAR bonus.
pub fn update_far_bonuses(
    timer: Res<SlowTickTimer>,
    buildings: Query<&Building>,
    mut state: ResMut<FarTransferState>,
) {
    if !timer.should_run() {
        return;
    }

    // Clear bonus data
    state.bonus_far.fill(0.0);
    state.bonus_flags.fill(0);
    state.total_bonus_far = 0.0;

    for building in &buildings {
        let x = building.grid_x;
        let y = building.grid_y;
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }

        let flags = eligible_bonuses(building.zone_type, building.level);
        if flags == 0 {
            continue;
        }

        let base_far = building.zone_type.default_far();
        let bonus = calculate_far_bonus(base_far, flags);

        let idx = y * GRID_WIDTH + x;
        state.bonus_far[idx] = bonus;
        state.bonus_flags[idx] = flags;
        state.total_bonus_far += bonus;
    }
}

/// Calculate available TDR FAR from source parcels (historic districts and parks).
/// Then distribute transferred FAR to eligible receiving cells.
pub fn update_far_transfers(
    timer: Res<SlowTickTimer>,
    preservation: Res<HistoricPreservationState>,
    district_map: Res<DistrictMap>,
    services: Query<&ServiceBuilding>,
    buildings: Query<&Building>,
    mut state: ResMut<FarTransferState>,
) {
    if !timer.should_run() {
        return;
    }

    let num_stat_districts = DISTRICTS_X * DISTRICTS_Y;

    // Reset transfer tracking
    state.transferred_far.fill(0.0);
    state.district_available_far.fill(0.0);
    state.district_transferred_far.fill(0.0);
    state.total_transferred_far = 0.0;

    // Ensure vectors are correctly sized
    state.district_available_far.resize(num_stat_districts, 0.0);
    state
        .district_transferred_far
        .resize(num_stat_districts, 0.0);

    // --- Step 1: Calculate available FAR from historic districts ---
    for &di in &preservation.preserved_districts {
        if di >= district_map.districts.len() {
            continue;
        }
        for &(cx, cy) in &district_map.districts[di].cells {
            if cx >= GRID_WIDTH || cy >= GRID_HEIGHT {
                continue;
            }
            // Each historic cell contributes unused FAR to its statistical district
            let (sdx, sdy) = stat_district_for_grid(cx, cy);
            let stat_idx = sdy * DISTRICTS_X + sdx;
            if stat_idx < num_stat_districts {
                state.district_available_far[stat_idx] += HISTORIC_UNUSED_FAR_PER_CELL;
            }
        }
    }

    // --- Step 2: Calculate available FAR from park service buildings ---
    for service in &services {
        if !is_park_service(service.service_type) {
            continue;
        }
        let x = service.grid_x;
        let y = service.grid_y;
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }
        let (sdx, sdy) = stat_district_for_grid(x, y);
        let stat_idx = sdy * DISTRICTS_X + sdx;
        if stat_idx < num_stat_districts {
            state.district_available_far[stat_idx] += PARK_UNUSED_FAR_PER_CELL;
        }
    }

    // --- Step 3: Distribute transferred FAR to eligible receiving cells ---
    // Receiving cells are buildings at level 2+ that are within transfer radius
    // of source districts.
    for building in &buildings {
        if building.level < 2 {
            continue;
        }
        let x = building.grid_x;
        let y = building.grid_y;
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }

        let (dst_dx, dst_dy) = stat_district_for_grid(x, y);

        // Find total available FAR from nearby source districts
        let mut available = 0.0_f32;
        let mut source_districts: Vec<usize> = Vec::new();

        for sdy in 0..DISTRICTS_Y {
            for sdx in 0..DISTRICTS_X {
                if !districts_within_transfer_radius(sdx, sdy, dst_dx, dst_dy) {
                    continue;
                }
                let stat_idx = sdy * DISTRICTS_X + sdx;
                let remaining = state.district_available_far[stat_idx]
                    - state.district_transferred_far[stat_idx];
                if remaining > 0.0 {
                    available += remaining;
                    source_districts.push(stat_idx);
                }
            }
        }

        if available <= 0.0 || source_districts.is_empty() {
            continue;
        }

        // Transfer up to MAX_TRANSFER_FAR_PER_CELL from available sources
        let transfer_amount = available.min(MAX_TRANSFER_FAR_PER_CELL);

        let idx = y * GRID_WIDTH + x;
        state.transferred_far[idx] = transfer_amount;
        state.total_transferred_far += transfer_amount;

        // Debit the transferred FAR from source districts proportionally
        let mut remaining_to_debit = transfer_amount;
        for &stat_idx in &source_districts {
            if remaining_to_debit <= 0.0 {
                break;
            }
            let source_remaining =
                state.district_available_far[stat_idx] - state.district_transferred_far[stat_idx];
            if source_remaining <= 0.0 {
                continue;
            }
            let debit = source_remaining.min(remaining_to_debit);
            state.district_transferred_far[stat_idx] += debit;
            remaining_to_debit -= debit;
        }
    }
}

/// Helper: convert grid coordinates to statistical district coordinates.
fn stat_district_for_grid(gx: usize, gy: usize) -> (usize, usize) {
    (
        (gx / DISTRICT_SIZE).min(DISTRICTS_X - 1),
        (gy / DISTRICT_SIZE).min(DISTRICTS_Y - 1),
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

// =============================================================================
// Plugin
// =============================================================================

pub struct FarTransferPlugin;

impl Plugin for FarTransferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FarTransferState>().add_systems(
            FixedUpdate,
            (update_far_bonuses, update_far_transfers)
                .chain()
                .after(crate::buildings::building_spawner),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FarTransferState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(AFFORDABLE_HOUSING_BONUS > 0.0);
        assert!(AFFORDABLE_HOUSING_BONUS < 1.0);
        assert!(PUBLIC_PLAZA_BONUS > 0.0);
        assert!(PUBLIC_PLAZA_BONUS < 1.0);
        assert!(TRANSIT_CONTRIBUTION_BONUS > 0.0);
        assert!(TRANSIT_CONTRIBUTION_BONUS < 1.0);
        assert!(MAX_BONUS_MULTIPLIER > 0.0);
        assert!(MAX_BONUS_MULTIPLIER <= 1.0);
        assert!(HISTORIC_UNUSED_FAR_PER_CELL > 0.0);
        assert!(PARK_UNUSED_FAR_PER_CELL > 0.0);
        assert!(TRANSFER_DISTRICT_RADIUS >= 1);
        assert!(MAX_TRANSFER_FAR_PER_CELL > 0.0);
    }

    #[test]
    fn test_bonus_values_match_spec() {
        assert!((AFFORDABLE_HOUSING_BONUS - 0.20).abs() < f32::EPSILON);
        assert!((PUBLIC_PLAZA_BONUS - 0.10).abs() < f32::EPSILON);
        assert!((TRANSIT_CONTRIBUTION_BONUS - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_bonus_equals_sum_of_all() {
        let sum = AFFORDABLE_HOUSING_BONUS + PUBLIC_PLAZA_BONUS + TRANSIT_CONTRIBUTION_BONUS;
        assert!(
            (MAX_BONUS_MULTIPLIER - sum).abs() < f32::EPSILON,
            "MAX_BONUS_MULTIPLIER should equal sum of all bonuses: {} vs {}",
            MAX_BONUS_MULTIPLIER,
            sum
        );
    }

    // -------------------------------------------------------------------------
    // Bonus type bit conversion tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bonus_type_bits_are_distinct() {
        let a = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let b = bonus_type_to_bit(FarBonusType::PublicPlaza);
        let c = bonus_type_to_bit(FarBonusType::TransitContribution);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
        // All should be powers of 2
        assert!(a.is_power_of_two());
        assert!(b.is_power_of_two());
        assert!(c.is_power_of_two());
    }

    #[test]
    fn test_bonus_type_multiplier() {
        assert!(
            (FarBonusType::AffordableHousing.multiplier() - AFFORDABLE_HOUSING_BONUS).abs()
                < f32::EPSILON
        );
        assert!((FarBonusType::PublicPlaza.multiplier() - PUBLIC_PLAZA_BONUS).abs() < f32::EPSILON);
        assert!(
            (FarBonusType::TransitContribution.multiplier() - TRANSIT_CONTRIBUTION_BONUS).abs()
                < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Bonus multiplier calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_bonus_multiplier_no_flags() {
        assert!((calculate_bonus_multiplier(0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_affordable_only() {
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let mult = calculate_bonus_multiplier(flags);
        assert!((mult - AFFORDABLE_HOUSING_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_plaza_only() {
        let flags = bonus_type_to_bit(FarBonusType::PublicPlaza);
        let mult = calculate_bonus_multiplier(flags);
        assert!((mult - PUBLIC_PLAZA_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_transit_only() {
        let flags = bonus_type_to_bit(FarBonusType::TransitContribution);
        let mult = calculate_bonus_multiplier(flags);
        assert!((mult - TRANSIT_CONTRIBUTION_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_all_flags() {
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing)
            | bonus_type_to_bit(FarBonusType::PublicPlaza)
            | bonus_type_to_bit(FarBonusType::TransitContribution);
        let mult = calculate_bonus_multiplier(flags);
        let expected = (AFFORDABLE_HOUSING_BONUS + PUBLIC_PLAZA_BONUS + TRANSIT_CONTRIBUTION_BONUS)
            .min(MAX_BONUS_MULTIPLIER);
        assert!((mult - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_bonus_multiplier_capped() {
        // Even with all bonuses, should not exceed MAX_BONUS_MULTIPLIER
        let flags = 0xFF; // all bits set
        let mult = calculate_bonus_multiplier(flags);
        assert!(mult <= MAX_BONUS_MULTIPLIER + f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // FAR bonus calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_far_bonus_residential_high_affordable() {
        let base_far = ZoneType::ResidentialHigh.default_far(); // 3.0
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let bonus = calculate_far_bonus(base_far, flags);
        let expected = 3.0 * 0.20; // 0.6
        assert!(
            (bonus - expected).abs() < f32::EPSILON,
            "expected {}, got {}",
            expected,
            bonus
        );
    }

    #[test]
    fn test_calculate_far_bonus_zero_flags() {
        let base_far = ZoneType::CommercialHigh.default_far();
        let bonus = calculate_far_bonus(base_far, 0);
        assert!(bonus.abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_housing_bonus_20_percent() {
        // Unit test from Definition of Done: building with affordable housing gets +20% FAR
        let base_far = 3.0; // e.g., ResidentialHigh
        let flags = bonus_type_to_bit(FarBonusType::AffordableHousing);
        let bonus = calculate_far_bonus(base_far, flags);
        let effective = effective_far(base_far, bonus, 0.0);
        // +20% means effective should be 3.6
        assert!(
            (effective - 3.6).abs() < f32::EPSILON,
            "affordable housing should give +20% FAR: base={}, effective={}",
            base_far,
            effective
        );
    }

    // -------------------------------------------------------------------------
    // Eligible bonuses tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_eligible_bonuses_low_level_no_bonuses() {
        // Level 1 buildings should not qualify for any bonuses
        let flags = eligible_bonuses(ZoneType::ResidentialLow, 1);
        assert_eq!(flags, 0, "level 1 should have no bonuses");
    }

    #[test]
    fn test_eligible_bonuses_level2_commercial_gets_plaza() {
        let flags = eligible_bonuses(ZoneType::CommercialHigh, 2);
        assert!(
            flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0,
            "level 2 commercial should get plaza bonus"
        );
        assert!(
            flags & bonus_type_to_bit(FarBonusType::AffordableHousing) == 0,
            "commercial should not get affordable housing bonus"
        );
    }

    #[test]
    fn test_eligible_bonuses_level3_residential_gets_affordable() {
        let flags = eligible_bonuses(ZoneType::ResidentialHigh, 3);
        assert!(
            flags & bonus_type_to_bit(FarBonusType::AffordableHousing) != 0,
            "level 3 residential should get affordable housing bonus"
        );
    }

    #[test]
    fn test_eligible_bonuses_level4_gets_transit() {
        let flags = eligible_bonuses(ZoneType::ResidentialHigh, 4);
        assert!(
            flags & bonus_type_to_bit(FarBonusType::TransitContribution) != 0,
            "level 4 should get transit contribution bonus"
        );
    }

    #[test]
    fn test_eligible_bonuses_level5_mixed_use_gets_all() {
        let flags = eligible_bonuses(ZoneType::MixedUse, 5);
        assert!(
            flags & bonus_type_to_bit(FarBonusType::AffordableHousing) != 0,
            "level 5 mixed-use should get affordable housing"
        );
        assert!(
            flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0,
            "level 5 mixed-use should get plaza"
        );
        assert!(
            flags & bonus_type_to_bit(FarBonusType::TransitContribution) != 0,
            "level 5 mixed-use should get transit"
        );
    }

    #[test]
    fn test_eligible_bonuses_industrial_no_affordable() {
        // Industrial zones should not get affordable housing or plaza bonuses
        let flags = eligible_bonuses(ZoneType::Industrial, 3);
        assert!(
            flags & bonus_type_to_bit(FarBonusType::AffordableHousing) == 0,
            "industrial should not get affordable housing"
        );
        assert!(
            flags & bonus_type_to_bit(FarBonusType::PublicPlaza) == 0,
            "industrial should not get plaza"
        );
    }

    #[test]
    fn test_eligible_bonuses_none_zone() {
        let flags = eligible_bonuses(ZoneType::None, 5);
        assert_eq!(flags, 0, "None zone type should have no bonuses");
    }

    #[test]
    fn test_eligible_bonuses_office_level2_gets_plaza() {
        let flags = eligible_bonuses(ZoneType::Office, 2);
        assert!(
            flags & bonus_type_to_bit(FarBonusType::PublicPlaza) != 0,
            "level 2 office should get plaza bonus"
        );
    }

    // -------------------------------------------------------------------------
    // District transfer radius tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_same_district_within_radius() {
        assert!(districts_within_transfer_radius(5, 5, 5, 5));
    }

    #[test]
    fn test_adjacent_district_within_radius() {
        assert!(districts_within_transfer_radius(5, 5, 6, 5)); // east
        assert!(districts_within_transfer_radius(5, 5, 5, 6)); // south
        assert!(districts_within_transfer_radius(5, 5, 6, 6)); // southeast
        assert!(districts_within_transfer_radius(5, 5, 4, 4)); // northwest
    }

    #[test]
    fn test_distant_district_outside_radius() {
        assert!(!districts_within_transfer_radius(0, 0, 3, 0));
        assert!(!districts_within_transfer_radius(0, 0, 0, 3));
        assert!(!districts_within_transfer_radius(0, 0, 2, 2));
    }

    // -------------------------------------------------------------------------
    // Effective FAR tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effective_far_base_only() {
        assert!((effective_far(3.0, 0.0, 0.0) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_with_bonus() {
        assert!((effective_far(3.0, 0.6, 0.0) - 3.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_with_transfer() {
        assert!((effective_far(3.0, 0.0, 1.5) - 4.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_with_both() {
        assert!((effective_far(3.0, 0.6, 1.5) - 5.1).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Park service detection tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_park_services_detected() {
        assert!(is_park_service(ServiceType::SmallPark));
        assert!(is_park_service(ServiceType::LargePark));
        assert!(is_park_service(ServiceType::Playground));
        assert!(is_park_service(ServiceType::Plaza));
        assert!(is_park_service(ServiceType::SportsField));
    }

    #[test]
    fn test_non_park_services_rejected() {
        assert!(!is_park_service(ServiceType::FireStation));
        assert!(!is_park_service(ServiceType::Hospital));
        assert!(!is_park_service(ServiceType::PoliceStation));
        assert!(!is_park_service(ServiceType::University));
    }

    // -------------------------------------------------------------------------
    // FarTransferState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = FarTransferState::default();
        assert_eq!(state.bonus_far.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.transferred_far.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.bonus_flags.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(
            state.district_available_far.len(),
            DISTRICTS_X * DISTRICTS_Y
        );
        assert_eq!(
            state.district_transferred_far.len(),
            DISTRICTS_X * DISTRICTS_Y
        );
        assert!(state.total_bonus_far.abs() < f32::EPSILON);
        assert!(state.total_transferred_far.abs() < f32::EPSILON);
    }

    #[test]
    fn test_bonus_at_default_zero() {
        let state = FarTransferState::default();
        assert!(state.bonus_at(0, 0).abs() < f32::EPSILON);
        assert!(state.bonus_at(128, 128).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transferred_at_default_zero() {
        let state = FarTransferState::default();
        assert!(state.transferred_at(0, 0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_far_adjustment() {
        let mut state = FarTransferState::default();
        let idx = 10 * GRID_WIDTH + 10;
        state.bonus_far[idx] = 0.5;
        state.transferred_far[idx] = 1.0;
        assert!((state.effective_far_adjustment(10, 10) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_has_bonus() {
        let mut state = FarTransferState::default();
        let idx = 10 * GRID_WIDTH + 10;
        state.bonus_flags[idx] = bonus_type_to_bit(FarBonusType::AffordableHousing)
            | bonus_type_to_bit(FarBonusType::PublicPlaza);
        assert!(state.has_bonus(10, 10, FarBonusType::AffordableHousing));
        assert!(state.has_bonus(10, 10, FarBonusType::PublicPlaza));
        assert!(!state.has_bonus(10, 10, FarBonusType::TransitContribution));
    }

    #[test]
    fn test_available_far_for_district() {
        let mut state = FarTransferState::default();
        state.district_available_far[0] = 10.0;
        state.district_transferred_far[0] = 3.0;
        assert!((state.available_far_for_district(0) - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_available_far_for_district_fully_transferred() {
        let mut state = FarTransferState::default();
        state.district_available_far[0] = 5.0;
        state.district_transferred_far[0] = 8.0; // over-transferred
        assert!(state.available_far_for_district(0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_available_far_for_district_out_of_bounds() {
        let state = FarTransferState::default();
        assert!(state.available_far_for_district(9999).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // TDR integration test: transfer FAR from park to adjacent development
    // -------------------------------------------------------------------------

    #[test]
    fn test_tdr_park_to_adjacent_site() {
        // Simulate: park in district (0,0) provides FAR, building in (0,0) receives it
        let mut state = FarTransferState::default();

        // Park provides FAR to district 0
        state.district_available_far[0] = PARK_UNUSED_FAR_PER_CELL;

        // Building in same district should be able to receive FAR
        let remaining = state.available_far_for_district(0);
        assert!(
            remaining > 0.0,
            "park should provide available FAR: {}",
            remaining
        );

        // Simulate transfer
        let transfer = remaining.min(MAX_TRANSFER_FAR_PER_CELL);
        state.district_transferred_far[0] += transfer;
        state.transferred_far[5 * GRID_WIDTH + 5] = transfer;

        // Verify accounting: source FAR is debited
        assert!(
            state.available_far_for_district(0).abs() < f32::EPSILON,
            "transferred FAR should be debited from source"
        );

        // Verify receiving cell has the transferred FAR
        assert!(
            (state.transferred_at(5, 5) - transfer).abs() < f32::EPSILON,
            "receiving cell should have transferred FAR"
        );
    }

    // -------------------------------------------------------------------------
    // Stat district helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_stat_district_for_grid_origin() {
        let (dx, dy) = stat_district_for_grid(0, 0);
        assert_eq!((dx, dy), (0, 0));
    }

    #[test]
    fn test_stat_district_for_grid_middle() {
        let (dx, dy) = stat_district_for_grid(128, 128);
        assert_eq!(dx, 128 / DISTRICT_SIZE);
        assert_eq!(dy, 128 / DISTRICT_SIZE);
    }

    #[test]
    fn test_stat_district_for_grid_max() {
        let (dx, dy) = stat_district_for_grid(255, 255);
        assert!(dx < DISTRICTS_X);
        assert!(dy < DISTRICTS_Y);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = FarTransferState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_bonus_active() {
        use crate::Saveable;
        let mut state = FarTransferState::default();
        state.total_bonus_far = 1.0;
        state.bonus_far[0] = 1.0;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_saves_when_transfer_active() {
        use crate::Saveable;
        let mut state = FarTransferState::default();
        state.total_transferred_far = 2.0;
        state.transferred_far[0] = 2.0;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = FarTransferState::default();
        state.bonus_far[100] = 0.6;
        state.transferred_far[200] = 1.5;
        state.bonus_flags[100] = bonus_type_to_bit(FarBonusType::AffordableHousing);
        state.district_available_far[0] = 5.0;
        state.district_transferred_far[0] = 2.0;
        state.total_bonus_far = 0.6;
        state.total_transferred_far = 1.5;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = FarTransferState::load_from_bytes(&bytes);

        assert!((restored.bonus_far[100] - 0.6).abs() < f32::EPSILON);
        assert!((restored.transferred_far[200] - 1.5).abs() < f32::EPSILON);
        assert_eq!(
            restored.bonus_flags[100],
            bonus_type_to_bit(FarBonusType::AffordableHousing)
        );
        assert!((restored.district_available_far[0] - 5.0).abs() < f32::EPSILON);
        assert!((restored.district_transferred_far[0] - 2.0).abs() < f32::EPSILON);
        assert!((restored.total_bonus_far - 0.6).abs() < f32::EPSILON);
        assert!((restored.total_transferred_far - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(FarTransferState::SAVE_KEY, "far_transfer");
    }
}
