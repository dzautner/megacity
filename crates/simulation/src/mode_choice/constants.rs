//! Constants for the mode choice system.

/// Maximum biking distance in cells (~5km at 16m/cell ≈ 312 cells).
/// We use a practical limit of ~80 cells for gameplay.
pub const MAX_PRACTICAL_BIKE_DISTANCE: f32 = 80.0;

/// Maximum distance to a transit stop for transit to be available (in cells).
pub const MAX_TRANSIT_ACCESS_DISTANCE: f32 = 15.0;

/// Maximum distance to a bike-friendly road (Path type) for biking (in cells).
pub const MAX_BIKE_ACCESS_DISTANCE: f32 = 10.0;

/// Speed multiplier for walking mode (relative to base citizen speed).
pub const WALK_SPEED_MULTIPLIER: f32 = 0.30;

/// Speed multiplier for biking mode.
pub const BIKE_SPEED_MULTIPLIER: f32 = 0.60;

/// Speed multiplier for driving mode (baseline).
pub const DRIVE_SPEED_MULTIPLIER: f32 = 1.00;

/// Speed multiplier for transit mode.
pub const TRANSIT_SPEED_MULTIPLIER: f32 = 0.80;

/// Comfort factor for walking (pleasant for short trips).
pub const WALK_COMFORT: f32 = 1.0;

/// Comfort factor for biking (slightly less comfortable).
pub const BIKE_COMFORT: f32 = 0.95;

/// Comfort factor for driving (parking stress, traffic stress).
pub const DRIVE_COMFORT: f32 = 0.90;

/// Comfort factor for transit (waiting, transfers, crowding).
pub const TRANSIT_COMFORT: f32 = 0.85;

/// Overhead time for driving (finding parking, walking to/from car), in
/// equivalent cells of travel distance.
pub const DRIVE_PARKING_OVERHEAD: f32 = 5.0;

/// Wait time overhead for transit (average wait at stop), in equivalent cells.
pub const TRANSIT_WAIT_OVERHEAD: f32 = 8.0;
