//! Integration tests for audio system infrastructure (PLAY-007).
//!
//! Verifies that `AudioSettings` is properly registered as a resource,
//! the `PlaySfxEvent` event type works, and the `Saveable` round-trip
//! preserves all settings.

use crate::audio_settings::{AudioSettings, PlaySfxEvent, SfxEvent};
use crate::test_harness::TestCity;

// =============================================================================
// Resource registration
// =============================================================================

#[test]
fn test_audio_settings_registered_with_defaults() {
    let city = TestCity::new();
    let settings = city.resource::<AudioSettings>();
    assert_eq!(settings.master_volume, 0.7);
    assert_eq!(settings.music_volume, 0.5);
    assert_eq!(settings.sfx_volume, 0.7);
    assert_eq!(settings.ui_volume, 0.7);
    assert!(!settings.muted);
}

// =============================================================================
// Volume calculations
// =============================================================================

#[test]
fn test_effective_volumes_unmuted() {
    let city = TestCity::new();
    let settings = city.resource::<AudioSettings>();

    let sfx = settings.effective_sfx_volume();
    let music = settings.effective_music_volume();
    let ui = settings.effective_ui_volume();

    // master (0.7) * channel
    assert!((sfx - 0.7 * 0.7).abs() < 1e-6);
    assert!((music - 0.7 * 0.5).abs() < 1e-6);
    assert!((ui - 0.7 * 0.7).abs() < 1e-6);
}

#[test]
fn test_muted_effective_volumes_are_zero() {
    let mut city = TestCity::new();
    city.world_mut()
        .resource_mut::<AudioSettings>()
        .toggle_mute();

    let settings = city.resource::<AudioSettings>();
    assert!(settings.muted);
    assert_eq!(settings.effective_sfx_volume(), 0.0);
    assert_eq!(settings.effective_music_volume(), 0.0);
    assert_eq!(settings.effective_ui_volume(), 0.0);
}

#[test]
fn test_toggle_mute_preserves_volumes() {
    let mut city = TestCity::new();
    {
        let mut settings = city.world_mut().resource_mut::<AudioSettings>();
        settings.set_master_volume(0.4);
        settings.set_sfx_volume(0.6);
        settings.toggle_mute();
    }

    let settings = city.resource::<AudioSettings>();
    assert!(settings.muted);
    assert_eq!(settings.master_volume, 0.4);
    assert_eq!(settings.sfx_volume, 0.6);
    assert_eq!(settings.effective_sfx_volume(), 0.0);
}

// =============================================================================
// Saveable round-trip
// =============================================================================

#[test]
fn test_audio_settings_saveable_roundtrip() {
    use crate::Saveable;

    let original = AudioSettings {
        master_volume: 0.3,
        music_volume: 0.8,
        sfx_volume: 0.1,
        ui_volume: 0.9,
        muted: true,
    };

    let bytes = original.save_to_bytes().expect("should serialize");
    let loaded = AudioSettings::load_from_bytes(&bytes);

    assert_eq!(loaded.master_volume, 0.3);
    assert_eq!(loaded.music_volume, 0.8);
    assert_eq!(loaded.sfx_volume, 0.1);
    assert_eq!(loaded.ui_volume, 0.9);
    assert!(loaded.muted);
}

// =============================================================================
// SfxEvent coverage
// =============================================================================

#[test]
fn test_sfx_event_all_variants_constructible() {
    let variants = [
        SfxEvent::ButtonClick,
        SfxEvent::RoadPlace,
        SfxEvent::ZonePaint,
        SfxEvent::BuildingPlace,
        SfxEvent::Demolish,
        SfxEvent::Notification,
        SfxEvent::Warning,
        SfxEvent::Error,
        SfxEvent::Save,
    ];
    assert_eq!(variants.len(), 9, "should have 9 SfxEvent variants");
}

#[test]
fn test_play_sfx_event_creation() {
    let event = PlaySfxEvent::new(SfxEvent::RoadPlace);
    assert_eq!(event.sfx, SfxEvent::RoadPlace);
    assert_eq!(event.volume_scale, 1.0);

    let event = PlaySfxEvent::with_volume(SfxEvent::Warning, 0.5);
    assert_eq!(event.sfx, SfxEvent::Warning);
    assert_eq!(event.volume_scale, 0.5);
}

// =============================================================================
// Volume clamping
// =============================================================================

#[test]
fn test_volume_clamping_in_context() {
    let mut city = TestCity::new();
    {
        let mut settings = city.world_mut().resource_mut::<AudioSettings>();
        settings.set_master_volume(2.0);
        settings.set_sfx_volume(-1.0);
    }

    let settings = city.resource::<AudioSettings>();
    assert_eq!(settings.master_volume, 1.0);
    assert_eq!(settings.sfx_volume, 0.0);
    assert_eq!(settings.effective_sfx_volume(), 0.0);
}

// =============================================================================
// Simulation ticking does not break audio settings
// =============================================================================

#[test]
fn test_audio_settings_survive_ticks() {
    let mut city = TestCity::new();
    {
        let mut settings = city.world_mut().resource_mut::<AudioSettings>();
        settings.set_master_volume(0.5);
        settings.set_music_volume(0.3);
    }

    // Run several simulation ticks.
    city.tick_slow_cycles(5);

    let settings = city.resource::<AudioSettings>();
    assert_eq!(settings.master_volume, 0.5);
    assert_eq!(settings.music_volume, 0.3);
}
