pub mod calculations;
pub mod constants;
pub mod systems;
mod tests;
pub mod types;

pub use constants::*;
pub use systems::{update_water_conservation, WaterConservationPlugin};
pub use types::WaterConservationState;
