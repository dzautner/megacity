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

mod intersection_lamps;
mod prop_lod;
mod seasonal_tint;

use bevy::prelude::*;

// Re-export all public items so external code can use `tree_props::Foo` as before.
pub use intersection_lamps::{
    is_intersection, road_neighbour_count, spawn_intersection_lamps, IntersectionLamp,
    IntersectionLampsSpawned,
};
pub use prop_lod::{should_show_prop, update_prop_lod};
pub use seasonal_tint::{
    blended_season_tint, season_tint, update_tree_seasonal_tint, LastTreeTintSeason,
};

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
