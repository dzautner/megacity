mod jobs;
mod life_events;
mod needs;
mod personality_health;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Timers
// ---------------------------------------------------------------------------

#[derive(Resource, Default, Serialize, Deserialize, Clone, Debug)]
pub struct LifeSimTimer {
    pub needs_tick: u32,
    pub life_event_tick: u32,
    pub salary_tick: u32,
    pub education_tick: u32,
    pub job_seek_tick: u32,
    pub personality_tick: u32,
    pub health_tick: u32,
}

const NEEDS_INTERVAL: u32 = 10; // every 10 ticks (~1 game minute)
const LIFE_EVENT_INTERVAL: u32 = 600; // every 600 ticks (~1 game hour)
const SALARY_INTERVAL: u32 = 43200; // every 43200 ticks (~30 game days)
const EDUCATION_INTERVAL: u32 = 1440; // every 1440 ticks (~1 game day)
const JOB_SEEK_INTERVAL: u32 = 300; // every 300 ticks (~30 game minutes)
const PERSONALITY_INTERVAL: u32 = 2880; // every 2880 ticks (~2 game days)
const HEALTH_INTERVAL: u32 = 1440; // every 1440 ticks (~1 game day)

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use jobs::{education_advancement, job_seeking, salary_payment};
pub use life_events::life_events;
pub use needs::update_needs;
pub use personality_health::{evolve_personality, retire_workers, update_health};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct LifeSimulationPlugin;

impl Plugin for LifeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LifeSimTimer>()
            .add_systems(
                FixedUpdate,
                (
                    update_needs,
                    education_advancement,
                    salary_payment,
                    job_seeking,
                    life_events,
                    retire_workers,
                )
                    .after(crate::happiness::update_happiness)
                    .in_set(crate::SimulationSet::Simulation),
            )
            .add_systems(
                FixedUpdate,
                (evolve_personality, update_health)
                    .after(update_needs)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
