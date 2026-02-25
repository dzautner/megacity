//! SAVE-014: Multiple Named Save Slots.
//!
//! Provides a save slot management system supporting multiple named save files.
//! Each slot has metadata (city name, timestamp, population, treasury, play time)
//! that can be read without loading the full save.
//!
//! The simulation crate owns the slot registry and events. The actual file I/O
//! is performed by the save crate via `PendingSavePath` + `SaveGameEvent`/`LoadGameEvent`.

use bevy::prelude::*;

use crate::Saveable;

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of manual save slots the player can create.
pub const MAX_SAVE_SLOTS: usize = 20;

/// File extension for save files.
pub const SAVE_EXTENSION: &str = "bin";

/// Directory prefix for named save files.
pub const SAVE_DIR: &str = "saves";

// =============================================================================
// Data types
// =============================================================================

/// Metadata for a single save slot, stored in-memory for the load screen.
#[derive(Debug, Clone, PartialEq, bitcode::Encode, bitcode::Decode)]
pub struct SaveSlotInfo {
    /// Unique slot identifier (0-based index).
    pub slot_index: u32,
    /// Player-chosen display name for this save.
    pub display_name: String,
    /// City classification name (e.g., "Town", "City", "Metropolis").
    pub city_name: String,
    /// Total population at time of save.
    pub population: u32,
    /// Treasury balance at time of save.
    pub treasury: f64,
    /// In-game day number.
    pub day: u32,
    /// In-game hour (0.0 .. 24.0).
    pub hour: f32,
    /// Total wall-clock play time in seconds.
    pub play_time_seconds: f64,
    /// Unix timestamp when the save was created.
    pub timestamp: u64,
}

impl Default for SaveSlotInfo {
    fn default() -> Self {
        Self {
            slot_index: 0,
            display_name: "New Save".to_string(),
            city_name: "Settlement".to_string(),
            population: 0,
            treasury: 0.0,
            day: 1,
            hour: 6.0,
            play_time_seconds: 0.0,
            timestamp: 0,
        }
    }
}

impl SaveSlotInfo {
    /// Returns the file path for this save slot.
    pub fn file_path(&self) -> String {
        slot_file_path(self.slot_index)
    }
}

/// Returns the file path for a given slot index.
pub fn slot_file_path(slot_index: u32) -> String {
    format!("{}/slot_{}.{}", SAVE_DIR, slot_index + 1, SAVE_EXTENSION)
}

// =============================================================================
// Resources
// =============================================================================

/// Manages the registry of available save slots.
///
/// Tracks metadata for all known save slots so the UI can display a list
/// without reading each file. The registry itself is persisted via `Saveable`
/// so it survives across sessions.
#[derive(Resource, Debug, Clone, Default, bitcode::Encode, bitcode::Decode)]
pub struct SaveSlotManager {
    /// All known save slots, ordered by slot index.
    pub slots: Vec<SaveSlotInfo>,
    /// The slot index that was most recently saved or loaded.
    pub active_slot: Option<u32>,
}

impl SaveSlotManager {
    /// Returns the number of occupied save slots.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Returns whether the maximum number of slots has been reached.
    pub fn is_full(&self) -> bool {
        self.slots.len() >= MAX_SAVE_SLOTS
    }

    /// Find a slot by its index.
    pub fn get_slot(&self, slot_index: u32) -> Option<&SaveSlotInfo> {
        self.slots.iter().find(|s| s.slot_index == slot_index)
    }

    /// Find a slot by its index (mutable).
    pub fn get_slot_mut(&mut self, slot_index: u32) -> Option<&mut SaveSlotInfo> {
        self.slots.iter_mut().find(|s| s.slot_index == slot_index)
    }

    /// Returns the next available slot index (lowest unused index).
    pub fn next_available_index(&self) -> Option<u32> {
        if self.is_full() {
            return None;
        }
        let used: std::collections::HashSet<u32> =
            self.slots.iter().map(|s| s.slot_index).collect();
        (0..MAX_SAVE_SLOTS as u32).find(|i| !used.contains(i))
    }

    /// Create a new slot with the given display name and metadata.
    /// Returns the slot index, or `None` if the manager is full.
    pub fn create_slot(&mut self, display_name: String, info: SaveSlotInfo) -> Option<u32> {
        if self.is_full() {
            return None;
        }
        let index = info.slot_index;
        // Remove existing slot at this index if any (overwrite).
        self.slots.retain(|s| s.slot_index != index);
        self.slots.push(SaveSlotInfo {
            display_name,
            ..info
        });
        self.slots.sort_by_key(|s| s.slot_index);
        self.active_slot = Some(index);
        Some(index)
    }

    /// Update an existing slot's metadata (e.g., after overwriting a save).
    pub fn update_slot(&mut self, slot_index: u32, info: SaveSlotInfo) -> bool {
        if let Some(slot) = self.get_slot_mut(slot_index) {
            slot.city_name = info.city_name;
            slot.population = info.population;
            slot.treasury = info.treasury;
            slot.day = info.day;
            slot.hour = info.hour;
            slot.play_time_seconds = info.play_time_seconds;
            slot.timestamp = info.timestamp;
            self.active_slot = Some(slot_index);
            true
        } else {
            false
        }
    }

    /// Delete a save slot by index. Returns `true` if the slot was found and removed.
    pub fn delete_slot(&mut self, slot_index: u32) -> bool {
        let before = self.slots.len();
        self.slots.retain(|s| s.slot_index != slot_index);
        let removed = self.slots.len() < before;
        if removed && self.active_slot == Some(slot_index) {
            self.active_slot = None;
        }
        removed
    }

    /// Returns all slot infos sorted by timestamp (most recent first).
    pub fn slots_by_recency(&self) -> Vec<&SaveSlotInfo> {
        let mut sorted: Vec<&SaveSlotInfo> = self.slots.iter().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted
    }

    /// Returns all slot display names.
    pub fn slot_names(&self) -> Vec<&str> {
        self.slots.iter().map(|s| s.display_name.as_str()).collect()
    }

    /// Returns all file paths for existing slots.
    pub fn all_file_paths(&self) -> Vec<String> {
        self.slots.iter().map(|s| s.file_path()).collect()
    }
}

// =============================================================================
// Events
// =============================================================================

/// Request to save the game to a specific named slot.
#[derive(Event, Debug, Clone)]
pub struct SaveToSlotEvent {
    /// The slot index to save to. If `None`, a new slot is created.
    pub slot_index: Option<u32>,
    /// Player-chosen display name for the save.
    pub display_name: String,
}

/// Request to load a game from a specific save slot.
#[derive(Event, Debug, Clone)]
pub struct LoadFromSlotEvent {
    /// The slot index to load from.
    pub slot_index: u32,
}

/// Request to delete a save slot.
#[derive(Event, Debug, Clone)]
pub struct DeleteSlotEvent {
    /// The slot index to delete.
    pub slot_index: u32,
}

/// Request to refresh the slot list by scanning save files on disk.
#[derive(Event, Debug, Clone)]
pub struct RefreshSlotsEvent;

// =============================================================================
// Saveable
// =============================================================================

impl Saveable for SaveSlotManager {
    const SAVE_KEY: &'static str = "save_slot_manager";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.slots.is_empty() && self.active_slot.is_none() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Handles `SaveToSlotEvent` by updating the slot manager metadata.
///
/// Runs in `Update`. The UI is responsible for also sending a `SaveGameEvent`
/// with the appropriate `PendingSavePath` to trigger the actual file write.
#[allow(clippy::too_many_arguments)]
pub fn handle_save_to_slot(
    mut events: EventReader<SaveToSlotEvent>,
    mut manager: ResMut<SaveSlotManager>,
    clock: Res<crate::time_of_day::GameClock>,
    budget: Res<crate::economy::CityBudget>,
    virtual_pop: Res<crate::virtual_population::VirtualPopulation>,
    play_time: Option<Res<crate::play_time::PlayTime>>,
) {
    for event in events.read() {
        let slot_index = match event.slot_index {
            Some(idx) => idx,
            None => match manager.next_available_index() {
                Some(idx) => idx,
                None => {
                    warn!(
                        "Cannot create save slot: maximum {} slots reached",
                        MAX_SAVE_SLOTS
                    );
                    continue;
                }
            },
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let play_secs = play_time
            .as_ref()
            .map(|pt| pt.total_seconds)
            .unwrap_or(0.0);

        let info = SaveSlotInfo {
            slot_index,
            display_name: event.display_name.clone(),
            city_name: String::new(),
            population: virtual_pop.total_virtual,
            treasury: budget.treasury,
            day: clock.day,
            hour: clock.hour,
            play_time_seconds: play_secs,
            timestamp,
        };

        if manager.get_slot(slot_index).is_some() {
            manager.update_slot(slot_index, info);
        } else {
            manager.create_slot(event.display_name.clone(), info);
        }

        info!(
            "Save slot {} ('{}') prepared at {}",
            slot_index + 1,
            event.display_name,
            slot_file_path(slot_index),
        );
    }
}

/// Handles `DeleteSlotEvent` by removing the slot from the manager
/// and deleting the save file from disk.
pub fn handle_delete_slot(
    mut events: EventReader<DeleteSlotEvent>,
    mut manager: ResMut<SaveSlotManager>,
) {
    for event in events.read() {
        let path = slot_file_path(event.slot_index);
        if manager.delete_slot(event.slot_index) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Err(e) = std::fs::remove_file(&path) {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        warn!("Failed to delete save file {}: {}", path, e);
                    }
                }
            }
            info!("Deleted save slot {} ({})", event.slot_index + 1, path);
        } else {
            warn!(
                "Cannot delete slot {}: not found in manager",
                event.slot_index + 1
            );
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SaveSlotsPlugin;

impl Plugin for SaveSlotsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveSlotManager>()
            .add_event::<SaveToSlotEvent>()
            .add_event::<LoadFromSlotEvent>()
            .add_event::<DeleteSlotEvent>()
            .add_event::<RefreshSlotsEvent>()
            .add_systems(Update, (handle_save_to_slot, handle_delete_slot));

        // Register for save/load via SaveableRegistry.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<SaveSlotManager>();
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_manager() {
        let manager = SaveSlotManager::default();
        assert_eq!(manager.slot_count(), 0);
        assert!(!manager.is_full());
        assert!(manager.active_slot.is_none());
    }

    #[test]
    fn test_slot_file_path() {
        assert_eq!(slot_file_path(0), "saves/slot_1.bin");
        assert_eq!(slot_file_path(4), "saves/slot_5.bin");
        assert_eq!(slot_file_path(19), "saves/slot_20.bin");
    }

    #[test]
    fn test_create_and_delete_slot() {
        let mut manager = SaveSlotManager::default();
        let info = SaveSlotInfo { slot_index: 0, ..Default::default() };
        assert_eq!(manager.create_slot("City".to_string(), info), Some(0));
        assert_eq!(manager.slot_count(), 1);
        assert!(manager.delete_slot(0));
        assert_eq!(manager.slot_count(), 0);
    }

    #[test]
    fn test_overwrite_existing_slot() {
        let mut manager = SaveSlotManager::default();
        let info1 = SaveSlotInfo { slot_index: 0, population: 100, ..Default::default() };
        manager.create_slot("First".to_string(), info1);
        let info2 = SaveSlotInfo { slot_index: 0, population: 500, ..Default::default() };
        manager.create_slot("Second".to_string(), info2);
        assert_eq!(manager.slot_count(), 1);
        assert_eq!(manager.get_slot(0).unwrap().population, 500);
    }

    #[test]
    fn test_next_available_index_fills_gaps() {
        let mut manager = SaveSlotManager::default();
        for i in [0u32, 2, 4] {
            let info = SaveSlotInfo { slot_index: i, ..Default::default() };
            manager.create_slot(format!("S{}", i), info);
        }
        assert_eq!(manager.next_available_index(), Some(1));
    }

    #[test]
    fn test_full_manager_rejects_new_slots() {
        let mut manager = SaveSlotManager::default();
        for i in 0..MAX_SAVE_SLOTS as u32 {
            let info = SaveSlotInfo { slot_index: i, ..Default::default() };
            manager.create_slot(format!("S{}", i), info);
        }
        assert!(manager.is_full());
        assert!(manager.next_available_index().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut manager = SaveSlotManager::default();
        let info = SaveSlotInfo {
            slot_index: 0,
            display_name: "My City".to_string(),
            population: 1500,
            treasury: 50000.0,
            day: 42,
            timestamp: 1700000000,
            ..Default::default()
        };
        manager.create_slot("My City".to_string(), info);
        let bytes = manager.save_to_bytes().unwrap();
        let loaded = SaveSlotManager::load_from_bytes(&bytes);
        assert_eq!(loaded.slot_count(), 1);
        assert_eq!(loaded.get_slot(0).unwrap().population, 1500);
    }

    #[test]
    fn test_saveable_skip_empty() {
        let manager = SaveSlotManager::default();
        assert!(manager.save_to_bytes().is_none());
    }

    #[test]
    fn test_slots_by_recency() {
        let mut manager = SaveSlotManager::default();
        for (i, ts) in [(0u32, 100u64), (1, 300), (2, 200)] {
            let info = SaveSlotInfo { slot_index: i, timestamp: ts, ..Default::default() };
            manager.create_slot(format!("S{}", i), info);
        }
        let sorted = manager.slots_by_recency();
        assert_eq!(sorted[0].slot_index, 1);
        assert_eq!(sorted[2].slot_index, 0);
    }
}
