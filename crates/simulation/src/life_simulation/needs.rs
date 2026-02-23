use bevy::prelude::*;

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, HomeLocation, Needs,
};
use crate::grid::WorldGrid;
use crate::time_of_day::GameClock;

use super::LifeSimTimer;

// ---------------------------------------------------------------------------
// Needs decay/fulfillment rates (per NEEDS_INTERVAL = 10 ticks)
// ---------------------------------------------------------------------------

// Decay rates (per interval, while active)
const HUNGER_DECAY: f32 = 2.1; // empty in ~8h
const ENERGY_DECAY: f32 = 1.0; // empty in ~16h
const SOCIAL_DECAY: f32 = 0.23; // empty in ~3 days
const FUN_DECAY: f32 = 0.35; // empty in ~2 days

// Restoration rates (per interval, at appropriate activity)
const HUNGER_RESTORE_HOME: f32 = 8.0;
const HUNGER_RESTORE_SHOP: f32 = 5.0;
const ENERGY_RESTORE_HOME_NIGHT: f32 = 4.0;
const ENERGY_RESTORE_HOME_DAY: f32 = 1.5;
const SOCIAL_RESTORE_WORK: f32 = 0.5;
const SOCIAL_RESTORE_LEISURE: f32 = 3.0;
const SOCIAL_RESTORE_SCHOOL: f32 = 2.0;
const FUN_RESTORE_LEISURE: f32 = 5.0;
const FUN_RESTORE_SHOP: f32 = 1.5;
const FUN_DRAIN_WORK: f32 = 0.3; // extra fun drain while working

// ---------------------------------------------------------------------------
// System: update_needs
// ---------------------------------------------------------------------------

pub fn update_needs(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    grid: Res<WorldGrid>,
    mut citizens: Query<
        (
            &CitizenStateComp,
            &mut Needs,
            &HomeLocation,
            &CitizenDetails,
        ),
        With<Citizen>,
    >,
) {
    if clock.paused {
        return;
    }
    timer.needs_tick += 1;
    if timer.needs_tick < super::NEEDS_INTERVAL {
        return;
    }
    timer.needs_tick = 0;

    let is_night = clock.hour < 6.0 || clock.hour >= 22.0;

    for (state, mut needs, home, _details) in &mut citizens {
        // --- Decay ---
        needs.hunger = (needs.hunger - HUNGER_DECAY).max(0.0);
        needs.energy = (needs.energy - ENERGY_DECAY).max(0.0);
        needs.social = (needs.social - SOCIAL_DECAY).max(0.0);
        needs.fun = (needs.fun - FUN_DECAY).max(0.0);

        // --- Fulfillment based on current activity ---
        match state.0 {
            CitizenState::AtHome => {
                // Eating restores hunger
                if needs.hunger < 80.0 {
                    needs.hunger = (needs.hunger + HUNGER_RESTORE_HOME).min(100.0);
                }
                // Resting restores energy
                if is_night {
                    needs.energy = (needs.energy + ENERGY_RESTORE_HOME_NIGHT).min(100.0);
                } else {
                    needs.energy = (needs.energy + ENERGY_RESTORE_HOME_DAY).min(100.0);
                }
            }
            CitizenState::Working => {
                needs.fun = (needs.fun - FUN_DRAIN_WORK).max(0.0);
                needs.social = (needs.social + SOCIAL_RESTORE_WORK).min(100.0);
            }
            CitizenState::Shopping => {
                needs.hunger = (needs.hunger + HUNGER_RESTORE_SHOP).min(100.0);
                needs.fun = (needs.fun + FUN_RESTORE_SHOP).min(100.0);
            }
            CitizenState::AtLeisure => {
                needs.fun = (needs.fun + FUN_RESTORE_LEISURE).min(100.0);
                needs.social = (needs.social + SOCIAL_RESTORE_LEISURE).min(100.0);
            }
            CitizenState::AtSchool => {
                needs.social = (needs.social + SOCIAL_RESTORE_SCHOOL).min(100.0);
            }
            _ => {} // commuting states: just decay
        }

        // --- Comfort: based on housing quality ---
        let home_cell = grid.get(home.grid_x, home.grid_y);
        let mut comfort = 40.0; // base
        if home_cell.has_power {
            comfort += 20.0;
        }
        if home_cell.has_water {
            comfort += 20.0;
        }
        // Smooth towards target comfort (don't snap)
        needs.comfort += (comfort - needs.comfort) * 0.1;
        needs.comfort = needs.comfort.clamp(0.0, 100.0);
    }
}
