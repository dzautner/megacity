//! Perceptually uniform color ramps for data overlays.
//!
//! Provides colorblind-friendly continuous ramps (viridis, inferno) for
//! scalar data (pollution, land value, traffic, etc.) and categorical
//! palettes for discrete overlays (zones, service coverage).
//!
//! All ramps are defined in sRGB space as lookup tables sampled from the
//! matplotlib originals and interpolated linearly for intermediate values.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Continuous color ramps
// ---------------------------------------------------------------------------

/// A continuous color ramp defined by evenly-spaced sRGB control points.
/// Interpolates linearly in sRGB space for a given `t` in `[0, 1]`.
pub struct ColorRamp {
    /// Control points as `[r, g, b]` in sRGB, evenly spaced from t=0..1.
    points: &'static [[f32; 3]],
}

impl ColorRamp {
    /// Sample the ramp at parameter `t` (clamped to `[0, 1]`).
    pub fn sample(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let n = self.points.len();
        if n == 0 {
            return Color::BLACK;
        }
        if n == 1 {
            let p = self.points[0];
            return Color::srgb(p[0], p[1], p[2]);
        }
        let max_idx = (n - 1) as f32;
        let scaled = t * max_idx;
        let lo = (scaled as usize).min(n - 2);
        let hi = lo + 1;
        let frac = scaled - lo as f32;
        let a = self.points[lo];
        let b = self.points[hi];
        Color::srgb(
            a[0] + (b[0] - a[0]) * frac,
            a[1] + (b[1] - a[1]) * frac,
            a[2] + (b[2] - a[2]) * frac,
        )
    }

    /// Sample the ramp and return as an `[f32; 4]` RGBA array (alpha = 1).
    pub fn sample_rgba(&self, t: f32) -> [f32; 4] {
        let c = self.sample(t);
        let s = c.to_srgba();
        [s.red, s.green, s.blue, 1.0]
    }
}

// ---------------------------------------------------------------------------
// Viridis ramp (32 control points sampled from matplotlib viridis)
// Perceptually uniform, colorblind-safe (deuteranopia + protanopia friendly).
// Good default for most continuous overlays.
// ---------------------------------------------------------------------------
pub static VIRIDIS: ColorRamp = ColorRamp {
    points: &[
        [0.267, 0.004, 0.329], // 0   - dark purple
        [0.282, 0.040, 0.363],
        [0.293, 0.075, 0.393],
        [0.298, 0.110, 0.420],
        [0.297, 0.147, 0.443],
        [0.290, 0.184, 0.460],
        [0.278, 0.220, 0.473],
        [0.263, 0.256, 0.482], // ~0.22
        [0.246, 0.290, 0.487],
        [0.228, 0.322, 0.489],
        [0.210, 0.354, 0.488],
        [0.192, 0.384, 0.484],
        [0.174, 0.413, 0.478],
        [0.156, 0.441, 0.470],
        [0.140, 0.468, 0.460],
        [0.127, 0.494, 0.448], // ~0.48 - teal
        [0.120, 0.519, 0.433],
        [0.122, 0.543, 0.415],
        [0.137, 0.566, 0.393],
        [0.163, 0.588, 0.368],
        [0.200, 0.609, 0.340],
        [0.246, 0.629, 0.308],
        [0.301, 0.647, 0.274],
        [0.363, 0.664, 0.237], // ~0.74
        [0.432, 0.679, 0.199],
        [0.505, 0.691, 0.162],
        [0.580, 0.700, 0.128],
        [0.655, 0.707, 0.101],
        [0.731, 0.710, 0.092],
        [0.804, 0.710, 0.105],
        [0.872, 0.706, 0.150],
        [0.993, 0.906, 0.144], // 1   - bright yellow
    ],
};

// ---------------------------------------------------------------------------
// Inferno ramp (32 control points sampled from matplotlib inferno)
// Perceptually uniform, high-contrast dark-to-bright.
// Best for "heat" data: pollution, congestion, noise.
// ---------------------------------------------------------------------------
pub static INFERNO: ColorRamp = ColorRamp {
    points: &[
        [0.001, 0.000, 0.014], // 0   - near-black
        [0.015, 0.011, 0.068],
        [0.044, 0.027, 0.130],
        [0.083, 0.040, 0.190],
        [0.125, 0.044, 0.247],
        [0.168, 0.040, 0.298],
        [0.212, 0.032, 0.339],
        [0.258, 0.027, 0.370], // ~0.22
        [0.306, 0.030, 0.389],
        [0.352, 0.040, 0.399],
        [0.398, 0.057, 0.400],
        [0.442, 0.077, 0.393],
        [0.486, 0.100, 0.378],
        [0.528, 0.126, 0.356],
        [0.569, 0.154, 0.329],
        [0.608, 0.185, 0.298], // ~0.48
        [0.646, 0.217, 0.265],
        [0.681, 0.252, 0.231],
        [0.715, 0.290, 0.197],
        [0.746, 0.330, 0.165],
        [0.775, 0.373, 0.135],
        [0.801, 0.419, 0.108],
        [0.824, 0.467, 0.085],
        [0.844, 0.518, 0.068], // ~0.74
        [0.860, 0.571, 0.058],
        [0.873, 0.626, 0.059],
        [0.882, 0.682, 0.076],
        [0.887, 0.739, 0.112],
        [0.888, 0.797, 0.170],
        [0.884, 0.854, 0.252],
        [0.882, 0.909, 0.357],
        [0.988, 0.998, 0.645], // 1   - pale yellow
    ],
};

// ---------------------------------------------------------------------------
// Cividis ramp (32 control points)
// Specifically designed for deuteranopia/protanopia color vision deficiency.
// Blue-to-yellow, avoids red-green entirely.
// Used for land value, education, and other "good = high" overlays.
// ---------------------------------------------------------------------------
pub static CIVIDIS: ColorRamp = ColorRamp {
    points: &[
        [0.000, 0.135, 0.305], // 0   - dark navy
        [0.000, 0.152, 0.321],
        [0.000, 0.170, 0.335],
        [0.000, 0.188, 0.347],
        [0.059, 0.206, 0.354],
        [0.107, 0.223, 0.358],
        [0.143, 0.240, 0.360],
        [0.173, 0.258, 0.361], // ~0.22
        [0.199, 0.275, 0.362],
        [0.223, 0.293, 0.362],
        [0.245, 0.311, 0.363],
        [0.266, 0.330, 0.364],
        [0.286, 0.349, 0.365],
        [0.307, 0.368, 0.365],
        [0.328, 0.388, 0.365],
        [0.349, 0.408, 0.364], // ~0.48
        [0.371, 0.428, 0.362],
        [0.394, 0.448, 0.358],
        [0.418, 0.469, 0.352],
        [0.444, 0.489, 0.344],
        [0.471, 0.510, 0.333],
        [0.501, 0.531, 0.319],
        [0.533, 0.552, 0.302],
        [0.567, 0.573, 0.281], // ~0.74
        [0.604, 0.594, 0.257],
        [0.644, 0.616, 0.229],
        [0.686, 0.637, 0.196],
        [0.731, 0.659, 0.158],
        [0.779, 0.681, 0.114],
        [0.829, 0.703, 0.063],
        [0.882, 0.726, 0.000],
        [0.940, 0.749, 0.000], // 1   - warm yellow
    ],
};

// ---------------------------------------------------------------------------
// Groundwater Level ramp (16 control points)
// Dry/depleted (red) -> warning (orange/yellow) -> moderate (light blue) ->
// saturated (deep blue). Designed for groundwater level overlay.
// ---------------------------------------------------------------------------
pub static GROUNDWATER_LEVEL: ColorRamp = ColorRamp {
    points: &[
        [0.75, 0.15, 0.10], // 0   - dry (red)
        [0.80, 0.25, 0.10],
        [0.85, 0.40, 0.12],
        [0.88, 0.55, 0.15], // ~0.20 - warning (orange)
        [0.85, 0.70, 0.20],
        [0.70, 0.78, 0.35], // ~0.33 - transitional (yellow-green)
        [0.45, 0.72, 0.55],
        [0.30, 0.65, 0.70], // ~0.47 - moderate (teal)
        [0.22, 0.55, 0.75],
        [0.18, 0.48, 0.78],
        [0.15, 0.40, 0.80], // ~0.67 - good (medium blue)
        [0.12, 0.35, 0.82],
        [0.10, 0.28, 0.82],
        [0.08, 0.22, 0.80], // ~0.87 - high (dark blue)
        [0.06, 0.16, 0.75],
        [0.04, 0.10, 0.68], // 1   - saturated (deep blue)
    ],
};

// ---------------------------------------------------------------------------
// Groundwater Quality ramp (16 control points)
// Contaminated (brown) -> poor (dark olive) -> moderate (olive-green) ->
// clean (bright green). Designed for groundwater quality overlay.
// ---------------------------------------------------------------------------
pub static GROUNDWATER_QUALITY: ColorRamp = ColorRamp {
    points: &[
        [0.40, 0.25, 0.12], // 0   - contaminated (brown)
        [0.45, 0.28, 0.14],
        [0.48, 0.32, 0.16],
        [0.50, 0.36, 0.18], // ~0.20 - poor (dark brown)
        [0.50, 0.40, 0.20],
        [0.48, 0.45, 0.22], // ~0.33 - poor-moderate (olive)
        [0.44, 0.50, 0.24],
        [0.38, 0.55, 0.26], // ~0.47 - moderate (olive-green)
        [0.32, 0.58, 0.28],
        [0.26, 0.60, 0.30],
        [0.20, 0.62, 0.32], // ~0.67 - good (green)
        [0.16, 0.65, 0.34],
        [0.12, 0.68, 0.36],
        [0.10, 0.72, 0.38], // ~0.87 - very good (bright green)
        [0.08, 0.76, 0.40],
        [0.06, 0.80, 0.42], // 1   - clean (bright green)
    ],
};

// ---------------------------------------------------------------------------
// Categorical / boolean palettes
// ---------------------------------------------------------------------------

/// Binary overlay colors for boolean states (e.g. power on/off, water connected/not).
pub struct BinaryPalette {
    /// Color when the value is true / active.
    pub on: Color,
    /// Color when the value is false / inactive.
    pub off: Color,
}

/// Power overlay: yellow = powered, red-brown = unpowered.
/// Yellow chosen over green to remain distinguishable for red-green CVD.
pub static POWER_PALETTE: BinaryPalette = BinaryPalette {
    on: Color::srgba(0.80, 0.78, 0.20, 0.45),
    off: Color::srgba(0.60, 0.15, 0.15, 0.55),
};

/// Water overlay: blue = connected, red-brown = disconnected.
pub static WATER_PALETTE: BinaryPalette = BinaryPalette {
    on: Color::srgba(0.18, 0.50, 0.82, 0.45),
    off: Color::srgba(0.60, 0.15, 0.15, 0.55),
};

// ---------------------------------------------------------------------------
// Public convenience helpers
// ---------------------------------------------------------------------------

/// Blend a base terrain color toward a tint with given alpha.
pub fn blend_tint(base: Color, tint: Color) -> Color {
    let b = base.to_srgba().to_f32_array();
    let t = tint.to_srgba().to_f32_array();
    let a = t[3];
    Color::srgb(
        b[0] * (1.0 - a) + t[0] * a,
        b[1] * (1.0 - a) + t[1] * a,
        b[2] * (1.0 - a) + t[2] * a,
    )
}

/// Darken a base color by multiplying RGB channels by `factor`.
pub fn darken(base: Color, factor: f32) -> Color {
    let b = base.to_srgba().to_f32_array();
    Color::srgb(b[0] * factor, b[1] * factor, b[2] * factor)
}

/// Sample a continuous ramp and return a fully opaque color blended over a
/// darkened base. Non-relevant cells (e.g. water on a land overlay) should
/// pass through `darken()` instead.
pub fn overlay_continuous(ramp: &ColorRamp, t: f32) -> Color {
    ramp.sample(t)
}

/// Return a binary overlay tint blended onto a base color.
pub fn overlay_binary(base: Color, palette: &BinaryPalette, active: bool) -> Color {
    let tint = if active { palette.on } else { palette.off };
    blend_tint(base, tint)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract sRGB components from a Color.
    fn rgb(c: Color) -> (f32, f32, f32) {
        let s = c.to_srgba();
        (s.red, s.green, s.blue)
    }

    #[test]
    fn viridis_endpoints() {
        let (r0, g0, b0) = rgb(VIRIDIS.sample(0.0));
        // Should be dark purple
        assert!(
            r0 < 0.30 && g0 < 0.05 && b0 > 0.30,
            "viridis(0) should be dark purple"
        );

        let (r1, g1, b1) = rgb(VIRIDIS.sample(1.0));
        // Should be bright yellow
        assert!(
            r1 > 0.90 && g1 > 0.85 && b1 < 0.20,
            "viridis(1) should be bright yellow"
        );
    }

    #[test]
    fn inferno_endpoints() {
        let (r0, g0, b0) = rgb(INFERNO.sample(0.0));
        // Should be near-black
        assert!(
            r0 < 0.05 && g0 < 0.05 && b0 < 0.05,
            "inferno(0) should be near-black"
        );

        let (r1, g1, b1) = rgb(INFERNO.sample(1.0));
        // Should be pale yellow
        assert!(r1 > 0.90 && g1 > 0.90, "inferno(1) should be pale yellow");
    }

    #[test]
    fn cividis_endpoints() {
        let (r0, g0, b0) = rgb(CIVIDIS.sample(0.0));
        // Should be dark navy
        assert!(r0 < 0.05 && b0 > 0.25, "cividis(0) should be dark navy");

        let (r1, g1, _b1) = rgb(CIVIDIS.sample(1.0));
        // Should be warm yellow
        assert!(r1 > 0.85 && g1 > 0.70, "cividis(1) should be warm yellow");
    }

    #[test]
    fn ramp_clamps_out_of_range() {
        let below = rgb(VIRIDIS.sample(-0.5));
        let at_zero = rgb(VIRIDIS.sample(0.0));
        assert_eq!(below, at_zero, "t < 0 should clamp to t = 0");

        let above = rgb(VIRIDIS.sample(1.5));
        let at_one = rgb(VIRIDIS.sample(1.0));
        assert_eq!(above, at_one, "t > 1 should clamp to t = 1");
    }

    #[test]
    fn ramp_midpoint_interpolation() {
        // At t=0.5 the color should be between endpoints (not equal to either).
        let (r0, g0, b0) = rgb(VIRIDIS.sample(0.0));
        let (r1, g1, b1) = rgb(VIRIDIS.sample(1.0));
        let (rm, gm, bm) = rgb(VIRIDIS.sample(0.5));

        // Midpoint should differ from both endpoints
        let diff_lo = (rm - r0).abs() + (gm - g0).abs() + (bm - b0).abs();
        let diff_hi = (rm - r1).abs() + (gm - g1).abs() + (bm - b1).abs();
        assert!(diff_lo > 0.1, "midpoint should differ from start");
        assert!(diff_hi > 0.1, "midpoint should differ from end");
    }

    #[test]
    fn ramp_monotonic_luminance_viridis() {
        // Viridis should have roughly monotonically increasing luminance.
        // We use a simple relative luminance approximation: 0.2126R + 0.7152G + 0.0722B
        let steps = 16;
        let mut prev_lum = 0.0_f32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let (r, g, b) = rgb(VIRIDIS.sample(t));
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            // Allow small tolerance for perceptual uniformity approximation
            assert!(
                lum >= prev_lum - 0.02,
                "viridis luminance should be roughly monotonic at t={t}: {lum} < {prev_lum}"
            );
            prev_lum = lum;
        }
    }

    #[test]
    fn sample_rgba_returns_full_alpha() {
        let c = VIRIDIS.sample_rgba(0.5);
        assert_eq!(c[3], 1.0, "sample_rgba alpha should be 1.0");
    }

    #[test]
    fn blend_tint_identity() {
        let base = Color::srgb(0.5, 0.5, 0.5);
        let tint = Color::srgba(1.0, 0.0, 0.0, 0.0);
        let result = rgb(blend_tint(base, tint));
        let expected = rgb(base);
        assert!(
            (result.0 - expected.0).abs() < 1e-5
                && (result.1 - expected.1).abs() < 1e-5
                && (result.2 - expected.2).abs() < 1e-5,
            "blend with alpha=0 should return base unchanged"
        );
    }

    #[test]
    fn darken_halves_rgb() {
        let base = Color::srgb(0.8, 0.6, 0.4);
        let (r, g, b) = rgb(darken(base, 0.5));
        assert!((r - 0.4).abs() < 1e-5);
        assert!((g - 0.3).abs() < 1e-5);
        assert!((b - 0.2).abs() < 1e-5);
    }

    #[test]
    fn groundwater_level_endpoints() {
        let (r0, _g0, b0) = rgb(GROUNDWATER_LEVEL.sample(0.0));
        // Should be red/warm (dry)
        assert!(r0 > 0.60, "groundwater_level(0) should be reddish (dry)");

        let (r1, _g1, b1) = rgb(GROUNDWATER_LEVEL.sample(1.0));
        // Should be deep blue (saturated)
        assert!(
            b1 > r1,
            "groundwater_level(1) should be blue (saturated), got r={r1} b={b1}"
        );
    }

    #[test]
    fn groundwater_quality_endpoints() {
        let (r0, g0, _b0) = rgb(GROUNDWATER_QUALITY.sample(0.0));
        // Should be brown (contaminated)
        assert!(
            r0 > g0,
            "groundwater_quality(0) should be brownish, got r={r0} g={g0}"
        );

        let (_r1, g1, _b1) = rgb(GROUNDWATER_QUALITY.sample(1.0));
        // Should be green (clean)
        assert!(
            g1 > 0.70,
            "groundwater_quality(1) should be green, got g={g1}"
        );
    }

    #[test]
    fn groundwater_ramps_different_from_each_other() {
        let level_mid = rgb(GROUNDWATER_LEVEL.sample(0.5));
        let quality_mid = rgb(GROUNDWATER_QUALITY.sample(0.5));
        let diff = (level_mid.0 - quality_mid.0).abs()
            + (level_mid.1 - quality_mid.1).abs()
            + (level_mid.2 - quality_mid.2).abs();
        assert!(
            diff > 0.1,
            "groundwater level and quality ramps should differ at midpoint"
        );
    }

    #[test]
    fn binary_palette_on_off() {
        let base = Color::srgb(0.5, 0.5, 0.5);
        let on_color = overlay_binary(base, &POWER_PALETTE, true);
        let off_color = overlay_binary(base, &POWER_PALETTE, false);
        let (on_r, _, _) = rgb(on_color);
        let (off_r, _, _) = rgb(off_color);
        // "On" should be more yellow (higher R+G), "Off" more red (higher R relative to G)
        assert!(on_r != off_r, "on and off colors should differ");
    }
}
