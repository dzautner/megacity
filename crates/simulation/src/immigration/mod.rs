mod attractiveness;
mod plugin;
mod random;
#[cfg(test)]
mod tests;
mod types;
mod waves;

pub use attractiveness::compute_attractiveness;
pub use plugin::ImmigrationPlugin;
pub use types::{CityAttractiveness, ImmigrationStats};
pub use waves::immigration_wave;
