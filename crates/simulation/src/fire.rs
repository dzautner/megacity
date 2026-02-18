use bevy::prelude::*;
use rand::Rng;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, CitizenStateComp, CitizenState, HomeLocation, WorkLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};
use crate::happiness::ServiceCoverageGrid;
use crate::SlowTickTimer;

// =============================================================================
// Resources & Components
// =============================================================================

/// Per-cell fire intensity grid. 0 = no fire, 1-100 = fire intensity.
#[derive(Resource)]
pub struct FireGrid {
    pub fire_levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for FireGrid {
    fn default() -> Self {
        Self {
            fire_levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl FireGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.fire_levels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.fire_levels[y * self.width + x] = val;
    }
}

/// Marker component for buildings that are currently on fire.
#[derive(Component)]
pub struct OnFire {
    pub intensity: f32,
    pub ticks_burning: u32,
}

// =============================================================================
// Constants
// =============================================================================

/// Base chance per industrial building per slow tick to catch fire (0.1%).
const BASE_FIRE_CHANCE: f32 = 0.001;

/// Multiplier when there is no fire coverage.
const NO_COVERAGE_MULTIPLIER: f32 = 3.0;

/// Chance per tick for fire to spread to an adjacent building.
const SPREAD_CHANCE: f32 = 0.05;

/// Rate at which fire intensity increases per tick (capped at 100).
const INTENSITY_GROWTH_RATE: f32 = 0.5;

/// How much fire coverage reduces intensity per tick.
const COVERAGE_REDUCTION_PER_TICK: f32 = 2.0;

/// Intensity threshold above which buildings take destruction damage.
const DESTRUCTION_INTENSITY_THRESHOLD: f32 = 50.0;

/// Number of ticks above the intensity threshold before destruction.
const DESTRUCTION_TICK_THRESHOLD: u32 = 200;

/// Health damage per tick to citizens in burning buildings.
const CITIZEN_FIRE_HEALTH_DAMAGE: f32 = 0.5;

// =============================================================================
// Systems
// =============================================================================

/// Randomly starts fires on industrial buildings. Runs on SlowTickTimer.
/// Buildings without fire coverage have a higher chance of catching fire.
#[allow(clippy::too_many_arguments)]
pub fn start_random_fires(
    slow_timer: Res<SlowTickTimer>,
    mut commands: Commands,
    buildings: Query<(Entity, &Building), Without<OnFire>>,
    coverage: Res<ServiceCoverageGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut rng = rand::thread_rng();

    for (entity, building) in &buildings {
        // Only industrial buildings can spontaneously catch fire
        if building.zone_type != ZoneType::Industrial {
            continue;
        }

        let idx = ServiceCoverageGrid::idx(building.grid_x, building.grid_y);
        let has_fire_coverage = coverage.has_fire(idx);

        let chance = if has_fire_coverage {
            BASE_FIRE_CHANCE
        } else {
            BASE_FIRE_CHANCE * NO_COVERAGE_MULTIPLIER
        };

        if rng.gen::<f32>() < chance {
            commands.entity(entity).insert(OnFire {
                intensity: 1.0,
                ticks_burning: 0,
            });
        }
    }
}

/// Spreads fire from burning buildings to adjacent buildings.
/// Fire intensity increases over time based on ticks_burning.
pub fn spread_fire(
    mut commands: Commands,
    mut fire_grid: ResMut<FireGrid>,
    grid: Res<WorldGrid>,
    mut burning: Query<(&Building, &mut OnFire)>,
    not_burning: Query<(Entity, &Building), Without<OnFire>>,
) {
    let mut rng = rand::thread_rng();

    // Collect adjacent building positions from burning buildings for spread candidates
    let mut spread_targets: Vec<(usize, usize)> = Vec::new();

    for (building, mut on_fire) in &mut burning {
        // Increase ticks and intensity
        on_fire.ticks_burning += 1;
        on_fire.intensity = (on_fire.ticks_burning as f32 * INTENSITY_GROWTH_RATE)
            .min(100.0)
            .max(on_fire.intensity);

        // Update fire grid
        fire_grid.set(
            building.grid_x,
            building.grid_y,
            on_fire.intensity as u8,
        );

        // Collect neighbors for potential spread
        let (neighbors, count) = grid.neighbors4(building.grid_x, building.grid_y);
        for &(nx, ny) in &neighbors[..count] {
            if rng.gen::<f32>() < SPREAD_CHANCE {
                spread_targets.push((nx, ny));
            }
        }
    }

    // Apply spread: find non-burning buildings at spread target positions and ignite them
    for (entity, building) in &not_burning {
        if spread_targets.contains(&(building.grid_x, building.grid_y)) {
            commands.entity(entity).insert(OnFire {
                intensity: 1.0,
                ticks_burning: 0,
            });
        }
    }
}

/// Extinguishes fires in areas with fire coverage.
/// Fire coverage reduces intensity by COVERAGE_REDUCTION_PER_TICK each tick.
/// When intensity reaches 0, the OnFire component is removed.
/// Without coverage, fire burns indefinitely (until building destruction).
pub fn extinguish_fires(
    mut commands: Commands,
    mut fire_grid: ResMut<FireGrid>,
    coverage: Res<ServiceCoverageGrid>,
    mut burning: Query<(Entity, &Building, &mut OnFire)>,
) {
    for (entity, building, mut on_fire) in &mut burning {
        let idx = ServiceCoverageGrid::idx(building.grid_x, building.grid_y);

        if coverage.has_fire(idx) {
            on_fire.intensity -= COVERAGE_REDUCTION_PER_TICK;

            if on_fire.intensity <= 0.0 {
                on_fire.intensity = 0.0;
                fire_grid.set(building.grid_x, building.grid_y, 0);
                commands.entity(entity).remove::<OnFire>();
            } else {
                fire_grid.set(building.grid_x, building.grid_y, on_fire.intensity as u8);
            }
        }
    }
}

/// Destroys buildings that have been burning intensely for too long.
/// Citizens in burning buildings lose health.
#[allow(clippy::too_many_arguments)]
pub fn fire_damage(
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    mut fire_grid: ResMut<FireGrid>,
    burning: Query<(Entity, &Building, &OnFire)>,
    mut citizens: Query<
        (&mut CitizenDetails, &HomeLocation, Option<&WorkLocation>, &CitizenStateComp),
        With<Citizen>,
    >,
) {
    // Collect buildings to destroy (can't despawn while iterating)
    let mut destroyed: Vec<(Entity, usize, usize)> = Vec::new();

    for (entity, building, on_fire) in &burning {
        // Destroy building if intensity > threshold for long enough
        if on_fire.intensity > DESTRUCTION_INTENSITY_THRESHOLD
            && on_fire.ticks_burning > DESTRUCTION_TICK_THRESHOLD
        {
            destroyed.push((entity, building.grid_x, building.grid_y));
        }
    }

    // Damage citizens in burning buildings
    for (mut details, home, work, state) in &mut citizens {
        let citizen_state = state.0;

        // Check if citizen is at home and home is on fire
        if citizen_state == CitizenState::AtHome {
            let home_fire = fire_grid.get(home.grid_x, home.grid_y);
            if home_fire > 0 {
                details.health = (details.health - CITIZEN_FIRE_HEALTH_DAMAGE).max(0.0);
            }
        }

        // Check if citizen is at work and work building is on fire
        if let Some(work_loc) = work {
            if citizen_state == CitizenState::Working {
                let work_fire = fire_grid.get(work_loc.grid_x, work_loc.grid_y);
                if work_fire > 0 {
                    details.health = (details.health - CITIZEN_FIRE_HEALTH_DAMAGE).max(0.0);
                }
            }
        }
    }

    // Despawn destroyed buildings and clear grid cells
    for (entity, gx, gy) in destroyed {
        fire_grid.set(gx, gy, 0);
        let cell = grid.get_mut(gx, gy);
        if cell.building_id == Some(entity) {
            cell.building_id = None;
        }
        cell.zone = ZoneType::None;
        commands.entity(entity).despawn();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_grid_default() {
        let grid = FireGrid::default();
        assert_eq!(grid.width, GRID_WIDTH);
        assert_eq!(grid.height, GRID_HEIGHT);
        assert_eq!(grid.fire_levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(grid.fire_levels.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_fire_grid_get_set() {
        let mut grid = FireGrid::default();
        assert_eq!(grid.get(10, 20), 0);
        grid.set(10, 20, 75);
        assert_eq!(grid.get(10, 20), 75);
    }

    #[test]
    fn test_fire_grid_boundary() {
        let mut grid = FireGrid::default();
        grid.set(0, 0, 100);
        assert_eq!(grid.get(0, 0), 100);
        grid.set(GRID_WIDTH - 1, GRID_HEIGHT - 1, 50);
        assert_eq!(grid.get(GRID_WIDTH - 1, GRID_HEIGHT - 1), 50);
    }

    #[test]
    fn test_on_fire_component() {
        let fire = OnFire {
            intensity: 25.0,
            ticks_burning: 10,
        };
        assert_eq!(fire.intensity, 25.0);
        assert_eq!(fire.ticks_burning, 10);
    }

    #[test]
    fn test_intensity_growth_capped() {
        // Simulate intensity growth: ticks * rate, capped at 100
        let ticks = 300u32;
        let intensity = (ticks as f32 * INTENSITY_GROWTH_RATE).min(100.0);
        assert_eq!(intensity, 100.0);
    }

    #[test]
    fn test_constants_valid() {
        assert!(BASE_FIRE_CHANCE > 0.0 && BASE_FIRE_CHANCE < 1.0);
        assert!(SPREAD_CHANCE > 0.0 && SPREAD_CHANCE < 1.0);
        assert!(INTENSITY_GROWTH_RATE > 0.0);
        assert!(COVERAGE_REDUCTION_PER_TICK > 0.0);
        assert!(DESTRUCTION_INTENSITY_THRESHOLD > 0.0);
        assert!(DESTRUCTION_TICK_THRESHOLD > 0);
        assert!(NO_COVERAGE_MULTIPLIER > 1.0);
    }
}
