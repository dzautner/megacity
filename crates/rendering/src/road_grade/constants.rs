//! Constants and color definitions for road grade indicators.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Distance between elevation sample points along the preview curve (world units).
pub(crate) const ELEVATION_SAMPLE_INTERVAL: f32 = 32.0;

/// Height offset for gizmo rendering above the ground plane.
pub(crate) const GIZMO_Y: f32 = 1.0;

/// Elevation scale factor: terrain elevation is [0,1], we scale for display.
/// In a 256x256 grid with CELL_SIZE=16, max world height is ~40m conceptually.
pub(crate) const ELEVATION_DISPLAY_SCALE: f32 = 40.0;

/// Grade thresholds (as fractions, not percentages).
pub(crate) const GRADE_LOW_THRESHOLD: f32 = 0.03;
pub(crate) const GRADE_MEDIUM_THRESHOLD: f32 = 0.06;

/// Radius of bridge/tunnel indicator circles.
pub(crate) const INDICATOR_RADIUS: f32 = 4.0;

/// Minimum hill elevation threshold for tunnel detection.
/// Cells with elevation above this are considered hills where tunnels would be needed.
pub(crate) const HILL_ELEVATION_THRESHOLD: f32 = 0.70;

// ---------------------------------------------------------------------------
// Grade colors
// ---------------------------------------------------------------------------

/// Green: gentle grade (0-3%).
pub(crate) const COLOR_GRADE_LOW: Color = Color::srgba(0.2, 0.85, 0.2, 0.9);

/// Yellow: moderate grade (3-6%).
pub(crate) const COLOR_GRADE_MEDIUM: Color = Color::srgba(0.9, 0.85, 0.1, 0.9);

/// Red: steep grade (6%+).
pub(crate) const COLOR_GRADE_HIGH: Color = Color::srgba(0.95, 0.15, 0.1, 0.9);

/// Blue: bridge indicator (water crossing).
pub(crate) const COLOR_BRIDGE: Color = Color::srgba(0.2, 0.5, 0.95, 0.9);

/// Orange: tunnel indicator (hill crossing).
pub(crate) const COLOR_TUNNEL: Color = Color::srgba(0.9, 0.5, 0.1, 0.9);
