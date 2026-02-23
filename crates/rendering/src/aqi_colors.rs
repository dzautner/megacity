//! EPA AQI (Air Quality Index) color mapping for the pollution overlay.
//!
//! Maps pollution concentration values (u8 0–255 or f32) to the standard
//! 6-tier AQI color scheme used by the U.S. EPA. Each tier has:
//! - A color band matching the EPA standard
//! - A tier name (Good, Moderate, etc.)
//! - A health advisory message for tooltip display
//!
//! The AQI value is derived from the raw pollution concentration by scaling
//! the u8 range (0–255) linearly to AQI 0–500.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// AQI Tier definitions
// ---------------------------------------------------------------------------

/// The six standard EPA AQI tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AqiTier {
    /// AQI 0–50: Air quality is satisfactory.
    Good,
    /// AQI 51–100: Acceptable; moderate health concern for sensitive groups.
    Moderate,
    /// AQI 101–150: Unhealthy for sensitive groups.
    UnhealthyForSensitive,
    /// AQI 151–200: Everyone may begin to experience health effects.
    Unhealthy,
    /// AQI 201–300: Health alert; serious health effects for everyone.
    VeryUnhealthy,
    /// AQI 301+: Health emergency; entire population affected.
    Hazardous,
}

impl AqiTier {
    /// Human-readable tier name for display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::Moderate => "Moderate",
            Self::UnhealthyForSensitive => "Unhealthy for Sensitive Groups",
            Self::Unhealthy => "Unhealthy",
            Self::VeryUnhealthy => "Very Unhealthy",
            Self::Hazardous => "Hazardous",
        }
    }

    /// Short label for compact legend display.
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::Moderate => "Moderate",
            Self::UnhealthyForSensitive => "Sensitive",
            Self::Unhealthy => "Unhealthy",
            Self::VeryUnhealthy => "Very Unhealthy",
            Self::Hazardous => "Hazardous",
        }
    }

    /// Health advisory text for tooltip display.
    pub fn health_advisory(self) -> &'static str {
        match self {
            Self::Good => "Air quality is satisfactory with little or no risk.",
            Self::Moderate => {
                "Acceptable quality. Sensitive individuals may experience minor effects."
            }
            Self::UnhealthyForSensitive => {
                "Sensitive groups (children, elderly) may experience health effects."
            }
            Self::Unhealthy => "Everyone may begin to experience health effects.",
            Self::VeryUnhealthy => "Health alert: serious effects for the entire population.",
            Self::Hazardous => "Health emergency: the entire population is affected.",
        }
    }
}

/// All tiers in order from best to worst, for iteration.
pub const ALL_TIERS: [AqiTier; 6] = [
    AqiTier::Good,
    AqiTier::Moderate,
    AqiTier::UnhealthyForSensitive,
    AqiTier::Unhealthy,
    AqiTier::VeryUnhealthy,
    AqiTier::Hazardous,
];

// ---------------------------------------------------------------------------
// AQI color definitions (EPA standard sRGB)
// ---------------------------------------------------------------------------

/// EPA standard AQI colors in sRGB.
const AQI_GREEN: [f32; 3] = [0.0, 0.58, 0.09]; // #009416 — Good
const AQI_YELLOW: [f32; 3] = [1.0, 0.87, 0.0]; // #FFDE00 — Moderate
const AQI_ORANGE: [f32; 3] = [1.0, 0.49, 0.0]; // #FF7D00 — USG
const AQI_RED: [f32; 3] = [1.0, 0.0, 0.0]; // #FF0000 — Unhealthy
const AQI_PURPLE: [f32; 3] = [0.60, 0.20, 0.60]; // #993399 — Very Unhealthy
const AQI_MAROON: [f32; 3] = [0.50, 0.0, 0.13]; // #800021 — Hazardous

// ---------------------------------------------------------------------------
// AQI value computation
// ---------------------------------------------------------------------------

/// Convert a raw pollution concentration (u8, 0–255) to an AQI value (0–500).
///
/// The game's pollution grid stores u8 values where 0 = clean and 255 = maximum.
/// We scale linearly: AQI = concentration * 500 / 255 ≈ concentration * 1.96.
pub fn concentration_to_aqi(concentration: u8) -> u16 {
    // Use u32 intermediate to avoid overflow: 255 * 500 = 127500
    ((concentration as u32 * 500) / 255) as u16
}

/// Convert a floating-point concentration (0.0–255.0) to an AQI value.
pub fn concentration_f32_to_aqi(concentration: f32) -> u16 {
    let clamped = concentration.clamp(0.0, 255.0);
    (clamped * 500.0 / 255.0) as u16
}

/// Classify an AQI value into its tier.
pub fn aqi_to_tier(aqi: u16) -> AqiTier {
    match aqi {
        0..=50 => AqiTier::Good,
        51..=100 => AqiTier::Moderate,
        101..=150 => AqiTier::UnhealthyForSensitive,
        151..=200 => AqiTier::Unhealthy,
        201..=300 => AqiTier::VeryUnhealthy,
        _ => AqiTier::Hazardous,
    }
}

/// Get the AQI tier for a raw u8 pollution concentration.
pub fn concentration_to_tier(concentration: u8) -> AqiTier {
    aqi_to_tier(concentration_to_aqi(concentration))
}

// ---------------------------------------------------------------------------
// Color mapping
// ---------------------------------------------------------------------------

/// Return the EPA standard color for an AQI tier as a Bevy `Color`.
pub fn tier_color(tier: AqiTier) -> Color {
    let c = match tier {
        AqiTier::Good => AQI_GREEN,
        AqiTier::Moderate => AQI_YELLOW,
        AqiTier::UnhealthyForSensitive => AQI_ORANGE,
        AqiTier::Unhealthy => AQI_RED,
        AqiTier::VeryUnhealthy => AQI_PURPLE,
        AqiTier::Hazardous => AQI_MAROON,
    };
    Color::srgb(c[0], c[1], c[2])
}

/// Return the EPA standard color for an AQI tier as sRGB `[f32; 3]`.
pub fn tier_color_rgb(tier: AqiTier) -> [f32; 3] {
    match tier {
        AqiTier::Good => AQI_GREEN,
        AqiTier::Moderate => AQI_YELLOW,
        AqiTier::UnhealthyForSensitive => AQI_ORANGE,
        AqiTier::Unhealthy => AQI_RED,
        AqiTier::VeryUnhealthy => AQI_PURPLE,
        AqiTier::Hazardous => AQI_MAROON,
    }
}

/// Map a u8 pollution concentration to an AQI overlay color.
///
/// Within each tier, the color smoothly interpolates from the lower tier's
/// color to the current tier's color, providing a continuous gradient that
/// still respects the 6-tier AQI banding.
pub fn aqi_overlay_color(concentration: u8) -> Color {
    let aqi = concentration_to_aqi(concentration);
    aqi_value_to_color(aqi)
}

/// Map a f32 pollution concentration (0.0–255.0) to an AQI overlay color.
pub fn aqi_overlay_color_f32(concentration: f32) -> Color {
    let aqi = concentration_f32_to_aqi(concentration);
    aqi_value_to_color(aqi)
}

/// Map an AQI value (0–500+) to a smoothly interpolated color within bands.
fn aqi_value_to_color(aqi: u16) -> Color {
    // Define the tier boundaries and their colors
    let bands: &[(u16, [f32; 3])] = &[
        (0, AQI_GREEN),
        (50, AQI_GREEN),
        (51, AQI_YELLOW),
        (100, AQI_YELLOW),
        (101, AQI_ORANGE),
        (150, AQI_ORANGE),
        (151, AQI_RED),
        (200, AQI_RED),
        (201, AQI_PURPLE),
        (300, AQI_PURPLE),
        (301, AQI_MAROON),
        (500, AQI_MAROON),
    ];

    // Find the interpolation range
    let aqi_clamped = aqi.min(500);
    let mut lo_idx = 0;
    for (i, &(threshold, _)) in bands.iter().enumerate() {
        if aqi_clamped >= threshold {
            lo_idx = i;
        }
    }
    let hi_idx = (lo_idx + 1).min(bands.len() - 1);

    let (lo_aqi, lo_color) = bands[lo_idx];
    let (hi_aqi, hi_color) = bands[hi_idx];

    let t = if hi_aqi > lo_aqi {
        (aqi_clamped as f32 - lo_aqi as f32) / (hi_aqi as f32 - lo_aqi as f32)
    } else {
        0.0
    };
    let t = t.clamp(0.0, 1.0);

    Color::srgb(
        lo_color[0] + (hi_color[0] - lo_color[0]) * t,
        lo_color[1] + (hi_color[1] - lo_color[1]) * t,
        lo_color[2] + (hi_color[2] - lo_color[2]) * t,
    )
}

/// Return the AQI range string for a tier (e.g., "0–50", "51–100").
pub fn tier_aqi_range(tier: AqiTier) -> &'static str {
    match tier {
        AqiTier::Good => "0-50",
        AqiTier::Moderate => "51-100",
        AqiTier::UnhealthyForSensitive => "101-150",
        AqiTier::Unhealthy => "151-200",
        AqiTier::VeryUnhealthy => "201-300",
        AqiTier::Hazardous => "301-500",
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concentration_zero_maps_to_aqi_zero() {
        assert_eq!(concentration_to_aqi(0), 0);
    }

    #[test]
    fn concentration_max_maps_to_aqi_500() {
        assert_eq!(concentration_to_aqi(255), 500);
    }

    #[test]
    fn aqi_tier_boundaries() {
        assert_eq!(aqi_to_tier(0), AqiTier::Good);
        assert_eq!(aqi_to_tier(50), AqiTier::Good);
        assert_eq!(aqi_to_tier(51), AqiTier::Moderate);
        assert_eq!(aqi_to_tier(100), AqiTier::Moderate);
        assert_eq!(aqi_to_tier(101), AqiTier::UnhealthyForSensitive);
        assert_eq!(aqi_to_tier(150), AqiTier::UnhealthyForSensitive);
        assert_eq!(aqi_to_tier(151), AqiTier::Unhealthy);
        assert_eq!(aqi_to_tier(200), AqiTier::Unhealthy);
        assert_eq!(aqi_to_tier(201), AqiTier::VeryUnhealthy);
        assert_eq!(aqi_to_tier(300), AqiTier::VeryUnhealthy);
        assert_eq!(aqi_to_tier(301), AqiTier::Hazardous);
        assert_eq!(aqi_to_tier(500), AqiTier::Hazardous);
    }

    #[test]
    fn concentration_to_tier_low_is_good() {
        // concentration 0–25 should be "Good" (AQI 0–49)
        assert_eq!(concentration_to_tier(0), AqiTier::Good);
        assert_eq!(concentration_to_tier(10), AqiTier::Good);
        assert_eq!(concentration_to_tier(25), AqiTier::Good);
    }

    #[test]
    fn concentration_to_tier_max_is_hazardous() {
        assert_eq!(concentration_to_tier(255), AqiTier::Hazardous);
    }

    #[test]
    fn tier_labels_are_non_empty() {
        for tier in ALL_TIERS {
            assert!(!tier.label().is_empty());
            assert!(!tier.short_label().is_empty());
            assert!(!tier.health_advisory().is_empty());
        }
    }

    #[test]
    fn aqi_color_zero_is_green() {
        let c = aqi_overlay_color(0);
        let s = c.to_srgba();
        // Should be green (high green channel, low red)
        assert!(
            s.green > s.red,
            "AQI 0 should be green, got r={} g={}",
            s.red,
            s.green
        );
    }

    #[test]
    fn aqi_color_max_is_maroon() {
        let c = aqi_overlay_color(255);
        let s = c.to_srgba();
        // Maroon: red > green, dark
        assert!(
            s.red > s.green,
            "AQI 500 should be maroon, got r={} g={}",
            s.red,
            s.green
        );
        assert!(s.red > 0.3, "Maroon should have red component > 0.3");
    }

    #[test]
    fn aqi_color_mid_is_orange_red() {
        // Concentration ~102 -> AQI ~200 -> Unhealthy (red)
        let c = aqi_overlay_color(102);
        let s = c.to_srgba();
        assert!(
            s.red > 0.5,
            "Mid-range AQI should have strong red, got r={}",
            s.red
        );
    }

    #[test]
    fn f32_conversion_matches_u8() {
        for val in [0u8, 50, 100, 150, 200, 255] {
            let aqi_u8 = concentration_to_aqi(val);
            let aqi_f32 = concentration_f32_to_aqi(val as f32);
            assert_eq!(aqi_u8, aqi_f32, "f32 and u8 should match for val={val}");
        }
    }

    #[test]
    fn tier_aqi_ranges_are_non_empty() {
        for tier in ALL_TIERS {
            assert!(!tier_aqi_range(tier).is_empty());
        }
    }

    #[test]
    fn all_six_tiers_have_distinct_colors() {
        let colors: Vec<[f32; 3]> = ALL_TIERS.iter().map(|t| tier_color_rgb(*t)).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                let diff = (colors[i][0] - colors[j][0]).abs()
                    + (colors[i][1] - colors[j][1]).abs()
                    + (colors[i][2] - colors[j][2]).abs();
                assert!(
                    diff > 0.1,
                    "Tiers {:?} and {:?} should have distinct colors",
                    ALL_TIERS[i],
                    ALL_TIERS[j]
                );
            }
        }
    }
}
