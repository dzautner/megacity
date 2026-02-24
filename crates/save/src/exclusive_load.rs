use bevy::prelude::*;
use simulation::reset_commuting_on_load::PostLoadResetPending;
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

use crate::despawn::despawn_all_game_entities;
use crate::file_header::{unwrap_header, UnwrapResult};
use crate::restore_resources::restore_resources_from_save;
use crate::save_plugin::PendingLoadBytes;
use crate::serialization::{migrate_save, SaveData, CURRENT_SAVE_VERSION};
use crate::spawn_entities::spawn_entities_from_save;

/// Exclusive system that performs the entire load operation with full world
/// access.  Entity despawns are immediate (no deferred Commands).
/// Runs on `OnEnter(SaveLoadState::Loading)`, then transitions back to `Idle`.
pub(crate) fn exclusive_load(world: &mut World) {
    // Take pending bytes (either from native file read or WASM IndexedDB).
    let bytes = world.resource_mut::<PendingLoadBytes>().0.take();
    let Some(bytes) = bytes else {
        eprintln!("exclusive_load: no pending bytes — skipping");
        world
            .resource_mut::<NextState<SaveLoadState>>()
            .set(SaveLoadState::Idle);
        return;
    };

    // -- Stage 0: Validate file header and extract payload --
    let payload = match unwrap_header(&bytes) {
        Ok(UnwrapResult::WithHeader { header, payload }) => {
            println!(
                "Save file header: format v{}, flags {:#X}, timestamp {}, \
                 data size {}, checksum {:#010X}",
                header.format_version,
                header.flags,
                header.timestamp,
                header.uncompressed_size,
                header.checksum,
            );
            payload
        }
        Ok(UnwrapResult::Legacy(payload)) => {
            println!("Loading legacy save file (no header)");
            payload
        }
        Err(e) => {
            eprintln!("Failed to read save file: {}", e);
            world
                .resource_mut::<NextState<SaveLoadState>>()
                .set(SaveLoadState::Idle);
            return;
        }
    };

    // -- Stage 1: Parse and migrate --
    let mut save = match SaveData::decode(payload) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to decode save: {}", e);
            world
                .resource_mut::<NextState<SaveLoadState>>()
                .set(SaveLoadState::Idle);
            return;
        }
    };

    let old_version = match migrate_save(&mut save) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Save migration failed: {}", e);
            world
                .resource_mut::<NextState<SaveLoadState>>()
                .set(SaveLoadState::Idle);
            return;
        }
    };
    if old_version != CURRENT_SAVE_VERSION {
        println!(
            "Migrated save from v{} to v{}",
            old_version, CURRENT_SAVE_VERSION
        );
    }

    // -- Stage 2: Despawn existing entities (immediate, not deferred) --
    despawn_all_game_entities(world);

    // -- Stage 3: Restore resources --
    restore_resources_from_save(world, &save);

    // -- Stage 4: Spawn entities --
    spawn_entities_from_save(world, &save);

    // -- Stage 5: Apply extension map via SaveableRegistry --
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    registry.load_all(world, &save.extensions);
    world.insert_resource(registry);

    // -- Stage 6: Signal post-load reset for commuting citizens (SAVE-008) --
    world.insert_resource(PostLoadResetPending);

    #[cfg(not(target_arch = "wasm32"))]
    println!("Loaded save from {}", crate::save_plugin::save_file_path());
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Loaded save from IndexedDB".into());

    // -- Stage 7: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}
