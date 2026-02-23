pub(crate) mod systems;
#[cfg(test)]
mod tests;
pub(crate) mod types;

pub use systems::{assign_industry_type, update_production_chains};
pub use types::{CityGoods, GoodsType, IndustryBuilding, IndustryType, ProductionChain};

use bevy::prelude::*;

pub struct ProductionPlugin;

impl Plugin for ProductionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityGoods>().add_systems(
            FixedUpdate,
            (assign_industry_type, update_production_chains)
                .chain()
                .after(crate::agriculture::update_agriculture)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
