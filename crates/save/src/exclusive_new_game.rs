use bevy::prelude::*;
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

use crate::despawn::despawn_all_game_entities;
use crate::reset_resources::reset_all_resources;

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

    // -- Stage 4: Generate starter terrain --
    {
        let (width, height) = {
            let grid = world.resource::<simulation::grid::WorldGrid>();
            (grid.width, grid.height)
        };
        let mut grid = world.resource_mut::<simulation::grid::WorldGrid>();
        for y in 0..height {
            for x in 0..width {
                let cell = grid.get_mut(x, y);
                if x < 10 {
                    cell.cell_type = simulation::grid::CellType::Water;
                    cell.elevation = 0.3;
                } else {
                    cell.cell_type = simulation::grid::CellType::Grass;
                    cell.elevation = 0.5;
                }
            }
        }
    }

    // -- Stage 5: Activate tutorial for new games --
    {
        let mut tutorial = world.resource_mut::<simulation::tutorial::TutorialState>();
        tutorial.active = true;
        tutorial.current_step = simulation::tutorial::TutorialStep::Welcome;
        tutorial.completed = false;
    }

    println!("New game started — blank map with $50,000 treasury");

    // -- Stage 6: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}
