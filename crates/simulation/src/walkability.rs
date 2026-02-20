//! 15-Minute City Walkability Scoring (ZONE-013).
//!
//! Each cell receives a walkability score (0-100) based on how many essential
//! service categories are reachable within walking distance. The scoring follows
//! the Walk Score methodology:
//!
//! - Full points within 400m (~25 cells at CELL_SIZE=16)
//! - Linear decay to 0 at 1600m (~100 cells)
//!
//! Categories and weights:
//! - Grocery/Commercial: 0.25
//! - School/Education:   0.15
//! - Healthcare:         0.20
//! - Park/Recreation:    0.15
//! - Transit:            0.15
//! - Employment:         0.10
//!
//! The composite score is a weighted average of per-category scores. It affects
//! citizen happiness, land value, and mode choice via the `WalkabilityGrid`
//! resource that other systems can read.
//!
//! Computed on the slow tick (every ~100 ticks) since scanning 65K cells is
//! expensive.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Full-score walking distance in cells (~400m / 16m per cell = 25 cells).
const FULL_SCORE_RADIUS: f32 = 25.0;

/// Maximum walking distance in cells (~1600m / 16m per cell = 100 cells).
const MAX_WALK_RADIUS: f32 = 100.0;

/// Category weights (must sum to 1.0).
const WEIGHT_GROCERY: f32 = 0.25;
const WEIGHT_SCHOOL: f32 = 0.15;
const WEIGHT_HEALTHCARE: f32 = 0.20;
const WEIGHT_PARK: f32 = 0.15;
const WEIGHT_TRANSIT: f32 = 0.15;
const WEIGHT_EMPLOYMENT: f32 = 0.10;

/// Maximum happiness bonus from walkability score.
pub const WALKABILITY_HAPPINESS_BONUS: f32 = 8.0;

/// Maximum land value bonus from walkability score.
pub const WALKABILITY_LAND_VALUE_BONUS: i32 = 15;

// =============================================================================
// Walkability category
// =============================================================================

/// The six service categories used for walkability scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkabilityCategory {
    Grocery,
    School,
    Healthcare,
    Park,
    Transit,
    Employment,
}

impl WalkabilityCategory {
    /// Weight of this category in the composite score.
    pub fn weight(self) -> f32 {
        match self {
            WalkabilityCategory::Grocery => WEIGHT_GROCERY,
            WalkabilityCategory::School => WEIGHT_SCHOOL,
            WalkabilityCategory::Healthcare => WEIGHT_HEALTHCARE,
            WalkabilityCategory::Park => WEIGHT_PARK,
            WalkabilityCategory::Transit => WEIGHT_TRANSIT,
            WalkabilityCategory::Employment => WEIGHT_EMPLOYMENT,
        }
    }
}

// =============================================================================
// Walkability grid resource
// =============================================================================

/// Per-cell walkability score (0-100), recomputed every slow tick.
#[derive(Resource, Clone, Encode, Decode)]
pub struct WalkabilityGrid {
    /// One score per cell, indexed as `y * GRID_WIDTH + x`.
    pub scores: Vec<u8>,
    /// City-wide average walkability score.
    pub city_average: f32,
}

impl Default for WalkabilityGrid {
    fn default() -> Self {
        Self {
            scores: vec![0; GRID_WIDTH * GRID_HEIGHT],
            city_average: 0.0,
        }
    }
}

impl WalkabilityGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.scores[y * GRID_WIDTH + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.scores[y * GRID_WIDTH + x] = val;
    }

    /// Returns the walkability score as a 0.0-1.0 fraction.
    #[inline]
    pub fn fraction(&self, x: usize, y: usize) -> f32 {
        self.get(x, y) as f32 / 100.0
    }
}

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
// Helper: classify service types into walkability categories
// =============================================================================

/// Classify a `ServiceType` into a walkability category, if applicable.
pub fn classify_service(service_type: ServiceType) -> Option<WalkabilityCategory> {
    match service_type {
        // Healthcare
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => {
            Some(WalkabilityCategory::Healthcare)
        }
        // School/Education
        ServiceType::ElementarySchool
        | ServiceType::HighSchool
        | ServiceType::University
        | ServiceType::Library
        | ServiceType::Kindergarten => Some(WalkabilityCategory::School),
        // Park/Recreation
        ServiceType::SmallPark
        | ServiceType::LargePark
        | ServiceType::Playground
        | ServiceType::Plaza
        | ServiceType::SportsField => Some(WalkabilityCategory::Park),
        // Transit
        ServiceType::BusDepot
        | ServiceType::TrainStation
        | ServiceType::SubwayStation
        | ServiceType::TramDepot
        | ServiceType::FerryPier => Some(WalkabilityCategory::Transit),
        _ => None,
    }
}

/// Classify a `ZoneType` into a walkability category, if applicable.
pub fn classify_zone(zone_type: ZoneType) -> Option<WalkabilityCategory> {
    match zone_type {
        // Grocery/Commercial: commercial zones and mixed-use with commercial ground floors
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::MixedUse => {
            Some(WalkabilityCategory::Grocery)
        }
        // Employment: industrial, office zones
        ZoneType::Industrial | ZoneType::Office => Some(WalkabilityCategory::Employment),
        _ => None,
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
fn category_score_for_cell(cx: usize, cy: usize, amenity_positions: &[(usize, usize)]) -> f32 {
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

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for WalkabilityGrid {
    const SAVE_KEY: &'static str = "walkability";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all scores are zero (no city built yet)
        if self.scores.iter().all(|&s| s == 0) {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WalkabilityPlugin;

impl Plugin for WalkabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WalkabilityGrid>().add_systems(
            FixedUpdate,
            update_walkability.after(crate::happiness::update_service_coverage),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WalkabilityGrid>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Distance decay tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_decay_within_full_radius() {
        assert!((distance_decay(0.0) - 1.0).abs() < f32::EPSILON);
        assert!((distance_decay(10.0) - 1.0).abs() < f32::EPSILON);
        assert!((distance_decay(25.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_at_max_radius() {
        assert!((distance_decay(100.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_beyond_max() {
        assert!((distance_decay(150.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_midpoint() {
        // At 62.5 cells (halfway between 25 and 100)
        let mid = (FULL_SCORE_RADIUS + MAX_WALK_RADIUS) / 2.0;
        let expected = 0.5;
        assert!((distance_decay(mid) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_monotonic() {
        let mut prev = distance_decay(0.0);
        for d in 1..=110 {
            let current = distance_decay(d as f32);
            assert!(
                current <= prev,
                "decay should be monotonically non-increasing"
            );
            prev = current;
        }
    }

    // -------------------------------------------------------------------------
    // Category weight tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_weights_sum_to_one() {
        let sum = WEIGHT_GROCERY
            + WEIGHT_SCHOOL
            + WEIGHT_HEALTHCARE
            + WEIGHT_PARK
            + WEIGHT_TRANSIT
            + WEIGHT_EMPLOYMENT;
        assert!(
            (sum - 1.0).abs() < f32::EPSILON,
            "category weights must sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn test_category_weights_match_constants() {
        assert!((WalkabilityCategory::Grocery.weight() - WEIGHT_GROCERY).abs() < f32::EPSILON);
        assert!((WalkabilityCategory::School.weight() - WEIGHT_SCHOOL).abs() < f32::EPSILON);
        assert!(
            (WalkabilityCategory::Healthcare.weight() - WEIGHT_HEALTHCARE).abs() < f32::EPSILON
        );
        assert!((WalkabilityCategory::Park.weight() - WEIGHT_PARK).abs() < f32::EPSILON);
        assert!((WalkabilityCategory::Transit.weight() - WEIGHT_TRANSIT).abs() < f32::EPSILON);
        assert!(
            (WalkabilityCategory::Employment.weight() - WEIGHT_EMPLOYMENT).abs() < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Classification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_classify_hospital_as_healthcare() {
        assert_eq!(
            classify_service(ServiceType::Hospital),
            Some(WalkabilityCategory::Healthcare)
        );
    }

    #[test]
    fn test_classify_clinic_as_healthcare() {
        assert_eq!(
            classify_service(ServiceType::MedicalClinic),
            Some(WalkabilityCategory::Healthcare)
        );
    }

    #[test]
    fn test_classify_school_as_school() {
        assert_eq!(
            classify_service(ServiceType::ElementarySchool),
            Some(WalkabilityCategory::School)
        );
        assert_eq!(
            classify_service(ServiceType::HighSchool),
            Some(WalkabilityCategory::School)
        );
        assert_eq!(
            classify_service(ServiceType::University),
            Some(WalkabilityCategory::School)
        );
    }

    #[test]
    fn test_classify_park_as_park() {
        assert_eq!(
            classify_service(ServiceType::SmallPark),
            Some(WalkabilityCategory::Park)
        );
        assert_eq!(
            classify_service(ServiceType::LargePark),
            Some(WalkabilityCategory::Park)
        );
        assert_eq!(
            classify_service(ServiceType::Playground),
            Some(WalkabilityCategory::Park)
        );
    }

    #[test]
    fn test_classify_transit() {
        assert_eq!(
            classify_service(ServiceType::BusDepot),
            Some(WalkabilityCategory::Transit)
        );
        assert_eq!(
            classify_service(ServiceType::TrainStation),
            Some(WalkabilityCategory::Transit)
        );
        assert_eq!(
            classify_service(ServiceType::SubwayStation),
            Some(WalkabilityCategory::Transit)
        );
    }

    #[test]
    fn test_classify_fire_station_is_none() {
        assert_eq!(classify_service(ServiceType::FireStation), None);
    }

    #[test]
    fn test_classify_commercial_zone_as_grocery() {
        assert_eq!(
            classify_zone(ZoneType::CommercialLow),
            Some(WalkabilityCategory::Grocery)
        );
        assert_eq!(
            classify_zone(ZoneType::CommercialHigh),
            Some(WalkabilityCategory::Grocery)
        );
        assert_eq!(
            classify_zone(ZoneType::MixedUse),
            Some(WalkabilityCategory::Grocery)
        );
    }

    #[test]
    fn test_classify_industrial_as_employment() {
        assert_eq!(
            classify_zone(ZoneType::Industrial),
            Some(WalkabilityCategory::Employment)
        );
        assert_eq!(
            classify_zone(ZoneType::Office),
            Some(WalkabilityCategory::Employment)
        );
    }

    #[test]
    fn test_classify_residential_is_none() {
        assert_eq!(classify_zone(ZoneType::ResidentialLow), None);
        assert_eq!(classify_zone(ZoneType::ResidentialHigh), None);
    }

    // -------------------------------------------------------------------------
    // Category score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_category_score_no_amenities() {
        let positions: Vec<(usize, usize)> = vec![];
        let score = category_score_for_cell(128, 128, &positions);
        assert!(score.abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_score_adjacent_amenity() {
        let positions = vec![(128, 128)];
        let score = category_score_for_cell(128, 128, &positions);
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_score_nearby_amenity() {
        let positions = vec![(128, 138)]; // 10 cells away
        let score = category_score_for_cell(128, 128, &positions);
        assert!((score - 1.0).abs() < f32::EPSILON); // within full-score radius
    }

    #[test]
    fn test_category_score_distant_amenity() {
        let positions = vec![(128, 228)]; // 100 cells away
        let score = category_score_for_cell(128, 128, &positions);
        assert!(score.abs() < 0.01); // at or beyond max walk radius
    }

    #[test]
    fn test_category_score_picks_nearest() {
        let positions = vec![(128, 228), (128, 130)]; // far and near
        let score = category_score_for_cell(128, 128, &positions);
        // Should pick the near one (2 cells away -> full score)
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Composite score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_composite_all_categories_full() {
        // If all categories score 1.0, composite should be 1.0 (100)
        let composite = 1.0 * WEIGHT_GROCERY
            + 1.0 * WEIGHT_SCHOOL
            + 1.0 * WEIGHT_HEALTHCARE
            + 1.0 * WEIGHT_PARK
            + 1.0 * WEIGHT_TRANSIT
            + 1.0 * WEIGHT_EMPLOYMENT;
        assert!((composite - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_no_categories() {
        // If all categories score 0, composite should be 0
        let composite = 0.0 * WEIGHT_GROCERY
            + 0.0 * WEIGHT_SCHOOL
            + 0.0 * WEIGHT_HEALTHCARE
            + 0.0 * WEIGHT_PARK
            + 0.0 * WEIGHT_TRANSIT
            + 0.0 * WEIGHT_EMPLOYMENT;
        assert!(composite.abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_only_grocery() {
        // Only grocery scores 1.0, rest are 0
        let composite = 1.0 * WEIGHT_GROCERY;
        let expected = WEIGHT_GROCERY;
        assert!((composite - expected).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Grid tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_grid_default_all_zero() {
        let grid = WalkabilityGrid::default();
        assert!(grid.scores.iter().all(|&s| s == 0));
        assert!(grid.city_average.abs() < f32::EPSILON);
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = WalkabilityGrid::default();
        grid.set(10, 20, 75);
        assert_eq!(grid.get(10, 20), 75);
    }

    #[test]
    fn test_grid_fraction() {
        let mut grid = WalkabilityGrid::default();
        grid.set(5, 5, 50);
        assert!((grid.fraction(5, 5) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_grid_fraction_full() {
        let mut grid = WalkabilityGrid::default();
        grid.set(5, 5, 100);
        assert!((grid.fraction(5, 5) - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let grid = WalkabilityGrid::default();
        assert!(grid.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_non_zero() {
        use crate::Saveable;
        let mut grid = WalkabilityGrid::default();
        grid.set(10, 10, 50);
        assert!(grid.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut grid = WalkabilityGrid::default();
        grid.set(50, 50, 85);
        grid.city_average = 42.5;

        let bytes = grid.save_to_bytes().expect("should serialize");
        let restored = WalkabilityGrid::load_from_bytes(&bytes);

        assert_eq!(restored.get(50, 50), 85);
        assert!((restored.city_average - 42.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(WalkabilityGrid::SAVE_KEY, "walkability");
    }

    // -------------------------------------------------------------------------
    // Constant value verification
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_score_radius() {
        assert_eq!(FULL_SCORE_RADIUS, 25.0);
    }

    #[test]
    fn test_max_walk_radius() {
        assert_eq!(MAX_WALK_RADIUS, 100.0);
    }

    #[test]
    fn test_happiness_bonus_positive() {
        assert!(WALKABILITY_HAPPINESS_BONUS > 0.0);
    }

    #[test]
    fn test_land_value_bonus_positive() {
        assert!(WALKABILITY_LAND_VALUE_BONUS > 0);
    }
}
