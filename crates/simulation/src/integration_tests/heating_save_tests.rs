//! Integration tests for SAVE-040: Serialize HeatingGrid.
//!
//! Verifies that heating network coverage persists across save/load cycles
//! so the heating overlay matches its pre-save state.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::heating::HeatingGrid;
use crate::Saveable;

// ---------------------------------------------------------------------------
// Round-trip: non-trivial heating grid survives save/load
// ---------------------------------------------------------------------------

#[test]
fn test_heating_grid_save_load_round_trip() {
    let mut grid = HeatingGrid::default();
    assert_eq!(grid.levels.len(), GRID_WIDTH * GRID_HEIGHT);

    // Set some non-trivial heating values
    grid.set(10, 10, 255);
    grid.set(11, 10, 200);
    grid.set(12, 10, 150);
    grid.set(13, 10, 100);
    grid.set(14, 10, 50);
    grid.set(10, 11, 180);
    grid.set(0, 0, 42);
    grid.set(GRID_WIDTH - 1, GRID_HEIGHT - 1, 99);

    // Serialize
    let bytes = grid
        .save_to_bytes()
        .expect("non-zero grid should produce Some bytes");

    // Deserialize
    let restored = HeatingGrid::load_from_bytes(&bytes);

    assert_eq!(restored.width, GRID_WIDTH);
    assert_eq!(restored.height, GRID_HEIGHT);
    assert_eq!(restored.levels.len(), GRID_WIDTH * GRID_HEIGHT);

    // Verify all cells match
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            assert_eq!(
                restored.get(x, y),
                grid.get(x, y),
                "mismatch at ({}, {}): expected {}, got {}",
                x,
                y,
                grid.get(x, y),
                restored.get(x, y),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Default (all-zero) grid returns None on save
// ---------------------------------------------------------------------------

#[test]
fn test_heating_grid_default_returns_none_on_save() {
    let grid = HeatingGrid::default();
    assert!(
        grid.save_to_bytes().is_none(),
        "all-zero heating grid should return None (skip saving)"
    );
}

// ---------------------------------------------------------------------------
// Corrupt / wrong-length bytes fall back to default
// ---------------------------------------------------------------------------

#[test]
fn test_heating_grid_load_from_corrupt_bytes_returns_default() {
    let corrupt = vec![0xFF, 0xFE, 0xFD];
    let restored = HeatingGrid::load_from_bytes(&corrupt);

    assert_eq!(restored.width, GRID_WIDTH);
    assert_eq!(restored.height, GRID_HEIGHT);
    assert_eq!(restored.levels.len(), GRID_WIDTH * GRID_HEIGHT);
    assert!(
        restored.levels.iter().all(|&v| v == 0),
        "corrupt bytes should produce an all-zero default grid"
    );
}

// ---------------------------------------------------------------------------
// Wrong-length valid decode falls back to default
// ---------------------------------------------------------------------------

#[test]
fn test_heating_grid_load_from_wrong_length_returns_default() {
    // Encode a smaller vector -- valid bitcode but wrong grid size
    let small_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
    let bytes = bitcode::encode(&small_vec);

    let restored = HeatingGrid::load_from_bytes(&bytes);
    assert_eq!(restored.levels.len(), GRID_WIDTH * GRID_HEIGHT);
    assert!(
        restored.levels.iter().all(|&v| v == 0),
        "wrong-length decode should produce an all-zero default grid"
    );
}

// ---------------------------------------------------------------------------
// Save key is correct
// ---------------------------------------------------------------------------

#[test]
fn test_heating_grid_save_key() {
    assert_eq!(HeatingGrid::SAVE_KEY, "heating_grid");
}
