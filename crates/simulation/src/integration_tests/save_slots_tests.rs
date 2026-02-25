//! Integration tests for the save slots system (SAVE-014).
//!
//! Tests verify that the `SaveSlotManager` resource is properly registered,
//! slot operations work, and the Saveable roundtrip preserves state.

use crate::save_slots::{
    SaveSlotInfo, SaveSlotManager, MAX_SAVE_SLOTS,
};
use crate::Saveable;
use crate::test_harness::TestCity;

// =============================================================================
// Resource registration
// =============================================================================

#[test]
fn test_save_slot_manager_registered_as_resource() {
    let city = TestCity::new();
    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 0, "Manager should start empty");
    assert!(manager.active_slot.is_none());
}

// =============================================================================
// Slot creation
// =============================================================================

#[test]
fn test_create_slot_with_metadata() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        let info = SaveSlotInfo {
            slot_index: 0,
            display_name: "My City".to_string(),
            city_name: "Town".to_string(),
            population: 1500,
            treasury: 50000.0,
            day: 42,
            hour: 14.5,
            play_time_seconds: 3600.0,
            timestamp: 1700000000,
        };
        let result = manager.create_slot("My City".to_string(), info);
        assert_eq!(result, Some(0));
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 1);
    let slot = manager.get_slot(0).unwrap();
    assert_eq!(slot.display_name, "My City");
    assert_eq!(slot.population, 1500);
    assert_eq!(slot.treasury, 50000.0);
    assert_eq!(slot.day, 42);
    assert_eq!(manager.active_slot, Some(0));
}

#[test]
fn test_create_multiple_slots() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        for i in 0..5u32 {
            let info = SaveSlotInfo {
                slot_index: i,
                display_name: format!("City {}", i + 1),
                population: (i + 1) * 1000,
                ..Default::default()
            };
            manager.create_slot(format!("City {}", i + 1), info);
        }
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 5);
    for i in 0..5u32 {
        let slot = manager.get_slot(i).unwrap();
        assert_eq!(slot.display_name, format!("City {}", i + 1));
        assert_eq!(slot.population, (i + 1) * 1000);
    }
}

#[test]
fn test_auto_assign_fills_gaps() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        // Create slots 0, 2, 4 (skipping 1, 3).
        for i in [0u32, 2, 4] {
            let info = SaveSlotInfo {
                slot_index: i,
                ..Default::default()
            };
            manager.create_slot(format!("S{}", i), info);
        }
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.next_available_index(), Some(1));
}

// =============================================================================
// Slot deletion
// =============================================================================

#[test]
fn test_delete_slot_removes_entry() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        let info = SaveSlotInfo {
            slot_index: 0,
            ..Default::default()
        };
        manager.create_slot("To Delete".to_string(), info);
        assert_eq!(manager.slot_count(), 1);
        assert!(manager.delete_slot(0));
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 0);
    assert!(manager.active_slot.is_none());
}

#[test]
fn test_delete_nonexistent_slot_is_noop() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        assert!(!manager.delete_slot(99));
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 0);
}

// =============================================================================
// Overwrite and update
// =============================================================================

#[test]
fn test_overwrite_existing_slot() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        let info1 = SaveSlotInfo {
            slot_index: 0,
            population: 100,
            ..Default::default()
        };
        manager.create_slot("First".to_string(), info1);

        let info2 = SaveSlotInfo {
            slot_index: 0,
            population: 500,
            ..Default::default()
        };
        manager.create_slot("Second".to_string(), info2);
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 1, "Should still have 1 slot");
    let slot = manager.get_slot(0).unwrap();
    assert_eq!(slot.display_name, "Second");
    assert_eq!(slot.population, 500);
}

#[test]
fn test_update_slot_preserves_name() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        let info = SaveSlotInfo {
            slot_index: 0,
            population: 100,
            ..Default::default()
        };
        manager.create_slot("Original".to_string(), info);

        let updated = SaveSlotInfo {
            slot_index: 0,
            population: 999,
            treasury: 12345.0,
            day: 10,
            ..Default::default()
        };
        assert!(manager.update_slot(0, updated));
    }

    let manager = city.resource::<SaveSlotManager>();
    let slot = manager.get_slot(0).unwrap();
    assert_eq!(slot.display_name, "Original", "Name should be preserved");
    assert_eq!(slot.population, 999);
    assert_eq!(slot.treasury, 12345.0);
}

// =============================================================================
// Max slots limit
// =============================================================================

#[test]
fn test_max_slots_limit_enforced() {
    let mut city = TestCity::new();

    {
        let mut manager = city.world_mut().resource_mut::<SaveSlotManager>();
        for i in 0..MAX_SAVE_SLOTS as u32 {
            let info = SaveSlotInfo {
                slot_index: i,
                ..Default::default()
            };
            manager.create_slot(format!("Save {}", i), info);
        }
        assert!(manager.is_full());
        assert!(manager.next_available_index().is_none());

        // Attempting to create beyond max should fail.
        let overflow = SaveSlotInfo {
            slot_index: MAX_SAVE_SLOTS as u32,
            ..Default::default()
        };
        assert!(manager.create_slot("Overflow".to_string(), overflow).is_none());
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), MAX_SAVE_SLOTS);
}

// =============================================================================
// Saveable persistence
// =============================================================================

#[test]
fn test_slot_manager_saveable_roundtrip() {
    let mut manager = SaveSlotManager::default();
    for i in 0..3u32 {
        let info = SaveSlotInfo {
            slot_index: i,
            display_name: format!("City {}", i + 1),
            population: (i + 1) * 1000,
            treasury: (i as f64 + 1.0) * 10000.0,
            timestamp: 1700000000 + i as u64 * 100,
            ..Default::default()
        };
        manager.create_slot(info.display_name.clone(), info);
    }
    manager.active_slot = Some(2);

    let bytes = manager.save_to_bytes().unwrap();
    let loaded = SaveSlotManager::load_from_bytes(&bytes);

    assert_eq!(loaded.slot_count(), 3);
    assert_eq!(loaded.active_slot, Some(2));
    for i in 0..3u32 {
        let slot = loaded.get_slot(i).unwrap();
        assert_eq!(slot.display_name, format!("City {}", i + 1));
        assert_eq!(slot.population, (i + 1) * 1000);
    }
}

#[test]
fn test_empty_manager_skips_save() {
    let manager = SaveSlotManager::default();
    assert!(
        manager.save_to_bytes().is_none(),
        "Empty manager should skip saving"
    );
}

// =============================================================================
// Slot ordering and querying
// =============================================================================

#[test]
fn test_slots_by_recency_ordering() {
    let mut manager = SaveSlotManager::default();
    let timestamps = [100u64, 300, 200];
    for (i, ts) in timestamps.iter().enumerate() {
        let info = SaveSlotInfo {
            slot_index: i as u32,
            timestamp: *ts,
            ..Default::default()
        };
        manager.create_slot(format!("S{}", i), info);
    }

    let recent = manager.slots_by_recency();
    assert_eq!(recent[0].slot_index, 1); // ts=300
    assert_eq!(recent[1].slot_index, 2); // ts=200
    assert_eq!(recent[2].slot_index, 0); // ts=100
}

#[test]
fn test_slot_file_paths() {
    let mut manager = SaveSlotManager::default();
    for i in 0..3u32 {
        let info = SaveSlotInfo {
            slot_index: i,
            ..Default::default()
        };
        manager.create_slot(format!("S{}", i), info);
    }
    let paths = manager.all_file_paths();
    assert_eq!(
        paths,
        vec!["saves/slot_1.bin", "saves/slot_2.bin", "saves/slot_3.bin"]
    );
}
