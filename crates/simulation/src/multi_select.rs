//! Multi-Select for Batch Operations (UX-059).
//!
//! Provides a `MultiSelectState` resource that tracks which entities (buildings
//! or road cells) the player has Ctrl+Clicked to add to a batch selection.
//!
//! Supports two batch operations:
//! - **Batch Bulldoze**: demolish all selected entities at once.
//! - **Batch Road Upgrade**: upgrade all selected road cells to their next tier.
//!
//! The selection count and total cost for the pending batch operation are
//! exposed for the UI status bar.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::Saveable;

// =============================================================================
// Types
// =============================================================================

/// Represents a selectable item in the multi-select system.
/// Can be either a building entity or a road cell (by grid coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectableItem {
    /// A building or service entity.
    Building(Entity),
    /// A road cell identified by grid coordinates.
    RoadCell { x: usize, y: usize },
}

// =============================================================================
// Resource
// =============================================================================

/// Resource tracking the current multi-select state.
///
/// Players add items via Ctrl+Click. The UI reads `selected_items` to display
/// the count and compute batch operation costs.
#[derive(Resource, Debug, Clone, Default)]
pub struct MultiSelectState {
    /// The set of currently selected items.
    pub selected_items: Vec<SelectableItem>,
}

impl MultiSelectState {
    /// Add an item to the selection if not already present.
    pub fn add(&mut self, item: SelectableItem) {
        if !self.selected_items.contains(&item) {
            self.selected_items.push(item);
        }
    }

    /// Remove an item from the selection (toggle off).
    pub fn remove(&mut self, item: &SelectableItem) {
        self.selected_items.retain(|i| i != item);
    }

    /// Toggle an item: add it if absent, remove it if present.
    pub fn toggle(&mut self, item: SelectableItem) {
        if self.selected_items.contains(&item) {
            self.remove(&item);
        } else {
            self.add(item);
        }
    }

    /// Returns the number of selected items.
    pub fn count(&self) -> usize {
        self.selected_items.len()
    }

    /// Check whether a specific item is selected.
    pub fn contains(&self, item: &SelectableItem) -> bool {
        self.selected_items.contains(item)
    }

    /// Clear the entire selection.
    pub fn clear(&mut self) {
        self.selected_items.clear();
    }

    /// Returns true if the selection is empty.
    pub fn is_empty(&self) -> bool {
        self.selected_items.is_empty()
    }

    /// Count of selected buildings.
    pub fn building_count(&self) -> usize {
        self.selected_items
            .iter()
            .filter(|i| matches!(i, SelectableItem::Building(_)))
            .count()
    }

    /// Count of selected road cells.
    pub fn road_count(&self) -> usize {
        self.selected_items
            .iter()
            .filter(|i| matches!(i, SelectableItem::RoadCell { .. }))
            .count()
    }

    /// Collect all building entities from the selection.
    pub fn buildings(&self) -> Vec<Entity> {
        self.selected_items
            .iter()
            .filter_map(|i| match i {
                SelectableItem::Building(e) => Some(*e),
                _ => None,
            })
            .collect()
    }

    /// Collect all road cell coordinates from the selection.
    pub fn road_cells(&self) -> Vec<(usize, usize)> {
        self.selected_items
            .iter()
            .filter_map(|i| match i {
                SelectableItem::RoadCell { x, y } => Some((*x, *y)),
                _ => None,
            })
            .collect()
    }
}

// =============================================================================
// Saveable
// =============================================================================

/// Serializable form for save/load. We don't persist Entity references across
/// saves (they're not stable), so we only save road cell selections.
/// Building selections are cleared on save/load since entity IDs change.
#[derive(Encode, Decode, Default)]
struct MultiSelectSave {
    road_cells: Vec<(u16, u16)>,
}

impl Saveable for MultiSelectState {
    const SAVE_KEY: &'static str = "multi_select";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.is_empty() {
            return None;
        }
        let save = MultiSelectSave {
            road_cells: self
                .road_cells()
                .iter()
                .map(|&(x, y)| (x as u16, y as u16))
                .collect(),
        };
        Some(bitcode::encode(&save))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let save: MultiSelectSave = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        let mut state = MultiSelectState::default();
        for (x, y) in save.road_cells {
            state.add(SelectableItem::RoadCell {
                x: x as usize,
                y: y as usize,
            });
        }
        state
    }
}

// =============================================================================
// Events
// =============================================================================

/// Event fired to request a batch bulldoze of all selected items.
#[derive(Event)]
pub struct BatchBulldozeEvent;

/// Event fired to request a batch road upgrade of all selected road cells.
#[derive(Event)]
pub struct BatchRoadUpgradeEvent;

// =============================================================================
// Plugin
// =============================================================================

pub struct MultiSelectPlugin;

impl Plugin for MultiSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiSelectState>()
            .add_event::<BatchBulldozeEvent>()
            .add_event::<BatchRoadUpgradeEvent>();

        // Register with save system
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<MultiSelectState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_contains() {
        let mut state = MultiSelectState::default();
        let item = SelectableItem::RoadCell { x: 5, y: 10 };
        state.add(item);
        assert!(state.contains(&item));
        assert_eq!(state.count(), 1);
    }

    #[test]
    fn test_add_duplicate_is_noop() {
        let mut state = MultiSelectState::default();
        let item = SelectableItem::RoadCell { x: 5, y: 10 };
        state.add(item);
        state.add(item);
        assert_eq!(state.count(), 1);
    }

    #[test]
    fn test_remove() {
        let mut state = MultiSelectState::default();
        let item = SelectableItem::RoadCell { x: 5, y: 10 };
        state.add(item);
        state.remove(&item);
        assert!(!state.contains(&item));
        assert_eq!(state.count(), 0);
    }

    #[test]
    fn test_toggle() {
        let mut state = MultiSelectState::default();
        let item = SelectableItem::RoadCell { x: 5, y: 10 };
        state.toggle(item);
        assert!(state.contains(&item));
        state.toggle(item);
        assert!(!state.contains(&item));
    }

    #[test]
    fn test_clear() {
        let mut state = MultiSelectState::default();
        state.add(SelectableItem::RoadCell { x: 1, y: 2 });
        state.add(SelectableItem::RoadCell { x: 3, y: 4 });
        assert_eq!(state.count(), 2);
        state.clear();
        assert!(state.is_empty());
    }

    #[test]
    fn test_building_and_road_counts() {
        let mut state = MultiSelectState::default();
        state.add(SelectableItem::RoadCell { x: 1, y: 2 });
        state.add(SelectableItem::RoadCell { x: 3, y: 4 });
        state.add(SelectableItem::Building(Entity::from_raw(42)));
        assert_eq!(state.building_count(), 1);
        assert_eq!(state.road_count(), 2);
        assert_eq!(state.count(), 3);
    }

    #[test]
    fn test_buildings_collection() {
        let mut state = MultiSelectState::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        state.add(SelectableItem::Building(e1));
        state.add(SelectableItem::Building(e2));
        state.add(SelectableItem::RoadCell { x: 0, y: 0 });
        let buildings = state.buildings();
        assert_eq!(buildings.len(), 2);
        assert!(buildings.contains(&e1));
        assert!(buildings.contains(&e2));
    }

    #[test]
    fn test_road_cells_collection() {
        let mut state = MultiSelectState::default();
        state.add(SelectableItem::RoadCell { x: 5, y: 10 });
        state.add(SelectableItem::RoadCell { x: 15, y: 20 });
        state.add(SelectableItem::Building(Entity::from_raw(1)));
        let roads = state.road_cells();
        assert_eq!(roads.len(), 2);
        assert!(roads.contains(&(5, 10)));
        assert!(roads.contains(&(15, 20)));
    }

    #[test]
    fn test_save_load_round_trip() {
        let mut state = MultiSelectState::default();
        state.add(SelectableItem::RoadCell { x: 10, y: 20 });
        state.add(SelectableItem::RoadCell { x: 30, y: 40 });
        // Building entities are NOT persisted
        state.add(SelectableItem::Building(Entity::from_raw(99)));

        let bytes = state.save_to_bytes().expect("should save non-empty state");
        let loaded = MultiSelectState::load_from_bytes(&bytes);

        // Only road cells survive save/load
        assert_eq!(loaded.road_count(), 2);
        assert_eq!(loaded.building_count(), 0);
        assert!(loaded.contains(&SelectableItem::RoadCell { x: 10, y: 20 }));
        assert!(loaded.contains(&SelectableItem::RoadCell { x: 30, y: 40 }));
    }

    #[test]
    fn test_save_empty_returns_none() {
        let state = MultiSelectState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_road_type_upgrade_tier() {
        use crate::grid::RoadType;
        assert_eq!(RoadType::Path.upgrade_tier(), Some(RoadType::Local));
        assert_eq!(RoadType::Local.upgrade_tier(), Some(RoadType::Avenue));
        assert_eq!(RoadType::Avenue.upgrade_tier(), Some(RoadType::Boulevard));
        assert_eq!(RoadType::OneWay.upgrade_tier(), Some(RoadType::Avenue));
        assert_eq!(RoadType::Boulevard.upgrade_tier(), None);
        assert_eq!(RoadType::Highway.upgrade_tier(), None);
    }

    #[test]
    fn test_road_type_upgrade_cost() {
        use crate::grid::RoadType;
        // Local ($10) -> Avenue ($20) = $10 upgrade cost
        assert_eq!(RoadType::Local.upgrade_cost(), Some(10.0));
        // Avenue ($20) -> Boulevard ($30) = $10 upgrade cost
        assert_eq!(RoadType::Avenue.upgrade_cost(), Some(10.0));
        // Path ($5) -> Local ($10) = $5 upgrade cost
        assert_eq!(RoadType::Path.upgrade_cost(), Some(5.0));
        // OneWay ($15) -> Avenue ($20) = $5 upgrade cost
        assert_eq!(RoadType::OneWay.upgrade_cost(), Some(5.0));
        // Boulevard has no upgrade
        assert_eq!(RoadType::Boulevard.upgrade_cost(), None);
        // Highway has no upgrade
        assert_eq!(RoadType::Highway.upgrade_cost(), None);
    }
}
