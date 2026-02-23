use bevy::prelude::*;

use super::pricing::update_market_prices;
use super::types::MarketPrices;

pub struct MarketPlugin;

impl Plugin for MarketPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketPrices>().add_systems(
            FixedUpdate,
            update_market_prices
                .after(crate::production::update_production_chains)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
