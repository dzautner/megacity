//! Road grade and elevation indicators shown during road preview.
//!
//! When the player is placing a road (freeform Bezier drawing, `DrawPhase::PlacedStart`),
//! this module overlays:
//!
//! - **Elevation numbers** at regular intervals along the preview curve
//! - **Grade color coding**: green (0-3%), yellow (3-6%), red (6%+)
//! - **Bridge indicator** where the road crosses water cells
//! - **Tunnel indicator** where the road goes through elevated terrain (hill)

mod constants;
mod helpers;
mod indicators;
mod tests;

pub use helpers::grade_to_color;
pub use indicators::draw_road_grade_indicators;
