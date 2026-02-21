use bevy::prelude::*;

use crate::buildings::{max_level_for_far, Building, MixedUseBuilding};
use crate::stats::CityStats;
use crate::urban_growth_boundary::UrbanGrowthBoundary;

const UPGRADE_INTERVAL: u32 = 30; // sim ticks between upgrade checks

#[derive(Resource, Default)]
pub struct UpgradeTimer {
    pub tick: u32,
    pub downgrade_tick: u32,
}

pub fn upgrade_buildings(
    stats: Res<CityStats>,
    mut timer: ResMut<UpgradeTimer>,
    mut buildings: Query<(&mut Building, Option<&mut MixedUseBuilding>)>,
    policies: Res<crate::policies::Policies>,
    ugb: Res<UrbanGrowthBoundary>,
) {
    timer.tick += 1;
    if timer.tick < UPGRADE_INTERVAL {
        return;
    }
    timer.tick = 0;

    let policy_max = policies.max_building_level();

    let mut upgraded = 0u32;
    let max_upgrades_per_tick = 50;

    for (mut building, mixed_use) in &mut buildings {
        if upgraded >= max_upgrades_per_tick {
            break;
        }

        // Buildings outside the UGB cannot be upgraded (ZONE-009).
        if !ugb.allows_upgrade(building.grid_x, building.grid_y) {
            continue;
        }

        let far_cap = max_level_for_far(building.zone_type) as u8;
        let max_level = building.zone_type.max_level().min(policy_max).min(far_cap);
        if building.level >= max_level {
            continue;
        }

        let occupancy = if building.capacity > 0 {
            building.occupants as f32 / building.capacity as f32
        } else {
            0.0
        };

        // Upgrade when occupancy is high and happiness is decent
        let should_upgrade = occupancy >= 0.75 && stats.average_happiness >= 45.0;

        if should_upgrade {
            building.level += 1;
            building.capacity = Building::capacity_for_level(building.zone_type, building.level);
            // Update MixedUseBuilding capacities if present
            if let Some(mut mu) = mixed_use {
                let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(building.level);
                mu.commercial_capacity = comm_cap;
                mu.residential_capacity = res_cap;
            }
            upgraded += 1;
        }
    }
}

/// Downgrade buildings when happiness is very low
pub fn downgrade_buildings(
    stats: Res<CityStats>,
    mut timer: ResMut<UpgradeTimer>,
    mut buildings: Query<(&mut Building, Option<&mut MixedUseBuilding>)>,
) {
    timer.downgrade_tick += 1;
    if timer.downgrade_tick < UPGRADE_INTERVAL {
        return;
    }
    timer.downgrade_tick = 0;

    if stats.average_happiness > 30.0 {
        return;
    }

    for (mut building, mixed_use) in &mut buildings {
        if building.level <= 1 {
            continue;
        }

        // Random chance of downgrade when happiness is very low
        if rand::random::<f32>() < 0.01 {
            building.level -= 1;
            building.capacity = Building::capacity_for_level(building.zone_type, building.level);
            // Evict excess occupants
            if building.occupants > building.capacity {
                building.occupants = building.capacity;
            }
            // Update MixedUseBuilding capacities if present
            if let Some(mut mu) = mixed_use {
                let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(building.level);
                mu.commercial_capacity = comm_cap;
                mu.residential_capacity = res_cap;
                if mu.commercial_occupants > mu.commercial_capacity {
                    mu.commercial_occupants = mu.commercial_capacity;
                }
                if mu.residential_occupants > mu.residential_capacity {
                    mu.residential_occupants = mu.residential_capacity;
                }
            }
        }
    }
}

pub struct BuildingUpgradePlugin;

impl Plugin for BuildingUpgradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpgradeTimer>().add_systems(
            FixedUpdate,
            (upgrade_buildings, downgrade_buildings)
                .chain()
                .after(crate::lifecycle::emigration)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
