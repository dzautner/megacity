use bevy::prelude::*;
use simulation::terrain_generation::{generate_procedural_terrain, TerrainConfig};
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

use crate::despawn::despawn_all_game_entities;
use crate::reset_resources::reset_all_resources;

/// Default number of hydraulic erosion iterations for new games.
const NEW_GAME_EROSION_ITERATIONS: u32 = 10_000;

/// Generate a random seed from the current system time.
fn random_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

/// Exclusive system that resets the world for a new game.  Entity despawns
/// are immediate (no deferred Commands).
/// Runs on `OnEnter(SaveLoadState::NewGame)`, then transitions back to `Idle`.
pub(crate) fn exclusive_new_game(world: &mut World) {
    // -- Stage 1: Despawn existing entities (immediate) --
    despawn_all_game_entities(world);

    // -- Stage 2: Reset all resources to defaults --
    reset_all_resources(world);

    // -- Stage 3: Reset extension-registered resources via SaveableRegistry --
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    registry.reset_all(world);
    world.insert_resource(registry);

    // -- Stage 4: Generate procedural terrain --
    let seed = random_seed();
    let biome_grid = {
        let mut grid = world.resource_mut::<simulation::grid::WorldGrid>();
        generate_procedural_terrain(&mut grid, seed, NEW_GAME_EROSION_ITERATIONS)
    };
    world.insert_resource(biome_grid);

    // Store the terrain configuration so it persists through saves.
    world.insert_resource(TerrainConfig {
        seed,
        erosion_iterations: NEW_GAME_EROSION_ITERATIONS,
        generated: true,
    });

    // -- Stage 5: Activate tutorial for new games --
    {
        let mut tutorial = world.resource_mut::<simulation::tutorial::TutorialState>();
        tutorial.active = true;
        tutorial.current_step = simulation::tutorial::TutorialStep::Welcome;
        tutorial.completed = false;
    }

    println!("New game started — procedural terrain (seed {seed}) with $50,000 treasury");

    // -- Stage 6: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}
