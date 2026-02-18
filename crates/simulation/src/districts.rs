use bevy::prelude::*;
use bevy::ecs::query::With;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

/// District size in grid cells
pub const DISTRICT_SIZE: usize = 16;
pub const DISTRICTS_X: usize = GRID_WIDTH / DISTRICT_SIZE;
pub const DISTRICTS_Y: usize = GRID_HEIGHT / DISTRICT_SIZE;

// ============================================================================
// Automatic statistical districts (existing Tier 3 aggregation)
// ============================================================================

/// Tier 3 citizens are stored as statistical aggregates per district.
/// This avoids creating ECS entities for ~750K citizens.
#[derive(Resource, Serialize, Deserialize)]
pub struct Districts {
    pub data: Vec<DistrictData>,
}

impl Default for Districts {
    fn default() -> Self {
        Self {
            data: vec![DistrictData::default(); DISTRICTS_X * DISTRICTS_Y],
        }
    }
}

impl Districts {
    pub fn get(&self, dx: usize, dy: usize) -> &DistrictData {
        &self.data[dy * DISTRICTS_X + dx]
    }

    pub fn get_mut(&mut self, dx: usize, dy: usize) -> &mut DistrictData {
        &mut self.data[dy * DISTRICTS_X + dx]
    }

    pub fn district_for_grid(gx: usize, gy: usize) -> (usize, usize) {
        (gx / DISTRICT_SIZE, gy / DISTRICT_SIZE)
    }

    pub fn total_statistical_population(&self) -> u32 {
        self.data.iter().map(|d| d.population).sum()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistrictData {
    pub population: u32,
    pub employed: u32,
    pub avg_happiness: f32,
    pub residential_capacity: u32,
    pub commercial_jobs: u32,
    pub industrial_jobs: u32,
    pub office_jobs: u32,
}

/// System that aggregates Tier 2/3 citizen data into districts
pub fn aggregate_districts(
    slow_tick: Res<crate::SlowTickTimer>,
    mut districts: ResMut<Districts>,
    buildings: Query<&crate::buildings::Building>,
    citizens: Query<(&crate::citizen::CitizenDetails, &crate::citizen::HomeLocation), With<crate::citizen::Citizen>>,
    _grid: Res<crate::grid::WorldGrid>,
) {
    if !slow_tick.should_run() {
        return;
    }
    // Reset district stats
    for d in &mut districts.data {
        d.population = 0;
        d.employed = 0;
        d.avg_happiness = 0.0;
        d.residential_capacity = 0;
        d.commercial_jobs = 0;
        d.industrial_jobs = 0;
        d.office_jobs = 0;
    }

    // Aggregate building data into districts
    for building in &buildings {
        let (dx, dy) = Districts::district_for_grid(building.grid_x, building.grid_y);
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            let d = districts.get_mut(dx, dy);
            if building.zone_type.is_residential() {
                d.residential_capacity += building.capacity;
                d.population += building.occupants;
            } else if building.zone_type.is_commercial() {
                d.commercial_jobs += building.capacity;
                d.employed += building.occupants;
            } else if building.zone_type == crate::grid::ZoneType::Industrial {
                d.industrial_jobs += building.capacity;
                d.employed += building.occupants;
            } else if building.zone_type == crate::grid::ZoneType::Office {
                d.office_jobs += building.capacity;
                d.employed += building.occupants;
            }
        }
    }

    // Compute happiness from actual citizens
    let mut district_happiness_sum: Vec<f32> = vec![0.0; DISTRICTS_X * DISTRICTS_Y];
    let mut district_citizen_count: Vec<u32> = vec![0; DISTRICTS_X * DISTRICTS_Y];
    for (details, home) in &citizens {
        let (dx, dy) = Districts::district_for_grid(home.grid_x, home.grid_y);
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            let idx = dy * DISTRICTS_X + dx;
            district_happiness_sum[idx] += details.happiness;
            district_citizen_count[idx] += 1;
        }
    }
    for i in 0..districts.data.len() {
        if district_citizen_count[i] > 0 {
            districts.data[i].avg_happiness = district_happiness_sum[i] / district_citizen_count[i] as f32;
        }
    }
}

// ============================================================================
// Player-defined district system
// ============================================================================

/// Per-district policy overrides that players can configure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictPolicies {
    /// Override tax rate for this district (None = use city-wide rate).
    pub tax_rate: Option<f32>,
    /// Override speed limit for this district (None = use default).
    pub speed_limit: Option<f32>,
    /// Whether a noise ordinance is active (restricts nighttime noise).
    pub noise_ordinance: bool,
    /// Whether heavy industry is banned in this district.
    pub heavy_industry_ban: bool,
}

impl Default for DistrictPolicies {
    fn default() -> Self {
        Self {
            tax_rate: None,
            speed_limit: None,
            noise_ordinance: false,
            heavy_industry_ban: false,
        }
    }
}

/// Computed per-district statistics (updated by the district_stats system).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerDistrictStats {
    pub population: u32,
    pub avg_happiness: f32,
    pub crime: f32,
}

/// A player-created district with a name, color, cells, and policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct District {
    pub name: String,
    pub color: [f32; 4],
    pub cells: HashSet<(usize, usize)>,
    pub policies: DistrictPolicies,
    pub stats: PlayerDistrictStats,
}

impl District {
    pub fn new(name: String, color: [f32; 4]) -> Self {
        Self {
            name,
            color,
            cells: HashSet::new(),
            policies: DistrictPolicies::default(),
            stats: PlayerDistrictStats::default(),
        }
    }
}

/// Default district definitions with distinct colors.
pub const DEFAULT_DISTRICTS: &[(&str, [f32; 4])] = &[
    ("Downtown", [0.2, 0.5, 1.0, 0.5]),
    ("Suburbs", [0.2, 0.8, 0.3, 0.5]),
    ("Industrial", [0.9, 0.7, 0.1, 0.5]),
    ("Waterfront", [0.1, 0.8, 0.9, 0.5]),
    ("Historic", [0.8, 0.3, 0.3, 0.5]),
    ("University", [0.7, 0.3, 0.9, 0.5]),
    ("Arts", [0.9, 0.4, 0.7, 0.5]),
    ("Tech Park", [0.3, 0.9, 0.6, 0.5]),
];

/// Resource that holds all player-created districts and a grid mapping
/// each cell to its district index.
#[derive(Resource, Serialize, Deserialize)]
pub struct DistrictMap {
    pub districts: Vec<District>,
    /// One entry per grid cell (GRID_WIDTH * GRID_HEIGHT). None = no district.
    pub cell_map: Vec<Option<usize>>,
}

impl Default for DistrictMap {
    fn default() -> Self {
        let mut districts = Vec::new();
        for &(name, color) in DEFAULT_DISTRICTS {
            districts.push(District::new(name.to_string(), color));
        }
        Self {
            districts,
            cell_map: vec![None; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}

impl DistrictMap {
    fn cell_idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    /// Assign a cell to a district. Removes it from any previous district first.
    pub fn assign_cell_to_district(&mut self, x: usize, y: usize, district_idx: usize) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT || district_idx >= self.districts.len() {
            return;
        }
        let idx = Self::cell_idx(x, y);
        // Remove from old district if any
        if let Some(old) = self.cell_map[idx] {
            if old < self.districts.len() {
                self.districts[old].cells.remove(&(x, y));
            }
        }
        self.cell_map[idx] = Some(district_idx);
        self.districts[district_idx].cells.insert((x, y));
    }

    /// Remove a cell's district assignment.
    pub fn remove_cell_from_district(&mut self, x: usize, y: usize) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return;
        }
        let idx = Self::cell_idx(x, y);
        if let Some(old) = self.cell_map[idx] {
            if old < self.districts.len() {
                self.districts[old].cells.remove(&(x, y));
            }
        }
        self.cell_map[idx] = None;
    }

    /// Get the district at a given cell, if any.
    pub fn get_district_at(&self, x: usize, y: usize) -> Option<&District> {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return None;
        }
        let idx = Self::cell_idx(x, y);
        self.cell_map[idx].and_then(|di| self.districts.get(di))
    }

    /// Get the district index at a given cell, if any.
    pub fn get_district_index_at(&self, x: usize, y: usize) -> Option<usize> {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return None;
        }
        self.cell_map[Self::cell_idx(x, y)]
    }
}

/// System that computes per-district population, happiness, and crime.
/// Runs every 50 ticks.
pub fn district_stats(
    tick: Res<crate::TickCounter>,
    mut district_map: ResMut<DistrictMap>,
    buildings: Query<&crate::buildings::Building>,
    citizens: Query<
        (&crate::citizen::CitizenDetails, &crate::citizen::HomeLocation),
        With<crate::citizen::Citizen>,
    >,
    crime_grid: Res<crate::crime::CrimeGrid>,
) {
    if !tick.0.is_multiple_of(50) {
        return;
    }

    let num_districts = district_map.districts.len();
    if num_districts == 0 {
        return;
    }

    // Accumulators
    let mut pop = vec![0u32; num_districts];
    let mut happiness_sum = vec![0.0f32; num_districts];
    let mut happiness_count = vec![0u32; num_districts];
    let mut crime_sum = vec![0.0f32; num_districts];
    let mut crime_count = vec![0u32; num_districts];

    // Population from buildings
    for building in &buildings {
        let idx = DistrictMap::cell_idx(building.grid_x, building.grid_y);
        if let Some(di) = district_map.cell_map.get(idx).copied().flatten() {
            if di < num_districts && building.zone_type.is_residential() {
                pop[di] += building.occupants;
            }
        }
    }

    // Happiness from citizens
    for (details, home) in &citizens {
        let idx = DistrictMap::cell_idx(home.grid_x, home.grid_y);
        if let Some(di) = district_map.cell_map.get(idx).copied().flatten() {
            if di < num_districts {
                happiness_sum[di] += details.happiness;
                happiness_count[di] += 1;
            }
        }
    }

    // Crime: average crime level across cells in each district
    for di in 0..num_districts {
        for &(cx, cy) in &district_map.districts[di].cells {
            if cx < GRID_WIDTH && cy < GRID_HEIGHT {
                crime_sum[di] += crime_grid.get(cx, cy) as f32;
                crime_count[di] += 1;
            }
        }
    }

    // Write stats
    for di in 0..num_districts {
        let d = &mut district_map.districts[di];
        d.stats.population = pop[di];
        d.stats.avg_happiness = if happiness_count[di] > 0 {
            happiness_sum[di] / happiness_count[di] as f32
        } else {
            0.0
        };
        d.stats.crime = if crime_count[di] > 0 {
            crime_sum[di] / crime_count[di] as f32
        } else {
            0.0
        };
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_district_for_grid() {
        assert_eq!(Districts::district_for_grid(0, 0), (0, 0));
        assert_eq!(Districts::district_for_grid(15, 15), (0, 0));
        assert_eq!(Districts::district_for_grid(16, 0), (1, 0));
        assert_eq!(Districts::district_for_grid(255, 255), (15, 15));
    }

    #[test]
    fn test_district_total_pop() {
        let mut districts = Districts::default();
        districts.get_mut(0, 0).population = 100;
        districts.get_mut(1, 0).population = 200;
        assert_eq!(districts.total_statistical_population(), 300);
    }

    #[test]
    fn test_district_map_assign_and_remove() {
        let mut map = DistrictMap::default();
        assert!(map.get_district_at(10, 10).is_none());

        map.assign_cell_to_district(10, 10, 0);
        assert!(map.get_district_at(10, 10).is_some());
        assert_eq!(map.get_district_index_at(10, 10), Some(0));
        assert!(map.districts[0].cells.contains(&(10, 10)));

        // Reassign to different district
        map.assign_cell_to_district(10, 10, 1);
        assert_eq!(map.get_district_index_at(10, 10), Some(1));
        assert!(!map.districts[0].cells.contains(&(10, 10)));
        assert!(map.districts[1].cells.contains(&(10, 10)));

        // Remove
        map.remove_cell_from_district(10, 10);
        assert!(map.get_district_at(10, 10).is_none());
        assert!(!map.districts[1].cells.contains(&(10, 10)));
    }

    #[test]
    fn test_district_map_bounds() {
        let mut map = DistrictMap::default();
        // Out of bounds should not panic
        map.assign_cell_to_district(999, 999, 0);
        map.remove_cell_from_district(999, 999);
        assert!(map.get_district_at(999, 999).is_none());
    }
}
