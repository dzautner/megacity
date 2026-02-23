//! Distance-decay scoring and the walkability update system.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

use super::categories::{
    classify_service, classify_zone, WalkabilityCategory, WEIGHT_EMPLOYMENT, WEIGHT_GROCERY,
    WEIGHT_HEALTHCARE, WEIGHT_PARK, WEIGHT_SCHOOL, WEIGHT_TRANSIT,
};
use super::grid::WalkabilityGrid;

// =============================================================================
// Constants
// =============================================================================

/// Full-score walking distance in cells (~400m / 16m per cell = 25 cells).
pub(crate) const FULL_SCORE_RADIUS: f32 = 25.0;

/// Maximum walking distance in cells (~1600m / 16m per cell = 100 cells).
pub(crate) const MAX_WALK_RADIUS: f32 = 100.0;

// =============================================================================
// Helper: distance decay
// =============================================================================

/// Compute a distance-based score for a single amenity.
///
/// - Within `FULL_SCORE_RADIUS` cells: returns 1.0 (full score)
/// - Between `FULL_SCORE_RADIUS` and `MAX_WALK_RADIUS`: linear decay from 1.0 to 0.0
/// - Beyond `MAX_WALK_RADIUS`: returns 0.0
pub fn distance_decay(distance_cells: f32) -> f32 {
    if distance_cells <= FULL_SCORE_RADIUS {
        1.0
    } else if distance_cells >= MAX_WALK_RADIUS {
        0.0
    } else {
        1.0 - (distance_cells - FULL_SCORE_RADIUS) / (MAX_WALK_RADIUS - FULL_SCORE_RADIUS)
    }
}

// =============================================================================
// Core scoring: per-cell category score
// =============================================================================

/// For a given cell (cx, cy), compute the best score for a single category
/// based on the nearest amenity of that category.
///
/// Walk Score methodology: the score for a category is determined by the
/// nearest amenity of that type. We use the best (closest) amenity's
/// distance-decayed score.
pub(crate) fn category_score_for_cell(
    cx: usize,
    cy: usize,
    amenity_positions: &[(usize, usize)],
) -> f32 {
    let mut best = 0.0_f32;
    for &(ax, ay) in amenity_positions {
        let dx = (cx as f32) - (ax as f32);
        let dy = (cy as f32) - (ay as f32);
        let dist = (dx * dx + dy * dy).sqrt();
        let score = distance_decay(dist);
        if score > best {
            best = score;
        }
        // Early exit: can't do better than 1.0
        if best >= 1.0 {
            break;
        }
    }
    best
}

// =============================================================================
// System: update walkability scores
// =============================================================================

/// System that recomputes walkability scores for all cells on the slow tick.
///
/// 1. Collects amenity positions grouped by category from service buildings
///    and zoned buildings with occupants.
/// 2. For each cell, computes per-category scores using nearest-amenity
///    distance decay.
/// 3. Computes composite score as weighted average.
/// 4. Updates city-wide average.
#[allow(clippy::too_many_arguments)]
pub fn update_walkability(
    timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    buildings: Query<&Building>,
    mut walkability: ResMut<WalkabilityGrid>,
) {
    if !timer.should_run() {
        return;
    }

    // Collect amenity positions per category
    let mut grocery_positions: Vec<(usize, usize)> = Vec::new();
    let mut school_positions: Vec<(usize, usize)> = Vec::new();
    let mut healthcare_positions: Vec<(usize, usize)> = Vec::new();
    let mut park_positions: Vec<(usize, usize)> = Vec::new();
    let mut transit_positions: Vec<(usize, usize)> = Vec::new();
    let mut employment_positions: Vec<(usize, usize)> = Vec::new();

    // Service buildings
    for service in &services {
        if let Some(cat) = classify_service(service.service_type) {
            let pos = (service.grid_x, service.grid_y);
            match cat {
                WalkabilityCategory::Grocery => grocery_positions.push(pos),
                WalkabilityCategory::School => school_positions.push(pos),
                WalkabilityCategory::Healthcare => healthcare_positions.push(pos),
                WalkabilityCategory::Park => park_positions.push(pos),
                WalkabilityCategory::Transit => transit_positions.push(pos),
                WalkabilityCategory::Employment => employment_positions.push(pos),
            }
        }
    }

    // Zoned buildings: commercial buildings count as grocery, industrial/office as employment
    for building in &buildings {
        if building.occupants == 0 {
            continue;
        }
        if let Some(cat) = classify_zone(building.zone_type) {
            let pos = (building.grid_x, building.grid_y);
            match cat {
                WalkabilityCategory::Grocery => grocery_positions.push(pos),
                WalkabilityCategory::Employment => employment_positions.push(pos),
                _ => {}
            }
        }
    }

    // Compute per-cell walkability scores
    let mut total_score: u64 = 0;
    let mut scored_cells: u64 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let grocery = category_score_for_cell(x, y, &grocery_positions);
            let school = category_score_for_cell(x, y, &school_positions);
            let healthcare = category_score_for_cell(x, y, &healthcare_positions);
            let park = category_score_for_cell(x, y, &park_positions);
            let transit = category_score_for_cell(x, y, &transit_positions);
            let employment = category_score_for_cell(x, y, &employment_positions);

            let composite = grocery * WEIGHT_GROCERY
                + school * WEIGHT_SCHOOL
                + healthcare * WEIGHT_HEALTHCARE
                + park * WEIGHT_PARK
                + transit * WEIGHT_TRANSIT
                + employment * WEIGHT_EMPLOYMENT;

            let score = (composite * 100.0).round().clamp(0.0, 100.0) as u8;
            walkability.set(x, y, score);
            total_score += score as u64;
            scored_cells += 1;
        }
    }

    walkability.city_average = if scored_cells > 0 {
        total_score as f32 / scored_cells as f32
    } else {
        0.0
    };
}
