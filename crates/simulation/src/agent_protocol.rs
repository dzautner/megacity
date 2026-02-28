//! Agent text protocol types for the `--agent` headless mode.
//!
//! Defines the JSON command/response envelope that external programs (LLMs,
//! scripts, test harnesses) use to interact with the simulation over
//! newline-delimited JSON on stdin/stdout.
//!
//! These types live in the `simulation` crate so they can be unit-tested
//! without pulling in the full app binary. The actual I/O loop lives in
//! `crates/app/src/agent_mode.rs`.

use serde::{Deserialize, Serialize};

use crate::city_observation::CityObservation;
use crate::game_actions::{ActionResult, GameAction};

// ---------------------------------------------------------------------------
// Commands (stdin → simulation)
// ---------------------------------------------------------------------------

/// A single command sent by the external agent over stdin.
///
/// Each line of stdin is parsed as one `AgentCommand`. The `cmd` field acts as
/// the discriminator tag.
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd")]
pub enum AgentCommand {
    /// Request the current city observation snapshot.
    #[serde(rename = "observe")]
    Observe,

    /// Execute a single game action.
    #[serde(rename = "act")]
    Act { action: GameAction },

    /// Execute multiple game actions in sequence.
    #[serde(rename = "batch_act")]
    BatchAct { actions: Vec<GameAction> },

    /// Advance the simulation by `ticks` fixed-update ticks.
    #[serde(rename = "step")]
    Step { ticks: u64 },

    /// Reset to a new game with the given seed.
    #[serde(rename = "new_game")]
    NewGame { seed: u64 },

    /// Save a replay file (stub — returns Ok).
    #[serde(rename = "save_replay")]
    SaveReplay { path: String },

    /// Load a replay file (stub — returns Ok).
    #[serde(rename = "load_replay")]
    LoadReplay { path: String },

    /// Request one or more spatial data layers.
    #[serde(rename = "query")]
    Query { layers: Vec<String> },

    /// Gracefully shut down the agent session.
    #[serde(rename = "quit")]
    Quit,
}

// ---------------------------------------------------------------------------
// Responses (simulation → stdout)
// ---------------------------------------------------------------------------

/// Every response includes the protocol version and a tagged payload.
#[derive(Debug, Serialize)]
pub struct AgentResponse {
    /// Monotonically increasing protocol version (currently 1).
    pub protocol_version: u32,
    /// The response payload, flattened into this object.
    #[serde(flatten)]
    pub payload: ResponsePayload,
}

/// Tagged payload variants for agent responses.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ResponsePayload {
    /// The simulation is ready to accept commands.
    #[serde(rename = "ready")]
    Ready,

    /// A city observation snapshot.
    #[serde(rename = "observation")]
    Observation { observation: CityObservation },

    /// Result of a single `act` command.
    #[serde(rename = "action_result")]
    ActionResult { result: ActionResult },

    /// Results of a `batch_act` command.
    #[serde(rename = "batch_result")]
    BatchResult { results: Vec<ActionResult> },

    /// The simulation has advanced; reports the current tick counter.
    #[serde(rename = "step_complete")]
    StepComplete { tick: u64 },

    /// Results of a `query` command — a JSON object keyed by layer name.
    #[serde(rename = "query_result")]
    QueryResult { layers: serde_json::Value },

    /// Generic success acknowledgement (used for stubs, new_game, etc.).
    #[serde(rename = "ok")]
    Ok,

    /// An error occurred while processing the command.
    #[serde(rename = "error")]
    Error { message: String },

    /// The session is ending (response to `quit`).
    #[serde(rename = "goodbye")]
    Goodbye,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Current protocol version. Bump when the command/response schema changes.
pub const PROTOCOL_VERSION: u32 = 1;

/// Convenience constructor that wraps a payload with the current protocol version.
pub fn make_response(payload: ResponsePayload) -> AgentResponse {
    AgentResponse {
        protocol_version: PROTOCOL_VERSION,
        payload,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_observe_command() {
        let json = r#"{"cmd":"observe"}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, AgentCommand::Observe));
    }

    #[test]
    fn deserialize_act_command() {
        let json = r#"{"cmd":"act","action":{"SetPaused":{"paused":true}}}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, AgentCommand::Act { .. }));
    }

    #[test]
    fn deserialize_batch_act_command() {
        let json = r#"{"cmd":"batch_act","actions":[{"SetSpeed":{"speed":2}},{"SetPaused":{"paused":false}}]}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        if let AgentCommand::BatchAct { actions } = cmd {
            assert_eq!(actions.len(), 2);
        } else {
            panic!("expected BatchAct");
        }
    }

    #[test]
    fn deserialize_step_command() {
        let json = r#"{"cmd":"step","ticks":100}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        if let AgentCommand::Step { ticks } = cmd {
            assert_eq!(ticks, 100);
        } else {
            panic!("expected Step");
        }
    }

    #[test]
    fn deserialize_new_game_command() {
        let json = r#"{"cmd":"new_game","seed":42}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        if let AgentCommand::NewGame { seed } = cmd {
            assert_eq!(seed, 42);
        } else {
            panic!("expected NewGame");
        }
    }

    #[test]
    fn deserialize_save_replay_command() {
        let json = r#"{"cmd":"save_replay","path":"/tmp/replay.bin"}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, AgentCommand::SaveReplay { .. }));
    }

    #[test]
    fn deserialize_load_replay_command() {
        let json = r#"{"cmd":"load_replay","path":"/tmp/replay.bin"}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, AgentCommand::LoadReplay { .. }));
    }

    #[test]
    fn deserialize_query_command() {
        let json = r#"{"cmd":"query","layers":["map","buildings"]}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        if let AgentCommand::Query { layers } = cmd {
            assert_eq!(layers, vec!["map", "buildings"]);
        } else {
            panic!("expected Query");
        }
    }

    #[test]
    fn deserialize_query_command_empty_layers() {
        let json = r#"{"cmd":"query","layers":[]}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        if let AgentCommand::Query { layers } = cmd {
            assert!(layers.is_empty());
        } else {
            panic!("expected Query");
        }
    }

    #[test]
    fn deserialize_quit_command() {
        let json = r#"{"cmd":"quit"}"#;
        let cmd: AgentCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, AgentCommand::Quit));
    }

    #[test]
    fn serialize_ready_response() {
        let resp = make_response(ResponsePayload::Ready);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"protocol_version\":1"));
        assert!(json.contains("\"type\":\"ready\""));
    }

    #[test]
    fn serialize_observation_response() {
        let obs = CityObservation::default();
        let resp = make_response(ResponsePayload::Observation { observation: obs });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"observation\""));
        assert!(json.contains("\"tick\":0"));
    }

    #[test]
    fn serialize_action_result_response() {
        let resp = make_response(ResponsePayload::ActionResult {
            result: ActionResult::Success,
        });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"action_result\""));
        assert!(json.contains("\"result\":\"Success\""));
    }

    #[test]
    fn serialize_batch_result_response() {
        let resp = make_response(ResponsePayload::BatchResult {
            results: vec![ActionResult::Success, ActionResult::Success],
        });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"batch_result\""));
    }

    #[test]
    fn serialize_step_complete_response() {
        let resp = make_response(ResponsePayload::StepComplete { tick: 42 });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"step_complete\""));
        assert!(json.contains("\"tick\":42"));
    }

    #[test]
    fn serialize_query_result_response() {
        let mut map = serde_json::Map::new();
        map.insert(
            "overview".to_string(),
            serde_json::Value::String("test map data".to_string()),
        );
        let resp = make_response(ResponsePayload::QueryResult {
            layers: serde_json::Value::Object(map),
        });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"query_result\""));
        assert!(json.contains("\"overview\""));
        assert!(json.contains("test map data"));
    }

    #[test]
    fn serialize_error_response() {
        let resp = make_response(ResponsePayload::Error {
            message: "something went wrong".to_string(),
        });
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("something went wrong"));
    }

    #[test]
    fn serialize_goodbye_response() {
        let resp = make_response(ResponsePayload::Goodbye);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"goodbye\""));
    }

    #[test]
    fn serialize_ok_response() {
        let resp = make_response(ResponsePayload::Ok);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"type\":\"ok\""));
    }

    #[test]
    fn invalid_command_returns_parse_error() {
        let json = r#"{"cmd":"nonexistent"}"#;
        let result = serde_json::from_str::<AgentCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn malformed_json_returns_parse_error() {
        let json = r#"{not valid json"#;
        let result = serde_json::from_str::<AgentCommand>(json);
        assert!(result.is_err());
    }
}
