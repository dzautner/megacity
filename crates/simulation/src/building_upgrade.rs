use bevy::prelude::*;

use crate::buildings::Building;
use crate::stats::CityStats;

const UPGRADE_INTERVAL: u32 = 30; // sim ticks between upgrade checks

#[derive(Resource, Default)]
pub struct UpgradeTimer {
    pub tick: u32,
    pub downgrade_tick: u32,
}

pub fn upgrade_buildings(
    stats: Res<CityStats>,
    mut timer: ResMut<UpgradeTimer>,
    mut buildings: Query<&mut Building>,
    policies: Res<crate::policies::Policies>,
) {
    timer.tick += 1;
    if timer.tick < UPGRADE_INTERVAL {
        return;
    }
    timer.tick = 0;

    let policy_max = policies.max_building_level();

    let mut upgraded = 0u32;
    let max_upgrades_per_tick = 50;

    for mut building in &mut buildings {
        if upgraded >= max_upgrades_per_tick {
            break;
        }

        let max_level = building.zone_type.max_level().min(policy_max);
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
            upgraded += 1;
        }
    }
}

/// Downgrade buildings when happiness is very low
pub fn downgrade_buildings(
    stats: Res<CityStats>,
    mut timer: ResMut<UpgradeTimer>,
    mut buildings: Query<&mut Building>,
) {
    timer.downgrade_tick += 1;
    if timer.downgrade_tick < UPGRADE_INTERVAL {
        return;
    }
    timer.downgrade_tick = 0;

    if stats.average_happiness > 30.0 {
        return;
    }

    for mut building in &mut buildings {
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
        }
    }
}
