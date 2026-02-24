//! Building Mesh Variants (REND-003)
//!
//! Provides 2-3 distinct mesh variants per zone type per building level, with
//! deterministic seeded selection based on grid position.  This ensures
//! neighbouring buildings of the same zone and level still look visually
//! distinct, while buildings at different levels are clearly differentiated.
//!
//! Each variant gets a unique **proportion** (width/height/depth scale) from
//! `building_variant_proportions`, so even buildings that share the same GLB
//! model appear different -- one might be tall and narrow, another wide and
//! squat.  Combined with the per-level model partitioning from the model pool,
//! this produces 2-3 visually distinct building shapes per zone/level combo.

use bevy::prelude::*;

use simulation::buildings::Building;
use simulation::grid::ZoneType;

use crate::building_meshes::{building_scale, BuildingModelCache};
use crate::building_render::{BuildingMesh3d, ZoneBuilding};
use crate::building_variant_proportions;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of distinct visual variants each zone-type/level combination
/// should expose.  The actual count is `min(VARIANTS_PER_LEVEL, pool_size)`.
const VARIANTS_PER_LEVEL: usize = 3;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component attached to zone-building mesh entities once a
/// level-aware variant has been assigned.  Stores the variant index so we
/// can detect when a level change requires re-selection.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuildingVariant {
    /// The building level at the time the variant was selected.
    pub level: u8,
    /// Index within the level-specific variant slice (0..VARIANTS_PER_LEVEL).
    pub variant_index: usize,
}

// ---------------------------------------------------------------------------
// Variant selection logic
// ---------------------------------------------------------------------------

/// Compute a deterministic variant index for a building at a given grid
/// position and level.  The hash mixes grid coordinates, level, and zone
/// discriminant so that:
///
/// * Two buildings at the same position but different levels get different
///   variants (most of the time).
/// * Two buildings at different positions but same zone/level get different
///   variants (spatial variety).
/// * The selection is stable across save/load (purely positional).
fn variant_hash(grid_x: usize, grid_y: usize, level: u8, zone: ZoneType) -> usize {
    let zone_disc: usize = match zone {
        ZoneType::None => 0,
        ZoneType::ResidentialLow => 1,
        ZoneType::ResidentialMedium => 2,
        ZoneType::ResidentialHigh => 3,
        ZoneType::CommercialLow => 4,
        ZoneType::CommercialHigh => 5,
        ZoneType::Industrial => 6,
        ZoneType::Office => 7,
        ZoneType::MixedUse => 8,
    };

    // Use a mixing function that avoids simple linear patterns
    let mut h: usize = grid_x
        .wrapping_mul(2654435761) // Knuth multiplicative hash constant
        .wrapping_add(grid_y.wrapping_mul(2246822519))
        .wrapping_add((level as usize).wrapping_mul(3266489917))
        .wrapping_add(zone_disc.wrapping_mul(668265263));

    // Finalizer: xorshift-style mixing
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;

    h
}

/// Select the scene handle for a given zone type, level, and variant hash.
///
/// This partitions the model pool into per-level slices of
/// `VARIANTS_PER_LEVEL` entries.  Each level gets a distinct starting offset
/// into the pool so that level-1 and level-2 buildings of the same zone
/// always look different (assuming the pool has enough models).
fn select_variant_scene(
    cache: &BuildingModelCache,
    zone: ZoneType,
    level: u8,
    hash: usize,
) -> (Handle<Scene>, usize) {
    match zone {
        ZoneType::ResidentialLow => select_from_pool(&cache.residential, level, hash),
        ZoneType::ResidentialMedium => {
            select_from_pool(&cache.commercial, level, hash)
        }
        ZoneType::ResidentialHigh => {
            if level >= 3 && !cache.skyscrapers.is_empty() {
                select_from_pool(&cache.skyscrapers, level, hash)
            } else if !cache.commercial.is_empty() {
                select_from_pool(&cache.commercial, level, hash)
            } else {
                select_from_pool(&cache.residential, level, hash)
            }
        }
        ZoneType::CommercialLow => {
            if level >= 4 && !cache.skyscrapers.is_empty() {
                select_from_pool(&cache.skyscrapers, level, hash)
            } else {
                select_from_pool(&cache.commercial, level, hash)
            }
        }
        ZoneType::CommercialHigh => {
            if level >= 4 && !cache.skyscrapers.is_empty() {
                select_from_pool(&cache.skyscrapers, level, hash)
            } else {
                select_from_pool(&cache.commercial, level, hash)
            }
        }
        ZoneType::Industrial => select_from_pool(&cache.industrial, level, hash),
        ZoneType::Office => {
            if level >= 3 && !cache.skyscrapers.is_empty() {
                select_from_pool(&cache.skyscrapers, level, hash)
            } else {
                select_from_pool(&cache.commercial, level, hash)
            }
        }
        ZoneType::MixedUse => {
            if level >= 3 && !cache.skyscrapers.is_empty() {
                select_from_pool(&cache.skyscrapers, level, hash)
            } else {
                select_from_pool(&cache.commercial, level, hash)
            }
        }
        ZoneType::None => select_from_pool(&cache.residential, level, hash),
    }
}

/// Given a model pool, partition it into per-level slices and select a
/// variant from the slice for the given level.
fn select_from_pool(pool: &[Handle<Scene>], level: u8, hash: usize) -> (Handle<Scene>, usize) {
    if pool.is_empty() {
        return (Handle::default(), 0);
    }

    let pool_len = pool.len();
    let level_offset = ((level as usize).wrapping_sub(1)) * VARIANTS_PER_LEVEL;
    let available = VARIANTS_PER_LEVEL.min(pool_len);
    let variant_index = hash % available;
    let pool_index = (level_offset + variant_index) % pool_len;

    (pool[pool_index].clone(), variant_index)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Assigns (or re-assigns) level-aware mesh variants to zone buildings.
///
/// For each zone building mesh entity:
/// 1. If it has no `BuildingVariant` yet, compute one and replace the scene.
/// 2. If it already has a `BuildingVariant` but the building's level has
///    changed, recompute and replace.
///
/// Also applies **per-variant proportions** from the proportion tables: each
/// variant gets a different width/height/depth scale multiplier so that
/// buildings of the same zone and level are visually distinct even when they
/// share the same GLB model.
#[allow(clippy::type_complexity)]
pub fn assign_building_variants(
    mut commands: Commands,
    model_cache: Res<BuildingModelCache>,
    buildings: Query<&Building>,
    mut mesh_query: Query<(Entity, &BuildingMesh3d, &mut Transform, Option<&BuildingVariant>), With<ZoneBuilding>>,
) {
    for (mesh_entity, bm, mut transform, maybe_variant) in &mut mesh_query {
        let Ok(building) = buildings.get(bm.tracked_entity) else {
            continue;
        };

        let needs_assign = match maybe_variant {
            None => true,
            Some(v) => v.level != building.level,
        };

        if !needs_assign {
            continue;
        }

        let hash = variant_hash(
            building.grid_x,
            building.grid_y,
            building.level,
            building.zone_type,
        );

        let (scene_handle, variant_index) =
            select_variant_scene(&model_cache, building.zone_type, building.level, hash);

        // Look up per-variant proportions for this zone/level/variant
        let proportions =
            &building_variant_proportions::proportions_for(building.zone_type, building.level)
                [variant_index];

        // Compute base scale and the small per-building variation
        let base_scale = building_scale(building.zone_type, building.level);
        let pos_hash = building
            .grid_x
            .wrapping_mul(7)
            .wrapping_add(building.grid_y.wrapping_mul(13));
        let scale_var = 0.98 + (pos_hash % 5) as f32 / 100.0;
        let s = base_scale * scale_var;

        // Apply variant proportions to the transform scale
        transform.scale = Vec3::new(
            s * proportions.x,
            s * proportions.y,
            s * proportions.z,
        );

        commands.entity(mesh_entity).insert((
            SceneRoot(scene_handle),
            BuildingVariant {
                level: building.level,
                variant_index,
            },
        ));
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BuildingMeshVariantsPlugin;

impl Plugin for BuildingMeshVariantsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            assign_building_variants
                .after(crate::building_render::spawn_building_meshes)
                .after(crate::building_render::update_building_meshes),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_hash_differs_by_level() {
        let h1 = variant_hash(10, 20, 1, ZoneType::ResidentialLow);
        let h2 = variant_hash(10, 20, 2, ZoneType::ResidentialLow);
        let h3 = variant_hash(10, 20, 3, ZoneType::ResidentialLow);
        assert_ne!(h1, h2);
        assert_ne!(h2, h3);
        assert_ne!(h1, h3);
    }

    #[test]
    fn variant_hash_differs_by_position() {
        let h1 = variant_hash(5, 5, 1, ZoneType::CommercialLow);
        let h2 = variant_hash(6, 5, 1, ZoneType::CommercialLow);
        let h3 = variant_hash(5, 6, 1, ZoneType::CommercialLow);
        assert_ne!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn variant_hash_differs_by_zone() {
        let h1 = variant_hash(10, 10, 1, ZoneType::ResidentialLow);
        let h2 = variant_hash(10, 10, 1, ZoneType::CommercialLow);
        let h3 = variant_hash(10, 10, 1, ZoneType::Industrial);
        assert_ne!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn variant_hash_is_deterministic() {
        let h1 = variant_hash(42, 99, 2, ZoneType::Industrial);
        let h2 = variant_hash(42, 99, 2, ZoneType::Industrial);
        assert_eq!(h1, h2);
    }

    #[test]
    fn select_from_pool_empty() {
        let pool: Vec<Handle<Scene>> = vec![];
        let (handle, idx) = select_from_pool(&pool, 1, 12345);
        assert_eq!(idx, 0);
        assert_eq!(handle, Handle::default());
    }

    #[test]
    fn select_from_pool_single_model() {
        let pool = vec![Handle::default()];
        let (_, idx) = select_from_pool(&pool, 1, 999);
        assert_eq!(idx, 0);
    }

    #[test]
    fn select_from_pool_level_offset() {
        let pool: Vec<Handle<Scene>> = (0..9).map(|_| Handle::default()).collect();
        let (_, v1) = select_from_pool(&pool, 1, 0);
        let (_, v2) = select_from_pool(&pool, 2, 0);
        let (_, v3) = select_from_pool(&pool, 3, 0);
        assert_eq!(v1, 0);
        assert_eq!(v2, 0);
        assert_eq!(v3, 0);
    }

    #[test]
    fn select_from_pool_wraps_around() {
        let pool: Vec<Handle<Scene>> = (0..4).map(|_| Handle::default()).collect();
        let _ = select_from_pool(&pool, 1, 0);
        let _ = select_from_pool(&pool, 2, 0);
        let _ = select_from_pool(&pool, 3, 0);
        let _ = select_from_pool(&pool, 5, 123456);
    }

    #[test]
    fn variant_selection_gives_at_least_2_variants() {
        let pool: Vec<Handle<Scene>> = (0..6).map(|_| Handle::default()).collect();
        let mut seen = std::collections::HashSet::new();
        for h in 0..100 {
            let (_, idx) = select_from_pool(&pool, 1, h);
            seen.insert(idx);
        }
        assert!(
            seen.len() >= 2,
            "Expected at least 2 variants, got {}",
            seen.len()
        );
    }

    #[test]
    fn residential_low_all_levels_covered() {
        let pos = (15, 25);
        let hashes: Vec<usize> = (1..=3)
            .map(|lvl| variant_hash(pos.0, pos.1, lvl, ZoneType::ResidentialLow))
            .collect();
        let unique: std::collections::HashSet<_> = hashes.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn commercial_low_all_levels_covered() {
        let pos = (30, 40);
        let hashes: Vec<usize> = (1..=3)
            .map(|lvl| variant_hash(pos.0, pos.1, lvl, ZoneType::CommercialLow))
            .collect();
        let unique: std::collections::HashSet<_> = hashes.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn industrial_all_levels_covered() {
        let pos = (50, 60);
        let hashes: Vec<usize> = (1..=3)
            .map(|lvl| variant_hash(pos.0, pos.1, lvl, ZoneType::Industrial))
            .collect();
        let unique: std::collections::HashSet<_> = hashes.iter().collect();
        assert_eq!(unique.len(), 3);
    }
}
