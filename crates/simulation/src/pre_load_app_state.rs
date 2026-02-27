//! Tracks the [`AppState`] before a save load begins so the game can roll
//! back on failure instead of leaving the player in a broken world.
//!
//! When the UI initiates a load (from main menu or pause menu), it stores
//! the current state here. If the load fails, the save crate reads this
//! resource and transitions back to the previous state.

use bevy::prelude::*;

use crate::app_state::AppState;

/// Stores the [`AppState`] that was active before a load operation started.
///
/// Written by the UI before sending [`LoadGameEvent`]. Consumed by the
/// save system on failure to roll back to the correct screen.
#[derive(Resource, Default)]
pub struct PreLoadAppState(pub Option<AppState>);
