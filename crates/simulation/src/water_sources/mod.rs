//! Water sources module: wells, surface intakes, reservoirs, and desalination plants.

mod plugin;
mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use plugin::WaterSourcesPlugin;
pub use systems::{aggregate_water_source_supply, replenish_reservoirs, update_water_sources};
pub use types::{WaterSource, WaterSourceType};
