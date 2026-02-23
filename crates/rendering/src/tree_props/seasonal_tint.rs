//! Seasonal tree tinting -- modifies `StandardMaterial` base color of tree props
//! to reflect the current season (spring, summer, autumn, winter).

use bevy::prelude::*;

use simulation::weather::Season;

use crate::props::TreeProp;

// =============================================================================
// Constants
// =============================================================================

/// Seasonal tint colors (applied as a multiplier to tree materials).
/// Spring: light green (budding foliage).
const SPRING_TINT: Color = Color::srgb(0.65, 0.85, 0.45);
/// Summer: lush deep green.
const SUMMER_TINT: Color = Color::srgb(0.35, 0.70, 0.30);
/// Autumn: warm orange-gold.
const AUTUMN_TINT: Color = Color::srgb(0.85, 0.55, 0.20);
/// Winter: grey-brown (bare branches).
const WINTER_TINT: Color = Color::srgb(0.55, 0.50, 0.40);

// =============================================================================
// Resources
// =============================================================================

/// Tracks the last season for which tree tinting was applied,
/// so we only update materials when the season changes.
#[derive(Resource, Default)]
pub struct LastTreeTintSeason(pub Option<u8>);

// =============================================================================
// Pure helper functions
// =============================================================================

/// Return the tint color for a given season.
pub fn season_tint(season: Season) -> Color {
    match season {
        Season::Spring => SPRING_TINT,
        Season::Summer => SUMMER_TINT,
        Season::Autumn => AUTUMN_TINT,
        Season::Winter => WINTER_TINT,
    }
}

/// Linearly interpolate between two sRGB colors.
fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    let t = t.clamp(0.0, 1.0);
    Color::srgb(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
    )
}

/// Compute a blended tint between the current season's colour and the next,
/// using `progress` (0.0 = start of season, 1.0 = end of season).
pub fn blended_season_tint(season: Season, progress: f32) -> Color {
    let next = match season {
        Season::Spring => Season::Summer,
        Season::Summer => Season::Autumn,
        Season::Autumn => Season::Winter,
        Season::Winter => Season::Spring,
    };
    color_lerp(season_tint(season), season_tint(next), progress)
}

// =============================================================================
// Systems
// =============================================================================

/// Apply seasonal color tinting to all tree prop scene materials.
///
/// When the season changes (tracked via `LastTreeTintSeason`), walks every
/// `StandardMaterial` in the asset store and applies the seasonal tint to
/// tree entities. Because tree meshes are shared GLB scenes whose materials
/// are loaded from asset files, we tint via material base_color directly.
///
/// This system is intentionally coarse-grained: it only runs when the season
/// id changes, not every frame.
pub fn update_tree_seasonal_tint(
    seasonal: Res<simulation::seasonal_rendering::SeasonalRenderingState>,
    mut last_season: ResMut<LastTreeTintSeason>,
    tree_query: Query<&Children, With<TreeProp>>,
    children_query: Query<&Children>,
    mesh_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let current_id = seasonal.current_season_id;

    // Only update when season changes.
    if last_season.0 == Some(current_id) {
        return;
    }
    last_season.0 = Some(current_id);

    let season = seasonal.active_season();
    let tint = season_tint(season);
    let tint_srgba = tint.to_srgba();

    // Walk each tree entity -> children -> children (scenes have nested hierarchies)
    // and find all StandardMaterial handles to tint.
    for tree_children in tree_query.iter() {
        tint_descendants(
            tree_children,
            &children_query,
            &mesh_query,
            &mut materials,
            &tint_srgba,
        );
    }
}

/// Recursively walk descendants to find and tint all StandardMaterial handles.
fn tint_descendants(
    children: &Children,
    children_query: &Query<&Children>,
    mesh_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    tint: &Srgba,
) {
    for &child in children.iter() {
        // If this child has a material handle, tint it.
        if let Ok(mat_handle) = mesh_query.get(child) {
            if let Some(material) = materials.get_mut(mat_handle) {
                // Blend the tint with the existing alpha (preserve transparency).
                let alpha = material.base_color.to_srgba().alpha;
                material.base_color = Color::srgba(tint.red, tint.green, tint.blue, alpha);
            }
        }
        // Recurse into deeper children.
        if let Ok(grandchildren) = children_query.get(child) {
            tint_descendants(grandchildren, children_query, mesh_query, materials, tint);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_tint_returns_distinct_colors() {
        let spring = season_tint(Season::Spring).to_srgba();
        let summer = season_tint(Season::Summer).to_srgba();
        let autumn = season_tint(Season::Autumn).to_srgba();
        let winter = season_tint(Season::Winter).to_srgba();

        assert_ne!(spring, summer, "spring and summer should differ");
        assert_ne!(summer, autumn, "summer and autumn should differ");
        assert_ne!(autumn, winter, "autumn and winter should differ");
        assert_ne!(winter, spring, "winter and spring should differ");
    }

    #[test]
    fn test_season_tint_valid_rgb_range() {
        for season in [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let c = season_tint(season).to_srgba();
            assert!(
                c.red >= 0.0 && c.red <= 1.0,
                "{:?} red out of range",
                season
            );
            assert!(
                c.green >= 0.0 && c.green <= 1.0,
                "{:?} green out of range",
                season
            );
            assert!(
                c.blue >= 0.0 && c.blue <= 1.0,
                "{:?} blue out of range",
                season
            );
        }
    }

    #[test]
    fn test_spring_is_greenish() {
        let c = season_tint(Season::Spring).to_srgba();
        assert!(
            c.green > c.red && c.green > c.blue,
            "spring should be green-dominant: r={} g={} b={}",
            c.red,
            c.green,
            c.blue
        );
    }

    #[test]
    fn test_summer_is_green_dominant() {
        let c = season_tint(Season::Summer).to_srgba();
        assert!(
            c.green > c.red && c.green > c.blue,
            "summer should be green-dominant: r={} g={} b={}",
            c.red,
            c.green,
            c.blue
        );
    }

    #[test]
    fn test_autumn_is_warm() {
        let c = season_tint(Season::Autumn).to_srgba();
        assert!(
            c.red > c.green && c.red > c.blue,
            "autumn should be red/orange-dominant: r={} g={} b={}",
            c.red,
            c.green,
            c.blue
        );
    }

    #[test]
    fn test_winter_is_muted() {
        let c = season_tint(Season::Winter).to_srgba();
        let spread = (c.red - c.blue).abs();
        assert!(
            spread < 0.2,
            "winter should be muted (low saturation), spread={}",
            spread
        );
    }

    #[test]
    fn test_blended_at_zero_equals_current_season() {
        for season in [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let blended = blended_season_tint(season, 0.0).to_srgba();
            let pure = season_tint(season).to_srgba();
            assert!(
                (blended.red - pure.red).abs() < 0.01,
                "{:?} red mismatch at t=0",
                season
            );
            assert!(
                (blended.green - pure.green).abs() < 0.01,
                "{:?} green mismatch at t=0",
                season
            );
            assert!(
                (blended.blue - pure.blue).abs() < 0.01,
                "{:?} blue mismatch at t=0",
                season
            );
        }
    }

    #[test]
    fn test_blended_at_one_equals_next_season() {
        let pairs = [
            (Season::Spring, Season::Summer),
            (Season::Summer, Season::Autumn),
            (Season::Autumn, Season::Winter),
            (Season::Winter, Season::Spring),
        ];
        for (current, next) in pairs {
            let blended = blended_season_tint(current, 1.0).to_srgba();
            let target = season_tint(next).to_srgba();
            assert!(
                (blended.red - target.red).abs() < 0.01,
                "{:?}->{:?} red mismatch at t=1",
                current,
                next
            );
            assert!(
                (blended.green - target.green).abs() < 0.01,
                "{:?}->{:?} green mismatch at t=1",
                current,
                next
            );
            assert!(
                (blended.blue - target.blue).abs() < 0.01,
                "{:?}->{:?} blue mismatch at t=1",
                current,
                next
            );
        }
    }

    #[test]
    fn test_blended_at_half_is_midpoint() {
        let blended = blended_season_tint(Season::Summer, 0.5).to_srgba();
        let summer = season_tint(Season::Summer).to_srgba();
        let autumn = season_tint(Season::Autumn).to_srgba();
        let expected_red = (summer.red + autumn.red) / 2.0;
        let expected_green = (summer.green + autumn.green) / 2.0;
        assert!(
            (blended.red - expected_red).abs() < 0.01,
            "midpoint red: expected ~{}, got {}",
            expected_red,
            blended.red
        );
        assert!(
            (blended.green - expected_green).abs() < 0.01,
            "midpoint green: expected ~{}, got {}",
            expected_green,
            blended.green
        );
    }

    #[test]
    fn test_color_lerp_at_zero() {
        let a = Color::srgb(1.0, 0.0, 0.0);
        let b = Color::srgb(0.0, 1.0, 0.0);
        let result = color_lerp(a, b, 0.0).to_srgba();
        assert!((result.red - 1.0).abs() < 0.001);
        assert!((result.green - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp_at_one() {
        let a = Color::srgb(1.0, 0.0, 0.0);
        let b = Color::srgb(0.0, 1.0, 0.0);
        let result = color_lerp(a, b, 1.0).to_srgba();
        assert!((result.red - 0.0).abs() < 0.001);
        assert!((result.green - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp_at_half() {
        let a = Color::srgb(1.0, 0.0, 0.0);
        let b = Color::srgb(0.0, 1.0, 0.0);
        let result = color_lerp(a, b, 0.5).to_srgba();
        assert!((result.red - 0.5).abs() < 0.001);
        assert!((result.green - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp_clamps_t() {
        let a = Color::srgb(0.0, 0.0, 0.0);
        let b = Color::srgb(1.0, 1.0, 1.0);
        let result = color_lerp(a, b, 2.0).to_srgba();
        assert!((result.red - 1.0).abs() < 0.001, "t>1 should clamp to 1");
        let result = color_lerp(a, b, -1.0).to_srgba();
        assert!((result.red - 0.0).abs() < 0.001, "t<0 should clamp to 0");
    }

    #[test]
    fn test_all_tint_colors_valid() {
        for season in [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let c = season_tint(season).to_srgba();
            assert!(c.red >= 0.0 && c.red <= 1.0);
            assert!(c.green >= 0.0 && c.green <= 1.0);
            assert!(c.blue >= 0.0 && c.blue <= 1.0);
        }
    }
}
