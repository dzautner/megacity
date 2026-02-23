pub mod systems;
#[cfg(test)]
mod tests;
pub mod types;

pub use systems::{update_wind_damage, WindDamagePlugin};
pub use types::{
    power_outage_probability, tree_knockdown_probability, wind_damage_amount, WindDamageEvent,
    WindDamageState, WindDamageTier,
};
