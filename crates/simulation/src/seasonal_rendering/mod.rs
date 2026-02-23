//! Enhanced seasonal rendering effects (WEATHER-018).
//!
//! Tracks visual effect state for each season: falling leaves in autumn,
//! snow accumulation on building roofs in winter, flower particles near parks
//! in spring, heat shimmer in summer, rain streaks during rain events,
//! storm darkening and lightning flashes during storms, and snowflake
//! particles during winter precipitation.
//!
//! The `SeasonalRenderingState` resource holds intensity values and active
//! effect flags that the rendering layer reads each frame. The
//! `update_seasonal_rendering` system runs every slow tick, reading the
//! current `Weather` resource to derive which effects should be active and
//! at what intensity.
//!
//! All effects are toggleable via `SeasonalEffectsConfig` for performance
//! tuning.

pub mod compute;
pub(crate) mod constants;
mod system;
pub mod types;

#[cfg(test)]
mod tests_compute;
#[cfg(test)]
mod tests_types;
#[cfg(test)]
mod tests_weather;

// Re-export all public items so callers don't need to change their imports.
pub use compute::*;
pub use system::{update_seasonal_rendering, SeasonalRenderingPlugin};
pub use types::{SeasonalEffectsConfig, SeasonalRenderingState};
