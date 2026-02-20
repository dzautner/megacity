use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::movement::ActivityTimer;
use crate::spatial_grid::SpatialGrid;

/// LOD tier for citizens. Stored as a component on ALL citizen entities
/// to avoid archetype fragmentation.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LodTier {
    /// Full simulation: individual pathfinding, rendered
    #[default]
    Full, // ~5K
    /// Simplified: pre-computed paths, rendered as pixels
    Simplified, // ~50K
    /// Abstract: state machine only, not rendered
    Abstract, // ~200K
              // Tier 3 (Statistical) is NOT stored in ECS; see Districts resource
}

/// Compressed representation for Tier 2 citizens (12 bytes)
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompressedCitizen {
    /// Packed: grid_x(8) | grid_y(8) | state(4) | age(4)
    pub packed_a: u32,
    /// happiness(8) | home_district(8) | work_district(8) | flags(8)
    pub packed_b: u32,
}

impl CompressedCitizen {
    pub fn new(
        gx: u8,
        gy: u8,
        state: CitizenState,
        age: u8,
        happiness: u8,
        home_d: u8,
        work_d: u8,
    ) -> Self {
        let state_bits = match state {
            CitizenState::AtHome => 0u32,
            CitizenState::CommutingToWork => 1,
            CitizenState::Working => 2,
            CitizenState::CommutingHome => 3,
            CitizenState::CommutingToShop => 4,
            CitizenState::Shopping => 5,
            CitizenState::CommutingToLeisure => 6,
            CitizenState::AtLeisure => 7,
            CitizenState::CommutingToSchool => 8,
            CitizenState::AtSchool => 9,
        };
        let packed_a =
            (gx as u32) << 24 | (gy as u32) << 16 | (state_bits << 12) | (age as u32 & 0xF);
        let packed_b = (happiness as u32) << 24 | (home_d as u32) << 16 | (work_d as u32) << 8;
        Self { packed_a, packed_b }
    }

    pub fn grid_x(&self) -> u8 {
        (self.packed_a >> 24) as u8
    }

    pub fn grid_y(&self) -> u8 {
        ((self.packed_a >> 16) & 0xFF) as u8
    }

    pub fn state(&self) -> CitizenState {
        match (self.packed_a >> 12) & 0xF {
            0 => CitizenState::AtHome,
            1 => CitizenState::CommutingToWork,
            2 => CitizenState::Working,
            3 => CitizenState::CommutingHome,
            4 => CitizenState::CommutingToShop,
            5 => CitizenState::Shopping,
            6 => CitizenState::CommutingToLeisure,
            7 => CitizenState::AtLeisure,
            8 => CitizenState::CommutingToSchool,
            9 => CitizenState::AtSchool,
            _ => CitizenState::AtHome,
        }
    }

    pub fn happiness(&self) -> u8 {
        (self.packed_b >> 24) as u8
    }
}

/// Camera viewport for LOD calculations
#[derive(Resource, Default)]
pub struct ViewportBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

pub fn update_viewport_bounds(
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut bounds: ResMut<ViewportBounds>,
) {
    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };

    // Estimate visible ground (Y=0) rectangle from 3D camera.
    // Cast rays from viewport corners to the Y=0 plane.
    let viewport_size = camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let corners = [
        Vec2::ZERO,
        Vec2::new(viewport_size.x, 0.0),
        Vec2::new(0.0, viewport_size.y),
        viewport_size,
    ];

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for corner in &corners {
        if let Ok(ray) = camera.viewport_to_world(cam_transform, *corner) {
            if ray.direction.y.abs() > 1e-6 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let hit = ray.origin + ray.direction * t;
                    min_x = min_x.min(hit.x);
                    max_x = max_x.max(hit.x);
                    min_z = min_z.min(hit.z);
                    max_z = max_z.max(hit.z);
                }
            }
        }
    }

    if min_x < f32::MAX {
        bounds.min_x = min_x;
        bounds.max_x = max_x;
        // LOD system uses min_y/max_y for the ground plane (which is XZ in 3D)
        bounds.min_y = min_z;
        bounds.max_y = max_z;
    }
}

pub fn update_spatial_grid(
    mut spatial: ResMut<SpatialGrid>,
    citizens: Query<(Entity, &Position), With<Citizen>>,
) {
    spatial.clear();
    for (entity, pos) in &citizens {
        spatial.insert(entity, pos.x, pos.y);
    }
}

/// Assign LOD tiers based on viewport distance
pub fn assign_lod_tiers(
    bounds: Res<ViewportBounds>,
    _spatial: Res<SpatialGrid>,
    mut citizens: Query<(Entity, &Position, &mut LodTier), With<Citizen>>,
) {
    // Expanded viewport for tier boundaries
    // Use tighter margins on WASM to reduce draw calls and memory pressure
    let margin = if cfg!(target_arch = "wasm32") {
        200.0
    } else {
        500.0
    };
    let full_min_x = bounds.min_x - margin;
    let full_max_x = bounds.max_x + margin;
    let full_min_y = bounds.min_y - margin;
    let full_max_y = bounds.max_y + margin;

    let simplified_margin = if cfg!(target_arch = "wasm32") {
        600.0
    } else {
        1500.0
    };

    for (_entity, pos, mut tier) in &mut citizens {
        let _in_viewport = pos.x >= bounds.min_x
            && pos.x <= bounds.max_x
            && pos.y >= bounds.min_y
            && pos.y <= bounds.max_y;

        let in_full_range = pos.x >= full_min_x
            && pos.x <= full_max_x
            && pos.y >= full_min_y
            && pos.y <= full_max_y;

        let in_simplified_range = pos.x >= bounds.min_x - simplified_margin
            && pos.x <= bounds.max_x + simplified_margin
            && pos.y >= bounds.min_y - simplified_margin
            && pos.y <= bounds.max_y + simplified_margin;

        let new_tier = if in_full_range {
            LodTier::Full
        } else if in_simplified_range {
            LodTier::Simplified
        } else {
            LodTier::Abstract
        };

        if *tier != new_tier {
            *tier = new_tier;
        }
    }
}

/// When a citizen enters Abstract tier, strip heavy components and insert CompressedCitizen.
/// This reduces per-entity memory from ~200 bytes + 12 components to ~50 bytes + 5 components.
#[allow(clippy::type_complexity)]
pub fn compress_abstract_citizens(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &LodTier,
            &CitizenStateComp,
            &CitizenDetails,
            &HomeLocation,
        ),
        (With<Citizen>, Changed<LodTier>, Without<CompressedCitizen>),
    >,
) {
    for (entity, lod, state, details, home) in &query {
        if *lod != LodTier::Abstract {
            continue;
        }
        let compressed = CompressedCitizen::new(
            home.grid_x as u8,
            home.grid_y as u8,
            state.0,
            details.age,
            details.happiness as u8,
            0, // home district placeholder
            0, // work district placeholder
        );
        commands.entity(entity).insert(compressed).remove::<(
            PathCache,
            Velocity,
            Needs,
            Personality,
            Family,
            ActivityTimer,
        )>();
    }
}

/// When a citizen leaves Abstract tier, restore full components from CompressedCitizen.
#[allow(clippy::type_complexity)]
pub fn decompress_active_citizens(
    mut commands: Commands,
    query: Query<(Entity, &LodTier, &CompressedCitizen), (With<Citizen>, Changed<LodTier>)>,
) {
    for (entity, lod, _compressed) in &query {
        if *lod == LodTier::Abstract {
            continue;
        }
        // Re-add the stripped components with defaults
        commands.entity(entity).insert((
            Velocity { x: 0.0, y: 0.0 },
            PathCache::new(Vec::new()),
            Needs::default(),
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Family::default(),
            ActivityTimer::default(),
        ));
        commands.entity(entity).remove::<CompressedCitizen>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_citizen_roundtrip() {
        let c = CompressedCitizen::new(100, 200, CitizenState::Working, 30, 75, 5, 10);
        assert_eq!(c.grid_x(), 100);
        assert_eq!(c.grid_y(), 200);
        assert_eq!(c.state(), CitizenState::Working);
        assert_eq!(c.happiness(), 75);
    }

    #[test]
    fn test_lod_tier_default() {
        assert_eq!(LodTier::default(), LodTier::Full);
    }

    #[test]
    fn test_compressed_all_states() {
        for (state, expected) in [
            (CitizenState::AtHome, CitizenState::AtHome),
            (CitizenState::CommutingToWork, CitizenState::CommutingToWork),
            (CitizenState::Working, CitizenState::Working),
            (CitizenState::CommutingHome, CitizenState::CommutingHome),
        ] {
            let c = CompressedCitizen::new(0, 0, state, 0, 0, 0, 0);
            assert_eq!(c.state(), expected);
        }
    }
}

pub struct LodPlugin;

impl Plugin for LodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportBounds>()
            .add_systems(
                Update,
                (
                    update_viewport_bounds,
                    update_spatial_grid.run_if(crate::lod_frame_ready),
                    assign_lod_tiers.run_if(crate::lod_frame_ready),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (compress_abstract_citizens, decompress_active_citizens).after(assign_lod_tiers),
            );
    }
}
