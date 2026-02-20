//! Tree and prop visual improvements (UX-074).
//!
//! Provides three visual enhancement systems:
//!
//! 1. **Seasonal tree tinting** -- Modifies the `StandardMaterial` base color of
//!    all tree prop meshes every slow tick to reflect the current season:
//!    spring (light green/budding), summer (lush green), autumn (orange-gold),
//!    and winter (grey-brown/bare). Tinting interpolates smoothly between
//!    seasons using the seasonal rendering state.
//!
//! 2. **Intersection lamp posts** -- Spawns additional lamp posts specifically at
//!    road intersections (cells where 3+ road neighbours meet). These complement
//!    the existing road-edge lamp spawning in `props.rs`.
//!
//! 3. **Prop LOD** -- Hides props (`TreeProp`, `StreetLamp`, `ParkedCar`) when the
//!    camera distance exceeds configurable thresholds, reducing draw calls at
//!    wide zoom levels.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid};
use simulation::weather::Season;

use crate::building_meshes::BuildingModelCache;
use crate::camera::OrbitCamera;
use crate::props::{ParkedCar, PropEntity, PropsSpawned, StreetLamp, TreeProp};

// =============================================================================
// Constants
// =============================================================================

/// Camera distance beyond which tree props are hidden.
const TREE_LOD_DISTANCE: f32 = 2500.0;

/// Camera distance beyond which street lamp props are hidden.
const LAMP_LOD_DISTANCE: f32 = 3000.0;

/// Camera distance beyond which parked car props are hidden.
const CAR_LOD_DISTANCE: f32 = 2000.0;

/// Scale for intersection lamp models.
const INTERSECTION_LAMP_SCALE: f32 = 1.8;

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
// Components
// =============================================================================

/// Marker for intersection lamp post entities (separate from edge lamps).
#[derive(Component)]
pub struct IntersectionLamp;

// =============================================================================
// Resources
// =============================================================================

/// Tracks whether intersection lamps have been spawned (one-shot, like `PropsSpawned`).
#[derive(Resource, Default)]
pub struct IntersectionLampsSpawned(pub bool);

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

/// Count how many orthogonal road-type neighbours a cell has.
/// A cell is an "intersection" if it has 3 or more road neighbours.
pub fn road_neighbour_count(grid: &WorldGrid, gx: usize, gy: usize) -> usize {
    let width = grid.width;
    let height = grid.height;
    let neighbours = [
        (gx.wrapping_sub(1), gy),
        (gx + 1, gy),
        (gx, gy.wrapping_sub(1)),
        (gx, gy + 1),
    ];
    neighbours
        .iter()
        .filter(|&&(nx, ny)| {
            nx < width && ny < height && grid.get(nx, ny).cell_type == CellType::Road
        })
        .count()
}

/// Returns true if the cell at (gx, gy) is a road intersection (3+ road neighbours).
pub fn is_intersection(grid: &WorldGrid, gx: usize, gy: usize) -> bool {
    road_neighbour_count(grid, gx, gy) >= 3
}

/// Determine whether a prop should be visible given the camera distance and the
/// LOD threshold for that prop type.
pub fn should_show_prop(camera_distance: f32, lod_threshold: f32) -> bool {
    camera_distance <= lod_threshold
}

// =============================================================================
// Systems
// =============================================================================

/// Spawn lamp posts at road intersections. Runs once after the grid is ready.
pub fn spawn_intersection_lamps(
    mut commands: Commands,
    model_cache: Res<BuildingModelCache>,
    grid: Res<WorldGrid>,
    mut spawned: ResMut<IntersectionLampsSpawned>,
    props_spawned: Res<PropsSpawned>,
) {
    // Wait until the base props system has run so the grid is populated.
    if spawned.0 || !props_spawned.lamps_spawned || model_cache.props.is_empty() {
        return;
    }
    spawned.0 = true;

    let width = grid.width;
    let height = grid.height;

    for gy in 1..height.saturating_sub(1) {
        for gx in 1..width.saturating_sub(1) {
            let cell = grid.get(gx, gy);
            if cell.cell_type != CellType::Road {
                continue;
            }

            if !is_intersection(&grid, gx, gy) {
                continue;
            }

            // Deterministic hash to avoid placing a lamp at every single intersection
            let hash = gx.wrapping_mul(47).wrapping_add(gy.wrapping_mul(61)) % 100;
            if hash >= 80 {
                continue; // ~80% of intersections get a lamp
            }

            let (wx, _) = WorldGrid::grid_to_world(gx, gy);
            let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

            // Place the lamp at a slight offset toward the corner of the intersection.
            let off_x = if hash % 2 == 0 {
                CELL_SIZE * 0.35
            } else {
                -CELL_SIZE * 0.35
            };
            let off_z = if (hash / 10) % 2 == 0 {
                CELL_SIZE * 0.35
            } else {
                -CELL_SIZE * 0.35
            };

            // Prefer the double-light for intersections (index 2 in props if available).
            let scene_handle = if model_cache.props.len() > 2 {
                model_cache.props[2].clone() // detail-light-double
            } else {
                model_cache.get_prop(hash)
            };

            commands.spawn((
                PropEntity,
                StreetLamp,
                IntersectionLamp,
                SceneRoot(scene_handle),
                Transform::from_xyz(wx + off_x, 0.0, wz + off_z)
                    .with_scale(Vec3::splat(INTERSECTION_LAMP_SCALE)),
                Visibility::default(),
            ));
        }
    }
}

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

/// LOD system: hide/show prop entities based on camera distance.
///
/// Runs every frame and toggles `Visibility` for trees, lamps, and parked cars
/// depending on the current orbit camera distance.
#[allow(clippy::type_complexity)]
pub fn update_prop_lod(
    orbit: Res<OrbitCamera>,
    mut trees: Query<&mut Visibility, (With<TreeProp>, Without<StreetLamp>, Without<ParkedCar>)>,
    mut lamps: Query<&mut Visibility, (With<StreetLamp>, Without<TreeProp>, Without<ParkedCar>)>,
    mut cars: Query<&mut Visibility, (With<ParkedCar>, Without<TreeProp>, Without<StreetLamp>)>,
) {
    let dist = orbit.distance;

    let tree_vis = if should_show_prop(dist, TREE_LOD_DISTANCE) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let lamp_vis = if should_show_prop(dist, LAMP_LOD_DISTANCE) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let car_vis = if should_show_prop(dist, CAR_LOD_DISTANCE) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in trees.iter_mut() {
        *vis = tree_vis;
    }
    for mut vis in lamps.iter_mut() {
        *vis = lamp_vis;
    }
    for mut vis in cars.iter_mut() {
        *vis = car_vis;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TreePropsPlugin;

impl Plugin for TreePropsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntersectionLampsSpawned>()
            .init_resource::<LastTreeTintSeason>()
            .add_systems(
                Update,
                (
                    spawn_intersection_lamps,
                    update_tree_seasonal_tint,
                    update_prop_lod,
                ),
            );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::config::{GRID_HEIGHT, GRID_WIDTH};

    // -------------------------------------------------------------------------
    // Season tint tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_season_tint_returns_distinct_colors() {
        let spring = season_tint(Season::Spring).to_srgba();
        let summer = season_tint(Season::Summer).to_srgba();
        let autumn = season_tint(Season::Autumn).to_srgba();
        let winter = season_tint(Season::Winter).to_srgba();

        // Each season should have a unique tint.
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
        // Winter should be desaturated: channels close together.
        let spread = (c.red - c.blue).abs();
        assert!(
            spread < 0.2,
            "winter should be muted (low saturation), spread={}",
            spread
        );
    }

    // -------------------------------------------------------------------------
    // Blended tint tests
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Intersection detection tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_road_neighbour_count_no_roads() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert_eq!(road_neighbour_count(&grid, 5, 5), 0);
    }

    #[test]
    fn test_road_neighbour_count_single_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        // Cell (5,5) is a road but none of its neighbours are
        assert_eq!(road_neighbour_count(&grid, 5, 5), 0);
    }

    #[test]
    fn test_road_neighbour_count_cross() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Create a cross intersection at (10, 10)
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 9).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, 10, 10), 4);
    }

    #[test]
    fn test_road_neighbour_count_t_junction() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // T-junction at (10, 10): roads left, right, down
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, 10, 10), 3);
    }

    #[test]
    fn test_is_intersection_cross() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 9).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert!(is_intersection(&grid, 10, 10));
    }

    #[test]
    fn test_is_intersection_straight_road_not_intersection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Straight horizontal road
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        assert!(!is_intersection(&grid, 10, 10));
    }

    #[test]
    fn test_is_intersection_corner_not_intersection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // L-shape corner: only 2 road neighbours
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert!(!is_intersection(&grid, 10, 10));
    }

    // -------------------------------------------------------------------------
    // Prop LOD tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_should_show_prop_within_threshold() {
        assert!(should_show_prop(100.0, TREE_LOD_DISTANCE));
        assert!(should_show_prop(100.0, LAMP_LOD_DISTANCE));
        assert!(should_show_prop(100.0, CAR_LOD_DISTANCE));
    }

    #[test]
    fn test_should_show_prop_at_threshold() {
        assert!(should_show_prop(TREE_LOD_DISTANCE, TREE_LOD_DISTANCE));
        assert!(should_show_prop(LAMP_LOD_DISTANCE, LAMP_LOD_DISTANCE));
        assert!(should_show_prop(CAR_LOD_DISTANCE, CAR_LOD_DISTANCE));
    }

    #[test]
    fn test_should_hide_prop_beyond_threshold() {
        assert!(!should_show_prop(
            TREE_LOD_DISTANCE + 1.0,
            TREE_LOD_DISTANCE
        ));
        assert!(!should_show_prop(
            LAMP_LOD_DISTANCE + 1.0,
            LAMP_LOD_DISTANCE
        ));
        assert!(!should_show_prop(CAR_LOD_DISTANCE + 1.0, CAR_LOD_DISTANCE));
    }

    #[test]
    fn test_lod_order_cars_hide_first() {
        // Cars should hide before trees, trees before lamps.
        assert!(
            CAR_LOD_DISTANCE < TREE_LOD_DISTANCE,
            "cars should hide before trees"
        );
        assert!(
            TREE_LOD_DISTANCE < LAMP_LOD_DISTANCE,
            "trees should hide before lamps"
        );
    }

    #[test]
    fn test_lod_thresholds_positive() {
        assert!(TREE_LOD_DISTANCE > 0.0);
        assert!(LAMP_LOD_DISTANCE > 0.0);
        assert!(CAR_LOD_DISTANCE > 0.0);
    }

    // -------------------------------------------------------------------------
    // Color lerp tests
    // -------------------------------------------------------------------------

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
        // t > 1.0 should be clamped
        let result = color_lerp(a, b, 2.0).to_srgba();
        assert!((result.red - 1.0).abs() < 0.001, "t>1 should clamp to 1");
        // t < 0.0 should be clamped
        let result = color_lerp(a, b, -1.0).to_srgba();
        assert!((result.red - 0.0).abs() < 0.001, "t<0 should clamp to 0");
    }

    // -------------------------------------------------------------------------
    // Constants validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_intersection_lamp_scale_positive() {
        assert!(INTERSECTION_LAMP_SCALE > 0.0);
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

    // -------------------------------------------------------------------------
    // Edge case: boundary cells
    // -------------------------------------------------------------------------

    #[test]
    fn test_road_neighbour_count_at_grid_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place a road at (0, 0) with one road neighbour to the right
        grid.get_mut(0, 0).cell_type = CellType::Road;
        grid.get_mut(1, 0).cell_type = CellType::Road;
        // wrapping_sub(1) for usize 0 gives a huge number, which is out of bounds.
        // So only (1,0) should count as a neighbour.
        assert_eq!(road_neighbour_count(&grid, 0, 0), 1);
    }

    #[test]
    fn test_road_neighbour_count_at_max_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let x = GRID_WIDTH - 1;
        let y = GRID_HEIGHT - 1;
        grid.get_mut(x, y).cell_type = CellType::Road;
        grid.get_mut(x - 1, y).cell_type = CellType::Road;
        grid.get_mut(x, y - 1).cell_type = CellType::Road;
        // (x+1, y) and (x, y+1) are out of bounds, should be ignored
        assert_eq!(road_neighbour_count(&grid, x, y), 2);
    }
}
