mod plugin;
mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use plugin::WindPlugin;
pub use systems::update_wind;
pub use types::{prevailing_direction_for_zone, WindState};
