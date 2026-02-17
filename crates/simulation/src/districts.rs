use bevy::prelude::*;
use bevy::ecs::query::With;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

/// District size in grid cells
pub const DISTRICT_SIZE: usize = 16;
pub const DISTRICTS_X: usize = GRID_WIDTH / DISTRICT_SIZE;
pub const DISTRICTS_Y: usize = GRID_HEIGHT / DISTRICT_SIZE;

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
}
