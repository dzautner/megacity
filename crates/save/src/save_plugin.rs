use bevy::prelude::*;
use simulation::notifications::{NotificationEvent, NotificationPriority};
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

#[cfg(target_arch = "wasm32")]
use crate::wasm_idb;

// ---------------------------------------------------------------------------
// Buffer resources
// ---------------------------------------------------------------------------

/// Holds raw bytes loaded from disk (native) or IndexedDB (WASM) that the
/// exclusive load system will parse and restore.
#[derive(Resource, Default)]
pub(crate) struct PendingLoadBytes(pub(crate) Option<Vec<u8>>);

/// On WASM, holds bytes arriving from an async IndexedDB read.
/// The `poll_wasm_load` system checks this each frame and, when data arrives,
/// stores it in `PendingLoadBytes` and triggers state transition.
#[cfg(target_arch = "wasm32")]
#[derive(Resource, Default)]
pub(crate) struct WasmLoadBuffer(
    pub(crate) std::sync::Arc<std::sync::Mutex<Option<Result<Vec<u8>, String>>>>,
);

/// On WASM, holds save error messages from async IndexedDB writes.
/// The `poll_wasm_save_error` system checks this each frame and, when an error
/// is present, emits a user-facing notification.
#[cfg(target_arch = "wasm32")]
#[derive(Resource, Default)]
pub(crate) struct WasmSaveErrorBuffer(pub(crate) std::sync::Arc<std::sync::Mutex<Option<String>>>);

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Event)]
pub struct SaveGameEvent;

#[derive(Event)]
pub struct LoadGameEvent;

#[derive(Event)]
pub struct NewGameEvent;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            .add_event::<NewGameEvent>()
            .init_resource::<SaveableRegistry>()
            .init_resource::<PendingLoadBytes>();

        // On WASM, register IndexedDB async load infrastructure.
        #[cfg(target_arch = "wasm32")]
        app.init_resource::<WasmLoadBuffer>();
        #[cfg(target_arch = "wasm32")]
        app.init_resource::<WasmSaveErrorBuffer>();

        // Event detection: runs every frame, reads events and triggers state
        // transitions.  These are lightweight systems that only read events.
        app.add_systems(Update, (detect_save_event, detect_new_game_event));

        // Native: synchronous load event detection (reads file, stores bytes,
        // transitions to Loading state).
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, detect_load_event);

        // WASM: async two-phase load detection.
        // 1) `start_wasm_load` kicks off async IndexedDB read
        // 2) `poll_wasm_load` checks for completed read and transitions to Loading
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            (
                start_wasm_load,
                poll_wasm_load.after(start_wasm_load),
                poll_wasm_save_error,
            ),
        );

        // Exclusive systems for each state: these run on state entry,
        // perform all work with exclusive world access, and transition back
        // to Idle.
        app.add_systems(
            OnEnter(SaveLoadState::Saving),
            crate::exclusive_save::exclusive_save,
        );
        app.add_systems(
            OnEnter(SaveLoadState::Loading),
            crate::exclusive_load::exclusive_load,
        );
        app.add_systems(
            OnEnter(SaveLoadState::NewGame),
            crate::exclusive_new_game::exclusive_new_game,
        );

        // Autosave bridge: converts simulation-side timer triggers into save events.
        app.add_plugins(crate::autosave_bridge::AutosaveBridgePlugin);
    }
}

// ---------------------------------------------------------------------------
// Event detection systems (lightweight, run in Update)
// ---------------------------------------------------------------------------

/// Detects `SaveGameEvent` and transitions to `Saving` state.
fn detect_save_event(
    mut events: EventReader<SaveGameEvent>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
) {
    if events.read().next().is_some() {
        // Drain remaining events (only process one per frame).
        events.read().for_each(drop);
        next_state.set(SaveLoadState::Saving);
    }
}

/// Detects `NewGameEvent` and transitions to `NewGame` state.
fn detect_new_game_event(
    mut events: EventReader<NewGameEvent>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
) {
    if events.read().next().is_some() {
        events.read().for_each(drop);
        next_state.set(SaveLoadState::NewGame);
    }
}

/// Native: detects `LoadGameEvent`, reads save file, stores bytes, and
/// transitions to `Loading` state.  File I/O errors are surfaced as
/// notifications instead of being silently swallowed.
#[cfg(not(target_arch = "wasm32"))]
fn detect_load_event(
    mut events: EventReader<LoadGameEvent>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
    mut pending: ResMut<PendingLoadBytes>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    if events.read().next().is_some() {
        events.read().for_each(drop);
        let path = save_file_path();
        match std::fs::read(&path) {
            Ok(bytes) => {
                pending.0 = Some(bytes);
                next_state.set(SaveLoadState::Loading);
            }
            Err(e) => {
                let save_err = crate::save_error::SaveError::from(e);
                let msg = format!("Load failed: {save_err}");
                error!("{msg}");
                notifications.send(NotificationEvent {
                    text: msg,
                    priority: NotificationPriority::Warning,
                    location: None,
                });
            }
        }
    }
}

/// WASM phase 1: consumes `LoadGameEvent` and kicks off an async IndexedDB read.
#[cfg(target_arch = "wasm32")]
fn start_wasm_load(mut events: EventReader<LoadGameEvent>, buffer: Res<WasmLoadBuffer>) {
    for _ in events.read() {
        let slot = buffer.0.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = wasm_idb::idb_load().await;
            *slot.lock().unwrap() = Some(result);
        });
    }
}

/// WASM phase 2: polls the shared buffer; when bytes arrive, stores them in
/// `PendingLoadBytes` and transitions to `Loading` state.
#[cfg(target_arch = "wasm32")]
fn poll_wasm_load(
    buffer: Res<WasmLoadBuffer>,
    mut pending: ResMut<PendingLoadBytes>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
) {
    let mut slot = buffer.0.lock().unwrap();
    if let Some(result) = slot.take() {
        match result {
            Ok(bytes) => {
                pending.0 = Some(bytes);
                next_state.set(SaveLoadState::Loading);
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load from IndexedDB: {}", e).into());
            }
        }
    }
}

/// Polls the WASM save error buffer each frame and, when an error is present,
/// emits a user-facing notification so the player knows the save failed.
#[cfg(target_arch = "wasm32")]
fn poll_wasm_save_error(
    buffer: Res<WasmSaveErrorBuffer>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    let mut slot = buffer.0.lock().unwrap();
    if let Some(error_msg) = slot.take() {
        web_sys::console::error_1(&format!("Save error surfaced to UI: {}", error_msg).into());
        notifications.send(NotificationEvent {
            text: error_msg,
            priority: NotificationPriority::Warning,
            location: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn save_file_path() -> String {
    "megacity_save.bin".to_string()
}
