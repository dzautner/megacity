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

/// Fade component for smooth LOD transitions.
///
/// When a citizen's LOD tier changes to a visible tier, we start with `alpha = 0.0`
/// and fade in over `FADE_DURATION` seconds. When transitioning to Abstract (invisible),
/// we fade out and then despawn the sprite. This avoids popping artifacts without
/// double-rendering (only one representation exists at a time).
#[derive(Component)]
pub struct LodFade {
    /// Current opacity factor (0.0 = invisible, 1.0 = fully visible)
    pub alpha: f32,
    /// Whether we are fading in (true) or fading out (false)
    pub fading_in: bool,
    /// Elapsed time for the current fade
    pub timer: f32,
}

/// Duration of fade transitions in seconds.
const FADE_DURATION: f32 = 0.3;

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

        // Start with a fade-in so newly spawned sprites don't pop in
        let fade = LodFade {
            alpha: 0.0,
            fading_in: true,
            timer: 0.0,
        };

        commands.entity(entity).insert((
            CitizenSprite,
            kind,
            fade,
            LodTier::default(),
            SceneRoot(scene_handle),
            Transform::from_scale(Vec3::splat(scale)),
            Visibility::default(),
        ));
        spawned += 1;
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
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
            Option<&LodFade>,
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
    for (
        entity,
        pos,
        vel,
        state,
        lod,
        mut mesh_kind,
        mut scene_root,
        mut transform,
        mut vis,
        fade,
    ) in &mut query
    {
        match lod {
            LodTier::Abstract => {
                // Don't hide immediately -- let the fade-out system handle it.
                // If there's no fade component yet (shouldn't happen), hide directly.
                if fade.is_none() {
                    *vis = Visibility::Hidden;
                }
                continue;
            }
            LodTier::Simplified | LodTier::Full => {}
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

        // Scale based on kind, LOD, and fade alpha
        let base_scale = match desired_kind {
            CitizenMeshKind::Humanoid => CHARACTER_SCALE,
            CitizenMeshKind::Car => VEHICLE_SCALE,
        };
        let lod_factor = match lod {
            LodTier::Simplified => 0.5,
            _ => 1.0,
        };
        let fade_factor = fade.map_or(1.0, |f| f.alpha);
        transform.scale = Vec3::splat(base_scale * lod_factor * fade_factor);

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

        // Visibility: hide at-home citizens, otherwise delegate to fade alpha
        *vis = match state.0 {
            CitizenState::CommutingToWork
            | CitizenState::CommutingHome
            | CitizenState::CommutingToShop
            | CitizenState::CommutingToLeisure
            | CitizenState::CommutingToSchool
            | CitizenState::Working
            | CitizenState::Shopping
            | CitizenState::AtLeisure
            | CitizenState::AtSchool => {
                if fade.is_some_and(|f| f.alpha <= 0.001) {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                }
            }
            CitizenState::AtHome => Visibility::Hidden,
        };
    }
}

/// Start a fade-out when a citizen transitions to Abstract tier, and a fade-in
/// when transitioning to a visible tier.
#[allow(clippy::type_complexity)]
pub fn trigger_lod_fade(
    mut commands: Commands,
    query: Query<(Entity, &LodTier), (With<CitizenSprite>, Changed<LodTier>)>,
) {
    for (entity, lod) in &query {
        match lod {
            LodTier::Abstract => {
                // Start fade-out before despawning
                commands.entity(entity).insert(LodFade {
                    alpha: 1.0,
                    fading_in: false,
                    timer: 0.0,
                });
            }
            LodTier::Full | LodTier::Simplified => {
                // Start fade-in for the new tier
                commands.entity(entity).insert(LodFade {
                    alpha: 0.0,
                    fading_in: true,
                    timer: 0.0,
                });
            }
        }
    }
}

/// Advance fade timers each frame. When a fade-in completes, remove the `LodFade`
/// component so the citizen renders at full opacity. When a fade-out completes,
/// despawn the sprite components entirely.
///
/// The fade alpha is applied as a scale multiplier by `update_citizen_sprites`
/// (which reads `Option<&LodFade>`). This system ensures the alpha value is
/// updated every frame so the scale smoothly interpolates even when Position
/// and CitizenState haven't changed.
#[allow(clippy::type_complexity)]
pub fn update_lod_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut LodFade,
            &mut Transform,
            &LodTier,
            &CitizenMeshKind,
        ),
        With<CitizenSprite>,
    >,
) {
    let dt = time.delta_secs();
    for (entity, mut fade, mut transform, lod, mesh_kind) in &mut query {
        fade.timer += dt;
        let progress = (fade.timer / FADE_DURATION).clamp(0.0, 1.0);

        if fade.fading_in {
            fade.alpha = progress;
        } else {
            fade.alpha = 1.0 - progress;
        }

        // If fade is complete, clean up
        if progress >= 1.0 {
            if fade.fading_in {
                // Fade-in complete: remove the LodFade component (citizen is fully visible)
                fade.alpha = 1.0;
                commands.entity(entity).remove::<LodFade>();
            } else {
                // Fade-out complete: despawn the sprite components
                commands
                    .entity(entity)
                    .remove::<(CitizenSprite, CitizenMeshKind, SceneRoot, LodFade)>();
                continue;
            }
        }

        // Apply fade alpha as a scale multiplier so the sprite smoothly grows/shrinks.
        // This runs every frame to keep the transition smooth even when
        // update_citizen_sprites doesn't fire (it only triggers on Changed<>).
        let base_scale = match mesh_kind {
            CitizenMeshKind::Humanoid => CHARACTER_SCALE,
            CitizenMeshKind::Car => VEHICLE_SCALE,
        };
        let lod_factor = match lod {
            LodTier::Simplified => 0.5,
            _ => 1.0,
        };
        transform.scale = Vec3::splat(base_scale * lod_factor * fade.alpha);
    }
}

/// Despawn CitizenSprite + SceneRoot when a citizen transitions to Abstract tier
/// AND has no active fade-out (i.e., the fade already completed or was never started).
/// Citizens with an active LodFade are handled by update_lod_fade when the fade completes.
#[allow(clippy::type_complexity)]
pub fn despawn_abstract_sprites(
    mut commands: Commands,
    query: Query<(Entity, &LodTier), (With<CitizenSprite>, Changed<LodTier>, Without<LodFade>)>,
) {
    for (entity, lod) in &query {
        if *lod == LodTier::Abstract {
            commands
                .entity(entity)
                .remove::<(CitizenSprite, CitizenMeshKind, SceneRoot)>();
        }
    }
}
