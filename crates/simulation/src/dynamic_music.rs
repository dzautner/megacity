//! PLAY-010: Dynamic Music System.
//!
//! Maintains a simulation-side resource describing which music mood should
//! be active based on game context (app state, time of day, budget health,
//! growth rate, milestones). Actual audio playback is handled downstream by
//! the rendering/audio layer; this module owns the data.

use bevy::prelude::*;

use crate::app_state::AppState;
use crate::economy::CityBudget;
use crate::hope_discontent::{CrisisState, HopeDiscontent};
use crate::milestones::MilestoneProgress;
use crate::stats::CityStats;
use crate::time_of_day::GameClock;

// =============================================================================
// Types
// =============================================================================

/// The current musical mood, driven by game context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MusicMood {
    /// Calm, inviting music for the main menu.
    #[default]
    MainMenu,
    /// Default gameplay — city growing at a steady pace.
    BuildingPeaceful,
    /// Rapid growth or lots of construction activity.
    BuildingEnergetic,
    /// Nighttime — slower, quieter musical feel.
    NightCalm,
    /// Budget deficit, disasters, low hope / high discontent.
    Crisis,
    /// Brief celebratory mood when a milestone tier is reached.
    Milestone,
    /// Game is paused — music fades out or goes very quiet.
    Paused,
}

// =============================================================================
// Resource
// =============================================================================

/// Tracks the current music mood and transition state.
///
/// Updated periodically by `update_music_mood`. Downstream audio systems
/// read this resource to drive actual music playback and crossfading.
#[derive(Resource, Debug, Clone)]
pub struct MusicState {
    /// The mood that should currently be playing.
    pub current_mood: MusicMood,
    /// How intense the current mood is (0.0 = subdued, 1.0 = full).
    pub intensity: f32,
    /// Seconds remaining in a mood transition (crossfade).
    pub transition_timer: f32,
    /// The previous population snapshot, used to compute growth rate.
    prev_population: u32,
    /// The milestone tier index last time we checked, for detecting new milestones.
    prev_milestone_index: usize,
    /// Remaining seconds for the Milestone mood before reverting.
    milestone_cooldown: f32,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            current_mood: MusicMood::MainMenu,
            intensity: 0.5,
            transition_timer: 0.0,
            prev_population: 0,
            prev_milestone_index: 0,
            milestone_cooldown: 0.0,
        }
    }
}

impl MusicState {
    /// Duration of a standard crossfade transition in seconds.
    const TRANSITION_DURATION: f32 = 3.0;

    /// Duration the Milestone mood plays before reverting (seconds).
    const MILESTONE_DURATION: f32 = 15.0;

    /// Set a new mood, starting a transition if the mood actually changed.
    fn set_mood(&mut self, mood: MusicMood, intensity: f32) {
        if self.current_mood != mood {
            self.current_mood = mood;
            self.transition_timer = Self::TRANSITION_DURATION;
        }
        self.intensity = intensity.clamp(0.0, 1.0);
    }
}

// =============================================================================
// Update timer
// =============================================================================

/// Controls how often the music mood recalculates (every 5 seconds).
#[derive(Resource)]
struct MusicUpdateTimer(Timer);

impl Default for MusicUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

// =============================================================================
// Constants
// =============================================================================

/// Night hours: music switches to NightCalm between these hours.
const NIGHT_START_HOUR: f32 = 22.0;
const NIGHT_END_HOUR: f32 = 5.0;

/// Budget deficit threshold: treasury below zero triggers crisis consideration.
const DEFICIT_THRESHOLD: f64 = 0.0;

/// Population growth rate (per-tick fraction) above which we consider "energetic".
const ENERGETIC_GROWTH_RATE: f32 = 0.02;

// =============================================================================
// System
// =============================================================================

/// Periodically evaluates game state and updates the music mood.
///
/// Priority order (highest to lowest):
/// 1. AppState::MainMenu → MainMenu
/// 2. AppState::Paused → Paused
/// 3. Milestone just reached → Milestone (temporary)
/// 4. Crisis conditions → Crisis
/// 5. Night hours → NightCalm
/// 6. High growth → BuildingEnergetic
/// 7. Default → BuildingPeaceful
#[allow(clippy::too_many_arguments)]
fn update_music_mood(
    time: Res<Time>,
    mut timer: ResMut<MusicUpdateTimer>,
    app_state: Res<State<AppState>>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    clock: Res<GameClock>,
    hope_discontent: Res<HopeDiscontent>,
    milestones: Res<MilestoneProgress>,
    mut state: ResMut<MusicState>,
) {
    // Tick the transition timer every frame (not gated by the 5s interval).
    if state.transition_timer > 0.0 {
        state.transition_timer = (state.transition_timer - time.delta_secs()).max(0.0);
    }

    // Tick milestone cooldown every frame too.
    if state.milestone_cooldown > 0.0 {
        state.milestone_cooldown = (state.milestone_cooldown - time.delta_secs()).max(0.0);
    }

    // Only re-evaluate mood every 5 seconds.
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    // --- 1. App state overrides ---
    match app_state.get() {
        AppState::MainMenu => {
            state.set_mood(MusicMood::MainMenu, 0.5);
            return;
        }
        AppState::Paused => {
            state.set_mood(MusicMood::Paused, 0.1);
            return;
        }
        AppState::Playing => {} // continue evaluation
    }

    // --- 2. Milestone detection ---
    let current_milestone_index = milestones.current_tier.index();
    if current_milestone_index > state.prev_milestone_index {
        state.prev_milestone_index = current_milestone_index;
        state.milestone_cooldown = MusicState::MILESTONE_DURATION;
        state.set_mood(MusicMood::Milestone, 1.0);
        state.prev_population = stats.population;
        return;
    }

    // Still in milestone celebration window.
    if state.milestone_cooldown > 0.0 {
        // Keep milestone mood active, don't override.
        state.prev_population = stats.population;
        return;
    }

    // --- 3. Crisis detection ---
    let in_crisis = hope_discontent.crisis_state == CrisisState::Crisis
        || budget.treasury < DEFICIT_THRESHOLD;

    if in_crisis {
        let crisis_intensity = if hope_discontent.crisis_state == CrisisState::Crisis {
            0.9
        } else {
            // Budget deficit: intensity based on how deep the deficit is.
            ((-budget.treasury) as f32 / 10_000.0).clamp(0.3, 0.8)
        };
        state.set_mood(MusicMood::Crisis, crisis_intensity);
        state.prev_population = stats.population;
        return;
    }

    // --- 4. Night hours ---
    let is_night = clock.hour >= NIGHT_START_HOUR || clock.hour < NIGHT_END_HOUR;
    if is_night {
        state.set_mood(MusicMood::NightCalm, 0.4);
        state.prev_population = stats.population;
        return;
    }

    // --- 5. Growth rate check ---
    let growth = if state.prev_population > 0 {
        (stats.population as f32 - state.prev_population as f32) / state.prev_population as f32
    } else {
        0.0
    };
    state.prev_population = stats.population;

    if growth > ENERGETIC_GROWTH_RATE {
        let energetic_intensity = (growth / (ENERGETIC_GROWTH_RATE * 5.0)).clamp(0.5, 1.0);
        state.set_mood(MusicMood::BuildingEnergetic, energetic_intensity);
        return;
    }

    // --- 6. Default: peaceful building ---
    // Intensity scales mildly with population.
    let peaceful_intensity = if stats.population > 0 {
        (stats.population as f32 / 50_000.0).clamp(0.3, 0.7)
    } else {
        0.3
    };
    state.set_mood(MusicMood::BuildingPeaceful, peaceful_intensity);
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers the dynamic music state and mood update system.
pub struct DynamicMusicPlugin;

impl Plugin for DynamicMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
            .init_resource::<MusicUpdateTimer>()
            .add_systems(
                Update,
                update_music_mood.in_set(crate::SimulationUpdateSet::Visual),
            );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_music_state_default() {
        let state = MusicState::default();
        assert_eq!(state.current_mood, MusicMood::MainMenu);
        assert_eq!(state.intensity, 0.5);
        assert_eq!(state.transition_timer, 0.0);
    }

    #[test]
    fn test_music_mood_default() {
        let mood = MusicMood::default();
        assert_eq!(mood, MusicMood::MainMenu);
    }

    #[test]
    fn test_set_mood_starts_transition() {
        let mut state = MusicState::default();
        state.set_mood(MusicMood::BuildingPeaceful, 0.6);
        assert_eq!(state.current_mood, MusicMood::BuildingPeaceful);
        assert_eq!(state.transition_timer, MusicState::TRANSITION_DURATION);
        assert!((state.intensity - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_mood_same_mood_no_transition() {
        let mut state = MusicState::default();
        // Already MainMenu, setting MainMenu again should not start transition.
        state.set_mood(MusicMood::MainMenu, 0.8);
        assert_eq!(state.transition_timer, 0.0);
        assert!((state.intensity - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_mood_clamps_intensity() {
        let mut state = MusicState::default();
        state.set_mood(MusicMood::Crisis, 1.5);
        assert!((state.intensity - 1.0).abs() < f32::EPSILON);

        state.set_mood(MusicMood::NightCalm, -0.5);
        assert!(state.intensity.abs() < f32::EPSILON);
    }

    #[test]
    fn test_milestone_detection_fields() {
        let mut state = MusicState::default();
        assert_eq!(state.prev_milestone_index, 0);
        assert_eq!(state.milestone_cooldown, 0.0);

        // Simulate a milestone being reached.
        state.prev_milestone_index = 3;
        state.milestone_cooldown = MusicState::MILESTONE_DURATION;
        state.set_mood(MusicMood::Milestone, 1.0);

        assert_eq!(state.current_mood, MusicMood::Milestone);
        assert_eq!(state.milestone_cooldown, MusicState::MILESTONE_DURATION);
    }

    #[test]
    fn test_all_mood_variants_are_distinct() {
        let moods = [
            MusicMood::MainMenu,
            MusicMood::BuildingPeaceful,
            MusicMood::BuildingEnergetic,
            MusicMood::NightCalm,
            MusicMood::Crisis,
            MusicMood::Milestone,
            MusicMood::Paused,
        ];
        for (i, a) in moods.iter().enumerate() {
            for (j, b) in moods.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }
}
