use bevy::prelude::*;

/// Marker resource: app is running in replay viewer mode.
///
/// Viewer mode is watch-only:
/// - camera movement remains enabled
/// - build/edit input is disabled
/// - UI is reduced to playback controls
#[derive(Resource, Default)]
pub struct ReplayViewerMode;

/// Metadata about the currently loaded replay, used by viewer UI.
#[derive(Resource, Default, Clone)]
pub struct ReplayViewerInfo {
    pub source: String,
    pub start_tick: u64,
    pub end_tick: u64,
    pub entry_count: u64,
}
