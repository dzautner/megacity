// ---------------------------------------------------------------------------
// Save Stages: typed intermediate structures for the staged save pipeline
// ---------------------------------------------------------------------------
//
// The save pipeline is split into focused stages, each responsible for
// collecting one domain of game state into `SaveData` fields. This avoids a
// single God-function with 40+ parameters and makes it safe to evolve each
// domain independently.
//
// ## Pipeline overview
//
// ```text
//   ECS World
//     |
//     +-- collect_grid_stage        -> GridStageOutput       (grid, roads, road_segments)
//     +-- collect_economy_stage     -> EconomyStageOutput    (clock, budget, demand, ext_budget, loans)
//     +-- collect_entity_stage      -> EntityStageOutput     (buildings, citizens, utilities, services, water_sources)
//     +-- collect_environment_stage -> EnvironmentStageOutput (weather, climate, UHI, stormwater, snow, ...)
//     +-- collect_disaster_stage    -> DisasterStageOutput   (drought, heat_wave, cold_snap, flood, ...)
//     +-- collect_policy_stage      -> PolicyStageOutput     (policies, unlocks, recycling, composting, ...)
//     |
//     +---> assemble_save_data(stages...) -> SaveData
// ```
//
// Each `collect_*` function takes only the references it needs, keeping call
// sites clean and type-safe.

mod assemble;
mod disaster_stage;
mod economy_stage;
mod entity_stage;
mod environment_stage;
mod grid_stage;
mod policy_stage;

// Re-export all public items so callers can continue using
// `crate::save_stages::*` without changes.
pub use assemble::*;
pub use disaster_stage::*;
pub use economy_stage::*;
pub use entity_stage::*;
pub use environment_stage::*;
pub use grid_stage::*;
pub use policy_stage::*;
