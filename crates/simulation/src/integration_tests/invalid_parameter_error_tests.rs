//! Tests for InvalidParameter(String) error serialization and unknown
//! service_type / utility_type parse failures.

use crate::agent_protocol::*;
use crate::game_actions::{ActionError, ActionResult};

#[test]
fn test_invalid_parameter_error_serializes_with_message() {
    let err = ActionError::InvalidParameter("bad value".to_string());
    let result = ActionResult::Error(err);
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("InvalidParameter"));
    assert!(json.contains("bad value"));
}

#[test]
fn test_invalid_parameter_response_includes_message() {
    let result =
        ActionResult::Error(ActionError::InvalidParameter("unknown type".to_string()));
    let resp = make_response(ResponsePayload::ActionResult { result });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"action_result\""));
    assert!(json.contains("InvalidParameter"));
    assert!(json.contains("unknown type"));
}

#[test]
fn test_unknown_service_type_serde_error_lists_valid_types() {
    let json =
        r#"{"cmd":"act","action":{"PlaceService":{"pos":[10,10],"service_type":"Park"}}}"#;
    let err = serde_json::from_str::<AgentCommand>(json).unwrap_err();
    let msg = format!("{err}");
    // The serde error should mention the unknown variant
    assert!(msg.contains("Park"), "Error should mention the unknown variant 'Park'");
    // The serde error should list some valid variants
    assert!(
        msg.contains("SmallPark"),
        "Error should list valid variant 'SmallPark'"
    );
    assert!(
        msg.contains("FireStation"),
        "Error should list valid variant 'FireStation'"
    );
}

#[test]
fn test_unknown_utility_type_serde_error_lists_valid_types() {
    let json = r#"{"cmd":"act","action":{"PlaceUtility":{"pos":[5,5],"utility_type":"SolarPanel"}}}"#;
    let err = serde_json::from_str::<AgentCommand>(json).unwrap_err();
    let msg = format!("{err}");
    // The serde error should mention the unknown variant
    assert!(
        msg.contains("SolarPanel"),
        "Error should mention the unknown variant 'SolarPanel'"
    );
    // The serde error should list some valid variants
    assert!(
        msg.contains("SolarFarm"),
        "Error should list valid variant 'SolarFarm'"
    );
    assert!(
        msg.contains("PowerPlant"),
        "Error should list valid variant 'PowerPlant'"
    );
}

#[test]
fn test_invalid_parameter_roundtrip() {
    let err = ActionError::InvalidParameter("test message".to_string());
    let result = ActionResult::Error(err.clone());
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ActionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, result);
}

#[test]
fn test_invalid_parameter_batch_result_serialization() {
    let result =
        ActionResult::Error(ActionError::InvalidParameter("bad param".to_string()));
    let resp = make_response(ResponsePayload::BatchResult {
        results: vec![result],
    });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"batch_result\""));
    assert!(json.contains("InvalidParameter"));
    assert!(json.contains("bad param"));
}
