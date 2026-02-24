use bevy::prelude::*;
use simulation::notifications::{NotificationEvent, NotificationPriority};
use simulation::reset_commuting_on_load::PostLoadResetPending;
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

use crate::despawn::despawn_all_game_entities;
use crate::file_header::{unwrap_header, UnwrapResult};
use crate::restore_resources::restore_resources_from_save;
use crate::save_error::SaveError;
use crate::save_plugin::PendingLoadBytes;
use crate::serialization::{migrate_save_with_report, SaveData};
use crate::spawn_entities::spawn_entities_from_save;

/// Exclusive system that performs the entire load operation with full world
/// access.  Entity despawns are immediate (no deferred Commands).
/// Runs on `OnEnter(SaveLoadState::Loading)`, then transitions back to `Idle`.
pub(crate) fn exclusive_load(world: &mut World) {
    if let Err(e) = exclusive_load_inner(world) {
        let msg = format!("Load failed: {e}");
        error!("{msg}");
        world.send_event(NotificationEvent {
            text: msg,
            priority: NotificationPriority::Warning,
            location: None,
        });
    }

    // Always transition back to Idle, even on error.
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}

/// Inner implementation that returns `Result` for proper error propagation.
fn exclusive_load_inner(world: &mut World) -> Result<(), SaveError> {
    // Take pending bytes (either from native file read or WASM IndexedDB).
    let bytes = world.resource_mut::<PendingLoadBytes>().0.take();
    let bytes = bytes.ok_or(SaveError::NoData)?;

    // -- Stage 0: Validate file header and extract payload --
    let payload = match unwrap_header(&bytes) {
        Ok(UnwrapResult::WithHeader { header, payload }) => {
            info!(
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
            info!("Loading legacy save file (no header)");
            payload
        }
        Err(e) => {
            return Err(SaveError::Decode(format!("Invalid file header: {e}")));
        }
    };

    // -- Stage 1: Parse and migrate --
    let mut save = SaveData::decode(payload)?;

    let report = migrate_save_with_report(&mut save)?;

    if report.steps_applied > 0 {
        info!(
            "Migrated save from v{} to v{} ({} steps applied)",
            report.original_version, report.final_version, report.steps_applied,
        );
        for desc in &report.step_descriptions {
            info!("  - {desc}");
        }
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
        .ok_or_else(|| SaveError::MissingResource("SaveableRegistry".to_string()))?;
    registry.load_all(world, &save.extensions);
    world.insert_resource(registry);

    // -- Stage 6: Signal post-load reset for commuting citizens (SAVE-008) --
    world.insert_resource(PostLoadResetPending);

    #[cfg(not(target_arch = "wasm32"))]
    info!("Loaded save from {}", crate::save_plugin::save_file_path());
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Loaded save from IndexedDB".into());

    Ok(())
}
