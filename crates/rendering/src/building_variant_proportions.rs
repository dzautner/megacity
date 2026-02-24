//! Per-variant proportion tables for building mesh variants (REND-003).
//!
//! Each zone type / level combination has 2-3 scale proportion vectors that
//! control the X (width), Y (height), and Z (depth) multipliers applied to the
//! base building scale.  This ensures that buildings of the same zone and level
//! look visually distinct -- one might be tall and narrow, another squat and
//! wide -- even when they share the same underlying GLB model.
//!
//! The proportions are purely cosmetic and do not affect simulation.

use simulation::grid::ZoneType;

/// Scale proportions for a single building variant: (x_scale, y_scale, z_scale).
///
/// These are multiplied against the uniform base scale from `building_scale()`.
/// A value of 1.0 means "no change from base", >1.0 stretches, <1.0 compresses.
#[derive(Debug, Clone, Copy)]
pub struct VariantProportion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl VariantProportion {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

// ---------------------------------------------------------------------------
// Residential Low: small houses, cottages, ranch houses
// ---------------------------------------------------------------------------

const RES_LOW_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),   // standard small house
    VariantProportion::new(1.15, 0.85, 1.05), // wider, squatter cottage
    VariantProportion::new(0.9, 1.1, 0.9),    // narrower, taller bungalow
];

const RES_LOW_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),   // standard larger house
    VariantProportion::new(1.2, 0.9, 1.0),    // wide duplex
    VariantProportion::new(0.85, 1.15, 1.1),  // tall row house
];

const RES_LOW_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),   // standard small apartment
    VariantProportion::new(1.1, 0.95, 1.15),  // wide townhouse complex
    VariantProportion::new(0.9, 1.2, 0.85),   // tall narrow townhouse
];

// ---------------------------------------------------------------------------
// Residential Medium: townhouses, duplexes, small apartments
// ---------------------------------------------------------------------------

const RES_MED_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.15, 0.9, 1.05),
    VariantProportion::new(0.9, 1.1, 0.95),
];

const RES_MED_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.95, 1.1),
    VariantProportion::new(0.85, 1.15, 0.9),
];

const RES_MED_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.9, 1.15),
    VariantProportion::new(0.9, 1.2, 0.85),
];

const RES_MED_L4: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.05, 0.95, 1.1),
    VariantProportion::new(0.9, 1.15, 0.9),
];

// ---------------------------------------------------------------------------
// Residential High: apartment blocks, mid-rises, towers
// ---------------------------------------------------------------------------

const RES_HIGH_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),    // standard apartment block
    VariantProportion::new(1.15, 0.9, 1.0),    // wide block
    VariantProportion::new(0.85, 1.15, 0.95),  // taller mid-rise
];

const RES_HIGH_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.9, 1.1),
    VariantProportion::new(0.85, 1.2, 0.85),
];

const RES_HIGH_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),    // standard tall tower
    VariantProportion::new(1.1, 0.85, 1.1),    // stocky tower
    VariantProportion::new(0.8, 1.25, 0.8),    // slender luxury tower
];

const RES_HIGH_L4: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.05, 0.9, 1.1),
    VariantProportion::new(0.85, 1.2, 0.85),
];

const RES_HIGH_L5: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.05),
    VariantProportion::new(0.8, 1.3, 0.8),    // super-tall slender tower
];

// ---------------------------------------------------------------------------
// Commercial Low: corner stores, cafes, small shops
// ---------------------------------------------------------------------------

const COM_LOW_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),    // standard corner store
    VariantProportion::new(1.2, 0.85, 1.0),    // wide storefront
    VariantProportion::new(0.9, 1.1, 1.15),    // deeper cafe
];

const COM_LOW_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.15, 0.9, 1.1),
    VariantProportion::new(0.85, 1.15, 0.9),
];

const COM_LOW_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.9, 1.15),
    VariantProportion::new(0.85, 1.2, 0.85),
];

// ---------------------------------------------------------------------------
// Commercial High: strip malls, retail stores, department stores
// ---------------------------------------------------------------------------

const COM_HIGH_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),    // standard retail
    VariantProportion::new(1.2, 0.85, 1.1),    // wide strip mall
    VariantProportion::new(0.85, 1.15, 0.9),   // tall retail
];

const COM_HIGH_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.15, 0.9, 1.05),
    VariantProportion::new(0.85, 1.2, 0.9),
];

const COM_HIGH_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.15),
    VariantProportion::new(0.8, 1.25, 0.85),
];

const COM_HIGH_L4: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.05, 0.9, 1.1),
    VariantProportion::new(0.85, 1.2, 0.85),
];

const COM_HIGH_L5: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.05),
    VariantProportion::new(0.8, 1.3, 0.8),
];

// ---------------------------------------------------------------------------
// Industrial: warehouses, small factories, heavy industry
// ---------------------------------------------------------------------------

const IND_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),    // standard warehouse
    VariantProportion::new(1.25, 0.8, 1.1),    // wide low warehouse
    VariantProportion::new(0.9, 1.15, 1.2),    // deep factory
];

const IND_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.2, 0.85, 1.15),
    VariantProportion::new(0.85, 1.2, 0.9),
];

const IND_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.15, 0.9, 1.1),
    VariantProportion::new(0.85, 1.15, 1.05),
];

const IND_L4: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.9, 1.15),
    VariantProportion::new(0.9, 1.15, 0.9),
];

const IND_L5: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.15, 0.85, 1.1),
    VariantProportion::new(0.85, 1.2, 0.9),
];

// ---------------------------------------------------------------------------
// Office: small offices, professional buildings, towers
// ---------------------------------------------------------------------------

const OFF_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),    // standard small office
    VariantProportion::new(1.15, 0.9, 1.0),    // wide professional building
    VariantProportion::new(0.85, 1.15, 1.05),  // taller office
];

const OFF_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.9, 1.1),
    VariantProportion::new(0.85, 1.2, 0.85),
];

const OFF_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.1),
    VariantProportion::new(0.8, 1.25, 0.85),
];

const OFF_L4: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.05, 0.9, 1.1),
    VariantProportion::new(0.8, 1.25, 0.85),
];

const OFF_L5: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.05),
    VariantProportion::new(0.75, 1.35, 0.75),  // very slender skyscraper
];

// ---------------------------------------------------------------------------
// Mixed-Use
// ---------------------------------------------------------------------------

const MIX_L1: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.15, 0.9, 1.05),
    VariantProportion::new(0.9, 1.1, 0.95),
];

const MIX_L2: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.95, 1.1),
    VariantProportion::new(0.85, 1.15, 0.9),
];

const MIX_L3: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.1),
    VariantProportion::new(0.8, 1.25, 0.85),
];

const MIX_L4: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.05, 0.9, 1.1),
    VariantProportion::new(0.85, 1.2, 0.85),
];

const MIX_L5: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.85, 1.05),
    VariantProportion::new(0.8, 1.3, 0.8),
];

// ---------------------------------------------------------------------------
// Default fallback
// ---------------------------------------------------------------------------

const DEFAULT_PROPORTIONS: [VariantProportion; 3] = [
    VariantProportion::new(1.0, 1.0, 1.0),
    VariantProportion::new(1.1, 0.9, 1.05),
    VariantProportion::new(0.9, 1.1, 0.95),
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up the proportion table for a given zone type and building level.
///
/// Returns a slice of 3 `VariantProportion` entries. The caller uses the
/// variant index (from `variant_hash`) to pick one.
pub fn proportions_for(zone: ZoneType, level: u8) -> &'static [VariantProportion; 3] {
    match (zone, level) {
        // Residential Low
        (ZoneType::ResidentialLow, 1) => &RES_LOW_L1,
        (ZoneType::ResidentialLow, 2) => &RES_LOW_L2,
        (ZoneType::ResidentialLow, _) => &RES_LOW_L3,

        // Residential Medium
        (ZoneType::ResidentialMedium, 1) => &RES_MED_L1,
        (ZoneType::ResidentialMedium, 2) => &RES_MED_L2,
        (ZoneType::ResidentialMedium, 3) => &RES_MED_L3,
        (ZoneType::ResidentialMedium, _) => &RES_MED_L4,

        // Residential High
        (ZoneType::ResidentialHigh, 1) => &RES_HIGH_L1,
        (ZoneType::ResidentialHigh, 2) => &RES_HIGH_L2,
        (ZoneType::ResidentialHigh, 3) => &RES_HIGH_L3,
        (ZoneType::ResidentialHigh, 4) => &RES_HIGH_L4,
        (ZoneType::ResidentialHigh, _) => &RES_HIGH_L5,

        // Commercial Low
        (ZoneType::CommercialLow, 1) => &COM_LOW_L1,
        (ZoneType::CommercialLow, 2) => &COM_LOW_L2,
        (ZoneType::CommercialLow, _) => &COM_LOW_L3,

        // Commercial High
        (ZoneType::CommercialHigh, 1) => &COM_HIGH_L1,
        (ZoneType::CommercialHigh, 2) => &COM_HIGH_L2,
        (ZoneType::CommercialHigh, 3) => &COM_HIGH_L3,
        (ZoneType::CommercialHigh, 4) => &COM_HIGH_L4,
        (ZoneType::CommercialHigh, _) => &COM_HIGH_L5,

        // Industrial
        (ZoneType::Industrial, 1) => &IND_L1,
        (ZoneType::Industrial, 2) => &IND_L2,
        (ZoneType::Industrial, 3) => &IND_L3,
        (ZoneType::Industrial, 4) => &IND_L4,
        (ZoneType::Industrial, _) => &IND_L5,

        // Office
        (ZoneType::Office, 1) => &OFF_L1,
        (ZoneType::Office, 2) => &OFF_L2,
        (ZoneType::Office, 3) => &OFF_L3,
        (ZoneType::Office, 4) => &OFF_L4,
        (ZoneType::Office, _) => &OFF_L5,

        // Mixed-Use
        (ZoneType::MixedUse, 1) => &MIX_L1,
        (ZoneType::MixedUse, 2) => &MIX_L2,
        (ZoneType::MixedUse, 3) => &MIX_L3,
        (ZoneType::MixedUse, 4) => &MIX_L4,
        (ZoneType::MixedUse, _) => &MIX_L5,

        // Fallback
        _ => &DEFAULT_PROPORTIONS,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_zone_levels_return_3_variants() {
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
        for zone in &zones {
            for level in 1..=5 {
                let props = proportions_for(*zone, level);
                assert_eq!(props.len(), 3, "{zone:?} L{level} must have 3 variants");
            }
        }
    }

    #[test]
    fn variant_0_is_always_uniform() {
        // Variant 0 should always be the "standard" 1.0/1.0/1.0
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
        ];
        for zone in &zones {
            for level in 1..=3 {
                let p = &proportions_for(*zone, level)[0];
                assert!(
                    (p.x - 1.0).abs() < 0.001
                        && (p.y - 1.0).abs() < 0.001
                        && (p.z - 1.0).abs() < 0.001,
                    "{zone:?} L{level} variant 0 should be uniform scale"
                );
            }
        }
    }

    #[test]
    fn variants_differ_from_each_other() {
        // Variants 1 and 2 should differ from variant 0
        let props = proportions_for(ZoneType::ResidentialLow, 1);
        let p0 = props[0];
        let p1 = props[1];
        let p2 = props[2];
        // At least one axis should differ
        assert!(
            (p0.x - p1.x).abs() > 0.01
                || (p0.y - p1.y).abs() > 0.01
                || (p0.z - p1.z).abs() > 0.01,
            "Variant 1 should differ from variant 0"
        );
        assert!(
            (p0.x - p2.x).abs() > 0.01
                || (p0.y - p2.y).abs() > 0.01
                || (p0.z - p2.z).abs() > 0.01,
            "Variant 2 should differ from variant 0"
        );
    }

    #[test]
    fn proportions_are_reasonable() {
        // All proportions should be between 0.5 and 2.0 for visual sanity
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
        for zone in &zones {
            for level in 1..=5 {
                for (i, p) in proportions_for(*zone, level).iter().enumerate() {
                    assert!(
                        p.x >= 0.5 && p.x <= 2.0,
                        "{zone:?} L{level} v{i} x={} out of range",
                        p.x
                    );
                    assert!(
                        p.y >= 0.5 && p.y <= 2.0,
                        "{zone:?} L{level} v{i} y={} out of range",
                        p.y
                    );
                    assert!(
                        p.z >= 0.5 && p.z <= 2.0,
                        "{zone:?} L{level} v{i} z={} out of range",
                        p.z
                    );
                }
            }
        }
    }

    #[test]
    fn none_zone_returns_default() {
        let props = proportions_for(ZoneType::None, 1);
        assert_eq!(props.len(), 3);
    }
}
