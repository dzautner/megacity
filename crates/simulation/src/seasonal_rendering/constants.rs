//! Constants for seasonal rendering effect tuning.

/// Maximum leaf particle intensity (arbitrary units, 0.0 - 1.0).
pub(crate) const MAX_LEAF_INTENSITY: f32 = 1.0;

/// Leaf intensity ramp-up per slow tick during autumn.
pub(crate) const LEAF_RAMP_RATE: f32 = 0.05;

/// Leaf intensity decay rate per slow tick outside autumn.
pub(crate) const LEAF_DECAY_RATE: f32 = 0.1;

/// Maximum flower particle intensity (0.0 - 1.0).
pub(crate) const MAX_FLOWER_INTENSITY: f32 = 1.0;

/// Flower intensity ramp-up per slow tick during spring.
pub(crate) const FLOWER_RAMP_RATE: f32 = 0.05;

/// Flower intensity decay rate per slow tick outside spring.
pub(crate) const FLOWER_DECAY_RATE: f32 = 0.1;

/// Maximum snow roof tint intensity (0.0 - 1.0), representing full white overlay.
pub(crate) const MAX_SNOW_ROOF_INTENSITY: f32 = 1.0;

/// Snow roof tint ramp-up per slow tick when snowing.
pub(crate) const SNOW_ROOF_RAMP_RATE: f32 = 0.04;

/// Snow roof tint decay rate per slow tick when not snowing and above freezing.
pub(crate) const SNOW_ROOF_DECAY_RATE: f32 = 0.02;

/// Temperature threshold (Celsius) above which heat shimmer can appear.
pub(crate) const HEAT_SHIMMER_THRESHOLD: f32 = 30.0;

/// Maximum heat shimmer intensity (0.0 - 1.0).
pub(crate) const MAX_HEAT_SHIMMER_INTENSITY: f32 = 1.0;

/// Rain streak intensity per inch/hr of precipitation (clamped to 1.0).
pub(crate) const RAIN_INTENSITY_SCALE: f32 = 0.5;

/// Maximum rain streak intensity (0.0 - 1.0).
pub(crate) const MAX_RAIN_INTENSITY: f32 = 1.0;

/// Maximum snowflake particle intensity (0.0 - 1.0).
pub(crate) const MAX_SNOWFLAKE_INTENSITY: f32 = 1.0;

/// Snowflake intensity per inch/hr of precipitation (clamped).
pub(crate) const SNOWFLAKE_INTENSITY_SCALE: f32 = 1.0;

/// Storm sky darkening intensity (0.0 = clear, 1.0 = fully dark).
pub(crate) const STORM_DARKENING_INTENSITY: f32 = 0.7;

/// Storm darkening ramp-up rate per slow tick.
pub(crate) const STORM_DARKEN_RAMP_RATE: f32 = 0.15;

/// Storm darkening decay rate per slow tick.
pub(crate) const STORM_DARKEN_DECAY_RATE: f32 = 0.1;

/// Lightning flash duration in slow ticks (each flash lasts ~1 tick).
pub(crate) const LIGHTNING_FLASH_DURATION: u32 = 1;

/// Probability per slow tick of a lightning flash during a storm (0.0 - 1.0).
pub(crate) const LIGHTNING_FLASH_PROBABILITY: f32 = 0.3;

/// Summer shadow length multiplier (longer shadows = more dramatic).
pub(crate) const SUMMER_SHADOW_MULTIPLIER: f32 = 1.5;

/// Spring brightness boost (fraction added to ambient light, 0.0 - 0.3).
pub(crate) const SPRING_BRIGHTNESS_BOOST: f32 = 0.15;

/// Freezing point for snow roof logic (Celsius).
pub(crate) const FREEZING_POINT_C: f32 = 0.0;
