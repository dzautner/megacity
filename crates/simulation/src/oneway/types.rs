use bevy::prelude::*;
use std::collections::HashMap;

use crate::road_segments::SegmentId;

/// Direction of traffic flow on a one-way segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OneWayDirection {
    /// Traffic flows from start_node to end_node.
    Forward,
    /// Traffic flows from end_node to start_node.
    Reverse,
}

/// Stores one-way direction overrides for road segments.
///
/// By default, all segments are bidirectional. When a segment is added to this map,
/// it becomes one-way in the specified direction.
#[derive(Resource, Default, Debug)]
pub struct OneWayDirectionMap {
    /// Map from segment ID to one-way direction.
    pub directions: HashMap<u32, OneWayDirection>,
    /// Incremented every time the map changes, so systems can detect changes cheaply.
    pub generation: u32,
}

impl OneWayDirectionMap {
    /// Get the one-way direction for a segment, if any.
    pub fn get(&self, id: SegmentId) -> Option<OneWayDirection> {
        self.directions.get(&id.0).copied()
    }

    /// Set a segment to one-way in the given direction.
    pub fn set(&mut self, id: SegmentId, direction: OneWayDirection) {
        self.directions.insert(id.0, direction);
        self.generation = self.generation.wrapping_add(1);
    }

    /// Remove one-way restriction (make bidirectional again).
    pub fn remove(&mut self, id: SegmentId) {
        self.directions.remove(&id.0);
        self.generation = self.generation.wrapping_add(1);
    }

    /// Toggle through: None -> Forward -> Reverse -> None
    pub fn toggle(&mut self, id: SegmentId) {
        match self.get(id) {
            None => self.set(id, OneWayDirection::Forward),
            Some(OneWayDirection::Forward) => self.set(id, OneWayDirection::Reverse),
            Some(OneWayDirection::Reverse) => self.remove(id),
        }
    }

    /// Check if a segment is one-way.
    pub fn is_oneway(&self, id: SegmentId) -> bool {
        self.directions.contains_key(&id.0)
    }
}

/// Event fired when user toggles one-way direction on a segment.
#[derive(Event)]
pub struct ToggleOneWayEvent {
    pub segment_id: SegmentId,
}
