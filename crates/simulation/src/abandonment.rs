use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::land_value::LandValueGrid;
use crate::TickCounter;

/// Marker component for buildings that have been abandoned.
/// Tracks how many ticks the building has been in an abandoned state.
#[derive(Component)]
pub struct Abandoned {
    pub ticks_abandoned: u32,
}

/// How often (in ticks) the abandonment check runs.
const CHECK_INTERVAL: u64 = 50;

/// Number of ticks an abandoned building survives before demolition.
const DEMOLISH_THRESHOLD: u32 = 500;

/// Land value penalty applied per abandoned building within the penalty radius.
const LAND_VALUE_PENALTY: i32 = 5;

/// Radius (in cells, Chebyshev distance) for the neighbor land value penalty.
const PENALTY_RADIUS: i32 = 2;

/// Checks non-abandoned buildings and marks them as `Abandoned` when conditions are met.
///
/// Runs every `CHECK_INTERVAL` ticks. A building becomes abandoned when:
/// - Its cell has neither power nor water, OR
/// - It has 0 occupants and its level is > 1 (upgraded building that emptied out).
pub fn check_building_abandonment(
    mut commands: Commands,
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    buildings: Query<(Entity, &Building), Without<Abandoned>>,
) {
    if !tick.0.is_multiple_of(CHECK_INTERVAL) {
        return;
    }

    for (entity, building) in &buildings {
        let x = building.grid_x;
        let y = building.grid_y;

        if !grid.in_bounds(x, y) {
            continue;
        }

        let cell = grid.get(x, y);

        // Condition 1: both power and water are missing
        let no_utilities = !cell.has_power && !cell.has_water;

        // Condition 2: upgraded building with zero occupants
        let empty_upgraded = building.occupants == 0 && building.level > 1;

        if no_utilities || empty_upgraded {
            commands
                .entity(entity)
                .insert(Abandoned { ticks_abandoned: 0 });
        }
    }
}

/// Processes buildings that are already abandoned.
///
/// Each tick:
/// - Increments `ticks_abandoned`.
/// - If the cell has regained both power AND water, the building recovers (Abandoned removed).
/// - If `ticks_abandoned` exceeds `DEMOLISH_THRESHOLD`, the building is demolished:
///   the entity is despawned and the grid cell's `building_id` and `zone` are cleared.
/// - While abandoned, building occupants are forced to 0.
pub fn process_abandoned_buildings(
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    mut buildings: Query<(Entity, &mut Building, &mut Abandoned)>,
) {
    for (entity, mut building, mut abandoned) in &mut buildings {
        let x = building.grid_x;
        let y = building.grid_y;

        abandoned.ticks_abandoned += 1;

        // Force occupants to 0 while abandoned
        building.occupants = 0;

        if !grid.in_bounds(x, y) {
            continue;
        }

        let cell = grid.get(x, y);
        let has_power = cell.has_power;
        let has_water = cell.has_water;

        // Recovery: if both utilities are restored, remove Abandoned
        if has_power && has_water {
            commands.entity(entity).remove::<Abandoned>();
            continue;
        }

        // Demolition: building has been abandoned too long
        if abandoned.ticks_abandoned > DEMOLISH_THRESHOLD {
            // Clear grid cell
            let cell_mut = grid.get_mut(x, y);
            cell_mut.building_id = None;
            cell_mut.zone = crate::grid::ZoneType::None;

            commands.entity(entity).despawn();
        }
    }
}

/// Reduces land value around abandoned buildings.
///
/// Each abandoned building applies a `LAND_VALUE_PENALTY` to all cells within
/// `PENALTY_RADIUS` (Chebyshev distance). This runs on the `SlowTickTimer` cadence
/// to align with the existing land-value update cycle.
pub fn abandoned_land_value_penalty(
    slow_timer: Res<crate::SlowTickTimer>,
    mut land_value: ResMut<LandValueGrid>,
    abandoned_buildings: Query<&Building, With<Abandoned>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for building in &abandoned_buildings {
        let bx = building.grid_x as i32;
        let by = building.grid_y as i32;

        for dy in -PENALTY_RADIUS..=PENALTY_RADIUS {
            for dx in -PENALTY_RADIUS..=PENALTY_RADIUS {
                let nx = bx + dx;
                let ny = by + dy;

                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let ux = nx as usize;
                    let uy = ny as usize;
                    let cur = land_value.get(ux, uy) as i32;
                    let new_val = (cur - LAND_VALUE_PENALTY).max(0) as u8;
                    land_value.set(ux, uy, new_val);
                }
            }
        }
    }
}

pub struct AbandonmentPlugin;

impl Plugin for AbandonmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                check_building_abandonment,
                bevy::ecs::schedule::apply_deferred,
                process_abandoned_buildings,
            )
                .chain()
                .after(crate::utilities::propagate_utilities)
                .in_set(crate::SimulationSet::Simulation),
        )
        .add_systems(
            FixedUpdate,
            abandoned_land_value_penalty
                .after(crate::land_value::update_land_value)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
