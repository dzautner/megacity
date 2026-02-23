//! Constants for the metro transit system.

/// Metro train speed in km/h.
pub const METRO_SPEED_KMH: f32 = 80.0;

/// Maximum passengers per train.
pub const TRAIN_CAPACITY: u32 = 180;

/// Walking speed for station access in grid-cells per tick.
/// Citizens walk ~4 km/h ≈ 4000m/h ÷ 16m/cell ≈ 250 cells/h.
pub const WALK_SPEED_CELLS_PER_HOUR: f32 = 250.0;

/// Maximum walking distance to a metro station (in grid cells).
/// ~800m / 16m = 50 cells.
pub const MAX_WALK_TO_STATION_CELLS: u32 = 50;

/// Station construction cost (one-time).
pub const STATION_CONSTRUCTION_COST: f64 = 5000.0;

/// Station weekly maintenance cost.
pub const STATION_WEEKLY_MAINTENANCE: f64 = 500.0;

/// Line weekly maintenance cost.
pub const LINE_WEEKLY_MAINTENANCE: f64 = 1200.0;

/// Land value boost at station center.
pub const STATION_LAND_VALUE_BOOST_CENTER: i32 = 25;

/// Land value boost radius (in grid cells).
pub const STATION_LAND_VALUE_BOOST_RADIUS: i32 = 12;

/// Minimum land value boost at edge of radius.
pub const STATION_LAND_VALUE_BOOST_MIN: i32 = 15;

/// Headway between trains in minutes (game time).
pub const TRAIN_HEADWAY_MINUTES: f32 = 5.0;

/// Average wait time is half the headway.
pub const AVERAGE_WAIT_MINUTES: f32 = TRAIN_HEADWAY_MINUTES / 2.0;

/// Metro speed in grid-cells per game-hour.
/// 80 km/h = 80_000 m/h ÷ 16 m/cell = 5000 cells/h.
pub const METRO_SPEED_CELLS_PER_HOUR: f32 = 5000.0;
