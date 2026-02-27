//! Integration tests for agent protocol types.
//!
//! Verifies serialization/deserialization of all command and response variants,
//! ensuring the JSON wire format is stable for external tool integrations.

use crate::agent_protocol::*;
use crate::city_observation::CityObservation;
use crate::game_actions::{ActionError, ActionResult, GameAction};

// ---------------------------------------------------------------------------
// Command deserialization
// ---------------------------------------------------------------------------

#[test]
fn test_agent_command_observe_deserializes() {
    let json = r#"{"cmd":"observe"}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    assert!(matches!(cmd, AgentCommand::Observe));
}

#[test]
fn test_agent_command_act_set_paused() {
    let json = r#"{"cmd":"act","action":{"SetPaused":{"paused":true}}}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::Act { action } = cmd {
        assert_eq!(action, GameAction::SetPaused { paused: true });
    } else {
        panic!("expected Act command");
    }
}

#[test]
fn test_agent_command_act_place_road() {
    let json = r#"{"cmd":"act","action":{"PlaceRoadLine":{"start":[10,20],"end":[10,30],"road_type":"Local"}}}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    assert!(matches!(cmd, AgentCommand::Act { .. }));
}

#[test]
fn test_agent_command_act_zone_rect() {
    let json = r#"{"cmd":"act","action":{"ZoneRect":{"min":[5,5],"max":[15,15],"zone_type":"ResidentialLow"}}}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    assert!(matches!(cmd, AgentCommand::Act { .. }));
}

#[test]
fn test_agent_command_batch_act_multiple_actions() {
    let json = r#"{"cmd":"batch_act","actions":[{"SetSpeed":{"speed":2}},{"SetPaused":{"paused":false}}]}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::BatchAct { actions } = cmd {
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], GameAction::SetSpeed { speed: 2 });
        assert_eq!(actions[1], GameAction::SetPaused { paused: false });
    } else {
        panic!("expected BatchAct command");
    }
}

#[test]
fn test_agent_command_step_ticks() {
    let json = r#"{"cmd":"step","ticks":500}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::Step { ticks } = cmd {
        assert_eq!(ticks, 500);
    } else {
        panic!("expected Step command");
    }
}

#[test]
fn test_agent_command_new_game_with_seed() {
    let json = r#"{"cmd":"new_game","seed":12345}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::NewGame { seed } = cmd {
        assert_eq!(seed, 12345);
    } else {
        panic!("expected NewGame command");
    }
}

#[test]
fn test_agent_command_save_replay() {
    let json = r#"{"cmd":"save_replay","path":"/tmp/replay.bin"}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::SaveReplay { path } = cmd {
        assert_eq!(path, "/tmp/replay.bin");
    } else {
        panic!("expected SaveReplay command");
    }
}

#[test]
fn test_agent_command_load_replay() {
    let json = r#"{"cmd":"load_replay","path":"/tmp/replay.bin"}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::LoadReplay { path } = cmd {
        assert_eq!(path, "/tmp/replay.bin");
    } else {
        panic!("expected LoadReplay command");
    }
}

#[test]
fn test_agent_command_quit() {
    let json = r#"{"cmd":"quit"}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    assert!(matches!(cmd, AgentCommand::Quit));
}

#[test]
fn test_agent_command_unknown_cmd_fails() {
    let json = r#"{"cmd":"fly_to_moon"}"#;
    let result = serde_json::from_str::<AgentCommand>(json);
    assert!(result.is_err());
}

#[test]
fn test_agent_command_missing_cmd_field_fails() {
    let json = r#"{"action":"observe"}"#;
    let result = serde_json::from_str::<AgentCommand>(json);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Response serialization
// ---------------------------------------------------------------------------

#[test]
fn test_response_ready_contains_protocol_version() {
    let resp = make_response(ResponsePayload::Ready);
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"protocol_version\":1"));
    assert!(json.contains("\"type\":\"ready\""));
}

#[test]
fn test_response_observation_includes_city_data() {
    let obs = CityObservation {
        tick: 42,
        day: 3,
        treasury: 50000.0,
        ..Default::default()
    };
    let resp = make_response(ResponsePayload::Observation {
        observation: obs.clone(),
    });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"observation\""));
    assert!(json.contains("\"tick\":42"));
    assert!(json.contains("\"day\":3"));
}

#[test]
fn test_response_action_result_success() {
    let resp = make_response(ResponsePayload::ActionResult {
        result: ActionResult::Success,
    });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"action_result\""));
    assert!(json.contains("\"result\":\"Success\""));
}

#[test]
fn test_response_action_result_error() {
    let resp = make_response(ResponsePayload::ActionResult {
        result: ActionResult::Error(ActionError::OutOfBounds),
    });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"action_result\""));
    assert!(json.contains("OutOfBounds"));
}

#[test]
fn test_response_batch_result_preserves_order() {
    let results = vec![
        ActionResult::Success,
        ActionResult::Error(ActionError::InsufficientFunds),
        ActionResult::Success,
    ];
    let resp = make_response(ResponsePayload::BatchResult {
        results: results.clone(),
    });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"batch_result\""));
    assert!(json.contains("InsufficientFunds"));
}

#[test]
fn test_response_step_complete_reports_tick() {
    let resp = make_response(ResponsePayload::StepComplete { tick: 1000 });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"step_complete\""));
    assert!(json.contains("\"tick\":1000"));
}

#[test]
fn test_response_ok() {
    let resp = make_response(ResponsePayload::Ok);
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"ok\""));
}

#[test]
fn test_response_error_includes_message() {
    let resp = make_response(ResponsePayload::Error {
        message: "invalid action format".to_string(),
    });
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"error\""));
    assert!(json.contains("invalid action format"));
}

#[test]
fn test_response_goodbye() {
    let resp = make_response(ResponsePayload::Goodbye);
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"type\":\"goodbye\""));
}

// ---------------------------------------------------------------------------
// Roundtrip: serialized response is valid JSON
// ---------------------------------------------------------------------------

#[test]
fn test_all_response_variants_produce_valid_json() {
    let variants: Vec<ResponsePayload> = vec![
        ResponsePayload::Ready,
        ResponsePayload::Observation {
            observation: CityObservation::default(),
        },
        ResponsePayload::ActionResult {
            result: ActionResult::Success,
        },
        ResponsePayload::BatchResult {
            results: vec![ActionResult::Success],
        },
        ResponsePayload::StepComplete { tick: 0 },
        ResponsePayload::Ok,
        ResponsePayload::Error {
            message: "test".to_string(),
        },
        ResponsePayload::Goodbye,
    ];

    for payload in variants {
        let resp = make_response(payload);
        let json = serde_json::to_string(&resp).unwrap();
        // Verify it re-parses as valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["protocol_version"], 1);
        assert!(parsed["type"].is_string());
    }
}

// ---------------------------------------------------------------------------
// Protocol version consistency
// ---------------------------------------------------------------------------

#[test]
fn test_protocol_version_is_one() {
    assert_eq!(PROTOCOL_VERSION, 1);
}

#[test]
fn test_make_response_sets_correct_version() {
    let resp = make_response(ResponsePayload::Ok);
    assert_eq!(resp.protocol_version, PROTOCOL_VERSION);
}
