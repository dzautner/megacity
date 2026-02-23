//! Constants for landfill capacity, environmental effects, and lifecycle timing.

/// Default capacity for a single landfill site in tons.
pub const DEFAULT_LANDFILL_CAPACITY_TONS: f64 = 500_000.0;

/// Groundwater pollution factor for unlined landfills (0.0-1.0).
pub const GROUNDWATER_POLLUTION_UNLINED: f32 = 0.80;

/// Groundwater pollution factor for lined landfills (0.0-1.0).
pub const GROUNDWATER_POLLUTION_LINED: f32 = 0.20;

/// Groundwater pollution factor for lined landfills with gas collection (0.0-1.0).
pub const GROUNDWATER_POLLUTION_LINED_COLLECTION: f32 = 0.05;

/// Odor radius in grid cells for unlined landfills.
pub const ODOR_RADIUS_UNLINED: u32 = 15;

/// Odor radius in grid cells for lined landfills.
pub const ODOR_RADIUS_LINED: u32 = 10;

/// Odor radius in grid cells for lined landfills with gas collection.
pub const ODOR_RADIUS_LINED_COLLECTION: u32 = 5;

/// Land value penalty fraction for unlined landfills (40%).
pub const LAND_VALUE_PENALTY_UNLINED: f32 = 0.40;

/// Land value penalty fraction for lined landfills (25%).
pub const LAND_VALUE_PENALTY_LINED: f32 = 0.25;

/// Land value penalty fraction for lined landfills with gas collection (15%).
pub const LAND_VALUE_PENALTY_LINED_COLLECTION: f32 = 0.15;

/// Megawatts of electricity generated per 1,000 tons/day of waste with gas collection.
pub const GAS_COLLECTION_MW_PER_1000_TONS_DAY: f64 = 1.0;

/// Number of years of post-closure monitoring required.
pub const POST_CLOSURE_MONITORING_YEARS: u32 = 30;

/// Number of slow ticks per game year (each slow tick ~ 1 game day).
pub const SLOW_TICKS_PER_YEAR: f64 = 365.0;

/// Days per year for years_remaining calculation.
pub const DAYS_PER_YEAR: f32 = 365.0;
