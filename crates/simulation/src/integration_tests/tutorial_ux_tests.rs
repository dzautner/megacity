//! Unit and integration tests for tutorial UX polish (issue #1702).
//!
//! Covers: TutorialStep::previous(), TutorialState::go_back(),
//! TutorialUiHint targets, and step metadata.

use crate::tutorial::{TutorialState, TutorialStep};
use crate::tutorial_hints::TutorialUiHint;
use crate::Saveable;

// ---- TutorialStep navigation ----

#[test]
fn test_tutorial_step_previous_from_welcome() {
    assert_eq!(TutorialStep::Welcome.previous(), None);
}

#[test]
fn test_tutorial_step_previous_from_place_road() {
    assert_eq!(
        TutorialStep::PlaceRoad.previous(),
        Some(TutorialStep::Welcome)
    );
}

#[test]
fn test_tutorial_step_previous_from_completed() {
    assert_eq!(
        TutorialStep::Completed.previous(),
        Some(TutorialStep::ManageBudget)
    );
}

#[test]
fn test_tutorial_step_previous_round_trip() {
    // For every step except Welcome, prev -> next should return the same step.
    for &step in TutorialStep::ALL.iter().skip(1) {
        let prev = step.previous().expect("non-Welcome should have previous");
        assert_eq!(prev.next(), Some(step));
    }
}

// ---- TutorialState::go_back ----

#[test]
fn test_tutorial_go_back_from_zone_residential() {
    let mut state = TutorialState::default();
    state.active = true;
    state.advance(); // Welcome -> PlaceRoad
    state.advance(); // PlaceRoad -> ZoneResidential
    assert_eq!(state.current_step, TutorialStep::ZoneResidential);

    assert!(state.go_back());
    assert_eq!(state.current_step, TutorialStep::PlaceRoad);

    assert!(state.go_back());
    assert_eq!(state.current_step, TutorialStep::Welcome);

    assert!(!state.go_back()); // Cannot go before Welcome
    assert_eq!(state.current_step, TutorialStep::Welcome);
}

#[test]
fn test_tutorial_go_back_when_completed() {
    let mut state = TutorialState::default();
    state.skip();
    assert!(!state.go_back());
}

// ---- Step metadata ----

#[test]
fn test_all_steps_have_nonempty_metadata() {
    for &step in TutorialStep::ALL {
        assert!(!step.title().is_empty(), "{:?} has empty title", step);
        assert!(
            !step.description().is_empty(),
            "{:?} has empty description",
            step
        );
        assert!(!step.hint().is_empty(), "{:?} has empty hint", step);
    }
}

#[test]
fn test_step_count_consistency() {
    assert_eq!(TutorialStep::ALL.len(), 9);
    assert_eq!(TutorialStep::total_steps(), 8);
}

#[test]
fn test_step_indices() {
    assert_eq!(TutorialStep::Welcome.index(), 0);
    assert_eq!(TutorialStep::PlaceRoad.index(), 1);
    assert_eq!(TutorialStep::Completed.index(), 8);
}

// ---- Full progression ----

#[test]
fn test_full_forward_progression() {
    let mut state = TutorialState::default();
    for i in 0..8 {
        assert_eq!(state.current_step, TutorialStep::ALL[i]);
        assert!(state.advance());
    }
    assert_eq!(state.current_step, TutorialStep::Completed);
    assert!(state.completed);
    assert!(!state.advance());
}

// ---- Saveable ----

#[test]
fn test_saveable_roundtrip_in_progress() {
    let mut state = TutorialState::default();
    state.advance();
    state.advance();
    let bytes = state.save_to_bytes().expect("should save");
    let restored = TutorialState::load_from_bytes(&bytes);
    assert_eq!(restored.current_step, TutorialStep::ZoneResidential);
}

#[test]
fn test_saveable_default_returns_none() {
    let state = TutorialState::default();
    assert!(state.save_to_bytes().is_none());
}

#[test]
fn test_saveable_completed_roundtrip() {
    let mut state = TutorialState::default();
    state.skip();
    let bytes = state.save_to_bytes().expect("should save");
    let restored = TutorialState::load_from_bytes(&bytes);
    assert!(restored.completed);
}

// ---- TutorialUiHint ----

#[test]
fn test_hint_highlight_roads() {
    let hint = TutorialUiHint::default();
    // We test via the public fields after the system would run, but we can
    // at least verify the default is None.
    assert!(hint.highlight_target.is_none());
    assert!(hint.camera_target.is_none());
}

// ---- is_manual_step ----

#[test]
fn test_manual_steps() {
    let manual = [
        TutorialStep::Welcome,
        TutorialStep::ManageBudget,
        TutorialStep::Completed,
    ];
    for step in manual {
        let state = TutorialState {
            current_step: step,
            ..Default::default()
        };
        assert!(state.is_manual_step(), "{:?} should be manual", step);
    }
}

#[test]
fn test_non_manual_steps() {
    let auto = [
        TutorialStep::PlaceRoad,
        TutorialStep::ZoneResidential,
        TutorialStep::ZoneCommercial,
        TutorialStep::PlacePowerPlant,
        TutorialStep::PlaceWaterTower,
        TutorialStep::ObserveGrowth,
    ];
    for step in auto {
        let state = TutorialState {
            current_step: step,
            ..Default::default()
        };
        assert!(!state.is_manual_step(), "{:?} should not be manual", step);
    }
}
