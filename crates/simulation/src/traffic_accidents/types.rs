use bevy::prelude::*;

/// A single traffic accident on the grid.
#[derive(Debug, Clone)]
pub struct TrafficAccident {
    pub grid_x: usize,
    pub grid_y: usize,
    /// Severity 1-3. Higher = worse.
    pub severity: u8,
    /// Ticks remaining before the accident clears.
    pub ticks_remaining: u32,
    /// Whether an emergency responder is en route or on scene.
    pub responding: bool,
    /// Whether an ambulance has been dispatched (for severity >= 2).
    pub ambulance_dispatched: bool,
}

/// Resource tracking all active and historical traffic accidents.
#[derive(Resource)]
pub struct AccidentTracker {
    pub active_accidents: Vec<TrafficAccident>,
    pub total_accidents: u32,
    pub accidents_this_month: u32,
    pub avg_response_time: f32,
    /// Maximum number of simultaneous active accidents.
    pub max_active: usize,
    /// Accumulated response time ticks for computing the average.
    pub response_time_accum: f32,
    pub response_count: u32,
}

impl Default for AccidentTracker {
    fn default() -> Self {
        Self {
            active_accidents: Vec::new(),
            total_accidents: 0,
            accidents_this_month: 0,
            avg_response_time: 0.0,
            max_active: 10,
            response_time_accum: 0.0,
            response_count: 0,
        }
    }
}
