//! Bridge between the simulation-side autosave timer and the save system.
//!
//! Reads `AutosavePending` (set by `simulation::autosave`) and, when true,
//! triggers a `SaveGameEvent`. After the save completes (state returns to
//! `Idle`), copies the save file to the current rotating autosave slot.
//!
//! On WASM, the slot rotation is skipped (IndexedDB saves use a single key).

use bevy::prelude::*;
use simulation::autosave::{AutosaveConfig, AutosavePending};
use simulation::SaveLoadState;

use crate::SaveGameEvent;

// =============================================================================
// State tracking
// =============================================================================

/// Tracks whether an autosave-initiated save is in flight so we know to
/// perform file rotation once it completes.
#[derive(Resource, Default)]
struct AutosaveInFlight {
    /// The slot filename to copy to after the save finishes.
    target_slot_filename: Option<String>,
}

// =============================================================================
// Systems
// =============================================================================

/// Detects `AutosavePending` and converts it into a `SaveGameEvent`.
///
/// Runs in `Update` alongside other event-detection systems. Only fires when
/// the save system is idle (not already saving/loading).
fn trigger_autosave(
    mut pending: ResMut<AutosavePending>,
    mut config: ResMut<AutosaveConfig>,
    mut in_flight: ResMut<AutosaveInFlight>,
    mut save_events: EventWriter<SaveGameEvent>,
    state: Res<State<SaveLoadState>>,
) {
    if !pending.pending {
        return;
    }

    // Only trigger if the save system is idle.
    if *state.get() != SaveLoadState::Idle {
        return;
    }

    // Record which slot to write to, then advance the slot counter.
    let filename = config.current_slot_filename();
    in_flight.target_slot_filename = Some(filename);
    config.advance_slot();

    // Clear the pending flag and fire the save event.
    pending.pending = false;
    save_events.send(SaveGameEvent);
}

/// After an autosave-triggered save completes, copies the main save file
/// to the target autosave slot.
///
/// Only runs on native platforms (WASM uses IndexedDB with a single key).
#[cfg(not(target_arch = "wasm32"))]
fn rotate_autosave_slot(mut in_flight: ResMut<AutosaveInFlight>, state: Res<State<SaveLoadState>>) {
    // Wait until the save operation completes.
    if *state.get() != SaveLoadState::Idle {
        return;
    }

    let filename = match in_flight.target_slot_filename.take() {
        Some(f) => f,
        None => return,
    };

    let source = crate::save_plugin::save_file_path();
    if let Err(e) = std::fs::copy(&source, &filename) {
        warn!(
            "Autosave slot rotation failed: could not copy {} -> {}: {}",
            source, filename, e
        );
    } else {
        info!("Autosave: copied {} -> {}", source, filename);
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub(crate) struct AutosaveBridgePlugin;

impl Plugin for AutosaveBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutosaveInFlight>()
            .add_systems(Update, trigger_autosave);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, rotate_autosave_slot.after(trigger_autosave));
    }
}
