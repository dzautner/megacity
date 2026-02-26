//! Integration tests for tutorial trigger behavior (issue #1701).
//!
//! Verifies that the tutorial only activates on New Game, not on default
//! resource initialization or save/load round-trips.

use crate::test_harness::TestCity;
use crate::tutorial::{TutorialState, TutorialStep};

#[test]
fn test_tutorial_default_is_inactive() {
    let state = TutorialState::default();
    assert!(!state.active, "Default TutorialState must be inactive");
    assert!(!state.completed);
    assert_eq!(state.current_step, TutorialStep::Welcome);
}

#[test]
fn test_tutorial_skip_works() {
    let mut state = TutorialState::default();
    state.active = true; // simulate new-game activation
    state.skip();
    assert!(!state.active);
    assert!(state.completed);
    assert_eq!(state.current_step, TutorialStep::Completed);
}

#[test]
fn test_tutorial_advance_from_active() {
    let mut state = TutorialState::default();
    state.active = true; // simulate new-game activation
    assert!(state.advance());
    assert_eq!(state.current_step, TutorialStep::PlaceRoad);
    assert!(state.active);
    assert!(!state.completed);
}

#[test]
fn test_tutorial_saveable_roundtrip_preserves_state() {
    use crate::Saveable;

    let mut state = TutorialState::default();
    state.active = true;
    state.advance(); // PlaceRoad
    state.advance(); // ZoneResidential

    let bytes = state.save_to_bytes().expect("in-progress should save");
    let restored = TutorialState::load_from_bytes(&bytes);
    assert_eq!(restored.current_step, TutorialStep::ZoneResidential);
    assert!(restored.active);
    assert!(!restored.completed);
}

#[test]
fn test_tutorial_inactive_in_test_city() {
    let city = TestCity::new();
    let tutorial = city.resource::<TutorialState>();
    assert!(!tutorial.active, "TestCity tutorial should be inactive");
    assert!(tutorial.completed, "TestCity tutorial should be completed");
}
