use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::services::{ServiceBuilding, ServiceType};

use super::types::{ConnectionType, OutsideConnection};

// =============================================================================
// Detection helpers
// =============================================================================

/// Cells within this distance of the map boundary count as "edge" cells.
pub(super) const EDGE_PROXIMITY: usize = 3;

/// Check if a grid coordinate is near the map edge.
pub(super) fn is_near_edge(x: usize, y: usize) -> bool {
    !(EDGE_PROXIMITY..GRID_WIDTH - EDGE_PROXIMITY).contains(&x)
        || !(EDGE_PROXIMITY..GRID_HEIGHT - EDGE_PROXIMITY).contains(&y)
}

/// Check if a grid coordinate is near a water edge (water cell within EDGE_PROXIMITY of map boundary).
pub(super) fn is_near_water_edge(x: usize, y: usize, grid: &WorldGrid) -> bool {
    if !is_near_edge(x, y) {
        return false;
    }
    // Check if there's water nearby (within 5 cells)
    let search = 5isize;
    for dy in -search..=search {
        for dx in -search..=search {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < GRID_WIDTH
                && (ny as usize) < GRID_HEIGHT
                && grid.get(nx as usize, ny as usize).cell_type == CellType::Water
            {
                return true;
            }
        }
    }
    false
}

/// Detect highway connections: road cells of type Highway at the map edge.
pub(super) fn detect_highway_connections(grid: &WorldGrid) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    let mut found_positions = Vec::new();

    // Check all four edges
    for x in 0..GRID_WIDTH {
        for &y in &[
            0usize,
            1,
            2,
            GRID_HEIGHT - 3,
            GRID_HEIGHT - 2,
            GRID_HEIGHT - 1,
        ] {
            if y >= GRID_HEIGHT {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road
                && matches!(cell.road_type, RoadType::Highway | RoadType::Boulevard)
            {
                // Avoid duplicate connections for the same road (cluster nearby cells)
                let too_close = found_positions
                    .iter()
                    .any(|&(fx, fy): &(usize, usize)| x.abs_diff(fx) + y.abs_diff(fy) < 10);
                if !too_close {
                    found_positions.push((x, y));
                    connections.push(OutsideConnection {
                        connection_type: ConnectionType::Highway,
                        grid_x: x,
                        grid_y: y,
                        capacity: 5000,
                        utilization: 0.0,
                    });
                }
            }
        }
    }

    for y in 0..GRID_HEIGHT {
        for &x in &[0usize, 1, 2, GRID_WIDTH - 3, GRID_WIDTH - 2, GRID_WIDTH - 1] {
            if x >= GRID_WIDTH {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road
                && matches!(cell.road_type, RoadType::Highway | RoadType::Boulevard)
            {
                let too_close = found_positions
                    .iter()
                    .any(|&(fx, fy): &(usize, usize)| x.abs_diff(fx) + y.abs_diff(fy) < 10);
                if !too_close {
                    found_positions.push((x, y));
                    connections.push(OutsideConnection {
                        connection_type: ConnectionType::Highway,
                        grid_x: x,
                        grid_y: y,
                        capacity: 5000,
                        utilization: 0.0,
                    });
                }
            }
        }
    }

    connections
}

/// Detect railway connections from TrainStation service buildings near map edge.
pub(super) fn detect_railway_connections(
    services: &[(&ServiceBuilding,)],
) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    for (service,) in services {
        if service.service_type == ServiceType::TrainStation
            && is_near_edge(service.grid_x, service.grid_y)
        {
            connections.push(OutsideConnection {
                connection_type: ConnectionType::Railway,
                grid_x: service.grid_x,
                grid_y: service.grid_y,
                capacity: 2000,
                utilization: 0.0,
            });
        }
    }
    connections
}

/// Detect sea port connections from FerryPier service buildings near water edge.
pub(super) fn detect_seaport_connections(
    services: &[(&ServiceBuilding,)],
    grid: &WorldGrid,
) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    for (service,) in services {
        if service.service_type == ServiceType::FerryPier
            && is_near_water_edge(service.grid_x, service.grid_y, grid)
        {
            connections.push(OutsideConnection {
                connection_type: ConnectionType::SeaPort,
                grid_x: service.grid_x,
                grid_y: service.grid_y,
                capacity: 3000,
                utilization: 0.0,
            });
        }
    }
    connections
}

/// Detect airport connections from SmallAirstrip, RegionalAirport, or InternationalAirport service buildings.
pub(super) fn detect_airport_connections(
    services: &[(&ServiceBuilding,)],
) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    for (service,) in services {
        match service.service_type {
            ServiceType::SmallAirstrip => {
                connections.push(OutsideConnection {
                    connection_type: ConnectionType::Airport,
                    grid_x: service.grid_x,
                    grid_y: service.grid_y,
                    capacity: 1000,
                    utilization: 0.0,
                });
            }
            ServiceType::RegionalAirport => {
                connections.push(OutsideConnection {
                    connection_type: ConnectionType::Airport,
                    grid_x: service.grid_x,
                    grid_y: service.grid_y,
                    capacity: 3000,
                    utilization: 0.0,
                });
            }
            ServiceType::InternationalAirport => {
                connections.push(OutsideConnection {
                    connection_type: ConnectionType::Airport,
                    grid_x: service.grid_x,
                    grid_y: service.grid_y,
                    capacity: 5000,
                    utilization: 0.0,
                });
            }
            _ => {}
        }
    }
    connections
}
