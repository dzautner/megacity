//! Types and resources for the freight traffic system.

use std::collections::HashMap;

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::roads::RoadNode;

/// A single freight truck moving along a pre-computed route.
#[derive(Debug, Clone)]
pub struct FreightTruck {
    /// Grid positions along the route.
    pub route: Vec<RoadNode>,
    /// Current position index in the route.
    pub current_index: usize,
    /// Origin building grid position.
    pub origin: (usize, usize),
    /// Destination building grid position.
    pub destination: (usize, usize),
}

impl FreightTruck {
    /// Returns the current grid position of the truck, or `None` if route is complete.
    pub fn current_position(&self) -> Option<&RoadNode> {
        self.route.get(self.current_index)
    }

    /// Advance the truck along its route by `steps` waypoints.
    pub fn advance(&mut self, steps: usize) {
        self.current_index = (self.current_index + steps).min(self.route.len());
    }

    /// Returns true if the truck has reached its destination.
    pub fn is_arrived(&self) -> bool {
        self.current_index >= self.route.len()
    }
}

/// City-wide freight traffic state resource.
#[derive(Resource, Debug, Clone)]
pub struct FreightTrafficState {
    /// Active freight trucks currently on the road network.
    pub trucks: Vec<FreightTruck>,
    /// Accumulated freight demand from industrial buildings (outbound).
    pub industrial_demand: f32,
    /// Accumulated freight demand from commercial buildings (inbound).
    pub commercial_demand: f32,
    /// Freight satisfaction ratio (0.0-1.0): fraction of demand met by deliveries.
    pub satisfaction: f32,
    /// Total trips completed since last reset.
    pub trips_completed: u64,
    /// Total trips generated since last reset.
    pub trips_generated: u64,
    /// Per-district heavy traffic ban. Key = district index, value = banned.
    pub heavy_traffic_ban: HashMap<usize, bool>,
}

impl Default for FreightTrafficState {
    fn default() -> Self {
        Self {
            trucks: Vec::new(),
            industrial_demand: 0.0,
            commercial_demand: 0.0,
            satisfaction: 1.0,
            trips_completed: 0,
            trips_generated: 0,
            heavy_traffic_ban: HashMap::new(),
        }
    }
}

impl FreightTrafficState {
    /// Toggle the heavy traffic ban for a specific district.
    pub fn toggle_heavy_traffic_ban(&mut self, district_idx: usize) {
        let entry = self.heavy_traffic_ban.entry(district_idx).or_insert(false);
        *entry = !*entry;
    }

    /// Check if heavy traffic is banned in a specific district.
    pub fn is_heavy_traffic_banned(&self, district_idx: usize) -> bool {
        self.heavy_traffic_ban
            .get(&district_idx)
            .copied()
            .unwrap_or(false)
    }
}

/// Serializable subset of FreightTrafficState for save/load.
#[derive(Debug, Clone, Default, Encode, Decode)]
pub(crate) struct FreightTrafficSaveData {
    pub(crate) satisfaction: f32,
    pub(crate) trips_completed: u64,
    pub(crate) trips_generated: u64,
    pub(crate) heavy_traffic_ban: Vec<(usize, bool)>,
}

impl crate::Saveable for FreightTrafficState {
    const SAVE_KEY: &'static str = "freight_traffic";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let data = FreightTrafficSaveData {
            satisfaction: self.satisfaction,
            trips_completed: self.trips_completed,
            trips_generated: self.trips_generated,
            heavy_traffic_ban: self
                .heavy_traffic_ban
                .iter()
                .filter(|(_, &v)| v)
                .map(|(&k, &v)| (k, v))
                .collect(),
        };
        // Skip saving if everything is at default
        if data.trips_completed == 0
            && data.trips_generated == 0
            && data.heavy_traffic_ban.is_empty()
        {
            return None;
        }
        Some(bitcode::encode(&data))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let data: FreightTrafficSaveData = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        Self {
            satisfaction: data.satisfaction,
            trips_completed: data.trips_completed,
            trips_generated: data.trips_generated,
            heavy_traffic_ban: data.heavy_traffic_ban.into_iter().collect(),
            ..Self::default()
        }
    }
}
