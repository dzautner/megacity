use super::*;
use crate::grid::{RoadType, ZoneType};

#[test]
fn test_game_action_serialization() {
    let action = GameAction::SetSpeed { speed: 2 };
    let json = serde_json::to_string(&action).unwrap();
    let decoded: GameAction = serde_json::from_str(&json).unwrap();
    assert_eq!(action, decoded);

    let action = GameAction::PlaceRoadLine { 
        start: (10, 20), 
        end: (30, 40), 
        road_type: RoadType::Avenue 
    };
    let json = serde_json::to_string(&action).unwrap();
    let decoded: GameAction = serde_json::from_str(&json).unwrap();
    assert_eq!(action, decoded);

    let action = GameAction::ZoneRect { 
        min: (5, 5), 
        max: (10, 10), 
        zone_type: ZoneType::ResidentialHigh 
    };
    let json = serde_json::to_string(&action).unwrap();
    let decoded: GameAction = serde_json::from_str(&json).unwrap();
    assert_eq!(action, decoded);
}

#[test]
fn test_action_result_serialization() {
    let res = ActionResult::Success;
    let json = serde_json::to_string(&res).unwrap();
    let decoded: ActionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(res, decoded);

    let res = ActionResult::Error(ActionError::InsufficientFunds);
    let json = serde_json::to_string(&res).unwrap();
    let decoded: ActionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(res, decoded);
}
