// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// UHI update frequency in simulation ticks.
pub(crate) const UHI_UPDATE_INTERVAL: u64 = 30;

/// Rural baseline green fraction (fraction of cells that are vegetated in
/// undeveloped areas).  The deficit between actual local green fraction and
/// this baseline drives the vegetation-deficit UHI contribution.
pub(crate) const RURAL_GREEN_BASELINE: f32 = 0.6;

/// Maximum vegetation-deficit contribution in degrees Fahrenheit.
pub(crate) const VEGETATION_DEFICIT_SCALE: f32 = 8.0;

/// Canyon-effect scale: building levels (stories) above 4 contribute to UHI
/// proportional to a height-to-width ratio approximation.
pub(crate) const CANYON_STORIES_THRESHOLD: u8 = 4;
pub(crate) const CANYON_EFFECT_SCALE: f32 = 1.5;

/// Nighttime amplification factor (UHI is doubled at night).
pub(crate) const NIGHTTIME_AMPLIFICATION: f32 = 2.0;

/// Hours considered nighttime for UHI amplification.
/// Night: 20:00 - 05:59 (inclusive).
pub(crate) const NIGHT_START_HOUR: u32 = 20;
pub(crate) const NIGHT_END_HOUR: u32 = 5;

// ---------------------------------------------------------------------------
// Surface heat factors (Fahrenheit)
// ---------------------------------------------------------------------------

/// Asphalt / dark roof surface heat factor.
pub(crate) const SURFACE_ASPHALT: f32 = 2.0;
/// Concrete surface heat factor.
pub(crate) const SURFACE_CONCRETE: f32 = 1.5;
/// Light roof surface heat factor.
pub(crate) const SURFACE_LIGHT_ROOF: f32 = 0.5;
/// Water surface heat factor (strong cooling).
pub(crate) const SURFACE_WATER: f32 = -2.0;
/// Vegetation surface heat factor (cooling).
pub(crate) const SURFACE_VEGETATION: f32 = -1.5;
