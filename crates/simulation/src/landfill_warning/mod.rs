mod calculations;
mod systems;
mod tests;
mod types;

pub use calculations::{
    advance_fill, compute_days_remaining, compute_remaining_pct, tier_from_remaining_pct,
    LANDFILL_CAPACITY_PER_BUILDING,
};
pub use systems::{update_landfill_capacity, LandfillWarningPlugin};
pub use types::{LandfillCapacityState, LandfillWarningEvent, LandfillWarningTier};
