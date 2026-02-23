//! Constants for hazardous waste management.

/// Capacity per hazardous waste facility in tons/day.
pub const FACILITY_CAPACITY_TONS_PER_DAY: f32 = 20.0;

/// Build cost for a single hazardous waste facility ($3M).
pub const FACILITY_BUILD_COST: f64 = 3_000_000.0;

/// Operating cost per facility per day ($5K).
pub const FACILITY_OPERATING_COST_PER_DAY: f64 = 5_000.0;

/// Federal fine per illegal dump event ($50K).
pub const FEDERAL_FINE_PER_EVENT: f64 = 50_000.0;

/// Groundwater quality reduction per unit of illegal dumping overflow.
/// Applied to cells around industrial buildings when dumping occurs.
pub const CONTAMINATION_PER_OVERFLOW_TON: f32 = 2.0;

/// Contamination natural decay rate per slow tick (1% reduction).
pub const CONTAMINATION_DECAY_RATE: f32 = 0.01;

/// Base hazardous waste generation rate per industrial building level (tons/day).
/// Level 1 = 0.5, Level 2 = 1.0, Level 3 = 2.0, Level 4 = 3.5, Level 5 = 5.0.
pub const INDUSTRIAL_WASTE_PER_LEVEL: [f32; 5] = [0.5, 1.0, 2.0, 3.5, 5.0];

/// Base hazardous waste generation for medical buildings (tons/day per facility).
pub const MEDICAL_WASTE_RATE: f32 = 0.8;

/// Radius (in grid cells) around industrial buildings affected by illegal dumping
/// contamination of groundwater.
pub const CONTAMINATION_RADIUS: i32 = 4;
