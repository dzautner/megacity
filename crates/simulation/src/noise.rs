//! POLL-010: Noise Pollution Logarithmic Attenuation Model
//!
//! Replaces linear Manhattan-distance decay with a physically-based logarithmic
//! attenuation model: `L(d) = L_source - 6.0 * log2(d) - 0.5 * d`
//!
//! The first term models inverse-square-law spherical spreading (6 dB per
//! doubling of distance), and the second term models atmospheric absorption.
//!
//! Source dB levels:
//! - Highway:              80 dB
//! - Boulevard:            55 dB
//! - Avenue:               45 dB
//! - Local / OneWay road:  35 dB
//! - Path:                  0 dB (no noise)
//! - Industrial building:  75 dB
//! - SmallAirstrip:        80 dB
//! - RegionalAirport:      90 dB
//! - InternationalAirport: 95 dB
//! - Stadium:              70 dB
//!
//! Output is mapped to u8 (0-100) for rendering and tier classification
//! compatibility.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};

// ---------------------------------------------------------------------------
// dB source levels
// ---------------------------------------------------------------------------

/// dB source level for a road type. Returns 0.0 for silent roads.
pub fn road_source_db(road_type: RoadType) -> f32 {
    match road_type {
        RoadType::Highway => 80.0,
        RoadType::Boulevard => 55.0,
        RoadType::Avenue => 45.0,
        RoadType::Local => 35.0,
        RoadType::OneWay => 35.0,
        RoadType::Path => 0.0,
    }
}

/// dB source level for an industrial building.
pub const INDUSTRIAL_SOURCE_DB: f32 = 75.0;

/// dB source level for airport service buildings.
pub fn airport_source_db(service_type: ServiceType) -> f32 {
    match service_type {
        ServiceType::SmallAirstrip => 80.0,
        ServiceType::RegionalAirport => 90.0,
        ServiceType::InternationalAirport => 95.0,
        _ => 0.0,
    }
}

/// dB source level for a stadium.
pub const STADIUM_SOURCE_DB: f32 = 70.0;

// ---------------------------------------------------------------------------
// Attenuation helpers
// ---------------------------------------------------------------------------

/// Logarithmic attenuation: `L(d) = source_db - 6.0 * log2(d) - 0.5 * d`.
/// At distance 0, returns the full source level.
/// Returns 0.0 if the attenuated level drops to or below zero.
pub fn attenuated_db(source_db: f32, distance: f32) -> f32 {
    if distance < 1.0 {
        return source_db;
    }
    let level = source_db - 6.0 * distance.log2() - 0.5 * distance;
    if level > 0.0 { level } else { 0.0 }
}

/// Convert a dB level (0-95 range) to a u8 grid value (0-100).
/// Maps linearly: 0 dB -> 0, 95 dB -> 100, clamped.
pub fn db_to_grid_u8(db: f32) -> u8 {
    let scaled = (db / 95.0) * 100.0;
    scaled.clamp(0.0, 100.0) as u8
}

/// Maximum Euclidean distance (in cells) at which a given source can still
/// contribute non-zero noise. Precomputed to limit the propagation radius.
pub fn max_radius(source_db: f32) -> i32 {
    // Binary search or iterate: find max d where attenuated_db > 0
    // For 95 dB that is around 34 cells; for 35 dB around 8 cells.
    let mut d = 1;
    while d < 50 {
        if attenuated_db(source_db, d as f32) <= 0.0 {
            break;
        }
        d += 1;
    }
    d
}

// ---------------------------------------------------------------------------
// Noise grid resource
// ---------------------------------------------------------------------------

/// Noise pollution grid -- higher values = louder area.
/// Values are capped at 100.
#[derive(Resource)]
pub struct NoisePollutionGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for NoisePollutionGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl NoisePollutionGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    /// Add noise at a position, capping at 100.
    fn add(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_add(amount).min(100);
    }

    /// Subtract noise at a position (floor at 0).
    fn sub(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_sub(amount);
    }
}

// ---------------------------------------------------------------------------
// Propagation helper
// ---------------------------------------------------------------------------

/// Propagate noise from a single source cell `(sx, sy)` with the given dB
/// level into surrounding cells using logarithmic attenuation.
fn propagate_noise(noise: &mut NoisePollutionGrid, sx: usize, sy: usize, source_db: f32) {
    let radius = max_radius(source_db);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = sx as i32 + dx;
            let ny = sy as i32 + dy;
            if nx < 0 || ny < 0 || nx as usize >= GRID_WIDTH || ny as usize >= GRID_HEIGHT {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            let db = attenuated_db(source_db, dist);
            if db > 0.0 {
                let val = db_to_grid_u8(db);
                if val > 0 {
                    noise.add(nx as usize, ny as usize, val);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main system
// ---------------------------------------------------------------------------

pub fn update_noise_pollution(
    slow_timer: Res<crate::SlowTickTimer>,
    mut noise: ResMut<NoisePollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Clear the grid each update
    noise.levels.fill(0);

    // --- Roads generate noise using logarithmic attenuation ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road {
                let db = road_source_db(cell.road_type);
                if db > 0.0 {
                    propagate_noise(&mut noise, x, y, db);
                }
            }
        }
    }

    // --- Industrial buildings ---
    for building in &buildings {
        if building.zone_type == ZoneType::Industrial {
            propagate_noise(
                &mut noise,
                building.grid_x,
                building.grid_y,
                INDUSTRIAL_SOURCE_DB,
            );
        }
    }

    // --- Airport and Stadium service buildings ---
    for service in &services {
        let db = match service.service_type {
            ServiceType::SmallAirstrip
            | ServiceType::RegionalAirport
            | ServiceType::InternationalAirport => airport_source_db(service.service_type),
            ServiceType::Stadium => STADIUM_SOURCE_DB,
            _ => 0.0,
        };
        if db > 0.0 {
            propagate_noise(
                &mut noise,
                service.grid_x,
                service.grid_y,
                db,
            );
        }
    }

    // --- Trees reduce noise: grass cells without buildings reduce noise ---
    let mut reductions: Vec<(usize, usize)> = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Grass && cell.building_id.is_none() {
                reductions.push((x, y));
            }
        }
    }
    for (cx, cy) in reductions {
        let radius = 1i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT
                {
                    noise.sub(nx as usize, ny as usize, 2);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoisePollutionGrid>().add_systems(
            FixedUpdate,
            update_noise_pollution
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attenuated_db_at_zero_distance() {
        assert!((attenuated_db(80.0, 0.0) - 80.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_attenuated_db_at_one_cell() {
        // log2(1) = 0, atmospheric = 0.5 => 80 - 0 - 0.5 = 79.5
        let result = attenuated_db(80.0, 1.0);
        assert!(
            (result - 79.5).abs() < 0.01,
            "expected 79.5, got {}",
            result
        );
    }

    #[test]
    fn test_attenuated_db_at_two_cells() {
        // log2(2) = 1, atmospheric = 1.0 => 80 - 6 - 1.0 = 73.0
        let result = attenuated_db(80.0, 2.0);
        assert!(
            (result - 73.0).abs() < 0.01,
            "expected 73.0, got {}",
            result
        );
    }

    #[test]
    fn test_attenuated_db_decays_with_distance() {
        let db1 = attenuated_db(80.0, 1.0);
        let db2 = attenuated_db(80.0, 5.0);
        let db3 = attenuated_db(80.0, 10.0);
        assert!(db1 > db2, "noise should decay: d=1 ({}) > d=5 ({})", db1, db2);
        assert!(
            db2 > db3,
            "noise should decay: d=5 ({}) > d=10 ({})",
            db2,
            db3
        );
    }

    #[test]
    fn test_attenuated_db_floors_at_zero() {
        let result = attenuated_db(10.0, 50.0);
        assert!(
            result == 0.0,
            "far away weak source should be 0, got {}",
            result
        );
    }

    #[test]
    fn test_db_to_grid_u8_mapping() {
        assert_eq!(db_to_grid_u8(0.0), 0);
        assert_eq!(db_to_grid_u8(95.0), 100);
        assert_eq!(db_to_grid_u8(47.5), 50);
    }

    #[test]
    fn test_db_to_grid_u8_clamps_above_95() {
        assert_eq!(db_to_grid_u8(120.0), 100);
    }

    #[test]
    fn test_road_source_db_values() {
        assert!((road_source_db(RoadType::Highway) - 80.0).abs() < f32::EPSILON);
        assert!((road_source_db(RoadType::Boulevard) - 55.0).abs() < f32::EPSILON);
        assert!((road_source_db(RoadType::Avenue) - 45.0).abs() < f32::EPSILON);
        assert!((road_source_db(RoadType::Local) - 35.0).abs() < f32::EPSILON);
        assert!((road_source_db(RoadType::OneWay) - 35.0).abs() < f32::EPSILON);
        assert!((road_source_db(RoadType::Path) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_radius_reasonable() {
        let r95 = max_radius(95.0);
        let r55 = max_radius(55.0);
        assert!(r95 > r55, "louder source should have larger radius");
        assert!(r95 <= 50, "radius should not exceed 50 cells");
        assert!(r55 >= 5, "55 dB source should reach at least 5 cells");
    }

    #[test]
    fn test_airport_source_db_values() {
        assert!(
            (airport_source_db(ServiceType::InternationalAirport) - 95.0).abs() < f32::EPSILON
        );
        assert!((airport_source_db(ServiceType::RegionalAirport) - 90.0).abs() < f32::EPSILON);
        assert!((airport_source_db(ServiceType::SmallAirstrip) - 80.0).abs() < f32::EPSILON);
    }
}
