//! Global save/load state machine.
//!
//! Defines [`SaveLoadState`], a Bevy [`States`] enum that isolates save/load
//! operations from gameplay.  Gameplay systems should only run while the state
//! is [`SaveLoadState::Idle`]; the save crate transitions through the other
//! variants when a save, load, or new-game operation is in progress.
//!
//! The state is defined here (in the `simulation` crate) rather than in `save`
//! so that simulation systems can add `run_if(in_state(SaveLoadState::Idle))`
//! guards without introducing a circular dependency.

use bevy::prelude::*;

/// Top-level state governing whether a save/load operation is in progress.
///
/// When the state is anything other than `Idle`, gameplay and rendering
/// systems should be suspended.  The save plugin enters the appropriate
/// non-idle state when an event is received, performs all work in an
/// exclusive system (so despawns/spawns apply immediately), then transitions
/// back to `Idle`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SaveLoadState {
    /// Normal gameplay — no save/load operation in progress.
    #[default]
    Idle,
    /// A save operation is in progress.
    Saving,
    /// A load operation is in progress.
    Loading,
    /// A new-game reset is in progress.
    NewGame,
}
