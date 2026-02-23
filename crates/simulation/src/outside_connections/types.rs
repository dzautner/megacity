use serde::{Deserialize, Serialize};

use bevy::prelude::*;

// =============================================================================
// Types
// =============================================================================

/// The kind of outside connection linking the city to the wider world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionType {
    Highway,
    Railway,
    SeaPort,
    Airport,
}

impl ConnectionType {
    pub fn name(self) -> &'static str {
        match self {
            ConnectionType::Highway => "Highway",
            ConnectionType::Railway => "Railway",
            ConnectionType::SeaPort => "Sea Port",
            ConnectionType::Airport => "Airport",
        }
    }

    /// All connection types, useful for iteration.
    pub fn all() -> &'static [ConnectionType] {
        &[
            ConnectionType::Highway,
            ConnectionType::Railway,
            ConnectionType::SeaPort,
            ConnectionType::Airport,
        ]
    }
}

/// A single outside connection point on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutsideConnection {
    pub connection_type: ConnectionType,
    pub grid_x: usize,
    pub grid_y: usize,
    /// Maximum throughput in vehicles (or equivalent units) per day.
    pub capacity: u32,
    /// Current utilization as a fraction 0.0..1.0.
    pub utilization: f32,
}

/// City-wide resource tracking all connections to the outside world.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutsideConnections {
    pub connections: Vec<OutsideConnection>,
}

impl OutsideConnections {
    /// Check if a given connection type is active (at least one connection exists).
    pub fn has_connection(&self, ct: ConnectionType) -> bool {
        self.connections.iter().any(|c| c.connection_type == ct)
    }

    /// Count the number of connections of a given type.
    pub fn count(&self, ct: ConnectionType) -> usize {
        self.connections
            .iter()
            .filter(|c| c.connection_type == ct)
            .count()
    }

    /// Average utilization across all connections of a given type.
    /// Returns 0.0 if no connections of that type exist.
    pub fn avg_utilization(&self, ct: ConnectionType) -> f32 {
        let conns: Vec<&OutsideConnection> = self
            .connections
            .iter()
            .filter(|c| c.connection_type == ct)
            .collect();
        if conns.is_empty() {
            return 0.0;
        }
        let total: f32 = conns.iter().map(|c| c.utilization).sum();
        total / conns.len() as f32
    }

    /// Human-readable effect description for a connection type.
    pub fn effect_description(ct: ConnectionType) -> &'static str {
        match ct {
            ConnectionType::Highway => "Trade import/export, +20% immigration, adds traffic",
            ConnectionType::Railway => "Bulk goods transport, cheaper imports, +10 tourism",
            ConnectionType::SeaPort => "Heavy cargo (oil/ore at 50% cost), boosts industry",
            ConnectionType::Airport => "Tourism +30, high-value exports, boosts office/tech",
        }
    }

    /// Generate UI-friendly stats for all connection types.
    pub fn stats(&self) -> Vec<ConnectionStat> {
        ConnectionType::all()
            .iter()
            .map(|&ct| ConnectionStat {
                connection_type: ct,
                active: self.has_connection(ct),
                count: self.count(ct),
                avg_utilization: self.avg_utilization(ct),
                effect_description: Self::effect_description(ct),
            })
            .collect()
    }
}

/// Summary for the UI panel.
pub struct ConnectionStat {
    pub connection_type: ConnectionType,
    pub active: bool,
    pub count: usize,
    pub avg_utilization: f32,
    pub effect_description: &'static str,
}
