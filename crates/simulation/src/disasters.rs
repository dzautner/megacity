use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::SlowTickTimer;
use crate::TickCounter;
use crate::TestSafetyNet;

// =============================================================================
// Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisasterType {
    Tornado,
    Earthquake,
    Flood,
}

impl DisasterType {
    pub fn name(self) -> &'static str {
        match self {
            DisasterType::Tornado => "Tornado",
            DisasterType::Earthquake => "Earthquake",
            DisasterType::Flood => "Flood",
        }
    }
}

pub struct DisasterInstance {
    pub disaster_type: DisasterType,
    pub center_x: usize,
    pub center_y: usize,
    pub radius: usize,
    pub ticks_remaining: u32,
    pub damage_applied: bool,
}

#[derive(Resource, Default)]
pub struct ActiveDisaster {
    pub current: Option<DisasterInstance>,
}

// =============================================================================
// Deterministic pseudo-random helpers (no rand crate)
// =============================================================================

/// Simple deterministic hash of a u64 value, producing a u64.
/// Uses the splitmix64 algorithm for good distribution.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
fn rand_f32(seed: u64) -> f32 {
    (splitmix64(seed) % 1_000_000) as f32 / 1_000_000.0
}

/// Returns a deterministic pseudo-random usize in [0, max) based on seed.
fn rand_usize(seed: u64, max: usize) -> usize {
    (splitmix64(seed) % max as u64) as usize
}

// =============================================================================
// Constants
// =============================================================================

/// Chance per slow tick that a disaster occurs (0.05%).
const DISASTER_CHANCE: f32 = 0.0005;

/// Tornado configuration.
const TORNADO_RADIUS: usize = 5;
const TORNADO_DURATION: u32 = 50;
const TORNADO_DESTROY_PCT: f32 = 0.30;

/// Earthquake configuration.
const EARTHQUAKE_RADIUS: usize = 10;
const EARTHQUAKE_DURATION: u32 = 20;
const EARTHQUAKE_DESTROY_PCT: f32 = 0.10;

/// Flood configuration.
const FLOOD_RADIUS: usize = 8;
const FLOOD_DURATION: u32 = 100;
const FLOOD_ELEVATION_THRESHOLD: f32 = 0.45;

// =============================================================================
// Systems
// =============================================================================

/// Triggers a random disaster with very low probability each slow tick.
/// Only triggers if no disaster is currently active and disasters are enabled.
pub fn trigger_random_disaster(
    slow_timer: Res<SlowTickTimer>,
    tick: Res<TickCounter>,
    mut active: ResMut<ActiveDisaster>,
    grid: Res<WorldGrid>,
    weather: Res<crate::weather::Weather>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Check if disasters are enabled via the weather resource toggle
    if !weather.disasters_enabled {
        return;
    }

    // Don't trigger if a disaster is already active
    if active.current.is_some() {
        return;
    }

    let seed = tick.0;
    let roll = rand_f32(seed.wrapping_mul(0xdeadbeef));
    if roll >= DISASTER_CHANCE {
        return;
    }

    // Pick a random disaster type (3 variants)
    let type_seed = splitmix64(seed.wrapping_mul(0xcafebabe));
    let disaster_type = match type_seed % 3 {
        0 => DisasterType::Tornado,
        1 => DisasterType::Earthquake,
        _ => DisasterType::Flood,
    };

    // Pick a random location that is on land (not water)
    // Try up to 20 times to find a valid land cell
    let mut center_x = 0;
    let mut center_y = 0;
    let mut found = false;

    for attempt in 0..20u64 {
        let loc_seed = splitmix64(seed.wrapping_add(attempt.wrapping_mul(0x12345)));
        let cx = rand_usize(loc_seed, GRID_WIDTH);
        let cy = rand_usize(splitmix64(loc_seed), GRID_HEIGHT);

        if grid.get(cx, cy).cell_type != CellType::Water {
            center_x = cx;
            center_y = cy;
            found = true;
            break;
        }
    }

    if !found {
        return;
    }

    let (radius, duration) = match disaster_type {
        DisasterType::Tornado => (TORNADO_RADIUS, TORNADO_DURATION),
        DisasterType::Earthquake => (EARTHQUAKE_RADIUS, EARTHQUAKE_DURATION),
        DisasterType::Flood => (FLOOD_RADIUS, FLOOD_DURATION),
    };

    info!(
        "DISASTER: {} struck at ({}, {}) with radius {}!",
        disaster_type.name(),
        center_x,
        center_y,
        radius,
    );

    active.current = Some(DisasterInstance {
        disaster_type,
        center_x,
        center_y,
        radius,
        ticks_remaining: duration,
        damage_applied: false,
    });
}

/// Processes the active disaster each tick: applies damage on first tick,
/// decrements duration, and clears when finished.
#[allow(clippy::too_many_arguments)]
pub fn process_active_disaster(
    mut commands: Commands,
    mut active: ResMut<ActiveDisaster>,
    mut grid: ResMut<WorldGrid>,
    buildings: Query<(Entity, &Building)>,
    tick: Res<TickCounter>,
    safety_net: Option<Res<TestSafetyNet>>,
) {
    if safety_net.is_some() {
        return;
    }
    let disaster = match active.current.as_mut() {
        Some(d) => d,
        None => return,
    };

    // Apply damage on the first tick only
    if !disaster.damage_applied {
        disaster.damage_applied = true;

        let dtype = disaster.disaster_type;
        let cx = disaster.center_x;
        let cy = disaster.center_y;
        let radius = disaster.radius;

        // Collect buildings within the disaster radius
        let mut buildings_in_radius: Vec<(Entity, usize, usize, u8, f32)> = Vec::new();
        for (entity, building) in &buildings {
            let dx = building.grid_x as isize - cx as isize;
            let dy = building.grid_y as isize - cy as isize;
            let dist_sq = (dx * dx + dy * dy) as usize;
            if dist_sq <= radius * radius {
                let elevation = grid.get(building.grid_x, building.grid_y).elevation;
                buildings_in_radius.push((
                    entity,
                    building.grid_x,
                    building.grid_y,
                    building.level,
                    elevation,
                ));
            }
        }

        let mut destroyed: Vec<(Entity, usize, usize)> = Vec::new();
        let mut downgraded: Vec<Entity> = Vec::new();

        match dtype {
            DisasterType::Tornado => {
                // Destroy 30% of buildings in radius
                for (idx, &(entity, gx, gy, _level, _elev)) in
                    buildings_in_radius.iter().enumerate()
                {
                    let hash_seed = tick.0.wrapping_add(idx as u64).wrapping_mul(0xfeedface);
                    if rand_f32(hash_seed) < TORNADO_DESTROY_PCT {
                        destroyed.push((entity, gx, gy));
                    }
                }
            }
            DisasterType::Earthquake => {
                // All buildings in radius lose 1 level (min 1), 10% destroyed
                for (idx, &(entity, gx, gy, level, _elev)) in buildings_in_radius.iter().enumerate()
                {
                    let hash_seed = tick.0.wrapping_add(idx as u64).wrapping_mul(0xbadf00d);
                    if rand_f32(hash_seed) < EARTHQUAKE_DESTROY_PCT {
                        destroyed.push((entity, gx, gy));
                    } else if level > 1 {
                        downgraded.push(entity);
                    }
                }
            }
            DisasterType::Flood => {
                // Destroy all buildings on cells with elevation < threshold within radius
                for &(entity, gx, gy, _level, elevation) in &buildings_in_radius {
                    if elevation < FLOOD_ELEVATION_THRESHOLD {
                        destroyed.push((entity, gx, gy));
                    }
                }
            }
        }

        // Apply downgrades (earthquake)
        if dtype == DisasterType::Earthquake {
            // We need to re-query for mutable access; collect entity IDs and
            // use commands to schedule level changes.
            // Since we cannot get mutable Building from the immutable query,
            // we log the intent here and apply via a helper approach:
            // Spawn a marker component that a later system could process,
            // or simply do it inline with commands + closure.
            // For simplicity, we iterate the query again:
            // Actually, we already have the entity IDs. We can't mutate through
            // an immutable query, so let's collect what we need and handle it
            // outside the borrow.
            for entity in &downgraded {
                // We'll use commands to insert a marker and handle downgrade below
                commands.entity(*entity).insert(EarthquakeDamaged);
            }
        }

        let destroyed_count = destroyed.len();

        // Despawn destroyed buildings and clear grid cells
        for (entity, gx, gy) in destroyed {
            let cell = grid.get_mut(gx, gy);
            if cell.building_id == Some(entity) {
                cell.building_id = None;
            }
            cell.zone = ZoneType::None;
            commands.entity(entity).despawn();
        }

        if destroyed_count > 0 || !downgraded.is_empty() {
            info!(
                "DISASTER DAMAGE: {} destroyed {} buildings, downgraded {} buildings",
                dtype.name(),
                destroyed_count,
                downgraded.len(),
            );
        }
    }

    // Decrement ticks remaining
    disaster.ticks_remaining = disaster.ticks_remaining.saturating_sub(1);

    if disaster.ticks_remaining == 0 {
        if let Some(ref d) = active.current {
            info!(
                "DISASTER ENDED: {} at ({}, {}) has subsided.",
                d.disaster_type.name(),
                d.center_x,
                d.center_y,
            );
        }
        active.current = None;
    }
}

/// Marker component for buildings damaged by an earthquake (lose 1 level).
#[derive(Component)]
pub struct EarthquakeDamaged;

/// System that applies earthquake damage to marked buildings, then removes the marker.
pub fn apply_earthquake_damage(
    mut commands: Commands,
    mut buildings: Query<(Entity, &mut Building), With<EarthquakeDamaged>>,
) {
    for (entity, mut building) in &mut buildings {
        if building.level > 1 {
            building.level -= 1;
            building.capacity = Building::capacity_for_level(building.zone_type, building.level);
            // Evict excess occupants
            if building.occupants > building.capacity {
                building.occupants = building.capacity;
            }
        }
        commands.entity(entity).remove::<EarthquakeDamaged>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disaster_type_name() {
        assert_eq!(DisasterType::Tornado.name(), "Tornado");
        assert_eq!(DisasterType::Earthquake.name(), "Earthquake");
        assert_eq!(DisasterType::Flood.name(), "Flood");
    }

    #[test]
    fn test_active_disaster_default() {
        let ad = ActiveDisaster::default();
        assert!(ad.current.is_none());
    }

    #[test]
    fn test_splitmix64_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        // Different seeds produce different values
        let c = splitmix64(43);
        assert_ne!(a, c);
    }

    #[test]
    fn test_rand_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_f32(seed);
            assert!(
                val >= 0.0 && val < 1.0,
                "rand_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }

    #[test]
    fn test_rand_usize_range() {
        for seed in 0..1000u64 {
            let val = rand_usize(seed, 256);
            assert!(
                val < 256,
                "rand_usize({}, 256) = {} out of range",
                seed,
                val
            );
        }
    }

    #[test]
    fn test_disaster_chance_very_low() {
        // Verify that disaster chance is indeed 0.05%
        assert!((DISASTER_CHANCE - 0.0005).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tornado_config() {
        assert_eq!(TORNADO_RADIUS, 5);
        assert_eq!(TORNADO_DURATION, 50);
        assert!((TORNADO_DESTROY_PCT - 0.30).abs() < f32::EPSILON);
    }

    #[test]
    fn test_earthquake_config() {
        assert_eq!(EARTHQUAKE_RADIUS, 10);
        assert_eq!(EARTHQUAKE_DURATION, 20);
        assert!((EARTHQUAKE_DESTROY_PCT - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_config() {
        assert_eq!(FLOOD_RADIUS, 8);
        assert_eq!(FLOOD_DURATION, 100);
        assert!((FLOOD_ELEVATION_THRESHOLD - 0.45).abs() < f32::EPSILON);
    }

    #[test]
    fn test_disaster_instance_creation() {
        let instance = DisasterInstance {
            disaster_type: DisasterType::Tornado,
            center_x: 100,
            center_y: 150,
            radius: TORNADO_RADIUS,
            ticks_remaining: TORNADO_DURATION,
            damage_applied: false,
        };
        assert_eq!(instance.disaster_type, DisasterType::Tornado);
        assert_eq!(instance.center_x, 100);
        assert_eq!(instance.center_y, 150);
        assert_eq!(instance.radius, 5);
        assert_eq!(instance.ticks_remaining, 50);
        assert!(!instance.damage_applied);
    }

    #[test]
    fn test_rand_distribution_reasonable() {
        // Check that over many samples, results spread reasonably
        let mut below_half = 0u32;
        let samples = 10_000u64;
        for seed in 0..samples {
            if rand_f32(seed.wrapping_mul(0x9876)) < 0.5 {
                below_half += 1;
            }
        }
        // Should be roughly 50% (allow wide margin for deterministic hash)
        let ratio = below_half as f64 / samples as f64;
        assert!(ratio > 0.3 && ratio < 0.7, "Distribution skewed: {}", ratio);
    }
}

pub struct DisastersPlugin;

impl Plugin for DisastersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDisaster>().add_systems(
            FixedUpdate,
            (
                trigger_random_disaster,
                process_active_disaster,
                bevy::ecs::schedule::apply_deferred,
                apply_earthquake_damage,
            )
                .chain()
                .after(crate::fire::fire_damage)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
