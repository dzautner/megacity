use bevy::prelude::*;
use std::collections::HashSet;

use rendering::building_render::BuildingMesh3d;
use rendering::citizen_render::CitizenSprite;
use simulation::buildings::Building;
use simulation::citizen::Citizen;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;
use simulation::water_sources::WaterSource;

/// Collects all game entities (buildings, citizens, utilities, services,
/// water sources, meshes, sprites) and despawns them immediately using
/// direct world access.  This avoids the deferred-Commands race condition.
pub(crate) fn despawn_all_game_entities(world: &mut World) {
    let mut entities = HashSet::new();

    // Collect entities from each component query.
    let mut q = world.query_filtered::<Entity, With<Building>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<Citizen>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<UtilitySource>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<ServiceBuilding>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<WaterSource>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<BuildingMesh3d>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<CitizenSprite>>();
    for e in q.iter(world) {
        entities.insert(e);
    }

    // Despawn each entity immediately.
    for entity in entities {
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
        }
    }
}
