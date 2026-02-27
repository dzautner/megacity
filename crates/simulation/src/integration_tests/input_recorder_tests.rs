//! Integration tests for the input action recorder (STAB-03).

use crate::input_recorder::{InputRecorder, RecorderMode, RecordedAction};
use crate::input_recorder_types::{
    RecordedRoadType, RecordedServiceType, RecordedUtilityType, RecordedZoneType,
};
use crate::test_harness::TestCity;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Enable recording mode on the InputRecorder resource.
fn enable_recording(city: &mut TestCity) {
    let world = city.world_mut();
    world.resource_mut::<InputRecorder>().start_recording();
}

/// Read the current tick from TickCounter.
fn current_tick(city: &mut TestCity) -> u64 {
    city.resource::<TickCounter>().0
}

// ---------------------------------------------------------------------------
// Tests: recording
// ---------------------------------------------------------------------------

#[test]
fn test_recorder_starts_off_by_default() {
    let city = TestCity::new();
    let recorder = city.resource::<InputRecorder>();
    assert_eq!(recorder.mode, RecorderMode::Off);
    assert!(recorder.actions.is_empty());
    assert_eq!(recorder.replay_cursor, 0);
}

#[test]
fn test_start_recording_clears_log() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut recorder = world.resource_mut::<InputRecorder>();
        // Manually push a dummy action
        recorder.actions.push((
            0,
            RecordedAction::PlaceGridRoad {
                x: 5,
                y: 5,
                road_type: RecordedRoadType::Local,
                cost: 10.0,
            },
        ));
        assert_eq!(recorder.action_count(), 1);
    }
    enable_recording(&mut city);
    let recorder = city.resource::<InputRecorder>();
    assert_eq!(recorder.mode, RecorderMode::Recording);
    assert!(recorder.actions.is_empty());
}

#[test]
fn test_stop_resets_mode() {
    let mut city = TestCity::new();
    enable_recording(&mut city);
    {
        let world = city.world_mut();
        world.resource_mut::<InputRecorder>().stop();
    }
    let recorder = city.resource::<InputRecorder>();
    assert_eq!(recorder.mode, RecorderMode::Off);
}

#[test]
fn test_record_only_when_recording() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut recorder = world.resource_mut::<InputRecorder>();
        // Off mode: record should be a no-op
        recorder.record(
            1,
            RecordedAction::PlaceGridRoad {
                x: 0,
                y: 0,
                road_type: RecordedRoadType::Avenue,
                cost: 20.0,
            },
        );
        assert!(recorder.actions.is_empty());
    }
}

#[test]
fn test_record_captures_with_tick() {
    let mut city = TestCity::new();
    enable_recording(&mut city);
    {
        let world = city.world_mut();
        let mut recorder = world.resource_mut::<InputRecorder>();
        recorder.record(
            42,
            RecordedAction::PlaceZone {
                cells: vec![(10, 10, RecordedZoneType::ResidentialLow)],
                cost: 5.0,
            },
        );
        recorder.record(
            43,
            RecordedAction::BulldozeRoad {
                x: 3,
                y: 4,
                road_type: RecordedRoadType::Highway,
                refund: 20.0,
            },
        );
    }
    let recorder = city.resource::<InputRecorder>();
    assert_eq!(recorder.action_count(), 2);
    assert_eq!(recorder.actions[0].0, 42);
    assert_eq!(recorder.actions[1].0, 43);
}

// ---------------------------------------------------------------------------
// Tests: serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_serialize_deserialize_roundtrip() {
    let mut recorder = InputRecorder::default();
    recorder.mode = RecorderMode::Recording;
    recorder.record(
        1,
        RecordedAction::PlaceGridRoad {
            x: 10,
            y: 20,
            road_type: RecordedRoadType::Boulevard,
            cost: 30.0,
        },
    );
    recorder.record(
        5,
        RecordedAction::PlaceService {
            service_type: RecordedServiceType::Hospital,
            grid_x: 50,
            grid_y: 60,
            cost: 500.0,
        },
    );
    recorder.record(
        10,
        RecordedAction::PlaceUtility {
            utility_type: RecordedUtilityType::SolarFarm,
            grid_x: 30,
            grid_y: 40,
            cost: 200.0,
        },
    );
    recorder.record(
        15,
        RecordedAction::BulldozeZone {
            x: 7,
            y: 8,
            zone: RecordedZoneType::Industrial,
        },
    );
    recorder.record(
        20,
        RecordedAction::Composite(vec![
            RecordedAction::PlaceGridRoad {
                x: 1,
                y: 1,
                road_type: RecordedRoadType::Local,
                cost: 10.0,
            },
            RecordedAction::PlaceGridRoad {
                x: 2,
                y: 1,
                road_type: RecordedRoadType::Local,
                cost: 10.0,
            },
        ]),
    );

    // Serialize
    let bytes = bitcode::encode(&recorder);

    // Deserialize
    let restored: InputRecorder = bitcode::decode(&bytes).expect("decode should succeed");

    assert_eq!(restored.actions.len(), 5);
    assert_eq!(restored.actions[0].0, 1);
    assert_eq!(restored.actions[1].0, 5);
    assert_eq!(restored.actions[2].0, 10);
    assert_eq!(restored.actions[3].0, 15);
    assert_eq!(restored.actions[4].0, 20);

    // Verify action data survived roundtrip
    assert_eq!(restored.actions[0].1, recorder.actions[0].1);
    assert_eq!(restored.actions[1].1, recorder.actions[1].1);
    assert_eq!(restored.actions[2].1, recorder.actions[2].1);
    assert_eq!(restored.actions[3].1, recorder.actions[3].1);
    assert_eq!(restored.actions[4].1, recorder.actions[4].1);
}

#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;

    let mut recorder = InputRecorder::default();
    recorder.mode = RecorderMode::Recording;
    recorder.record(
        7,
        RecordedAction::BulldozeService {
            service_type: RecordedServiceType::FireStation,
            grid_x: 15,
            grid_y: 25,
            refund: 100.0,
        },
    );

    let bytes = recorder.save_to_bytes().expect("non-empty should save");
    let restored = InputRecorder::load_from_bytes(&bytes);

    assert_eq!(restored.actions.len(), 1);
    assert_eq!(restored.actions[0].0, 7);
    assert_eq!(restored.actions[0].1, recorder.actions[0].1);
}

#[test]
fn test_saveable_skips_empty() {
    use crate::Saveable;

    let recorder = InputRecorder::default();
    assert!(recorder.save_to_bytes().is_none());
}

// ---------------------------------------------------------------------------
// Tests: replay state
// ---------------------------------------------------------------------------

#[test]
fn test_replay_cursor_and_finished() {
    let mut recorder = InputRecorder::default();
    recorder.mode = RecorderMode::Recording;
    recorder.record(
        1,
        RecordedAction::PlaceGridRoad {
            x: 0,
            y: 0,
            road_type: RecordedRoadType::Path,
            cost: 5.0,
        },
    );
    recorder.record(
        2,
        RecordedAction::PlaceGridRoad {
            x: 1,
            y: 0,
            road_type: RecordedRoadType::Path,
            cost: 5.0,
        },
    );

    recorder.start_replay();
    assert_eq!(recorder.mode, RecorderMode::Replaying);
    assert_eq!(recorder.replay_cursor, 0);
    assert!(!recorder.replay_finished());

    recorder.replay_cursor = 1;
    assert!(!recorder.replay_finished());

    recorder.replay_cursor = 2;
    assert!(recorder.replay_finished());
}

// ---------------------------------------------------------------------------
// Tests: CityAction conversion
// ---------------------------------------------------------------------------

#[test]
fn test_city_action_conversion_place_zone() {
    use crate::grid::ZoneType;
    use crate::undo_redo::CityAction;

    let action = CityAction::PlaceZone {
        cells: vec![
            (10, 20, ZoneType::CommercialHigh),
            (11, 20, ZoneType::Office),
        ],
        cost: 100.0,
    };
    let recorded = RecordedAction::from_city_action(&action);

    match recorded {
        RecordedAction::PlaceZone { cells, cost } => {
            assert_eq!(cells.len(), 2);
            assert_eq!(cells[0], (10, 20, RecordedZoneType::CommercialHigh));
            assert_eq!(cells[1], (11, 20, RecordedZoneType::Office));
            assert!((cost - 100.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected PlaceZone variant"),
    }
}

#[test]
fn test_city_action_conversion_composite() {
    use crate::grid::RoadType;
    use crate::undo_redo::CityAction;

    let action = CityAction::Composite(vec![
        CityAction::PlaceGridRoad {
            x: 5,
            y: 6,
            road_type: RoadType::Avenue,
            cost: 20.0,
        },
        CityAction::PlaceGridRoad {
            x: 6,
            y: 6,
            road_type: RoadType::Avenue,
            cost: 20.0,
        },
    ]);
    let recorded = RecordedAction::from_city_action(&action);

    match recorded {
        RecordedAction::Composite(inner) => {
            assert_eq!(inner.len(), 2);
        }
        _ => panic!("Expected Composite variant"),
    }
}
