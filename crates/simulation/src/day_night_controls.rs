//! Day/Night cycle visual controls (UX-069).
//!
//! Exposes player controls for the day/night cycle rendering:
//! - Time-of-day slider: set a specific hour for rendering
//! - Lock time option: freeze the visual hour at a specific time (e.g. always daytime)
//! - Cycle speed: normal, fast, or disabled
//!
//! These settings only affect the *visual* rendering of the day/night cycle.
//! The simulation's `GameClock` continues to tick normally regardless of these settings.
//! Settings persist across saves via the `Saveable` trait (bitcode serialization).

use bevy::prelude::*;

use crate::Saveable;

// =============================================================================
// Types
// =============================================================================

/// Controls the speed of the visual day/night cycle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, bitcode::Encode, bitcode::Decode)]
pub enum CycleSpeed {
    /// Normal cycle speed (follows game clock 1:1).
    #[default]
    Normal,
    /// Fast cycle speed (visual hour advances at 2x game clock rate).
    Fast,
    /// Visual cycle is disabled; hour stays fixed (equivalent to lock at current hour).
    Disabled,
}

// =============================================================================
// Resource
// =============================================================================

/// Player-facing controls for the day/night visual cycle.
///
/// This resource controls how the rendering layer interprets the game clock
/// for lighting purposes. The simulation clock itself is unaffected.
#[derive(Resource, Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct DayNightControls {
    /// If `Some(hour)`, the visual hour is locked to this value (0.0..24.0).
    /// The day/night rendering will use this hour instead of the game clock.
    pub locked_hour: Option<f32>,

    /// Speed multiplier for the visual day/night cycle.
    pub cycle_speed: CycleSpeed,

    /// The visual hour used by the rendering system.
    /// Updated each frame based on locked_hour, cycle_speed, and the game clock.
    /// When not locked, this tracks the game clock (possibly at a different speed).
    pub visual_hour: f32,
}

impl Default for DayNightControls {
    fn default() -> Self {
        Self {
            locked_hour: None,
            cycle_speed: CycleSpeed::Normal,
            visual_hour: 6.0, // default start: 6 AM
        }
    }
}

impl DayNightControls {
    /// Returns the effective hour for rendering.
    ///
    /// If a locked hour is set, returns that. Otherwise returns `visual_hour`.
    pub fn effective_hour(&self) -> f32 {
        if let Some(locked) = self.locked_hour {
            locked
        } else {
            self.visual_hour
        }
    }

    /// Returns the visual cycle speed multiplier.
    pub fn speed_multiplier(&self) -> f32 {
        match self.cycle_speed {
            CycleSpeed::Normal => 1.0,
            CycleSpeed::Fast => 2.0,
            CycleSpeed::Disabled => 0.0,
        }
    }
}

// =============================================================================
// Saveable
// =============================================================================

impl Saveable for DayNightControls {
    const SAVE_KEY: &'static str = "day_night_controls";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Always save: even default settings should persist so the player's
        // choice of "normal" is explicit after loading.
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Updates the visual hour based on the game clock and day/night control settings.
///
/// - When locked, `visual_hour` stays at the locked value.
/// - When in `Disabled` cycle speed, `visual_hour` freezes at its current value.
/// - Otherwise, `visual_hour` follows the game clock hour (possibly at an altered rate).
pub fn update_visual_hour(
    clock: Res<crate::time_of_day::GameClock>,
    mut controls: ResMut<DayNightControls>,
) {
    if let Some(locked) = controls.locked_hour {
        // Locked: visual hour is always the locked value
        controls.visual_hour = locked;
        return;
    }

    match controls.cycle_speed {
        CycleSpeed::Normal => {
            // Follow game clock directly
            controls.visual_hour = clock.hour;
        }
        CycleSpeed::Fast => {
            // Visual hour advances at 2x the game clock rate.
            // We compute a doubled-speed offset from the game clock.
            controls.visual_hour = (clock.hour * 2.0) % 24.0;
        }
        CycleSpeed::Disabled => {
            // Visual hour stays frozen at its current value -- no update needed.
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct DayNightControlsPlugin;

impl Plugin for DayNightControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayNightControls>().add_systems(
            Update,
            update_visual_hour.in_set(crate::SimulationUpdateSet::Input),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DayNightControls>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_controls() {
        let controls = DayNightControls::default();
        assert_eq!(controls.locked_hour, None);
        assert_eq!(controls.cycle_speed, CycleSpeed::Normal);
        assert!((controls.visual_hour - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_hour_unlocked() {
        let controls = DayNightControls {
            locked_hour: None,
            visual_hour: 14.5,
            ..Default::default()
        };
        assert!((controls.effective_hour() - 14.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_hour_locked() {
        let controls = DayNightControls {
            locked_hour: Some(12.0),
            visual_hour: 20.0, // should be ignored
            ..Default::default()
        };
        assert!((controls.effective_hour() - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_multiplier() {
        assert!(
            (DayNightControls {
                cycle_speed: CycleSpeed::Normal,
                ..Default::default()
            }
            .speed_multiplier()
                - 1.0)
                .abs()
                < f32::EPSILON
        );

        assert!(
            (DayNightControls {
                cycle_speed: CycleSpeed::Fast,
                ..Default::default()
            }
            .speed_multiplier()
                - 2.0)
                .abs()
                < f32::EPSILON
        );

        assert!(
            (DayNightControls {
                cycle_speed: CycleSpeed::Disabled,
                ..Default::default()
            }
            .speed_multiplier()
                - 0.0)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let controls = DayNightControls {
            locked_hour: Some(15.5),
            cycle_speed: CycleSpeed::Fast,
            visual_hour: 15.5,
        };
        let bytes = controls.save_to_bytes().unwrap();
        let loaded = DayNightControls::load_from_bytes(&bytes);
        assert!((loaded.locked_hour.unwrap() - 15.5).abs() < f32::EPSILON);
        assert_eq!(loaded.cycle_speed, CycleSpeed::Fast);
    }

    #[test]
    fn test_saveable_default_still_saves() {
        let controls = DayNightControls::default();
        // Even default state should save (player explicitly chose Normal)
        assert!(controls.save_to_bytes().is_some());
    }
}
