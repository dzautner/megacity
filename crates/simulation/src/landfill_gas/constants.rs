//! Constants for landfill gas generation, collection, and energy conversion.

/// Cubic feet of landfill gas generated per ton of waste per year.
pub const GAS_GENERATION_CF_PER_TON_PER_YEAR: f64 = 100.0;

/// Fraction of landfill gas that is methane (CH4).
pub const METHANE_FRACTION: f32 = 0.50;

/// Fraction of landfill gas that is carbon dioxide (CO2).
pub const CO2_FRACTION: f32 = 0.50;

/// Megawatts of electricity generated per 1,000 tons/day of waste in landfill.
pub const MW_PER_1000_TONS_DAY: f64 = 1.0;

/// Default collection efficiency (75% of generated gas is captured).
pub const COLLECTION_EFFICIENCY_DEFAULT: f32 = 0.75;

/// Capital cost to install gas collection infrastructure at one landfill ($500K).
pub const COLLECTION_INFRA_COST_PER_LANDFILL: f64 = 500_000.0;

/// Annual maintenance cost per landfill with gas collection ($20K/year).
pub const MAINTENANCE_COST_PER_LANDFILL_YEAR: f64 = 20_000.0;

/// Annual probability of fire/explosion at a landfill without gas collection.
pub const FIRE_RISK_ANNUAL_NO_COLLECTION: f32 = 0.001;

/// Number of slow ticks that represent one year (for annualizing per-tick calculations).
/// Each slow tick represents roughly 1 game-day, so 365 ticks = 1 year.
pub const SLOW_TICKS_PER_YEAR: f64 = 365.0;
