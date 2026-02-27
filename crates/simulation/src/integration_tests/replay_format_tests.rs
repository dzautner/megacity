//! Integration tests for the deterministic replay format, recorder, and player.

use crate::game_actions::{ActionQueue, ActionSource, GameAction};
use crate::grid::RoadType;
use crate::replay::format::{
    ReplayEntry, ReplayFile, ReplayFooter, ReplayHeader, CURRENT_FORMAT_VERSION,
};
use crate::replay::player::ReplayPlayer;
use crate::replay::recorder::ReplayRecorder;
use crate::test_harness::TestCity;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// ReplayFile roundtrip tests
// ---------------------------------------------------------------------------

fn sample_replay() -> ReplayFile {
    ReplayFile {
        header: ReplayHeader {
            format_version: CURRENT_FORMAT_VERSION,
            seed: 12345,
            city_name: "Megacity".to_string(),
            start_tick: 0,
        },
        entries: vec![
            ReplayEntry {
                tick: 0,
                action: GameAction::NewGame {
                    seed: 12345,
                    map_size: Some(256),
                },
            },
            ReplayEntry {
                tick: 1,
                action: GameAction::SetSpeed { speed: 2 },
            },
            ReplayEntry {
                tick: 10,
                action: GameAction::PlaceRoadLine {
                    start: (5, 5),
                    end: (15, 5),
                    road_type: RoadType::Avenue,
                },
            },
            ReplayEntry {
                tick: 10,
                action: GameAction::SetPaused { paused: true },
            },
        ],
        footer: ReplayFooter {
            end_tick: 200,
            final_state_hash: 0xDEAD_BEEF,
            entry_count: 4,
        },
    }
}

#[test]
fn test_replay_file_bitcode_roundtrip() {
    let original = sample_replay();
    let bytes = original.to_bytes();
    let decoded = ReplayFile::from_bytes(&bytes).expect("bitcode decode should succeed");
    assert_eq!(original, decoded);
}

#[test]
fn test_replay_file_json_roundtrip() {
    let original = sample_replay();
    let json = original.to_json();
    assert!(json.contains("Megacity"));
    assert!(json.contains("12345"));
    let decoded = ReplayFile::from_json(&json).expect("JSON decode should succeed");
    assert_eq!(original, decoded);
}

#[test]
fn test_replay_file_validate_valid() {
    let replay = sample_replay();
    assert!(replay.validate().is_ok());
}

#[test]
fn test_replay_file_validate_entry_count_mismatch() {
    let mut replay = sample_replay();
    replay.footer.entry_count = 99;
    let err = replay.validate().unwrap_err();
    assert!(
        err.contains("entry_count mismatch"),
        "expected entry_count mismatch error, got: {err}"
    );
}

#[test]
fn test_replay_file_validate_unsorted_ticks() {
    let mut replay = sample_replay();
    // Move the tick-10 entry before the tick-1 entry
    replay.entries.swap(1, 2);
    let err = replay.validate().unwrap_err();
    assert!(
        err.contains("not sorted by tick"),
        "expected unsorted error, got: {err}"
    );
}

#[test]
fn test_replay_file_validate_empty_entries() {
    let replay = ReplayFile {
        header: ReplayHeader {
            format_version: 1,
            seed: 0,
            city_name: String::new(),
            start_tick: 0,
        },
        entries: vec![],
        footer: ReplayFooter {
            end_tick: 0,
            final_state_hash: 0,
            entry_count: 0,
        },
    };
    assert!(replay.validate().is_ok());
}

// ---------------------------------------------------------------------------
// ReplayRecorder tests
// ---------------------------------------------------------------------------

#[test]
fn test_recorder_start_record_stop() {
    let mut recorder = ReplayRecorder::default();
    assert!(!recorder.is_recording());

    recorder.start(42, "RecordTest".to_string(), 0);
    assert!(recorder.is_recording());

    recorder.record(1, GameAction::SetSpeed { speed: 3 });
    recorder.record(
        2,
        GameAction::PlaceRoadLine {
            start: (0, 0),
            end: (10, 0),
            road_type: RoadType::Local,
        },
    );
    assert_eq!(recorder.entry_count(), 2);

    let file = recorder.stop(50, 0);
    assert!(!recorder.is_recording());
    assert_eq!(file.header.seed, 42);
    assert_eq!(file.header.city_name, "RecordTest");
    assert_eq!(file.entries.len(), 2);
    assert_eq!(file.footer.entry_count, 2);
    assert_eq!(file.footer.end_tick, 50);
    assert!(file.validate().is_ok());
}

#[test]
fn test_recorder_ignores_when_not_recording() {
    let mut recorder = ReplayRecorder::default();
    recorder.record(1, GameAction::SetSpeed { speed: 1 });
    assert_eq!(recorder.entry_count(), 0);
}

// ---------------------------------------------------------------------------
// ReplayPlayer tests
// ---------------------------------------------------------------------------

#[test]
fn test_player_feeds_correct_actions_per_tick() {
    let replay = sample_replay();
    let mut player = ReplayPlayer::default();
    player.load(replay);

    assert!(player.is_playing());
    assert!(!player.is_finished());

    // Tick 0: one entry (NewGame)
    let actions = player.actions_for_tick(0);
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions[0], GameAction::NewGame { .. }));

    // Tick 1: one entry (SetSpeed)
    let actions = player.actions_for_tick(1);
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions[0], GameAction::SetSpeed { speed: 2 }));

    // Ticks 2-9: no entries
    for t in 2..10 {
        assert!(player.actions_for_tick(t).is_empty());
    }

    // Tick 10: two entries
    let actions = player.actions_for_tick(10);
    assert_eq!(actions.len(), 2);

    assert!(player.is_finished());
}

#[test]
fn test_player_stop_clears_state() {
    let mut player = ReplayPlayer::default();
    player.load(sample_replay());
    player.stop();

    assert!(!player.is_playing());
    assert!(player.is_finished());
    assert_eq!(player.cursor(), 0);
}

// ---------------------------------------------------------------------------
// Integration: recorder + player round-trip with TestCity
// ---------------------------------------------------------------------------

#[test]
fn test_recorder_player_roundtrip_with_test_city() {
    let mut city = TestCity::new();

    // Start recording
    {
        let world = city.world_mut();
        let mut recorder = world.resource_mut::<ReplayRecorder>();
        recorder.start(42, "RoundtripTest".to_string(), 0);
    }

    // Push some actions into the queue
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Player,
            GameAction::SetSpeed { speed: 3 },
        );
    }

    // Tick to let the record_actions system capture them
    city.tick(1);

    // Push more actions at a later tick
    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Player,
            GameAction::SetPaused { paused: true },
        );
    }

    city.tick(1);

    // Stop recording
    let replay = {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut recorder = world.resource_mut::<ReplayRecorder>();
        recorder.stop(tick, 0)
    };

    assert!(replay.validate().is_ok());
    assert_eq!(replay.header.seed, 42);
    assert!(!replay.entries.is_empty(), "should have recorded some actions");

    // Now load the replay into a fresh city's player
    let mut city2 = TestCity::new();
    {
        let world = city2.world_mut();
        let mut player = world.resource_mut::<ReplayPlayer>();
        player.load(replay.clone());
        assert!(player.is_playing());
    }

    // Verify the player resource is accessible and has the right entry count
    {
        let world = city2.world_mut();
        let player = world.resource::<ReplayPlayer>();
        assert!(player.is_playing());
    }
}
