//! Landfill Gas (LFG) Collection and Energy Generation (WASTE-956).
//!
//! Landfills produce landfill gas (LFG) — a mixture of roughly 50% methane and
//! 50% CO2 — as organic waste decomposes anaerobically. Without active gas
//! collection infrastructure, all methane escapes into the atmosphere as a potent
//! greenhouse gas and poses fire/explosion risks.
//!
//! When gas collection is active, a fraction of the generated gas is captured
//! (default 75% efficiency) and can be converted to electricity via gas-to-energy
//! turbines. The conversion rate is approximately 1 MW per 1,000 tons/day of
//! waste in the landfill.
//!
//! Key design points:
//! - Gas generation: 100 cubic feet per ton of waste per year
//! - Methane/CO2 split: 50/50
//! - Collection efficiency: 75% default (configurable)
//! - Electricity conversion: 1 MW per 1,000 tons/day of waste
//! - Fire/explosion risk: 0.1% annual probability without collection
//! - Infrastructure cost: $500K per landfill, $20K/year maintenance

pub mod calculations;
pub mod constants;
pub mod state;
pub mod systems;

#[cfg(test)]
mod tests_calculations;
#[cfg(test)]
mod tests_state;

// Re-export all public items so callers don't need to change their imports.
pub use calculations::*;
pub use constants::*;
pub use state::*;
pub use systems::*;

use bevy::prelude::*;

pub struct LandfillGasPlugin;

impl Plugin for LandfillGasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LandfillGasState>().add_systems(
            FixedUpdate,
            update_landfill_gas
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
