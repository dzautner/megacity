mod calculations;
mod systems;
mod tests;
mod types;

pub use systems::{process_accidents, spawn_accidents, TrafficAccidentsPlugin};
pub use types::{AccidentTracker, TrafficAccident};
