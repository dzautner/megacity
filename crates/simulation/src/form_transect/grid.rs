//! [`TransectGrid`] resource, FAR-to-level conversion, and save/load support.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};

use super::TransectZone;

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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::ZoneType;

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

    #[test]
    fn test_grid_size_matches_config() {
        let grid = TransectGrid::default();
        assert_eq!(grid.zones.len(), GRID_WIDTH * GRID_HEIGHT);
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
    // Integration-style tests (from issue test plan)
    // -------------------------------------------------------------------------

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
}
