//! Intersection Auto-Detection Preview (UX-023)
//!
//! When drawing roads in freeform mode, this system detects where the preview
//! road crosses existing road segments and renders colored markers at each
//! intersection point:
//!
//! - **Green diamond**: A valid new intersection that will create a new node
//!   in the road network when the road is placed.
//! - **Yellow diamond**: The intersection is close to an existing node and will
//!   snap to it rather than creating a new one.
//!
//! This gives players visual feedback about how their road will connect to
//! the existing network before they commit to placing it.

mod geometry;
mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use systems::IntersectionPreviewPlugin;
pub use types::{DetectedIntersection, IntersectionKind, IntersectionPreviewState};
