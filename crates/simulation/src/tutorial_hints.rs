use bevy::prelude::*;

use crate::tutorial::{TutorialState, TutorialStep};

// =============================================================================
// Tutorial UI Hint Resource
// =============================================================================

/// Provides per-step UI hints for the tutorial overlay.
///
/// Updated each frame by [`update_tutorial_hints`] based on the current tutorial
/// step. The UI crate reads these fields to draw pulsing highlights and
/// position the camera.
#[derive(Resource, Debug, Clone, Default)]
pub struct TutorialUiHint {
    /// Name of the toolbar category the player should interact with (e.g. "Roads").
    pub highlight_target: Option<&'static str>,
    /// World-space position the camera should auto-focus to at tutorial start.
    pub camera_target: Option<(f32, f32)>,
}

impl TutorialUiHint {
    /// Returns the toolbar highlight target for the given step.
    fn target_for_step(step: TutorialStep) -> Option<&'static str> {
        match step {
            TutorialStep::PlaceRoad => Some("Roads"),
            TutorialStep::ZoneResidential | TutorialStep::ZoneCommercial => Some("Zones"),
            TutorialStep::PlacePowerPlant | TutorialStep::PlaceWaterTower => Some("Utilities"),
            _ => None,
        }
    }

    /// Returns an optional camera focus position for the given step.
    /// The camera moves to the center of the map at the start of the tutorial.
    fn camera_for_step(step: TutorialStep) -> Option<(f32, f32)> {
        match step {
            // Center the camera on the map for the welcome/first-build steps.
            // 256 cells * 16.0 CELL_SIZE / 2 = 2048.0 center.
            TutorialStep::Welcome => Some((2048.0, 2048.0)),
            _ => None,
        }
    }
}

// =============================================================================
// System
// =============================================================================

/// Updates [`TutorialUiHint`] based on the current tutorial step.
pub fn update_tutorial_hints(tutorial: Res<TutorialState>, mut hint: ResMut<TutorialUiHint>) {
    if !tutorial.active {
        hint.highlight_target = None;
        hint.camera_target = None;
        return;
    }

    let step = tutorial.current_step;
    hint.highlight_target = TutorialUiHint::target_for_step(step);
    hint.camera_target = TutorialUiHint::camera_for_step(step);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TutorialHintsPlugin;

impl Plugin for TutorialHintsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialUiHint>()
            .add_systems(Update, update_tutorial_hints.in_set(crate::SimulationUpdateSet::Visual));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_target_for_road_step() {
        assert_eq!(
            TutorialUiHint::target_for_step(TutorialStep::PlaceRoad),
            Some("Roads")
        );
    }

    #[test]
    fn test_hint_target_for_zone_steps() {
        assert_eq!(
            TutorialUiHint::target_for_step(TutorialStep::ZoneResidential),
            Some("Zones")
        );
        assert_eq!(
            TutorialUiHint::target_for_step(TutorialStep::ZoneCommercial),
            Some("Zones")
        );
    }

    #[test]
    fn test_hint_target_for_utility_steps() {
        assert_eq!(
            TutorialUiHint::target_for_step(TutorialStep::PlacePowerPlant),
            Some("Utilities")
        );
        assert_eq!(
            TutorialUiHint::target_for_step(TutorialStep::PlaceWaterTower),
            Some("Utilities")
        );
    }

    #[test]
    fn test_hint_target_none_for_welcome() {
        assert_eq!(
            TutorialUiHint::target_for_step(TutorialStep::Welcome),
            None
        );
    }

    #[test]
    fn test_camera_target_welcome() {
        assert_eq!(
            TutorialUiHint::camera_for_step(TutorialStep::Welcome),
            Some((2048.0, 2048.0))
        );
    }

    #[test]
    fn test_camera_target_none_for_other_steps() {
        assert_eq!(
            TutorialUiHint::camera_for_step(TutorialStep::PlaceRoad),
            None
        );
        assert_eq!(
            TutorialUiHint::camera_for_step(TutorialStep::Completed),
            None
        );
    }
}
