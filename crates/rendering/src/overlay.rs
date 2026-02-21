use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverlayMode {
    #[default]
    None,
    Power,
    Water,
    Traffic,
    Pollution,
    LandValue,
    Education,
    Garbage,
    Noise,
    WaterPollution,
    GroundwaterLevel,
    GroundwaterQuality,
    Wind,
}

/// Ordered list of all overlay modes for Tab/Shift+Tab cycling.
const ALL_OVERLAYS: [OverlayMode; 13] = [
    OverlayMode::None,
    OverlayMode::Power,
    OverlayMode::Water,
    OverlayMode::Traffic,
    OverlayMode::Pollution,
    OverlayMode::LandValue,
    OverlayMode::Education,
    OverlayMode::Garbage,
    OverlayMode::Noise,
    OverlayMode::WaterPollution,
    OverlayMode::GroundwaterLevel,
    OverlayMode::GroundwaterQuality,
    OverlayMode::Wind,
];

/// List of overlay modes excluding None, for UI dropdowns.
pub const OVERLAY_CHOICES: [OverlayMode; 12] = [
    OverlayMode::Power,
    OverlayMode::Water,
    OverlayMode::Traffic,
    OverlayMode::Pollution,
    OverlayMode::LandValue,
    OverlayMode::Education,
    OverlayMode::Garbage,
    OverlayMode::Noise,
    OverlayMode::WaterPollution,
    OverlayMode::GroundwaterLevel,
    OverlayMode::GroundwaterQuality,
    OverlayMode::Wind,
];

impl OverlayMode {
    /// Returns the next overlay mode in the cycle (wraps around).
    pub fn next(self) -> Self {
        let idx = ALL_OVERLAYS.iter().position(|&m| m == self).unwrap_or(0);
        ALL_OVERLAYS[(idx + 1) % ALL_OVERLAYS.len()]
    }

    /// Returns the previous overlay mode in the cycle (wraps around).
    pub fn prev(self) -> Self {
        let idx = ALL_OVERLAYS.iter().position(|&m| m == self).unwrap_or(0);
        ALL_OVERLAYS[(idx + ALL_OVERLAYS.len() - 1) % ALL_OVERLAYS.len()]
    }

    /// Human-readable label for display in status bar.
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Power => "Power",
            Self::Water => "Water",
            Self::Traffic => "Traffic",
            Self::Pollution => "Pollution",
            Self::LandValue => "Land Value",
            Self::Education => "Education",
            Self::Garbage => "Garbage",
            Self::Noise => "Noise",
            Self::WaterPollution => "Water Pollution",
            Self::GroundwaterLevel => "Groundwater Level",
            Self::GroundwaterQuality => "Groundwater Quality",
            Self::Wind => "Wind",
        }
    }
}

#[derive(Resource, Default)]
pub struct OverlayState {
    pub mode: OverlayMode,
}

/// How two overlays are combined when dual overlay is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DualOverlayMode {
    /// Alpha-composite both overlays with a configurable blend factor.
    #[default]
    Blend,
    /// Left half of the screen shows overlay A, right half shows overlay B.
    Split,
}

impl DualOverlayMode {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Blend => "Blend",
            Self::Split => "Split",
        }
    }
}

/// State for the dual-overlay feature (UX-029).
///
/// When `secondary` is set to something other than `None`, the rendering
/// system will composite or split-screen the primary overlay (from
/// `OverlayState`) with this secondary overlay.
#[derive(Resource)]
pub struct DualOverlayState {
    /// The secondary overlay to display alongside the primary.
    pub secondary: OverlayMode,
    /// How the two overlays are combined.
    pub mode: DualOverlayMode,
    /// Blend factor for `DualOverlayMode::Blend` (0.0 = only primary, 1.0 = only secondary).
    /// Default is 0.5 (50/50).
    pub blend_factor: f32,
    /// Whether the dual overlay panel is open in the UI.
    pub panel_open: bool,
}

impl Default for DualOverlayState {
    fn default() -> Self {
        Self {
            secondary: OverlayMode::None,
            mode: DualOverlayMode::Blend,
            blend_factor: 0.5,
            panel_open: false,
        }
    }
}

impl DualOverlayState {
    /// Returns true when dual overlay is effectively active
    /// (primary is not None and secondary is not None).
    pub fn is_active(&self, primary: OverlayMode) -> bool {
        primary != OverlayMode::None && self.secondary != OverlayMode::None
    }
}

/// Cycle overlays with Tab (forward) / Shift+Tab (backward).
/// Individual letter-key shortcuts have been removed to resolve keybinding
/// conflicts (see issue #905).  All overlays are reachable via Tab cycling.
pub fn toggle_overlay_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<OverlayState>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if keys.just_pressed(bindings.overlay_cycle_next.key) {
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        overlay.mode = if shift {
            overlay.mode.prev()
        } else {
            overlay.mode.next()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cycles_forward_through_all_overlays() {
        let mut mode = OverlayMode::None;
        let expected = [
            OverlayMode::Power,
            OverlayMode::Water,
            OverlayMode::Traffic,
            OverlayMode::Pollution,
            OverlayMode::LandValue,
            OverlayMode::Education,
            OverlayMode::Garbage,
            OverlayMode::Noise,
            OverlayMode::WaterPollution,
            OverlayMode::GroundwaterLevel,
            OverlayMode::GroundwaterQuality,
            OverlayMode::Wind,
            OverlayMode::None, // wraps back
        ];
        for &exp in &expected {
            mode = mode.next();
            assert_eq!(mode, exp);
        }
    }

    #[test]
    fn prev_cycles_backward_through_all_overlays() {
        let mut mode = OverlayMode::None;
        let expected = [
            OverlayMode::Wind,
            OverlayMode::GroundwaterQuality,
            OverlayMode::GroundwaterLevel,
            OverlayMode::WaterPollution,
            OverlayMode::Noise,
            OverlayMode::Garbage,
            OverlayMode::Education,
            OverlayMode::LandValue,
            OverlayMode::Pollution,
            OverlayMode::Traffic,
            OverlayMode::Water,
            OverlayMode::Power,
            OverlayMode::None, // wraps back
        ];
        for &exp in &expected {
            mode = mode.prev();
            assert_eq!(mode, exp);
        }
    }

    #[test]
    fn next_then_prev_returns_to_original() {
        for &start in &ALL_OVERLAYS {
            assert_eq!(start.next().prev(), start);
            assert_eq!(start.prev().next(), start);
        }
    }

    #[test]
    fn label_returns_non_empty_for_all_variants() {
        for &mode in &ALL_OVERLAYS {
            assert!(!mode.label().is_empty());
        }
    }

    #[test]
    fn dual_overlay_default_blend_factor_is_half() {
        let state = DualOverlayState::default();
        assert!((state.blend_factor - 0.5).abs() < f32::EPSILON);
        assert_eq!(state.mode, DualOverlayMode::Blend);
        assert_eq!(state.secondary, OverlayMode::None);
    }

    #[test]
    fn dual_overlay_is_active_only_when_both_set() {
        let state = DualOverlayState {
            secondary: OverlayMode::Traffic,
            ..Default::default()
        };
        assert!(state.is_active(OverlayMode::Pollution));
        assert!(!state.is_active(OverlayMode::None));

        let state2 = DualOverlayState {
            secondary: OverlayMode::None,
            ..Default::default()
        };
        assert!(!state2.is_active(OverlayMode::Pollution));
    }

    #[test]
    fn dual_overlay_mode_labels() {
        assert_eq!(DualOverlayMode::Blend.label(), "Blend");
        assert_eq!(DualOverlayMode::Split.label(), "Split");
    }

    #[test]
    fn overlay_choices_excludes_none() {
        for &mode in &OVERLAY_CHOICES {
            assert_ne!(mode, OverlayMode::None);
        }
        assert_eq!(OVERLAY_CHOICES.len(), 12);
    }
}
