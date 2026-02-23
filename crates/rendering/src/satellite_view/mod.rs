//! Satellite View: 2D top-down map overlay at maximum zoom-out.
//!
//! When the camera zooms far enough out, this module renders a flat textured
//! quad covering the entire city showing terrain colors, road lines, and
//! building area fills. The overlay smoothly fades in as the camera distance
//! increases from `TRANSITION_START` to `TRANSITION_END`, and 3D objects
//! (buildings, roads, citizens, props) fade out simultaneously.

mod colors;
mod image_gen;
mod painting;
mod systems;
mod tests;
mod types;

pub use systems::SatelliteViewPlugin;
pub use types::{SatelliteQuad, SatelliteView};
