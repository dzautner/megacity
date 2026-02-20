use bevy::prelude::*;

use simulation::citizen::{Citizen, CitizenState, CitizenStateComp, Position, Velocity};
use simulation::lod::LodTier;

use crate::building_meshes::BuildingModelCache;

#[derive(Component)]
pub struct CitizenSprite;

/// Tracks whether this citizen is currently showing the car or humanoid mesh
#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum CitizenMeshKind {
    Humanoid,
    Car,
}

/// Scale for character GLB models (Kenney mini-characters are ~1.5 units tall, already human-sized)
const CHARACTER_SCALE: f32 = 2.0;

/// Scale for vehicle GLB models (Kenney car-kit models are ~4 units long, already car-sized)
const VEHICLE_SCALE: f32 = 3.0;

/// Max GLTF scene roots to instantiate per frame.
/// WebGL2 can't handle thousands of scene allocations in one frame.
const MAX_SPRITES_PER_FRAME: usize = if cfg!(target_arch = "wasm32") {
    50
} else {
    500
};

#[allow(clippy::type_complexity)]
pub fn spawn_citizen_sprites(
    mut commands: Commands,
    query: Query<
        (Entity, Option<&CitizenStateComp>, Option<&LodTier>),
        (With<Citizen>, Without<CitizenSprite>),
    >,
    model_cache: Res<BuildingModelCache>,
) {
    if query.is_empty() {
        return;
    }

    let mut spawned = 0;
    for (entity, state_opt, lod) in &query {
        if spawned >= MAX_SPRITES_PER_FRAME {
            break;
        }
        // Skip Abstract-tier citizens — they don't need scene roots
        if lod == Some(&LodTier::Abstract) {
            continue;
        }
        let hash = entity.index() as usize;
        let is_commuting = state_opt.is_some_and(|s| s.0.is_commuting());

        let (scene_handle, kind, scale) = if is_commuting {
            (
                model_cache.get_vehicle(hash),
                CitizenMeshKind::Car,
                VEHICLE_SCALE,
            )
        } else {
            (
                model_cache.get_character(hash),
                CitizenMeshKind::Humanoid,
                CHARACTER_SCALE,
            )
        };

        commands.entity(entity).insert((
            CitizenSprite,
            kind,
            LodTier::default(),
            SceneRoot(scene_handle),
            Transform::from_scale(Vec3::splat(scale)),
            Visibility::default(),
        ));
        spawned += 1;
    }
}

#[allow(clippy::type_complexity)]
pub fn update_citizen_sprites(
    mut query: Query<
        (
            Entity,
            &Position,
            &Velocity,
            &CitizenStateComp,
            &LodTier,
            &mut CitizenMeshKind,
            &mut SceneRoot,
            &mut Transform,
            &mut Visibility,
        ),
        (
            With<CitizenSprite>,
            Or<(
                Changed<Position>,
                Changed<CitizenStateComp>,
                Changed<LodTier>,
            )>,
        ),
    >,
    model_cache: Res<BuildingModelCache>,
) {
    for (entity, pos, vel, state, lod, mut mesh_kind, mut scene_root, mut transform, mut vis) in
        &mut query
    {
        match lod {
            LodTier::Abstract => {
                *vis = Visibility::Hidden;
                continue;
            }
            LodTier::Simplified => {
                // Keep scale but reduce it
            }
            LodTier::Full => {}
        }

        let is_commuting = state.0.is_commuting();
        let hash = entity.index() as usize;

        // Determine which mesh to show
        let desired_kind = if is_commuting {
            CitizenMeshKind::Car
        } else {
            CitizenMeshKind::Humanoid
        };

        // Swap scene if kind changed
        if *mesh_kind != desired_kind {
            *mesh_kind = desired_kind;
            match desired_kind {
                CitizenMeshKind::Humanoid => {
                    *scene_root = SceneRoot(model_cache.get_character(hash));
                }
                CitizenMeshKind::Car => {
                    *scene_root = SceneRoot(model_cache.get_vehicle(hash));
                }
            }
        }

        // Scale based on kind and LOD
        let base_scale = match desired_kind {
            CitizenMeshKind::Humanoid => CHARACTER_SCALE,
            CitizenMeshKind::Car => VEHICLE_SCALE,
        };
        let lod_factor = match lod {
            LodTier::Simplified => 0.5,
            _ => 1.0,
        };
        transform.scale = Vec3::splat(base_scale * lod_factor);

        // Position with lane offset: offset perpendicular to travel direction
        // so vehicles on the same road don't stack on top of each other
        let mut render_x = pos.x;
        let mut render_z = pos.y;

        if is_commuting {
            let speed_sq = vel.x * vel.x + vel.y * vel.y;
            if speed_sq > 0.01 {
                let speed = speed_sq.sqrt();
                let nx = vel.x / speed;
                let ny = vel.y / speed;
                // Perpendicular to travel direction
                let perp_x = -ny;
                let perp_z = nx;
                // Lane assignment based on entity ID: spread across road width
                let lane = (entity.index() % 4) as f32 - 1.5; // -1.5, -0.5, 0.5, 1.5
                let lane_width = 3.0; // ~3m per lane
                render_x += perp_x * lane * lane_width;
                render_z += perp_z * lane * lane_width;
            }
        }

        transform.translation.x = render_x;
        transform.translation.y = 0.0;
        transform.translation.z = render_z;

        // Orientation: face travel direction when commuting
        if is_commuting {
            let speed_sq = vel.x * vel.x + vel.y * vel.y;
            if speed_sq > 0.01 {
                let heading = (-vel.x).atan2(-vel.y);
                transform.rotation = Quat::from_rotation_y(heading);
            }
        } else {
            let angle = (entity.index() % 8) as f32 * std::f32::consts::FRAC_PI_4;
            transform.rotation = Quat::from_rotation_y(angle);
        }

        // Visibility
        *vis = match state.0 {
            CitizenState::CommutingToWork
            | CitizenState::CommutingHome
            | CitizenState::CommutingToShop
            | CitizenState::CommutingToLeisure
            | CitizenState::CommutingToSchool
            | CitizenState::Working
            | CitizenState::Shopping
            | CitizenState::AtLeisure
            | CitizenState::AtSchool => Visibility::Visible,
            CitizenState::AtHome => Visibility::Hidden,
        };
    }
}

/// Despawn CitizenSprite + SceneRoot when a citizen transitions to Abstract tier.
/// This prevents 150K-600K unnecessary GLTF child entities from accumulating.
#[allow(clippy::type_complexity)]
pub fn despawn_abstract_sprites(
    mut commands: Commands,
    query: Query<(Entity, &LodTier), (With<CitizenSprite>, Changed<LodTier>)>,
) {
    for (entity, lod) in &query {
        if *lod == LodTier::Abstract {
            commands
                .entity(entity)
                .remove::<(CitizenSprite, CitizenMeshKind, SceneRoot)>();
        }
    }
}
