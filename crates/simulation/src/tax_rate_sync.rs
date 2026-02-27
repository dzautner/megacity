//! Keeps `CityBudget.tax_rate` synchronized with the per-zone rates in
//! `ExtendedBudget.zone_taxes`.
//!
//! The UI exposes per-zone tax sliders that directly modify
//! `ExtendedBudget.zone_taxes`, which is what `collect_taxes()` reads.
//! Several other systems (happiness, immigration attractiveness, advisors,
//! chart data) read the single `CityBudget.tax_rate` field. This system
//! keeps that field equal to the average of the four zone rates so those
//! downstream systems reflect the player's chosen tax policy.

use bevy::prelude::*;

use crate::budget::ExtendedBudget;
use crate::economy::CityBudget;

/// Sync `CityBudget.tax_rate` to the mean of the four zone tax rates.
pub fn sync_tax_rate_from_zones(
    extended: Res<ExtendedBudget>,
    mut budget: ResMut<CityBudget>,
) {
    let zt = &extended.zone_taxes;
    let avg = (zt.residential + zt.commercial + zt.industrial + zt.office) / 4.0;
    budget.tax_rate = avg;
}

pub struct TaxRateSyncPlugin;

impl Plugin for TaxRateSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            sync_tax_rate_from_zones
                .before(crate::economy::collect_taxes)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
