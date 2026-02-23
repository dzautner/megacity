//! Types and constants for the satellite view overlay.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Camera distance at which the satellite overlay starts fading in.
pub(crate) const TRANSITION_START: f32 = 2500.0;

/// Camera distance at which the satellite overlay is fully opaque.
pub(crate) const TRANSITION_END: f32 = 3800.0;

/// Resolution of the satellite map texture (pixels per axis).
pub(crate) const TEX_SIZE: usize = 512;

/// Y position of the satellite quad (above terrain at Y=0).
pub(crate) const SATELLITE_Y: f32 = 5.0;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the current satellite view blend factor and dirty state.
#[derive(Resource)]
pub struct SatelliteView {
    /// 0.0 = fully 3D, 1.0 = fully satellite.
    pub blend: f32,
    /// Whether the satellite texture needs regeneration.
    pub dirty: bool,
}

impl Default for SatelliteView {
    fn default() -> Self {
        Self {
            blend: 0.0,
            dirty: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker for the satellite overlay quad entity.
#[derive(Component)]
pub struct SatelliteQuad;
