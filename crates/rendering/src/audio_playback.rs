//! Audio playback system that consumes `PlaySfxEvent` events.
//!
//! Currently logs each event at debug level since no audio asset files
//! exist yet. When `.ogg` assets are added, this module can be extended
//! to load and play them via Bevy's `AudioPlayer` API.

use bevy::prelude::*;

use simulation::audio_settings::{AudioSettings, PlaySfxEvent};

/// System that reads [`PlaySfxEvent`] events each frame and logs them.
///
/// Respects [`AudioSettings`]: skips events when muted, and calculates
/// the effective volume as `event.volume_scale * settings.effective_sfx_volume()`.
fn consume_sfx_events(
    mut events: EventReader<PlaySfxEvent>,
    settings: Res<AudioSettings>,
) {
    for event in events.read() {
        let channel_volume = settings.effective_sfx_volume();
        if channel_volume == 0.0 {
            // Muted or zero volume — discard the event silently.
            continue;
        }
        let effective_volume = event.volume_scale * channel_volume;
        debug!("SFX: {:?} vol={:.2}", event.sfx, effective_volume);
    }
}

/// Plugin that wires up the SFX event consumer system.
pub struct AudioPlaybackPlugin;

impl Plugin for AudioPlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, consume_sfx_events);
    }
}
