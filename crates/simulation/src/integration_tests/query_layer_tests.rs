//! Integration tests for the `query` command and `query_result` response
//! protocol types.

use crate::agent_protocol::{make_response, AgentCommand, ResponsePayload};

// ---------------------------------------------------------------------------
// Command deserialization
// ---------------------------------------------------------------------------

#[test]
fn test_query_command_deserializes() {
    let json = r#"{"cmd":"query","layers":["map","buildings"]}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::Query { layers } = cmd {
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], "map");
        assert_eq!(layers[1], "buildings");
    } else {
        panic!("expected Query command");
    }
}

#[test]
fn test_query_command_empty_layers() {
    let json = r#"{"cmd":"query","layers":[]}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::Query { layers } = cmd {
        assert!(layers.is_empty());
    } else {
        panic!("expected Query command");
    }
}

#[test]
fn test_query_command_single_layer() {
    let json = r#"{"cmd":"query","layers":["overview"]}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::Query { layers } = cmd {
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0], "overview");
    } else {
        panic!("expected Query command");
    }
}

#[test]
fn test_query_command_all_known_layers() {
    let json = r#"{"cmd":"query","layers":["map","overview","buildings","services","utilities","roads","zones","terrain"]}"#;
    let cmd: AgentCommand = serde_json::from_str(json).unwrap();
    if let AgentCommand::Query { layers } = cmd {
        assert_eq!(layers.len(), 8);
    } else {
        panic!("expected Query command");
    }
}

// ---------------------------------------------------------------------------
// Response serialization
// ---------------------------------------------------------------------------

#[test]
fn test_query_result_serializes() {
    let mut map = serde_json::Map::new();
    map.insert(
        "overview".to_string(),
        serde_json::Value::String("...overview data...".to_string()),
    );
    map.insert(
        "buildings".to_string(),
        serde_json::Value::String("Buildings (3 total):".to_string()),
    );

    let resp = make_response(ResponsePayload::QueryResult {
        layers: serde_json::Value::Object(map),
    });
    let json = serde_json::to_string(&resp).unwrap();

    assert!(json.contains("\"type\":\"query_result\""));
    assert!(json.contains("\"protocol_version\":1"));
    assert!(json.contains("\"overview\""));
    assert!(json.contains("\"buildings\""));
    assert!(json.contains("...overview data..."));
}

#[test]
fn test_query_result_empty_layers() {
    let map = serde_json::Map::new();
    let resp = make_response(ResponsePayload::QueryResult {
        layers: serde_json::Value::Object(map),
    });
    let json = serde_json::to_string(&resp).unwrap();

    assert!(json.contains("\"type\":\"query_result\""));
    assert!(json.contains("\"layers\":{}"));
}

#[test]
fn test_query_result_with_unknown_layer() {
    let mut map = serde_json::Map::new();
    map.insert(
        "nonexistent".to_string(),
        serde_json::Value::String("Unknown layer: nonexistent".to_string()),
    );

    let resp = make_response(ResponsePayload::QueryResult {
        layers: serde_json::Value::Object(map),
    });
    let json = serde_json::to_string(&resp).unwrap();

    assert!(json.contains("\"nonexistent\""));
    assert!(json.contains("Unknown layer: nonexistent"));
}

#[test]
fn test_query_result_roundtrip_valid_json() {
    let mut map = serde_json::Map::new();
    map.insert(
        "roads".to_string(),
        serde_json::Value::String("Roads (42 total cells):".to_string()),
    );

    let resp = make_response(ResponsePayload::QueryResult {
        layers: serde_json::Value::Object(map),
    });
    let json = serde_json::to_string(&resp).unwrap();

    // Verify it's valid JSON by parsing it back
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["type"], "query_result");
    assert!(parsed["layers"]["roads"].is_string());
}
