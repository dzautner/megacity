use bevy::prelude::*;

use crate::buildings::{Building, MixedUseBuilding};
use crate::grid::{CellType, WorldGrid, ZoneType};

// ---------------------------------------------------------------------------
// Intermediate struct for tallying building stats from the grid + ECS.
// ---------------------------------------------------------------------------

pub struct ZoneStats {
    /// Total population living in residential buildings.
    pub(crate) population: u32,
    /// Total capacity of residential buildings.
    pub(crate) residential_capacity: u32,
    /// Total occupants of residential buildings.
    pub(crate) residential_occupants: u32,
    /// Total capacity of commercial buildings.
    pub(crate) commercial_capacity: u32,
    /// Total occupants of commercial buildings.
    pub(crate) commercial_occupants: u32,
    /// Total capacity of industrial buildings.
    pub(crate) industrial_capacity: u32,
    /// Total occupants of industrial buildings.
    pub(crate) industrial_occupants: u32,
    /// Total capacity of office buildings.
    pub(crate) office_capacity: u32,
    /// Total occupants of office buildings.
    pub(crate) office_occupants: u32,
    /// Total job capacity (commercial + industrial + office).
    pub(crate) total_job_capacity: u32,
    /// Total job occupants (commercial + industrial + office).
    pub(crate) total_job_occupants: u32,
    /// Whether any roads exist (needed for bootstrapping).
    pub(crate) has_roads: bool,
}

pub fn gather_zone_stats(
    grid: &WorldGrid,
    buildings: &Query<&Building>,
    mixed_use_buildings: &Query<&MixedUseBuilding>,
) -> ZoneStats {
    let mut stats = ZoneStats {
        population: 0,
        residential_capacity: 0,
        residential_occupants: 0,
        commercial_capacity: 0,
        commercial_occupants: 0,
        industrial_capacity: 0,
        industrial_occupants: 0,
        office_capacity: 0,
        office_occupants: 0,
        total_job_capacity: 0,
        total_job_occupants: 0,
        has_roads: false,
    };

    for cell in &grid.cells {
        if cell.cell_type == CellType::Road {
            stats.has_roads = true;
        }

        if let Some(entity) = cell.building_id {
            if let Ok(b) = buildings.get(entity) {
                match cell.zone {
                    ZoneType::ResidentialLow
                    | ZoneType::ResidentialMedium
                    | ZoneType::ResidentialHigh => {
                        stats.residential_capacity += b.capacity;
                        stats.residential_occupants += b.occupants;
                        stats.population += b.occupants;
                    }
                    ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                        stats.commercial_capacity += b.capacity;
                        stats.commercial_occupants += b.occupants;
                        stats.total_job_capacity += b.capacity;
                        stats.total_job_occupants += b.occupants;
                    }
                    ZoneType::Industrial => {
                        stats.industrial_capacity += b.capacity;
                        stats.industrial_occupants += b.occupants;
                        stats.total_job_capacity += b.capacity;
                        stats.total_job_occupants += b.occupants;
                    }
                    ZoneType::Office => {
                        stats.office_capacity += b.capacity;
                        stats.office_occupants += b.occupants;
                        stats.total_job_capacity += b.capacity;
                        stats.total_job_occupants += b.occupants;
                    }
                    ZoneType::MixedUse => {
                        // MixedUse counts toward both residential and commercial
                        if let Ok(mu) = mixed_use_buildings.get(entity) {
                            stats.residential_capacity += mu.residential_capacity;
                            stats.residential_occupants += mu.residential_occupants;
                            stats.population += mu.residential_occupants;
                            stats.commercial_capacity += mu.commercial_capacity;
                            stats.commercial_occupants += mu.commercial_occupants;
                            stats.total_job_capacity += mu.commercial_capacity;
                            stats.total_job_occupants += mu.commercial_occupants;
                        }
                    }
                    ZoneType::None => {}
                }
            }
        }
    }

    stats
}
