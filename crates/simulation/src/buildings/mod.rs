mod construction;
mod plugin;
mod spawning;
#[cfg(test)]
mod tests;
pub mod types;

pub use construction::progress_construction;
pub use plugin::BuildingsPlugin;
pub use spawning::{building_spawner, rebuild_eligible_cells, BuildingSpawnTimer, EligibleCells};
pub use types::{max_level_for_far, Building, MixedUseBuilding, UnderConstruction};
