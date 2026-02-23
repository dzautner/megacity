/// Demand reduction from low-flow fixtures (applied to residential buildings only).
pub const LOW_FLOW_DEMAND_REDUCTION: f32 = 0.20;

/// Demand reduction from xeriscaping (reduced irrigation area, applies to total demand).
pub const XERISCAPING_DEMAND_REDUCTION: f32 = 0.10;

/// Demand reduction from tiered water pricing (behavioural change, applies to total demand).
pub const TIERED_PRICING_DEMAND_REDUCTION: f32 = 0.15;

/// Demand reduction from greywater recycling (applies to total demand).
pub const GREYWATER_DEMAND_REDUCTION: f32 = 0.15;

/// Sewage volume reduction from greywater recycling (greywater reused instead of discharged).
pub const GREYWATER_SEWAGE_REDUCTION: f32 = 0.30;

/// Demand reduction from rainwater harvesting (effective only when precipitation > 0).
pub const RAINWATER_DEMAND_REDUCTION: f32 = 0.10;

/// Hard cap on total demand reduction from all combined conservation policies.
pub const MAX_TOTAL_DEMAND_REDUCTION: f32 = 0.60;

/// Per-building retrofit cost for low-flow fixtures (dollars).
pub const LOW_FLOW_COST_PER_BUILDING: f64 = 500.0;

/// Per-building retrofit cost for greywater recycling (dollars).
pub const GREYWATER_COST_PER_BUILDING: f64 = 3000.0;

/// Per-building retrofit cost for rainwater harvesting (dollars).
pub const RAINWATER_COST_PER_BUILDING: f64 = 1000.0;

/// Base daily water demand per building used for annual savings estimates (gallons).
/// This is a rough average across building types for estimating conservation savings.
pub(crate) const BASE_DAILY_DEMAND_PER_BUILDING: f64 = 1200.0;

/// Days in a year for annual savings calculation.
pub(crate) const DAYS_PER_YEAR: f64 = 365.0;
