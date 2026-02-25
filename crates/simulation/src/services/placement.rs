use bevy::prelude::*;

use crate::grid::{CellType, WorldGrid};
use crate::services::{ServiceBuilding, ServiceType};
use crate::utilities::{UtilitySource, UtilityType};

pub fn place_service(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    service_type: ServiceType,
    gx: usize,
    gy: usize,
) -> bool {
    let (fw, fh) = ServiceBuilding::footprint(service_type);

    // Check all cells in footprint are valid
    for dy in 0..fh {
        for dx in 0..fw {
            let cx = gx + dx;
            let cy = gy + dy;
            if !grid.in_bounds(cx, cy) {
                return false;
            }
            let cell = grid.get(cx, cy);
            if cell.cell_type != CellType::Grass || cell.building_id.is_some() {
                return false;
            }
        }
    }

    let entity = commands
        .spawn(ServiceBuilding {
            service_type,
            grid_x: gx,
            grid_y: gy,
            radius: ServiceBuilding::coverage_radius(service_type),
        })
        .id();

    // Mark all cells in footprint
    for dy in 0..fh {
        for dx in 0..fw {
            grid.get_mut(gx + dx, gy + dy).building_id = Some(entity);
        }
    }
    true
}

pub fn place_utility_source(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    utility_type: UtilityType,
    gx: usize,
    gy: usize,
) -> bool {
    if !grid.in_bounds(gx, gy) {
        return false;
    }
    let cell = grid.get(gx, gy);
    if cell.cell_type != CellType::Grass || cell.building_id.is_some() {
        return false;
    }

    let range = match utility_type {
        UtilityType::PowerPlant => 30,
        UtilityType::SolarFarm => 25,
        UtilityType::WindTurbine => 20,
        UtilityType::WaterTower => 25,
        UtilityType::SewagePlant => 20,
        UtilityType::NuclearPlant => 50,
        UtilityType::Geothermal => 35,
        UtilityType::PumpingStation => 15,
        UtilityType::WaterTreatment => 35,
        UtilityType::HydroDam => 40,
    };

    let entity = commands
        .spawn(UtilitySource {
            utility_type,
            grid_x: gx,
            grid_y: gy,
            range,
        })
        .id();

    grid.get_mut(gx, gy).building_id = Some(entity);
    true
}

pub fn utility_cost(utility_type: UtilityType) -> f64 {
    match utility_type {
        UtilityType::PowerPlant => 800.0,
        UtilityType::SolarFarm => 1200.0,
        UtilityType::WindTurbine => 600.0,
        UtilityType::WaterTower => 600.0,
        UtilityType::SewagePlant => 500.0,
        UtilityType::NuclearPlant => 5000.0,
        UtilityType::Geothermal => 3000.0,
        UtilityType::PumpingStation => 400.0,
        UtilityType::WaterTreatment => 1000.0,
        UtilityType::HydroDam => 5000.0,
    }
}
