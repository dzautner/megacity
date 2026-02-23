mod stats;
mod systems;
mod tier;

#[cfg(test)]
mod tests;

pub use stats::AirportStats;
pub use systems::{update_airports, AirportPlugin};
pub use tier::AirportTier;
