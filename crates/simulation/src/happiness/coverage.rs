use bevy::prelude::*;

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};

use super::constants::*;

/// Per-cell service coverage flags, precomputed when service buildings change.
/// Uses bitflags packed into a single Vec<u8> — 5x less memory than 5 separate Vec<bool>.
#[derive(Resource)]
pub struct ServiceCoverageGrid {
    /// One byte per cell, with bits for each service type.
    pub flags: Vec<u8>,
    pub dirty: bool,
}

impl Default for ServiceCoverageGrid {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            flags: vec![0; n],
            dirty: true,
        }
    }
}

impl ServiceCoverageGrid {
    pub fn clear(&mut self) {
        self.flags.fill(0);
    }

    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    #[inline]
    pub fn has_health(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_HEALTH != 0
    }
    #[inline]
    pub fn has_education(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_EDUCATION != 0
    }
    #[inline]
    pub fn has_police(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_POLICE != 0
    }
    #[inline]
    pub fn has_park(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_PARK != 0
    }
    #[inline]
    pub fn has_entertainment(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_ENTERTAINMENT != 0
    }
    #[inline]
    pub fn has_telecom(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_TELECOM != 0
    }
    #[inline]
    pub fn has_transport(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_TRANSPORT != 0
    }
    #[inline]
    pub fn has_fire(&self, idx: usize) -> bool {
        self.flags[idx] & COVERAGE_FIRE != 0
    }
}

pub fn update_service_coverage(
    services: Query<&ServiceBuilding>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    mut coverage: ResMut<ServiceCoverageGrid>,
    ext_budget: Res<crate::budget::ExtendedBudget>,
) {
    if !added_services.is_empty() {
        coverage.dirty = true;
    }
    if ext_budget.is_changed() {
        coverage.dirty = true;
    }
    if !coverage.dirty {
        return;
    }
    coverage.dirty = false;
    coverage.clear();

    for service in &services {
        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let effective_radius = service.radius * budget_level;
        let radius_cells = (effective_radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = effective_radius * effective_radius;

        // Determine which coverage bits this service sets
        let bits = match service.service_type {
            ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => {
                COVERAGE_HEALTH
            }
            ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::University
            | ServiceType::Library
            | ServiceType::Kindergarten => COVERAGE_EDUCATION,
            ServiceType::PoliceStation
            | ServiceType::PoliceKiosk
            | ServiceType::PoliceHQ
            | ServiceType::Prison => COVERAGE_POLICE,
            ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground => {
                COVERAGE_PARK
            }
            ServiceType::Stadium | ServiceType::Plaza | ServiceType::SportsField => {
                COVERAGE_ENTERTAINMENT
            }
            ServiceType::CellTower | ServiceType::DataCenter => COVERAGE_TELECOM,
            ServiceType::BusDepot
            | ServiceType::TrainStation
            | ServiceType::SubwayStation
            | ServiceType::TramDepot
            | ServiceType::FerryPier
            | ServiceType::SmallAirstrip
            | ServiceType::RegionalAirport
            | ServiceType::InternationalAirport => COVERAGE_TRANSPORT,
            ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ => {
                COVERAGE_FIRE
            }
            _ => continue,
        };

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let idx = ServiceCoverageGrid::idx(cx as usize, cy as usize);
                coverage.flags[idx] |= bits;
            }
        }
    }
}
