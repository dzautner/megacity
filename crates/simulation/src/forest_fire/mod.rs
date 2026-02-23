mod constants;
mod helpers;
mod resources;
mod systems;
mod tests;

pub use resources::{ForestFireGrid, ForestFireStats};
pub use systems::{update_forest_fire, ForestFirePlugin};
