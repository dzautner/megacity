//! Top-level application state machine.
//!
//! Defines [`AppState`], a Bevy [`States`] enum that governs top-level game
//! flow: main menu, playing, and paused.  Simulation systems should only run
//! when the state is [`AppState::Playing`].
//!
//! The state is defined here (in the `simulation` crate) rather than in `app`
//! or `ui` so that all crates can gate systems on it without circular
//! dependencies — the same pattern used by [`SaveLoadState`].

use bevy::prelude::*;

/// Top-level game state governing application flow.
///
/// Simulation sets are gated behind `in_state(AppState::Playing)`, so the
/// game world is frozen in both `MainMenu` and `Paused` states.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// The main menu screen — no simulation running.
    #[default]
    MainMenu,
    /// Active gameplay — simulation ticks are running.
    Playing,
    /// Gameplay is paused — world is visible but simulation is frozen.
    Paused,
}
