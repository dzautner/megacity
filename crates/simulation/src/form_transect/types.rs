//! [`TransectZone`] enum and associated form-based code constants.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

    // -------------------------------------------------------------------------
    // Constants validation
    // -------------------------------------------------------------------------

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
