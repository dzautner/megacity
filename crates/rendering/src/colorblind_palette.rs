//! Colorblind-adapted color palettes.
//!
//! Provides functions that return adapted colors based on the active
//! `ColorblindMode`. All color adaptations follow established guidelines:
//!
//! - **Protanopia / Deuteranopia** (red-green CVD): Replace red-green scales
//!   with blue-orange/yellow scales. Use distinct luminance to maintain
//!   discriminability.
//! - **Tritanopia** (blue-yellow CVD): Replace blue-yellow scales with
//!   red-cyan/magenta scales. Avoid blue-yellow pairs.
//!
//! The ramps themselves (viridis, inferno, cividis) are already largely
//! colorblind-safe for protan/deutan. The main adjustments are for:
//! 1. Traffic LOS colors (red-green -> adapted)
//! 2. Zone ground colors (subtle tint adjustments)
//! 3. Status icon colors (distinct per mode)

use bevy::prelude::*;

use simulation::colorblind::ColorblindMode;
use simulation::traffic_los::LosGrade;

// ---------------------------------------------------------------------------
// Traffic LOS colors (the biggest red-green offender)
// ---------------------------------------------------------------------------

/// Return the traffic LOS color adapted for the given colorblind mode.
///
/// Normal: green -> yellow -> orange -> red (classic traffic light)
/// Protan/Deutan: blue -> cyan -> yellow -> orange (avoids red-green)
/// Tritan: dark teal -> light teal -> peach -> magenta (avoids blue-yellow)
pub fn los_color(grade: LosGrade, mode: ColorblindMode) -> Color {
    match mode {
        ColorblindMode::Normal => match grade {
            LosGrade::A => Color::srgb(0.20, 0.72, 0.20), // green
            LosGrade::B => Color::srgb(0.55, 0.78, 0.22), // yellow-green
            LosGrade::C => Color::srgb(0.90, 0.82, 0.15), // yellow
            LosGrade::D => Color::srgb(0.95, 0.55, 0.10), // orange
            LosGrade::E => Color::srgb(0.90, 0.25, 0.10), // red-orange
            LosGrade::F => Color::srgb(0.75, 0.08, 0.08), // deep red
        },
        ColorblindMode::Protanopia | ColorblindMode::Deuteranopia => match grade {
            // Blue-to-orange ramp: discriminable via both hue and luminance
            LosGrade::A => Color::srgb(0.12, 0.40, 0.80), // blue
            LosGrade::B => Color::srgb(0.20, 0.60, 0.78), // cyan-blue
            LosGrade::C => Color::srgb(0.55, 0.75, 0.55), // muted teal
            LosGrade::D => Color::srgb(0.85, 0.75, 0.25), // yellow
            LosGrade::E => Color::srgb(0.92, 0.55, 0.12), // orange
            LosGrade::F => Color::srgb(0.85, 0.35, 0.05), // dark orange
        },
        ColorblindMode::Tritanopia => match grade {
            // Teal-to-magenta ramp: avoids blue-yellow confusion
            LosGrade::A => Color::srgb(0.10, 0.55, 0.50), // dark teal
            LosGrade::B => Color::srgb(0.25, 0.65, 0.55), // teal
            LosGrade::C => Color::srgb(0.55, 0.70, 0.60), // light teal
            LosGrade::D => Color::srgb(0.80, 0.60, 0.55), // peach
            LosGrade::E => Color::srgb(0.85, 0.40, 0.50), // rose
            LosGrade::F => Color::srgb(0.75, 0.15, 0.45), // magenta
        },
    }
}

// ---------------------------------------------------------------------------
// Zone ground colors
// ---------------------------------------------------------------------------

/// Zone type identifier for palette lookup (avoids importing grid types into rendering).
#[derive(Debug, Clone, Copy)]
pub enum ZoneColorKind {
    ResidentialLow,
    ResidentialMedium,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
    MixedUse,
}

/// Return zone ground color adapted for the given colorblind mode.
///
/// Normal mode uses the existing subtle earth tones. Colorblind modes adjust
/// hues slightly to increase discriminability while keeping the overall
/// aesthetic. The differences are subtle since zone colors are already
/// fairly muted.
pub fn zone_color(kind: ZoneColorKind, mode: ColorblindMode) -> (f32, f32, f32) {
    match mode {
        ColorblindMode::Normal => match kind {
            ZoneColorKind::ResidentialLow => (0.52, 0.56, 0.46),
            ZoneColorKind::ResidentialMedium => (0.57, 0.58, 0.51),
            ZoneColorKind::ResidentialHigh => (0.62, 0.60, 0.57),
            ZoneColorKind::CommercialLow => (0.58, 0.57, 0.54),
            ZoneColorKind::CommercialHigh => (0.60, 0.58, 0.55),
            ZoneColorKind::Industrial => (0.55, 0.52, 0.47),
            ZoneColorKind::Office => (0.64, 0.62, 0.58),
            ZoneColorKind::MixedUse => (0.60, 0.58, 0.52),
        },
        ColorblindMode::Protanopia | ColorblindMode::Deuteranopia => {
            // Shift residential toward blue tints, commercial toward warm yellow,
            // industrial toward brown. Increases contrast between zone types
            // for red-green CVD viewers.
            match kind {
                ZoneColorKind::ResidentialLow => (0.48, 0.53, 0.55),
                ZoneColorKind::ResidentialMedium => (0.52, 0.55, 0.58),
                ZoneColorKind::ResidentialHigh => (0.56, 0.57, 0.62),
                ZoneColorKind::CommercialLow => (0.60, 0.56, 0.48),
                ZoneColorKind::CommercialHigh => (0.63, 0.58, 0.46),
                ZoneColorKind::Industrial => (0.55, 0.50, 0.44),
                ZoneColorKind::Office => (0.62, 0.60, 0.62),
                ZoneColorKind::MixedUse => (0.58, 0.56, 0.52),
            }
        }
        ColorblindMode::Tritanopia => {
            // Shift residential toward warm tints, commercial toward cooler tones.
            // Avoids blue-yellow confusion.
            match kind {
                ZoneColorKind::ResidentialLow => (0.54, 0.52, 0.48),
                ZoneColorKind::ResidentialMedium => (0.58, 0.55, 0.50),
                ZoneColorKind::ResidentialHigh => (0.62, 0.58, 0.55),
                ZoneColorKind::CommercialLow => (0.52, 0.58, 0.56),
                ZoneColorKind::CommercialHigh => (0.50, 0.60, 0.58),
                ZoneColorKind::Industrial => (0.56, 0.50, 0.48),
                ZoneColorKind::Office => (0.60, 0.62, 0.60),
                ZoneColorKind::MixedUse => (0.56, 0.58, 0.54),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Status icon colors
// ---------------------------------------------------------------------------

/// Utility status icon kind.
#[derive(Debug, Clone, Copy)]
pub enum UtilityIconKind {
    NoPower,
    NoWater,
    NoPowerNoWater,
}

/// Return utility status icon color adapted for the given colorblind mode.
pub fn utility_icon_color(kind: UtilityIconKind, mode: ColorblindMode) -> Color {
    match mode {
        ColorblindMode::Normal => match kind {
            UtilityIconKind::NoPower => Color::srgb(1.0, 0.15, 0.15), // red
            UtilityIconKind::NoWater => Color::srgb(0.2, 0.45, 1.0),  // blue
            UtilityIconKind::NoPowerNoWater => Color::srgb(1.0, 0.85, 0.1), // yellow
        },
        ColorblindMode::Protanopia | ColorblindMode::Deuteranopia => match kind {
            // Replace red with orange (more visible), keep blue, use bright yellow
            UtilityIconKind::NoPower => Color::srgb(0.95, 0.55, 0.05), // orange
            UtilityIconKind::NoWater => Color::srgb(0.15, 0.40, 0.90), // blue
            UtilityIconKind::NoPowerNoWater => Color::srgb(0.95, 0.85, 0.15), // yellow
        },
        ColorblindMode::Tritanopia => match kind {
            // Replace blue with teal (more visible), keep red, use peach
            UtilityIconKind::NoPower => Color::srgb(0.95, 0.20, 0.20), // red
            UtilityIconKind::NoWater => Color::srgb(0.10, 0.65, 0.55), // teal
            UtilityIconKind::NoPowerNoWater => Color::srgb(0.90, 0.55, 0.35), // peach
        },
    }
}

/// Enhanced building status icon kind.
#[derive(Debug, Clone, Copy)]
pub enum EnhancedIconKindCb {
    Fire,
    UnderConstruction,
    HighCrime,
    CapacityFull,
    Abandoned,
}

/// Return enhanced status icon color adapted for the given colorblind mode.
pub fn enhanced_icon_color(kind: EnhancedIconKindCb, mode: ColorblindMode) -> Color {
    match mode {
        ColorblindMode::Normal => match kind {
            EnhancedIconKindCb::Fire => Color::srgb(1.0, 0.35, 0.0),
            EnhancedIconKindCb::UnderConstruction => Color::srgb(1.0, 0.75, 0.0),
            EnhancedIconKindCb::HighCrime => Color::srgb(0.6, 0.0, 0.1),
            EnhancedIconKindCb::CapacityFull => Color::srgb(0.0, 0.8, 0.6),
            EnhancedIconKindCb::Abandoned => Color::srgb(0.5, 0.5, 0.5),
        },
        ColorblindMode::Protanopia | ColorblindMode::Deuteranopia => match kind {
            // Shift fire to bright orange, crime to dark purple, capacity to blue
            EnhancedIconKindCb::Fire => Color::srgb(0.95, 0.50, 0.05),
            EnhancedIconKindCb::UnderConstruction => Color::srgb(0.90, 0.75, 0.15),
            EnhancedIconKindCb::HighCrime => Color::srgb(0.45, 0.05, 0.40),
            EnhancedIconKindCb::CapacityFull => Color::srgb(0.15, 0.55, 0.85),
            EnhancedIconKindCb::Abandoned => Color::srgb(0.5, 0.5, 0.5),
        },
        ColorblindMode::Tritanopia => match kind {
            // Shift capacity to red-pink, crime to dark maroon, construction to peach
            EnhancedIconKindCb::Fire => Color::srgb(1.0, 0.35, 0.0),
            EnhancedIconKindCb::UnderConstruction => Color::srgb(0.85, 0.60, 0.35),
            EnhancedIconKindCb::HighCrime => Color::srgb(0.55, 0.05, 0.15),
            EnhancedIconKindCb::CapacityFull => Color::srgb(0.75, 0.20, 0.45),
            EnhancedIconKindCb::Abandoned => Color::srgb(0.5, 0.5, 0.5),
        },
    }
}

// ---------------------------------------------------------------------------
// Binary overlay palette adjustments
// ---------------------------------------------------------------------------

use crate::color_ramps::BinaryPalette;

/// Return adapted binary palette colors for power overlay.
pub fn power_palette(mode: ColorblindMode) -> BinaryPalette {
    match mode {
        ColorblindMode::Normal => BinaryPalette {
            on: Color::srgba(0.80, 0.78, 0.20, 0.45),
            off: Color::srgba(0.60, 0.15, 0.15, 0.55),
        },
        ColorblindMode::Protanopia | ColorblindMode::Deuteranopia => BinaryPalette {
            on: Color::srgba(0.80, 0.78, 0.20, 0.45),
            off: Color::srgba(0.85, 0.45, 0.05, 0.55), // orange instead of red
        },
        ColorblindMode::Tritanopia => BinaryPalette {
            on: Color::srgba(0.75, 0.55, 0.30, 0.45), // warm peach instead of yellow
            off: Color::srgba(0.60, 0.15, 0.15, 0.55),
        },
    }
}

/// Return adapted binary palette colors for water overlay.
pub fn water_palette(mode: ColorblindMode) -> BinaryPalette {
    match mode {
        ColorblindMode::Normal => BinaryPalette {
            on: Color::srgba(0.18, 0.50, 0.82, 0.45),
            off: Color::srgba(0.60, 0.15, 0.15, 0.55),
        },
        ColorblindMode::Protanopia | ColorblindMode::Deuteranopia => BinaryPalette {
            on: Color::srgba(0.18, 0.50, 0.82, 0.45),
            off: Color::srgba(0.85, 0.45, 0.05, 0.55), // orange instead of red
        },
        ColorblindMode::Tritanopia => BinaryPalette {
            on: Color::srgba(0.15, 0.60, 0.50, 0.45), // teal instead of blue
            off: Color::srgba(0.60, 0.15, 0.15, 0.55),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_colors_distinct_per_grade_all_modes() {
        for mode in ColorblindMode::ALL {
            let grades = [
                LosGrade::A,
                LosGrade::B,
                LosGrade::C,
                LosGrade::D,
                LosGrade::E,
                LosGrade::F,
            ];
            for i in 0..grades.len() {
                for j in (i + 1)..grades.len() {
                    let ci = los_color(grades[i], mode).to_srgba();
                    let cj = los_color(grades[j], mode).to_srgba();
                    let diff = (ci.red - cj.red).abs()
                        + (ci.green - cj.green).abs()
                        + (ci.blue - cj.blue).abs();
                    assert!(
                        diff > 0.05,
                        "LOS {:?} and {:?} should have distinct colors in {:?} mode",
                        grades[i],
                        grades[j],
                        mode
                    );
                }
            }
        }
    }

    #[test]
    fn zone_colors_valid_range_all_modes() {
        let kinds = [
            ZoneColorKind::ResidentialLow,
            ZoneColorKind::ResidentialMedium,
            ZoneColorKind::ResidentialHigh,
            ZoneColorKind::CommercialLow,
            ZoneColorKind::CommercialHigh,
            ZoneColorKind::Industrial,
            ZoneColorKind::Office,
            ZoneColorKind::MixedUse,
        ];
        for mode in ColorblindMode::ALL {
            for kind in &kinds {
                let (r, g, b) = zone_color(*kind, mode);
                assert!(
                    (0.0..=1.0).contains(&r)
                        && (0.0..=1.0).contains(&g)
                        && (0.0..=1.0).contains(&b),
                    "Zone color out of range for {:?} in {:?} mode: ({}, {}, {})",
                    kind,
                    mode,
                    r,
                    g,
                    b
                );
            }
        }
    }

    #[test]
    fn utility_icon_colors_distinct_per_kind_all_modes() {
        let kinds = [
            UtilityIconKind::NoPower,
            UtilityIconKind::NoWater,
            UtilityIconKind::NoPowerNoWater,
        ];
        for mode in ColorblindMode::ALL {
            for i in 0..kinds.len() {
                for j in (i + 1)..kinds.len() {
                    let ci = utility_icon_color(kinds[i], mode).to_srgba();
                    let cj = utility_icon_color(kinds[j], mode).to_srgba();
                    let diff = (ci.red - cj.red).abs()
                        + (ci.green - cj.green).abs()
                        + (ci.blue - cj.blue).abs();
                    assert!(
                        diff > 0.1,
                        "Utility icons {:?} and {:?} should be distinct in {:?} mode",
                        kinds[i],
                        kinds[j],
                        mode
                    );
                }
            }
        }
    }

    #[test]
    fn enhanced_icon_colors_distinct_per_kind_all_modes() {
        let kinds = [
            EnhancedIconKindCb::Fire,
            EnhancedIconKindCb::UnderConstruction,
            EnhancedIconKindCb::HighCrime,
            EnhancedIconKindCb::CapacityFull,
            EnhancedIconKindCb::Abandoned,
        ];
        for mode in ColorblindMode::ALL {
            for i in 0..kinds.len() {
                for j in (i + 1)..kinds.len() {
                    let ci = enhanced_icon_color(kinds[i], mode).to_srgba();
                    let cj = enhanced_icon_color(kinds[j], mode).to_srgba();
                    let diff = (ci.red - cj.red).abs()
                        + (ci.green - cj.green).abs()
                        + (ci.blue - cj.blue).abs();
                    assert!(
                        diff > 0.05,
                        "Enhanced icons {:?} and {:?} should be distinct in {:?} mode (diff={:.3})",
                        kinds[i],
                        kinds[j],
                        mode,
                        diff
                    );
                }
            }
        }
    }

    #[test]
    fn power_palette_on_off_distinct_all_modes() {
        for mode in ColorblindMode::ALL {
            let p = power_palette(mode);
            let on = p.on.to_srgba();
            let off = p.off.to_srgba();
            let diff = (on.red - off.red).abs()
                + (on.green - off.green).abs()
                + (on.blue - off.blue).abs();
            assert!(
                diff > 0.2,
                "Power palette on/off should be distinct in {:?} mode",
                mode
            );
        }
    }

    #[test]
    fn water_palette_on_off_distinct_all_modes() {
        for mode in ColorblindMode::ALL {
            let p = water_palette(mode);
            let on = p.on.to_srgba();
            let off = p.off.to_srgba();
            let diff = (on.red - off.red).abs()
                + (on.green - off.green).abs()
                + (on.blue - off.blue).abs();
            assert!(
                diff > 0.2,
                "Water palette on/off should be distinct in {:?} mode",
                mode
            );
        }
    }

    #[test]
    fn protan_deutan_los_avoids_red_green() {
        // For protan/deutan modes, LOS A should NOT be green and LOS F should NOT be red
        for mode in [ColorblindMode::Protanopia, ColorblindMode::Deuteranopia] {
            let a = los_color(LosGrade::A, mode).to_srgba();
            // LOS A should be blue-ish (blue > green and blue > red)
            assert!(
                a.blue > a.red,
                "LOS A in {:?} mode should be blue, not red/green: r={} g={} b={}",
                mode,
                a.red,
                a.green,
                a.blue
            );
            let f = los_color(LosGrade::F, mode).to_srgba();
            // LOS F should be orange-ish (red > blue, not pure red)
            assert!(
                f.red > f.blue,
                "LOS F in {:?} mode should be warm, not blue: r={} b={}",
                mode,
                f.red,
                f.blue
            );
        }
    }

    #[test]
    fn tritan_los_avoids_blue_yellow() {
        let a = los_color(LosGrade::A, ColorblindMode::Tritanopia).to_srgba();
        // LOS A should be teal-ish
        assert!(
            a.green > a.red && a.green > 0.4,
            "LOS A in tritanopia should be teal: r={} g={} b={}",
            a.red,
            a.green,
            a.blue
        );
        let f = los_color(LosGrade::F, ColorblindMode::Tritanopia).to_srgba();
        // LOS F should be magenta-ish
        assert!(
            f.red > f.green,
            "LOS F in tritanopia should be magenta: r={} g={}",
            f.red,
            f.green
        );
    }
}
