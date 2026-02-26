//! PLAY-007: Audio System Infrastructure.
//!
//! Provides the foundational audio settings resource (`AudioSettings`) with
//! per-channel volume controls, mute toggle, and persistence via `Saveable`.
//! Also defines `SfxEvent` / `PlaySfxEvent` for triggering sound effects
//! from any system. Actual audio playback is handled downstream (rendering
//! or app crate); this module owns the data layer.

use bevy::prelude::*;

use crate::keybindings::KeyBindings;
use crate::Saveable;

// =============================================================================
// Sound effect event types
// =============================================================================

/// Categories of sound effects that can be triggered throughout the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfxEvent {
    /// UI button click.
    ButtonClick,
    /// Road segment placed on the map.
    RoadPlace,
    /// Zone brush painted on the map.
    ZonePaint,
    /// Building placed on the map.
    BuildingPlace,
    /// Structure demolished / bulldozed.
    Demolish,
    /// Informational notification.
    Notification,
    /// Warning notification (budget, disasters, etc.).
    Warning,
    /// Error feedback (invalid placement, insufficient funds).
    Error,
    /// Game saved successfully.
    Save,
}

/// Event sent by gameplay systems to request a sound effect.
///
/// Downstream audio playback systems (in rendering or app crates) should
/// consume this event each frame and map `SfxEvent` to actual audio assets.
#[derive(Event, Debug, Clone)]
pub struct PlaySfxEvent {
    /// Which sound effect to play.
    pub sfx: SfxEvent,
    /// Volume multiplier on top of the channel volume (0.0-1.0).
    /// Defaults to 1.0 (full channel volume).
    pub volume_scale: f32,
}

impl PlaySfxEvent {
    /// Create an event for the given sound effect at full channel volume.
    pub fn new(sfx: SfxEvent) -> Self {
        Self {
            sfx,
            volume_scale: 1.0,
        }
    }

    /// Create an event with a custom volume scale.
    pub fn with_volume(sfx: SfxEvent, volume_scale: f32) -> Self {
        Self {
            sfx,
            volume_scale: volume_scale.clamp(0.0, 1.0),
        }
    }
}

// =============================================================================
// AudioSettings resource
// =============================================================================

/// Central audio configuration resource.
///
/// All volume values are in the range `0.0` (silent) to `1.0` (full).
/// The `muted` flag overrides all channels to zero without losing the
/// stored volume levels, so un-muting restores previous settings.
#[derive(Resource, Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct AudioSettings {
    /// Master volume multiplier applied to all channels.
    pub master_volume: f32,
    /// Background music volume.
    pub music_volume: f32,
    /// Sound effects volume (actions, impacts, ambient loops).
    pub sfx_volume: f32,
    /// UI interaction sounds volume (clicks, hovers).
    pub ui_volume: f32,
    /// When `true`, all effective volumes return 0.
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            music_volume: 0.5,
            sfx_volume: 0.7,
            ui_volume: 0.7,
            muted: false,
        }
    }
}

impl AudioSettings {
    /// Effective SFX volume: returns 0 when muted, otherwise `master * sfx`.
    pub fn effective_sfx_volume(&self) -> f32 {
        if self.muted {
            return 0.0;
        }
        self.master_volume * self.sfx_volume
    }

    /// Effective music volume: returns 0 when muted, otherwise `master * music`.
    pub fn effective_music_volume(&self) -> f32 {
        if self.muted {
            return 0.0;
        }
        self.master_volume * self.music_volume
    }

    /// Effective UI volume: returns 0 when muted, otherwise `master * ui`.
    pub fn effective_ui_volume(&self) -> f32 {
        if self.muted {
            return 0.0;
        }
        self.master_volume * self.ui_volume
    }

    /// Toggle the mute state.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Set master volume, clamped to `[0.0, 1.0]`.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume, clamped to `[0.0, 1.0]`.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Set SFX volume, clamped to `[0.0, 1.0]`.
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Set UI volume, clamped to `[0.0, 1.0]`.
    pub fn set_ui_volume(&mut self, volume: f32) {
        self.ui_volume = volume.clamp(0.0, 1.0);
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl Saveable for AudioSettings {
    const SAVE_KEY: &'static str = "audio_settings";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Mute toggle system
// =============================================================================

/// System that listens for the `M` key to toggle audio mute.
///
/// Uses `Option<Res<ButtonInput<KeyCode>>>` so the system is a no-op in
/// headless test contexts where Bevy's `InputPlugin` is not present.
fn mute_toggle_system(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    _bindings: Res<KeyBindings>,
    mut settings: ResMut<AudioSettings>,
) {
    let Some(keys) = keys else {
        return;
    };
    // M key for mute toggle (not bound in the keybindings system).
    if keys.just_pressed(KeyCode::KeyM) {
        settings.toggle_mute();
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers audio settings, events, and the mute toggle system.
pub struct AudioSettingsPlugin;

impl Plugin for AudioSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>()
            .add_event::<PlaySfxEvent>()
            .add_systems(
                Update,
                mute_toggle_system.in_set(crate::SimulationUpdateSet::Input),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<AudioSettings>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_volumes() {
        let s = AudioSettings::default();
        assert_eq!(s.master_volume, 0.7);
        assert_eq!(s.music_volume, 0.5);
        assert_eq!(s.sfx_volume, 0.7);
        assert_eq!(s.ui_volume, 0.7);
        assert!(!s.muted);
    }

    #[test]
    fn test_effective_sfx_volume() {
        let s = AudioSettings::default();
        let expected = 0.7 * 0.7;
        assert!((s.effective_sfx_volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_effective_music_volume() {
        let s = AudioSettings::default();
        let expected = 0.7 * 0.5;
        assert!((s.effective_music_volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_effective_ui_volume() {
        let s = AudioSettings::default();
        let expected = 0.7 * 0.7;
        assert!((s.effective_ui_volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_muted_returns_zero() {
        let mut s = AudioSettings::default();
        s.muted = true;
        assert_eq!(s.effective_sfx_volume(), 0.0);
        assert_eq!(s.effective_music_volume(), 0.0);
        assert_eq!(s.effective_ui_volume(), 0.0);
    }

    #[test]
    fn test_toggle_mute() {
        let mut s = AudioSettings::default();
        assert!(!s.muted);
        s.toggle_mute();
        assert!(s.muted);
        s.toggle_mute();
        assert!(!s.muted);
    }

    #[test]
    fn test_set_volume_clamps() {
        let mut s = AudioSettings::default();
        s.set_master_volume(1.5);
        assert_eq!(s.master_volume, 1.0);
        s.set_master_volume(-0.5);
        assert_eq!(s.master_volume, 0.0);
        s.set_sfx_volume(2.0);
        assert_eq!(s.sfx_volume, 1.0);
        s.set_music_volume(-1.0);
        assert_eq!(s.music_volume, 0.0);
        s.set_ui_volume(0.5);
        assert_eq!(s.ui_volume, 0.5);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let settings = AudioSettings {
            master_volume: 0.3,
            music_volume: 0.8,
            sfx_volume: 0.1,
            ui_volume: 0.9,
            muted: true,
        };
        let bytes = settings.save_to_bytes().unwrap();
        let loaded = AudioSettings::load_from_bytes(&bytes);
        assert_eq!(loaded.master_volume, 0.3);
        assert_eq!(loaded.music_volume, 0.8);
        assert_eq!(loaded.sfx_volume, 0.1);
        assert_eq!(loaded.ui_volume, 0.9);
        assert!(loaded.muted);
    }

    #[test]
    fn test_play_sfx_event_new() {
        let event = PlaySfxEvent::new(SfxEvent::ButtonClick);
        assert_eq!(event.sfx, SfxEvent::ButtonClick);
        assert_eq!(event.volume_scale, 1.0);
    }

    #[test]
    fn test_play_sfx_event_with_volume_clamps() {
        let event = PlaySfxEvent::with_volume(SfxEvent::Save, 2.0);
        assert_eq!(event.volume_scale, 1.0);
        let event = PlaySfxEvent::with_volume(SfxEvent::Error, -1.0);
        assert_eq!(event.volume_scale, 0.0);
    }
}
