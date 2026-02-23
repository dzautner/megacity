//! ECS systems for updating neighborhood quality indices.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::crime::CrimeGrid;
use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};
use crate::grid::WorldGrid;
use crate::happiness::ServiceCoverageGrid;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::SlowTickTimer;

use super::compute::{
    compute_composite_index, compute_environment_quality, compute_park_access, compute_safety,
    compute_service_coverage, compute_walkability,
};
use super::types::{DistrictQuality, NeighborhoodQualityIndex, MAX_BUILDING_LEVEL};

/// System that computes the neighborhood quality index per district on the slow tick.
#[allow(clippy::too_many_arguments)]
pub fn update_neighborhood_quality(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    coverage: Res<ServiceCoverageGrid>,
    pollution: Res<PollutionGrid>,
    noise: Res<NoisePollutionGrid>,
    crime: Res<CrimeGrid>,
    buildings: Query<&Building>,
    mut quality_index: ResMut<NeighborhoodQualityIndex>,
) {
    if !timer.should_run() {
        return;
    }

    // Pre-compute per-district building quality accumulators
    let num_districts = DISTRICTS_X * DISTRICTS_Y;
    let mut building_level_sum = vec![0u32; num_districts];
    let mut building_count = vec![0u32; num_districts];

    for building in &buildings {
        let (dx, dy) = Districts::district_for_grid(building.grid_x, building.grid_y);
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            let idx = dy * DISTRICTS_X + dx;
            building_level_sum[idx] += building.level as u32;
            building_count[idx] += 1;
        }
    }

    let mut city_quality_sum = 0.0f32;
    let mut city_district_count = 0u32;

    for dy in 0..DISTRICTS_Y {
        for dx in 0..DISTRICTS_X {
            let idx = dy * DISTRICTS_X + dx;
            let start_x = dx * DISTRICT_SIZE;
            let start_y = dy * DISTRICT_SIZE;

            let walkability = compute_walkability(&grid, start_x, start_y, DISTRICT_SIZE);
            let service_cov = compute_service_coverage(&coverage, start_x, start_y, DISTRICT_SIZE);
            let environment =
                compute_environment_quality(&pollution, &noise, start_x, start_y, DISTRICT_SIZE);
            let safety = compute_safety(&crime, start_x, start_y, DISTRICT_SIZE);
            let park = compute_park_access(&coverage, start_x, start_y, DISTRICT_SIZE);

            let bq = if building_count[idx] > 0 {
                (building_level_sum[idx] as f32 / building_count[idx] as f32 / MAX_BUILDING_LEVEL)
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };

            let mut dq = DistrictQuality {
                overall: 0.0,
                walkability,
                service_coverage: service_cov,
                environment,
                safety,
                park_access: park,
                building_quality: bq,
            };
            dq.overall = compute_composite_index(&dq);
            quality_index.districts[idx] = dq;

            city_quality_sum += quality_index.districts[idx].overall;
            city_district_count += 1;
        }
    }

    quality_index.city_average = if city_district_count > 0 {
        city_quality_sum / city_district_count as f32
    } else {
        0.0
    };
}
