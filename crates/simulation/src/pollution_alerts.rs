//! POLL-022: Pollution Alert Event System
//!
//! Emits pollution alerts when pollution levels exceed health thresholds over
//! sustained periods. Tracks per-cell exceedance counters so that transient
//! single-tick spikes do not trigger alerts.
//!
//! Alert types:
//! - **Air quality**: residential cell AQI > 150 for 3+ slow ticks
//! - **Water quality**: water quality drops below "Clean" tier (< 180) at any cell
//! - **Noise complaint**: residential cell noise > 80 dB for 3+ slow ticks
//! - **Soil contamination**: industrial building adjacent to residential zone
//!   with high water pollution (proxy for soil contamination, since no dedicated
//!   soil grid exists yet)

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::groundwater::WaterQualityGrid;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::water_pollution::WaterPollutionGrid;
use crate::SlowTickTimer;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The category of pollution that triggered the alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum PollutionAlertType {
    AirQuality,
    WaterQuality,
    NoiseComplaint,
    SoilContamination,
}

/// Severity level for a pollution alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum AlertSeverity {
    Advisory,
    Warning,
    Emergency,
}

// ---------------------------------------------------------------------------
// Alert struct
// ---------------------------------------------------------------------------

/// A single pollution alert record.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PollutionAlert {
    pub alert_type: PollutionAlertType,
    pub severity: AlertSeverity,
    pub grid_x: usize,
    pub grid_y: usize,
    pub tick: u64,
}

// ---------------------------------------------------------------------------
// Exceedance tracker (not saved — transient runtime state)
// ---------------------------------------------------------------------------

/// Tracks how many consecutive slow ticks each cell has exceeded a threshold.
/// Used internally to implement sustained-exceedance logic.
#[derive(Resource)]
pub struct ExceedanceTracker {
    /// Air quality exceedance counters per cell.
    pub air: Vec<u8>,
    /// Noise exceedance counters per cell.
    pub noise: Vec<u8>,
}

impl Default for ExceedanceTracker {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            air: vec![0; n],
            noise: vec![0; n],
        }
    }
}

// ---------------------------------------------------------------------------
// Alert log resource (saved)
// ---------------------------------------------------------------------------

/// Maximum number of alerts retained in the log.
const MAX_ALERT_LOG: usize = 200;

/// Persistent log of pollution alerts, newest first.
#[derive(Resource, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PollutionAlertLog {
    pub alerts: Vec<PollutionAlert>,
}

impl PollutionAlertLog {
    /// Push a new alert, evicting the oldest if the log is full.
    pub fn push(&mut self, alert: PollutionAlert) {
        if self.alerts.len() >= MAX_ALERT_LOG {
            self.alerts.remove(0);
        }
        self.alerts.push(alert);
    }

    /// Return all alerts of a given type.
    pub fn alerts_of_type(&self, t: PollutionAlertType) -> Vec<&PollutionAlert> {
        self.alerts.iter().filter(|a| a.alert_type == t).collect()
    }
}

impl crate::Saveable for PollutionAlertLog {
    const SAVE_KEY: &'static str = "pollution_alert_log";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.alerts.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Thresholds
// ---------------------------------------------------------------------------

/// AQI concentration threshold (u8 pollution value) for air quality alerts.
/// Corresponds to AqiTier::Unhealthy (151+).
const AIR_THRESHOLD: u8 = 151;
/// Noise level (out of 100) threshold for noise complaints.
const NOISE_THRESHOLD: u8 = 80;
/// Water quality value below which a water-quality alert is raised.
/// Values below 180 are considered non-Clean (WaterQualityGrid: 0=bad, 255=pure).
const WATER_QUALITY_CLEAN_THRESHOLD: u8 = 180;
/// Water pollution value above which soil contamination alert is raised near residential.
const SOIL_CONTAMINATION_THRESHOLD: u8 = 100;
/// Number of consecutive slow ticks a cell must exceed the threshold before an alert fires.
const SUSTAINED_TICKS: u8 = 3;

// ---------------------------------------------------------------------------
// Air quality alert system
// ---------------------------------------------------------------------------

/// Checks residential cells for sustained high air pollution and emits alerts.
#[allow(clippy::too_many_arguments)]
pub fn check_air_quality_alerts(
    slow_timer: Res<SlowTickTimer>,
    tick_counter: Res<TickCounter>,
    pollution: Res<PollutionGrid>,
    grid: Res<WorldGrid>,
    mut tracker: ResMut<ExceedanceTracker>,
    mut log: ResMut<PollutionAlertLog>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let cell = grid.get(x, y);

            if !cell.zone.is_residential() {
                tracker.air[idx] = 0;
                continue;
            }

            let concentration = pollution.get(x, y);
            if concentration >= AIR_THRESHOLD {
                tracker.air[idx] = tracker.air[idx].saturating_add(1);
            } else {
                tracker.air[idx] = tracker.air[idx].saturating_sub(1);
            }

            if tracker.air[idx] >= SUSTAINED_TICKS {
                let severity = if concentration >= 251 {
                    AlertSeverity::Emergency
                } else if concentration >= 201 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Advisory
                };
                log.push(PollutionAlert {
                    alert_type: PollutionAlertType::AirQuality,
                    severity,
                    grid_x: x,
                    grid_y: y,
                    tick: tick_counter.0,
                });
                // Reset counter so we don't spam every tick
                tracker.air[idx] = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Water quality alert system
// ---------------------------------------------------------------------------

/// Checks for cells where drinking water quality has dropped below the Clean tier.
pub fn check_water_quality_alerts(
    slow_timer: Res<SlowTickTimer>,
    tick_counter: Res<TickCounter>,
    water_quality: Res<WaterQualityGrid>,
    mut log: ResMut<PollutionAlertLog>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Sample every 8th cell to avoid O(n) full-grid scan each slow tick.
    // This still catches any meaningful contamination zone.
    for y in (0..GRID_HEIGHT).step_by(8) {
        for x in (0..GRID_WIDTH).step_by(8) {
            let quality = water_quality.get(x, y);
            if quality < WATER_QUALITY_CLEAN_THRESHOLD {
                let severity = if quality < 80 {
                    AlertSeverity::Emergency
                } else if quality < 140 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Advisory
                };
                log.push(PollutionAlert {
                    alert_type: PollutionAlertType::WaterQuality,
                    severity,
                    grid_x: x,
                    grid_y: y,
                    tick: tick_counter.0,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Noise complaint system
// ---------------------------------------------------------------------------

/// Checks residential cells for sustained high noise and emits complaints.
pub fn check_noise_complaints(
    slow_timer: Res<SlowTickTimer>,
    tick_counter: Res<TickCounter>,
    noise: Res<NoisePollutionGrid>,
    grid: Res<WorldGrid>,
    mut tracker: ResMut<ExceedanceTracker>,
    mut log: ResMut<PollutionAlertLog>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let cell = grid.get(x, y);

            if !cell.zone.is_residential() {
                tracker.noise[idx] = 0;
                continue;
            }

            let level = noise.get(x, y);
            if level >= NOISE_THRESHOLD {
                tracker.noise[idx] = tracker.noise[idx].saturating_add(1);
            } else {
                tracker.noise[idx] = tracker.noise[idx].saturating_sub(1);
            }

            if tracker.noise[idx] >= SUSTAINED_TICKS {
                let severity = if level >= 95 {
                    AlertSeverity::Emergency
                } else if level >= 90 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Advisory
                };
                log.push(PollutionAlert {
                    alert_type: PollutionAlertType::NoiseComplaint,
                    severity,
                    grid_x: x,
                    grid_y: y,
                    tick: tick_counter.0,
                });
                tracker.noise[idx] = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Soil contamination alert system
// ---------------------------------------------------------------------------

/// Checks for high water pollution near residential zones as a proxy for soil
/// contamination (no dedicated soil grid exists yet).
pub fn check_soil_contamination_alerts(
    slow_timer: Res<SlowTickTimer>,
    tick_counter: Res<TickCounter>,
    water_pollution: Res<WaterPollutionGrid>,
    grid: Res<WorldGrid>,
    mut log: ResMut<PollutionAlertLog>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Check every 4th cell for performance (contamination zones are broad).
    for y in (0..GRID_HEIGHT).step_by(4) {
        for x in (0..GRID_WIDTH).step_by(4) {
            let wp = water_pollution.get(x, y);
            if wp < SOIL_CONTAMINATION_THRESHOLD {
                continue;
            }

            // Check if any neighboring cell (radius 2) is residential
            let has_nearby_residential = check_nearby_residential(&grid, x, y, 2);

            if has_nearby_residential {
                let severity = if wp >= 200 {
                    AlertSeverity::Emergency
                } else if wp >= 150 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Advisory
                };
                log.push(PollutionAlert {
                    alert_type: PollutionAlertType::SoilContamination,
                    severity,
                    grid_x: x,
                    grid_y: y,
                    tick: tick_counter.0,
                });
            }
        }
    }
}

/// Returns true if any cell within `radius` Manhattan distance of (cx,cy) is
/// a residential zone.
fn check_nearby_residential(grid: &WorldGrid, cx: usize, cy: usize, radius: i32) -> bool {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            if grid.get(nx as usize, ny as usize).zone.is_residential() {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PollutionAlertPlugin;

impl Plugin for PollutionAlertPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PollutionAlertLog>()
            .init_resource::<ExceedanceTracker>()
            .add_systems(
                FixedUpdate,
                (
                    check_air_quality_alerts,
                    check_water_quality_alerts,
                    check_noise_complaints,
                    check_soil_contamination_alerts,
                )
                    .after(crate::pollution_health::apply_pollution_health_effects)
                    .after(crate::noise::update_noise_pollution)
                    .after(crate::water_pollution::update_water_pollution)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<PollutionAlertLog>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Advisory < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Emergency);
    }

    #[test]
    fn test_alert_log_push_and_eviction() {
        let mut log = PollutionAlertLog::default();
        for i in 0..MAX_ALERT_LOG + 10 {
            log.push(PollutionAlert {
                alert_type: PollutionAlertType::AirQuality,
                severity: AlertSeverity::Advisory,
                grid_x: 0,
                grid_y: 0,
                tick: i as u64,
            });
        }
        assert_eq!(log.alerts.len(), MAX_ALERT_LOG);
        // Oldest should have been evicted; first alert tick should be 10
        assert_eq!(log.alerts[0].tick, 10);
    }

    #[test]
    fn test_alert_log_filter_by_type() {
        let mut log = PollutionAlertLog::default();
        log.push(PollutionAlert {
            alert_type: PollutionAlertType::AirQuality,
            severity: AlertSeverity::Advisory,
            grid_x: 0,
            grid_y: 0,
            tick: 1,
        });
        log.push(PollutionAlert {
            alert_type: PollutionAlertType::NoiseComplaint,
            severity: AlertSeverity::Warning,
            grid_x: 5,
            grid_y: 5,
            tick: 2,
        });
        log.push(PollutionAlert {
            alert_type: PollutionAlertType::AirQuality,
            severity: AlertSeverity::Emergency,
            grid_x: 10,
            grid_y: 10,
            tick: 3,
        });

        let air = log.alerts_of_type(PollutionAlertType::AirQuality);
        assert_eq!(air.len(), 2);
        let noise = log.alerts_of_type(PollutionAlertType::NoiseComplaint);
        assert_eq!(noise.len(), 1);
        let water = log.alerts_of_type(PollutionAlertType::WaterQuality);
        assert_eq!(water.len(), 0);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut log = PollutionAlertLog::default();
        log.push(PollutionAlert {
            alert_type: PollutionAlertType::WaterQuality,
            severity: AlertSeverity::Warning,
            grid_x: 42,
            grid_y: 99,
            tick: 12345,
        });

        let bytes = log.save_to_bytes().expect("should produce bytes");
        let restored = PollutionAlertLog::load_from_bytes(&bytes);
        assert_eq!(restored.alerts.len(), 1);
        assert_eq!(restored.alerts[0].grid_x, 42);
        assert_eq!(restored.alerts[0].grid_y, 99);
        assert_eq!(restored.alerts[0].tick, 12345);
        assert_eq!(restored.alerts[0].alert_type, PollutionAlertType::WaterQuality);
        assert_eq!(restored.alerts[0].severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_empty_log_save_returns_none() {
        let log = PollutionAlertLog::default();
        assert!(log.save_to_bytes().is_none());
    }
}
