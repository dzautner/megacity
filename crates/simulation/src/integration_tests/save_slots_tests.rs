//! Integration tests for the save slots system (SAVE-014).
//!
//! Tests verify that the `SaveSlotManager` resource is properly registered,
//! events are handled, and slot operations work within the full ECS pipeline.

use crate::save_slots::{
    DeleteSlotEvent, SaveSlotInfo, SaveSlotManager, SaveToSlotEvent, MAX_SAVE_SLOTS,
};
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
// Slot creation via events
// =============================================================================

#[test]
fn test_save_to_slot_creates_entry() {
    let mut city = TestCity::new();

    // Send a save-to-slot event.
    city.world_mut().send_event(SaveToSlotEvent {
        slot_index: Some(0),
        display_name: "Test City".to_string(),
    });

    // Run a frame so Update systems process the event.
    city.app_mut().update();

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 1);
    let slot = manager.get_slot(0).unwrap();
    assert_eq!(slot.display_name, "Test City");
    assert_eq!(manager.active_slot, Some(0));
}

#[test]
fn test_save_to_slot_auto_assigns_index() {
    let mut city = TestCity::new();

    // Send with slot_index = None to auto-assign.
    city.world_mut().send_event(SaveToSlotEvent {
        slot_index: None,
        display_name: "Auto Slot".to_string(),
    });

    city.app_mut().update();

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 1);
    // Should auto-assign slot index 0.
    assert!(manager.get_slot(0).is_some());
    assert_eq!(manager.get_slot(0).unwrap().display_name, "Auto Slot");
}

#[test]
fn test_save_to_slot_overwrites_existing() {
    let mut city = TestCity::new();

    // Create initial save.
    city.world_mut().send_event(SaveToSlotEvent {
        slot_index: Some(0),
        display_name: "First Save".to_string(),
    });
    city.app_mut().update();

    // Overwrite with new name.
    city.world_mut().send_event(SaveToSlotEvent {
        slot_index: Some(0),
        display_name: "Updated Save".to_string(),
    });
    city.app_mut().update();

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 1, "Should still have just 1 slot");
    assert_eq!(manager.get_slot(0).unwrap().display_name, "Updated Save");
}

// =============================================================================
// Slot deletion via events
// =============================================================================

#[test]
fn test_delete_slot_removes_entry() {
    let mut city = TestCity::new();

    // Create a slot first.
    city.world_mut().send_event(SaveToSlotEvent {
        slot_index: Some(0),
        display_name: "To Delete".to_string(),
    });
    city.app_mut().update();
    assert_eq!(city.resource::<SaveSlotManager>().slot_count(), 1);

    // Delete it.
    city.world_mut().send_event(DeleteSlotEvent { slot_index: 0 });
    city.app_mut().update();

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 0);
    assert!(manager.active_slot.is_none());
}

#[test]
fn test_delete_nonexistent_slot_is_noop() {
    let mut city = TestCity::new();

    city.world_mut()
        .send_event(DeleteSlotEvent { slot_index: 99 });
    city.app_mut().update();

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 0);
}

// =============================================================================
// Multiple slots management
// =============================================================================

#[test]
fn test_multiple_slots_independent() {
    let mut city = TestCity::new();

    for i in 0..3u32 {
        city.world_mut().send_event(SaveToSlotEvent {
            slot_index: Some(i),
            display_name: format!("City {}", i + 1),
        });
        city.app_mut().update();
    }

    let manager = city.resource::<SaveSlotManager>();
    assert_eq!(manager.slot_count(), 3);
    assert_eq!(manager.get_slot(0).unwrap().display_name, "City 1");
    assert_eq!(manager.get_slot(1).unwrap().display_name, "City 2");
    assert_eq!(manager.get_slot(2).unwrap().display_name, "City 3");
}

#[test]
fn test_max_slots_limit() {
    let mut manager = SaveSlotManager::default();
    for i in 0..MAX_SAVE_SLOTS as u32 {
        let info = SaveSlotInfo {
            slot_index: i,
            ..Default::default()
        };
        manager.create_slot(format!("Save {}", i), info);
    }
    assert!(manager.is_full());

    // Trying to create one more should fail.
    let info = SaveSlotInfo {
        slot_index: MAX_SAVE_SLOTS as u32,
        ..Default::default()
    };
    assert!(manager.create_slot("Overflow".to_string(), info).is_none());
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

// =============================================================================
// Slot ordering and querying
// =============================================================================

#[test]
fn test_slots_sorted_by_index() {
    let mut city = TestCity::new();

    // Create slots in reverse order.
    for i in (0..3u32).rev() {
        city.world_mut().send_event(SaveToSlotEvent {
            slot_index: Some(i),
            display_name: format!("Slot {}", i),
        });
        city.app_mut().update();
    }

    let manager = city.resource::<SaveSlotManager>();
    let indices: Vec<u32> = manager.slots.iter().map(|s| s.slot_index).collect();
    assert_eq!(indices, vec![0, 1, 2], "Slots should be sorted by index");
}

use crate::Saveable;

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
