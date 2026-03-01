//! Integration tests verifying that action error reasons appear in
//! `CityObservation::recent_action_results` (#1949).

use crate::game_actions::queue::ActionSource;
use crate::game_actions::{ActionQueue, GameAction};
use crate::grid::RoadType;
use crate::observation_builder::CurrentObservation;
use crate::test_harness::TestCity;

#[test]
fn test_action_error_reason_appears_in_observation() {
    let mut city = TestCity::new().with_budget(100_000.0);

    // Push an out-of-bounds road action that will fail
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (300, 300),
                end: (310, 300),
                road_type: RoadType::Local,
            },
        );
    }

    city.tick(1);

    let obs = &city.resource::<CurrentObservation>().observation;
    assert!(!obs.recent_action_results.is_empty(), "Should have action results");

    let entry = &obs.recent_action_results.last().unwrap();
    assert!(!entry.success, "Action should have failed");
    assert_eq!(
        entry.error.as_deref(),
        Some("OutOfBounds"),
        "Error reason should be OutOfBounds"
    );
}

#[test]
fn test_successful_action_has_no_error_field() {
    let mut city = TestCity::new().with_budget(100_000.0);

    // Push a valid road action that will succeed
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (10, 10),
                end: (15, 10),
                road_type: RoadType::Local,
            },
        );
    }

    city.tick(1);

    let obs = &city.resource::<CurrentObservation>().observation;
    assert!(!obs.recent_action_results.is_empty(), "Should have action results");

    let entry = &obs.recent_action_results.last().unwrap();
    assert!(entry.success, "Action should have succeeded");
    assert!(
        entry.error.is_none(),
        "Successful action should not have an error"
    );
}

#[test]
fn test_insufficient_funds_error_in_observation() {
    let mut city = TestCity::new().with_budget(0.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (5, 5),
                end: (50, 5),
                road_type: RoadType::Highway,
            },
        );
    }

    city.tick(1);

    let obs = &city.resource::<CurrentObservation>().observation;
    let entry = &obs.recent_action_results.last().unwrap();
    assert!(!entry.success);
    assert_eq!(
        entry.error.as_deref(),
        Some("InsufficientFunds"),
        "Error reason should be InsufficientFunds"
    );
}

#[test]
fn test_error_field_serializes_correctly() {
    use crate::city_observation::ActionResultEntry;

    // Error case: error field present
    let entry_with_error = ActionResultEntry {
        action_summary: "PlaceRoadLine".into(),
        success: false,
        warning: None,
        error: Some("OutOfBounds".into()),
    };
    let json = serde_json::to_string(&entry_with_error).unwrap();
    assert!(json.contains("\"error\":\"OutOfBounds\""));

    // Success case: error field omitted from JSON
    let entry_success = ActionResultEntry {
        action_summary: "PlaceRoadLine".into(),
        success: true,
        warning: None,
        error: None,
    };
    let json = serde_json::to_string(&entry_success).unwrap();
    assert!(
        !json.contains("error"),
        "error field should be skipped when None"
    );

    // Backward compat: deserialize JSON without error field
    let old_json = r#"{"action_summary":"test","success":true}"#;
    let entry: ActionResultEntry = serde_json::from_str(old_json).unwrap();
    assert!(entry.error.is_none());
}
