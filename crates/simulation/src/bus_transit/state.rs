//! Methods on `BusTransitState` and the `Saveable` implementation.

use crate::grid::{CellType, WorldGrid};

use super::types::*;

// =============================================================================
// Public API for placing stops and creating routes
// =============================================================================

impl BusTransitState {
    /// Place a bus stop on a road cell. Returns the new stop ID, or None if
    /// the cell is not a road or a stop already exists there.
    pub fn add_stop(&mut self, grid: &WorldGrid, x: usize, y: usize) -> Option<BusStopId> {
        if !grid.in_bounds(x, y) || grid.get(x, y).cell_type != CellType::Road {
            return None;
        }
        // Check for duplicate stop at same location
        if self.stops.iter().any(|s| s.grid_x == x && s.grid_y == y) {
            return None;
        }
        let id = self.next_stop_id;
        self.next_stop_id += 1;
        self.stops.push(BusStop {
            id,
            grid_x: x,
            grid_y: y,
            waiting: 0,
        });
        Some(id)
    }

    /// Remove a bus stop by ID. Also removes the stop from any routes.
    pub fn remove_stop(&mut self, stop_id: BusStopId) {
        self.stops.retain(|s| s.id != stop_id);
        for route in &mut self.routes {
            route.stop_ids.retain(|&id| id != stop_id);
        }
        // Remove routes that now have fewer than 2 stops
        let removed_route_ids: Vec<BusRouteId> = self
            .routes
            .iter()
            .filter(|r| r.stop_ids.len() < 2)
            .map(|r| r.id)
            .collect();
        for route_id in &removed_route_ids {
            self.remove_route(*route_id);
        }
    }

    /// Create a new bus route from an ordered list of stop IDs.
    /// Returns the route ID, or None if fewer than 2 valid stops.
    pub fn add_route(&mut self, name: String, stop_ids: Vec<BusStopId>) -> Option<BusRouteId> {
        // Validate all stop IDs exist
        let valid_stops: Vec<BusStopId> = stop_ids
            .into_iter()
            .filter(|id| self.stops.iter().any(|s| s.id == *id))
            .collect();

        if valid_stops.len() < 2 || valid_stops.len() > MAX_STOPS_PER_ROUTE {
            return None;
        }

        let id = self.next_route_id;
        self.next_route_id += 1;
        self.routes.push(BusRoute {
            id,
            name,
            stop_ids: valid_stops,
            active: false, // activated by depot check
            total_ridership: 0,
            monthly_ridership: 0,
        });
        Some(id)
    }

    /// Remove a bus route and its buses.
    pub fn remove_route(&mut self, route_id: BusRouteId) {
        self.routes.retain(|r| r.id != route_id);
        self.buses.retain(|b| b.route_id != route_id);
    }

    /// Find the bus stop nearest to the given grid position within MAX_WALK_DISTANCE.
    pub fn nearest_stop(&self, gx: usize, gy: usize) -> Option<&BusStop> {
        self.stops
            .iter()
            .filter(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy <= MAX_WALK_DISTANCE
            })
            .min_by_key(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy
            })
    }

    /// Find the bus stop nearest to a destination, considering only stops
    /// on active routes.
    pub fn nearest_active_stop(&self, gx: usize, gy: usize) -> Option<&BusStop> {
        let active_route_stop_ids: Vec<BusStopId> = self
            .routes
            .iter()
            .filter(|r| r.active)
            .flat_map(|r| r.stop_ids.iter().copied())
            .collect();

        self.stops
            .iter()
            .filter(|s| active_route_stop_ids.contains(&s.id))
            .filter(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy <= MAX_WALK_DISTANCE
            })
            .min_by_key(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy
            })
    }

    /// Estimate transit time in ticks for traveling from (sx,sy) to (dx,dy)
    /// using the bus system. Returns None if no viable route exists.
    pub fn estimate_transit_time(
        &self,
        src_x: usize,
        src_y: usize,
        dst_x: usize,
        dst_y: usize,
    ) -> Option<u32> {
        let origin_stop = self.nearest_active_stop(src_x, src_y)?;
        let dest_stop = self.nearest_active_stop(dst_x, dst_y)?;

        // Walk to origin stop
        let walk_to = manhattan_distance(src_x, src_y, origin_stop.grid_x, origin_stop.grid_y);

        // Check if both stops are on the same route
        let _shared_route = self.routes.iter().find(|r| {
            r.active && r.stop_ids.contains(&origin_stop.id) && r.stop_ids.contains(&dest_stop.id)
        })?;

        // Ride distance (Manhattan between stops as approximation)
        let ride_dist = manhattan_distance(
            origin_stop.grid_x,
            origin_stop.grid_y,
            dest_stop.grid_x,
            dest_stop.grid_y,
        );
        let ride_ticks = (ride_dist as f32 / BUS_SPEED_CELLS_PER_TICK) as u32;

        // Walk from destination stop
        let walk_from = manhattan_distance(dest_stop.grid_x, dest_stop.grid_y, dst_x, dst_y);

        // Total: walk + wait + ride + walk
        Some(walk_to + AVERAGE_WAIT_TICKS + ride_ticks + walk_from)
    }

    /// Get total number of active routes.
    pub fn active_route_count(&self) -> usize {
        self.routes.iter().filter(|r| r.active).count()
    }

    /// Get total ridership across all routes.
    pub fn total_ridership(&self) -> u64 {
        self.routes.iter().map(|r| r.total_ridership).sum()
    }

    /// Get the stop by ID.
    pub fn stop_by_id(&self, id: BusStopId) -> Option<&BusStop> {
        self.stops.iter().find(|s| s.id == id)
    }
}

/// Manhattan distance between two grid cells.
pub(crate) fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> u32 {
    let dx = (x1 as i32 - x2 as i32).unsigned_abs();
    let dy = (y1 as i32 - y2 as i32).unsigned_abs();
    dx + dy
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for BusTransitState {
    const SAVE_KEY: &'static str = "bus_transit";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.stops.is_empty() && self.routes.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
