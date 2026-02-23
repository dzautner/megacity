//! Constants for waste reduction policy costs, bonuses, and penalties.

/// Waste generation reduction from plastic bag ban (5%).
pub const PLASTIC_BAG_BAN_WASTE_REDUCTION: f32 = 0.05;

/// Happiness penalty from plastic bag ban (minor citizen convenience impact).
pub const PLASTIC_BAG_BAN_HAPPINESS_PENALTY: f32 = 1.0;

/// Monthly upkeep cost for plastic bag ban enforcement.
pub const PLASTIC_BAG_BAN_MONTHLY_COST: f64 = 5_000.0;

/// Recycling rate bonus from deposit/return program (10 percentage points).
pub const DEPOSIT_RETURN_RECYCLING_BONUS: f32 = 0.10;

/// One-time infrastructure cost for deposit/return program.
pub const DEPOSIT_RETURN_INFRASTRUCTURE_COST: f64 = 500_000.0;

/// Monthly operating cost for deposit/return program.
pub const DEPOSIT_RETURN_MONTHLY_COST: f64 = 15_000.0;

/// Composting diversion bonus from composting mandate (15 percentage points).
pub const COMPOSTING_MANDATE_DIVERSION_BONUS: f32 = 0.15;

/// Happiness penalty from composting mandate (mandatory sorting is annoying).
pub const COMPOSTING_MANDATE_HAPPINESS_PENALTY: f32 = 2.0;

/// One-time enforcement setup cost for composting mandate.
pub const COMPOSTING_MANDATE_ENFORCEMENT_COST: f64 = 1_000_000.0;

/// Monthly enforcement cost for composting mandate.
pub const COMPOSTING_MANDATE_MONTHLY_COST: f64 = 25_000.0;

/// Monthly cost for WTE mandate administration.
pub const WTE_MANDATE_MONTHLY_COST: f64 = 10_000.0;

/// Fraction of landfill-bound waste diverted to WTE when mandate is active
/// and incinerator capacity is available.
pub const WTE_MANDATE_DIVERSION_FRACTION: f32 = 0.80;
