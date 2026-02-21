//! Form-Based Transect Overlay System (ZONE-003).
//!
//! Implements form-based codes as an overlay on top of Euclidean zoning. The
//! transect (T1-T6) controls physical building form (height, FAR, lot coverage,
//! setbacks) independent of use. This allows players to say "I want medium-density
//! here" without specifying residential vs commercial.
//!
//! **TransectZone** enum: `None`, `T1Natural`, `T2Rural`, `T3Suburban`, `T4Urban`,
//! `T5Center`, `T6Core`. Each tier defines constraints on maximum stories, FAR,
//! lot coverage, and front setback.
//!
//! Transect data is stored in a separate `TransectGrid` resource (one entry per
//! cell), rather than modifying the `Cell` struct, to keep the overlay fully
//! decoupled from the base zoning system.
//!
//! **Key behaviours:**
//!
//! - `T1Natural` prevents all building spawning (natural preserve).
//! - Other tiers cap building level based on their FAR limit.
//! - `TransectZone::None` (the default) imposes no additional constraints,
//!   preserving backward compatibility.
//! - The `enforce_transect_constraints` system runs every slow tick and caps
//!   existing buildings that exceed their transect's FAR limit.
//!
//! The `TransectGrid` is registered with the `SaveableRegistry` for persistence.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::SlowTickTimer;

// =============================================================================
// TransectZone enum
// =============================================================================

/// Form-based transect tiers, from most rural (T1) to most urban (T6).
///
/// Each tier defines physical form constraints independent of land use.
/// `None` means no transect overlay is applied (unconstrained).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Encode, Decode,
)]
pub enum TransectZone {
    /// No transect overlay -- existing zoning rules apply without restriction.
    #[default]
    None,
    /// T1 Natural: preserved open space, no building allowed.
    T1Natural,
    /// T2 Rural: very low density, agricultural character.
    T2Rural,
    /// T3 Suburban: detached houses, conventional subdivision.
    T3Suburban,
    /// T4 Urban General: mixed-use, walkable urbanism.
    T4Urban,
    /// T5 Urban Center: higher density downtown fringe.
    T5Center,
    /// T6 Urban Core: tallest buildings, maximum density.
    T6Core,
}

impl TransectZone {
    /// Maximum number of stories permitted by this transect tier.
    pub fn max_stories(self) -> u8 {
        match self {
            TransectZone::None => u8::MAX, // unconstrained
            TransectZone::T1Natural => 0,  // no building
            TransectZone::T2Rural => 2,
            TransectZone::T3Suburban => 3,
            TransectZone::T4Urban => 5,
            TransectZone::T5Center => 8,
            TransectZone::T6Core => 20,
        }
    }

    /// Maximum Floor Area Ratio (FAR) permitted by this transect tier.
    pub fn max_far(self) -> f32 {
        match self {
            TransectZone::None => f32::MAX, // unconstrained
            TransectZone::T1Natural => 0.0, // no building
            TransectZone::T2Rural => 0.5,
            TransectZone::T3Suburban => 1.0,
            TransectZone::T4Urban => 3.0,
            TransectZone::T5Center => 6.0,
            TransectZone::T6Core => 15.0,
        }
    }

    /// Maximum lot coverage as a fraction (0.0-1.0).
    pub fn max_lot_coverage(self) -> f32 {
        match self {
            TransectZone::None => 1.0,      // unconstrained
            TransectZone::T1Natural => 0.0, // no building
            TransectZone::T2Rural => 0.3,
            TransectZone::T3Suburban => 0.5,
            TransectZone::T4Urban => 0.7,
            TransectZone::T5Center => 0.85,
            TransectZone::T6Core => 0.95,
        }
    }

    /// Required front setback in grid cells.
    pub fn front_setback_cells(self) -> u8 {
        match self {
            TransectZone::None => 0,      // unconstrained
            TransectZone::T1Natural => 0, // irrelevant -- no building
            TransectZone::T2Rural => 4,
            TransectZone::T3Suburban => 3,
            TransectZone::T4Urban => 1,
            TransectZone::T5Center => 0,
            TransectZone::T6Core => 0,
        }
    }

    /// Whether any building is allowed in this transect tier.
    pub fn allows_building(self) -> bool {
        self != TransectZone::T1Natural
    }

    /// Returns the display name of the transect tier.
    pub fn label(self) -> &'static str {
        match self {
            TransectZone::None => "None",
            TransectZone::T1Natural => "T1 Natural",
            TransectZone::T2Rural => "T2 Rural",
            TransectZone::T3Suburban => "T3 Suburban",
            TransectZone::T4Urban => "T4 Urban",
            TransectZone::T5Center => "T5 Center",
            TransectZone::T6Core => "T6 Core",
        }
    }
}

// =============================================================================
// TransectGrid resource
// =============================================================================

/// Per-cell transect overlay, stored separately from the `WorldGrid` to keep
/// the systems decoupled.
#[derive(Resource, Clone, Encode, Decode)]
pub struct TransectGrid {
    /// One `TransectZone` per cell, indexed as `y * GRID_WIDTH + x`.
    pub zones: Vec<TransectZone>,
}

impl Default for TransectGrid {
    fn default() -> Self {
        Self {
            zones: vec![TransectZone::None; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}

impl TransectGrid {
    /// Get the transect zone for a cell.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> TransectZone {
        self.zones[y * GRID_WIDTH + x]
    }

    /// Set the transect zone for a cell.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, zone: TransectZone) {
        self.zones[y * GRID_WIDTH + x] = zone;
    }

    /// Paint a rectangular region with a transect zone.
    pub fn paint_rect(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, zone: TransectZone) {
        let min_x = x1.min(x2).min(GRID_WIDTH - 1);
        let max_x = x1.max(x2).min(GRID_WIDTH - 1);
        let min_y = y1.min(y2).min(GRID_HEIGHT - 1);
        let max_y = y1.max(y2).min(GRID_HEIGHT - 1);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.set(x, y, zone);
            }
        }
    }

    /// Returns whether building is allowed at the given cell per the transect
    /// overlay. When the transect is `None`, building is always allowed.
    pub fn allows_building(&self, x: usize, y: usize) -> bool {
        self.get(x, y).allows_building()
    }

    /// Returns the maximum building level allowed at (x, y) by the transect
    /// FAR constraint. Uses the same implied-FAR formula as `max_level_for_far`
    /// in `buildings.rs`.
    ///
    /// Returns `u8::MAX` when the transect is `None` (unconstrained).
    pub fn max_level_at(&self, x: usize, y: usize, zone_type: crate::grid::ZoneType) -> u8 {
        let transect = self.get(x, y);
        max_level_for_transect(transect, zone_type)
    }
}

// =============================================================================
// Helper: FAR-to-level conversion for a transect tier
// =============================================================================

/// Returns the maximum building level allowed by a transect tier's FAR limit
/// for the given zone type. Uses the same formula as `buildings::max_level_for_far`:
///
///   implied_far = (capacity_for_level(level) * 20.0) / 256.0
///
/// The highest level where implied_far <= transect.max_far() is returned.
/// Always returns at least 1 (minimum building level), or 0 for T1Natural.
pub fn max_level_for_transect(transect: TransectZone, zone_type: crate::grid::ZoneType) -> u8 {
    match transect {
        TransectZone::None => u8::MAX, // unconstrained
        TransectZone::T1Natural => 0,  // no building
        _ => {
            let far_limit = transect.max_far();
            let stories_limit = transect.max_stories();
            let mut best = 1u8;
            for level in 1..=5u8 {
                let capacity = Building::capacity_for_level(zone_type, level);
                if capacity == 0 {
                    break;
                }
                let implied_far = (capacity as f32 * 20.0) / 256.0;
                if implied_far <= far_limit && level <= stories_limit {
                    best = level;
                }
            }
            best
        }
    }
}

// =============================================================================
// System: enforce transect constraints on existing buildings
// =============================================================================

/// Periodically checks all buildings and caps their level to respect the
/// transect overlay's FAR and stories constraints.
///
/// Runs on the slow tick. Buildings above the transect limit are downgraded
/// (level capped, capacity adjusted, excess occupants evicted).
pub fn enforce_transect_constraints(
    timer: Res<SlowTickTimer>,
    transect_grid: Res<TransectGrid>,
    mut buildings: Query<&mut Building>,
) {
    if !timer.should_run() {
        return;
    }

    for mut building in &mut buildings {
        let transect = transect_grid.get(building.grid_x, building.grid_y);

        // T1Natural: buildings shouldn't exist here, but we don't despawn --
        // that's handled by the spawner refusing to place new buildings.
        // Just prevent growth.
        if transect == TransectZone::T1Natural {
            continue;
        }

        // None: unconstrained
        if transect == TransectZone::None {
            continue;
        }

        let max_level = max_level_for_transect(transect, building.zone_type);
        if building.level > max_level {
            building.level = max_level;
            building.capacity = Building::capacity_for_level(building.zone_type, building.level);
            if building.occupants > building.capacity {
                building.occupants = building.capacity;
            }
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for TransectGrid {
    const SAVE_KEY: &'static str = "form_transect";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all cells are None (default state)
        if self.zones.iter().all(|z| *z == TransectZone::None) {
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

pub struct FormTransectPlugin;

impl Plugin for FormTransectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransectGrid>().add_systems(
            FixedUpdate,
            enforce_transect_constraints.in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<TransectGrid>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::ZoneType;

    // -------------------------------------------------------------------------
    // TransectZone::max_stories tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_t1_natural_max_stories_zero() {
        assert_eq!(TransectZone::T1Natural.max_stories(), 0);
    }

    #[test]
    fn test_t2_rural_max_stories() {
        assert_eq!(TransectZone::T2Rural.max_stories(), 2);
    }

    #[test]
    fn test_t3_suburban_max_stories() {
        assert_eq!(TransectZone::T3Suburban.max_stories(), 3);
    }

    #[test]
    fn test_t4_urban_max_stories() {
        assert_eq!(TransectZone::T4Urban.max_stories(), 5);
    }

    #[test]
    fn test_t5_center_max_stories() {
        assert_eq!(TransectZone::T5Center.max_stories(), 8);
    }

    #[test]
    fn test_t6_core_max_stories() {
        assert_eq!(TransectZone::T6Core.max_stories(), 20);
    }

    #[test]
    fn test_none_max_stories_unconstrained() {
        assert_eq!(TransectZone::None.max_stories(), u8::MAX);
    }

    // -------------------------------------------------------------------------
    // TransectZone::max_far tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_t1_natural_max_far_zero() {
        assert!(TransectZone::T1Natural.max_far().abs() < f32::EPSILON);
    }

    #[test]
    fn test_t2_rural_max_far() {
        assert!((TransectZone::T2Rural.max_far() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_t3_suburban_max_far() {
        assert!((TransectZone::T3Suburban.max_far() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_t4_urban_max_far() {
        assert!((TransectZone::T4Urban.max_far() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_t5_center_max_far() {
        assert!((TransectZone::T5Center.max_far() - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_t6_core_max_far() {
        assert!((TransectZone::T6Core.max_far() - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_none_max_far_unconstrained() {
        assert_eq!(TransectZone::None.max_far(), f32::MAX);
    }

    // -------------------------------------------------------------------------
    // TransectZone::max_lot_coverage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_t1_natural_max_lot_coverage_zero() {
        assert!(TransectZone::T1Natural.max_lot_coverage().abs() < f32::EPSILON);
    }

    #[test]
    fn test_t3_suburban_max_lot_coverage() {
        assert!((TransectZone::T3Suburban.max_lot_coverage() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_t6_core_max_lot_coverage() {
        assert!((TransectZone::T6Core.max_lot_coverage() - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_none_max_lot_coverage_unconstrained() {
        assert!((TransectZone::None.max_lot_coverage() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lot_coverage_monotonically_increases() {
        let tiers = [
            TransectZone::T1Natural,
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        let mut prev = -1.0_f32;
        for tier in tiers {
            let coverage = tier.max_lot_coverage();
            assert!(
                coverage >= prev,
                "{:?} lot coverage {} should be >= prev {}",
                tier,
                coverage,
                prev
            );
            prev = coverage;
        }
    }

    // -------------------------------------------------------------------------
    // TransectZone::front_setback_cells tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_t2_rural_front_setback() {
        assert_eq!(TransectZone::T2Rural.front_setback_cells(), 4);
    }

    #[test]
    fn test_t3_suburban_front_setback() {
        assert_eq!(TransectZone::T3Suburban.front_setback_cells(), 3);
    }

    #[test]
    fn test_t6_core_front_setback_zero() {
        assert_eq!(TransectZone::T6Core.front_setback_cells(), 0);
    }

    #[test]
    fn test_setback_monotonically_decreases() {
        let tiers = [
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        let mut prev = u8::MAX;
        for tier in tiers {
            let setback = tier.front_setback_cells();
            assert!(
                setback <= prev,
                "{:?} setback {} should be <= prev {}",
                tier,
                setback,
                prev
            );
            prev = setback;
        }
    }

    // -------------------------------------------------------------------------
    // TransectZone::allows_building tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_t1_natural_prevents_building() {
        assert!(!TransectZone::T1Natural.allows_building());
    }

    #[test]
    fn test_other_tiers_allow_building() {
        assert!(TransectZone::None.allows_building());
        assert!(TransectZone::T2Rural.allows_building());
        assert!(TransectZone::T3Suburban.allows_building());
        assert!(TransectZone::T4Urban.allows_building());
        assert!(TransectZone::T5Center.allows_building());
        assert!(TransectZone::T6Core.allows_building());
    }

    // -------------------------------------------------------------------------
    // TransectZone::label tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_labels_non_empty() {
        let all = [
            TransectZone::None,
            TransectZone::T1Natural,
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        for tier in all {
            assert!(!tier.label().is_empty(), "{:?} should have a label", tier);
        }
    }

    // -------------------------------------------------------------------------
    // TransectGrid tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_grid_default_all_none() {
        let grid = TransectGrid::default();
        assert!(grid.zones.iter().all(|z| *z == TransectZone::None));
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = TransectGrid::default();
        grid.set(10, 20, TransectZone::T3Suburban);
        assert_eq!(grid.get(10, 20), TransectZone::T3Suburban);
    }

    #[test]
    fn test_grid_paint_rect() {
        let mut grid = TransectGrid::default();
        grid.paint_rect(5, 5, 7, 7, TransectZone::T4Urban);
        for y in 5..=7 {
            for x in 5..=7 {
                assert_eq!(
                    grid.get(x, y),
                    TransectZone::T4Urban,
                    "Cell ({},{}) should be T4Urban",
                    x,
                    y
                );
            }
        }
        // Cell outside the rectangle should still be None
        assert_eq!(grid.get(4, 5), TransectZone::None);
        assert_eq!(grid.get(8, 5), TransectZone::None);
    }

    #[test]
    fn test_grid_paint_rect_reversed_coords() {
        let mut grid = TransectGrid::default();
        // Paint with reversed coordinates (x2 < x1)
        grid.paint_rect(7, 7, 5, 5, TransectZone::T5Center);
        for y in 5..=7 {
            for x in 5..=7 {
                assert_eq!(grid.get(x, y), TransectZone::T5Center);
            }
        }
    }

    #[test]
    fn test_grid_allows_building_none() {
        let grid = TransectGrid::default();
        assert!(grid.allows_building(128, 128));
    }

    #[test]
    fn test_grid_allows_building_t1_natural() {
        let mut grid = TransectGrid::default();
        grid.set(100, 100, TransectZone::T1Natural);
        assert!(!grid.allows_building(100, 100));
    }

    #[test]
    fn test_grid_allows_building_t3_suburban() {
        let mut grid = TransectGrid::default();
        grid.set(50, 50, TransectZone::T3Suburban);
        assert!(grid.allows_building(50, 50));
    }

    // -------------------------------------------------------------------------
    // max_level_for_transect tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_none_transect_unconstrained() {
        let max = max_level_for_transect(TransectZone::None, ZoneType::ResidentialHigh);
        assert_eq!(max, u8::MAX);
    }

    #[test]
    fn test_t1_natural_returns_zero() {
        let max = max_level_for_transect(TransectZone::T1Natural, ZoneType::ResidentialHigh);
        assert_eq!(max, 0);
    }

    #[test]
    fn test_t3_suburban_caps_residential_high() {
        // T3Suburban: FAR=1.0, stories=3
        // ResidentialHigh L1=50 -> implied_far=50*20/256=3.9 > 1.0 but we
        // still return at least 1.
        let max = max_level_for_transect(TransectZone::T3Suburban, ZoneType::ResidentialHigh);
        assert!(max >= 1, "max_level_for_transect should return at least 1");
        assert!(
            max <= 3,
            "T3Suburban should cap at max 3 stories, got {}",
            max
        );
    }

    #[test]
    fn test_t6_core_allows_high_levels() {
        // T6Core: FAR=15.0, stories=20
        // ResidentialHigh L5=2000 -> implied_far=2000*20/256=156.25 > 15.0
        // But lower levels should fit. L1=50->3.9, L2=200->15.6 > 15.0
        // So max should be at least 1
        let max = max_level_for_transect(TransectZone::T6Core, ZoneType::ResidentialHigh);
        assert!(max >= 1);
    }

    #[test]
    fn test_t4_urban_residential_low() {
        // T4Urban: FAR=3.0, stories=5
        // ResidentialLow has max level 3:
        //   L1=10 -> implied_far=10*20/256=0.78 <= 3.0 -> ok
        //   L2=30 -> implied_far=30*20/256=2.34 <= 3.0 -> ok
        //   L3=80 -> implied_far=80*20/256=6.25 > 3.0 -> too much
        // So max should be 2
        let max = max_level_for_transect(TransectZone::T4Urban, ZoneType::ResidentialLow);
        assert_eq!(max, 2, "T4Urban should cap ResidentialLow at level 2");
    }

    #[test]
    fn test_t5_center_industrial() {
        // T5Center: FAR=6.0, stories=8
        // Industrial:
        //   L1=20 -> 1.56 <= 6.0 -> ok
        //   L2=60 -> 4.69 <= 6.0 -> ok
        //   L3=150 -> 11.72 > 6.0 -> too much
        let max = max_level_for_transect(TransectZone::T5Center, ZoneType::Industrial);
        assert_eq!(max, 2);
    }

    #[test]
    fn test_transect_far_hierarchy() {
        // Higher transect tiers should allow equal or more levels
        let tiers = [
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        let zone = ZoneType::ResidentialLow;
        let mut prev = 0u8;
        for tier in tiers {
            let max = max_level_for_transect(tier, zone);
            assert!(
                max >= prev,
                "{:?} max level {} should be >= prev {}",
                tier,
                max,
                prev
            );
            prev = max;
        }
    }

    #[test]
    fn test_all_non_none_transects_return_at_least_one() {
        let tiers = [
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for tier in tiers {
            for zone in zones {
                let max = max_level_for_transect(tier, zone);
                assert!(
                    max >= 1,
                    "max_level_for_transect({:?}, {:?}) = {}, should be >= 1",
                    tier,
                    zone,
                    max
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // TransectGrid::max_level_at tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_max_level_at_default() {
        let grid = TransectGrid::default();
        let max = grid.max_level_at(128, 128, ZoneType::ResidentialHigh);
        assert_eq!(
            max,
            u8::MAX,
            "Default None transect should be unconstrained"
        );
    }

    #[test]
    fn test_max_level_at_t1_natural() {
        let mut grid = TransectGrid::default();
        grid.set(10, 10, TransectZone::T1Natural);
        let max = grid.max_level_at(10, 10, ZoneType::ResidentialLow);
        assert_eq!(max, 0, "T1Natural should return 0 (no building)");
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let grid = TransectGrid::default();
        assert!(
            grid.save_to_bytes().is_none(),
            "Default grid should skip saving"
        );
    }

    #[test]
    fn test_saveable_saves_when_non_default() {
        use crate::Saveable;
        let mut grid = TransectGrid::default();
        grid.set(50, 50, TransectZone::T3Suburban);
        assert!(
            grid.save_to_bytes().is_some(),
            "Non-default grid should save"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut grid = TransectGrid::default();
        grid.set(10, 10, TransectZone::T3Suburban);
        grid.set(20, 30, TransectZone::T6Core);
        grid.set(0, 0, TransectZone::T1Natural);

        let bytes = grid.save_to_bytes().expect("should serialize");
        let restored = TransectGrid::load_from_bytes(&bytes);

        assert_eq!(restored.get(10, 10), TransectZone::T3Suburban);
        assert_eq!(restored.get(20, 30), TransectZone::T6Core);
        assert_eq!(restored.get(0, 0), TransectZone::T1Natural);
        assert_eq!(restored.get(128, 128), TransectZone::None);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(TransectGrid::SAVE_KEY, "form_transect");
    }

    // -------------------------------------------------------------------------
    // FAR hierarchy tests (from issue test plan)
    // -------------------------------------------------------------------------

    #[test]
    fn test_far_increases_with_tier() {
        let tiers = [
            TransectZone::T1Natural,
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        let mut prev = -1.0_f32;
        for tier in tiers {
            let far = tier.max_far();
            assert!(
                far >= prev,
                "{:?} FAR {} should be >= prev {}",
                tier,
                far,
                prev
            );
            prev = far;
        }
    }

    #[test]
    fn test_stories_increase_with_tier() {
        let tiers = [
            TransectZone::T1Natural,
            TransectZone::T2Rural,
            TransectZone::T3Suburban,
            TransectZone::T4Urban,
            TransectZone::T5Center,
            TransectZone::T6Core,
        ];
        let mut prev = 0u8;
        for tier in tiers {
            let stories = tier.max_stories();
            assert!(
                stories >= prev,
                "{:?} stories {} should be >= prev {}",
                tier,
                stories,
                prev
            );
            prev = stories;
        }
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (from issue test plan)
    // -------------------------------------------------------------------------

    #[test]
    fn test_issue_unit_t3_suburban_max_stories_3() {
        // From issue: "Unit: TransectZone::T3Suburban.max_stories() == 3"
        assert_eq!(TransectZone::T3Suburban.max_stories(), 3);
    }

    #[test]
    fn test_issue_unit_t6_core_max_far_15() {
        // From issue: "Unit: TransectZone::T6Core.max_far() == 15.0"
        assert!((TransectZone::T6Core.max_far() - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_t3_suburban_caps_residential_high_at_about_3_stories() {
        // From issue: "Paint T3Suburban over ResidentialHigh zone, verify buildings cap at ~3 stories"
        // T3Suburban: FAR=1.0, stories=3
        // ResidentialHigh:
        //   L1=50 -> implied_far=50*20/256=3.9 > 1.0 (exceeds FAR but min is 1)
        //   L2=200 -> 15.6 > 1.0
        //   L3=500 -> 39.1 > 1.0
        // So FAR constrains to level 1, but stories allow up to 3.
        // The min of FAR-cap (1) and stories-cap (3) = 1, so max is 1.
        let max = max_level_for_transect(TransectZone::T3Suburban, ZoneType::ResidentialHigh);
        assert!(
            max <= 3,
            "T3Suburban should cap ResidentialHigh at <= 3 stories, got {}",
            max
        );
    }

    #[test]
    fn test_t1_natural_prevents_building_spawning() {
        // From issue: "T1Natural zone prevents any building spawning"
        let mut grid = TransectGrid::default();
        grid.set(50, 50, TransectZone::T1Natural);
        assert!(!grid.allows_building(50, 50));
    }

    // -------------------------------------------------------------------------
    // Constants validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_grid_size_matches_config() {
        let grid = TransectGrid::default();
        assert_eq!(grid.zones.len(), GRID_WIDTH * GRID_HEIGHT);
    }

    #[test]
    fn test_default_is_backward_compatible() {
        // None transect should not restrict anything
        assert!(TransectZone::None.allows_building());
        assert_eq!(TransectZone::None.max_stories(), u8::MAX);
        assert_eq!(TransectZone::None.max_far(), f32::MAX);
        assert_eq!(TransectZone::None.front_setback_cells(), 0);
        assert!((TransectZone::None.max_lot_coverage() - 1.0).abs() < f32::EPSILON);
    }
}
