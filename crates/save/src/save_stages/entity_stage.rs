use crate::save_codec::*;
use crate::save_types::*;

use bevy::prelude::Entity;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::{CitizenState, Gender};
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;
use simulation::water_sources::WaterSource;

use std::collections::HashMap;

/// Spawned entities: buildings, citizens, utilities, services, water sources.
pub struct EntityStageOutput {
    pub buildings: Vec<SaveBuilding>,
    pub citizens: Vec<SaveCitizen>,
    pub utility_sources: Vec<SaveUtilitySource>,
    pub service_buildings: Vec<SaveServiceBuilding>,
    pub water_sources: Option<Vec<SaveWaterSource>>,
}

/// Collect entity data: buildings, citizens, utilities, services, water sources.
pub fn collect_entity_stage(
    buildings: &[(Building, Option<MixedUseBuilding>)],
    citizens: &[CitizenSaveInput],
    utility_sources: &[UtilitySource],
    service_buildings: &[(ServiceBuilding,)],
    water_sources: Option<&[WaterSource]>,
) -> EntityStageOutput {
    // Build Entity -> citizen-array-index map for family reference serialization
    let entity_to_idx: HashMap<Entity, u32> = citizens
        .iter()
        .enumerate()
        .map(|(i, c)| (c.entity, i as u32))
        .collect();

    EntityStageOutput {
        buildings: buildings
            .iter()
            .map(|(b, mu)| SaveBuilding {
                zone_type: zone_type_to_u8(b.zone_type),
                level: b.level,
                grid_x: b.grid_x,
                grid_y: b.grid_y,
                capacity: b.capacity,
                occupants: b.occupants,
                commercial_capacity: mu.as_ref().map_or(0, |m| m.commercial_capacity),
                commercial_occupants: mu.as_ref().map_or(0, |m| m.commercial_occupants),
                residential_capacity: mu.as_ref().map_or(0, |m| m.residential_capacity),
                residential_occupants: mu.as_ref().map_or(0, |m| m.residential_occupants),
            })
            .collect(),
        citizens: citizens
            .iter()
            .map(|c| SaveCitizen {
                age: c.details.age,
                happiness: c.details.happiness,
                education: c.details.education,
                state: match c.state {
                    CitizenState::AtHome => 0,
                    CitizenState::CommutingToWork => 1,
                    CitizenState::Working => 2,
                    CitizenState::CommutingHome => 3,
                    CitizenState::CommutingToShop => 4,
                    CitizenState::Shopping => 5,
                    CitizenState::CommutingToLeisure => 6,
                    CitizenState::AtLeisure => 7,
                    CitizenState::CommutingToSchool => 8,
                    CitizenState::AtSchool => 9,
                },
                home_x: c.home_x,
                home_y: c.home_y,
                work_x: c.work_x,
                work_y: c.work_y,
                path_waypoints: c.path.waypoints.iter().map(|n| (n.0, n.1)).collect(),
                path_current_index: c.path.current_index,
                velocity_x: c.velocity.x,
                velocity_y: c.velocity.y,
                pos_x: c.position.x,
                pos_y: c.position.y,
                gender: match c.details.gender {
                    Gender::Male => 0,
                    Gender::Female => 1,
                },
                health: c.details.health,
                salary: c.details.salary,
                savings: c.details.savings,
                ambition: c.personality.ambition,
                sociability: c.personality.sociability,
                materialism: c.personality.materialism,
                resilience: c.personality.resilience,
                need_hunger: c.needs.hunger,
                need_energy: c.needs.energy,
                need_social: c.needs.social,
                need_fun: c.needs.fun,
                need_comfort: c.needs.comfort,
                activity_timer: c.activity_timer,
                family_partner: c
                    .family
                    .partner
                    .and_then(|e| entity_to_idx.get(&e).copied())
                    .unwrap_or(u32::MAX),
                family_children: c
                    .family
                    .children
                    .iter()
                    .filter_map(|e| entity_to_idx.get(e).copied())
                    .collect(),
                family_parent: c
                    .family
                    .parent
                    .and_then(|e| entity_to_idx.get(&e).copied())
                    .unwrap_or(u32::MAX),
            })
            .collect(),
        utility_sources: utility_sources
            .iter()
            .map(|u| SaveUtilitySource {
                utility_type: utility_type_to_u8(u.utility_type),
                grid_x: u.grid_x,
                grid_y: u.grid_y,
                range: u.range,
            })
            .collect(),
        service_buildings: service_buildings
            .iter()
            .map(|(sb,)| SaveServiceBuilding {
                service_type: service_type_to_u8(sb.service_type),
                grid_x: sb.grid_x,
                grid_y: sb.grid_y,
                radius_cells: (sb.radius / simulation::config::CELL_SIZE) as u32,
            })
            .collect(),
        water_sources: water_sources.map(|ws| {
            ws.iter()
                .map(|s| SaveWaterSource {
                    source_type: water_source_type_to_u8(s.source_type),
                    grid_x: s.grid_x,
                    grid_y: s.grid_y,
                    capacity_mgd: s.capacity_mgd,
                    quality: s.quality,
                    operating_cost: s.operating_cost,
                    stored_gallons: s.stored_gallons,
                    storage_capacity: s.storage_capacity,
                })
                .collect()
        }),
    }
}
