//! SERV-007: Park District System with Levels
//!
//! Players draw park district boundaries and place park buildings/props within
//! them. Districts level up based on visitor count, number of attractions, and
//! attraction variety. Higher-level districts provide stronger happiness, land
//! value, and tourism bonuses.
//!
//! Park types: CityPark, AmusementPark, NatureReserve, Zoo.
//! Levels 1-5, each requiring more attractions and visitors.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const LEVEL_VISITOR_THRESHOLDS: [u32; 5] = [0, 50, 200, 500, 1500];
const LEVEL_ATTRACTION_THRESHOLDS: [u32; 5] = [0, 2, 5, 10, 20];
const HAPPINESS_PER_LEVEL: [f32; 5] = [3.0, 6.0, 10.0, 15.0, 20.0];
const LAND_VALUE_PER_LEVEL: [f32; 5] = [2.0, 5.0, 8.0, 12.0, 18.0];
const TOURISM_PER_LEVEL: [f32; 5] = [0.0, 1.0, 3.0, 8.0, 15.0];
const RADIUS_PER_LEVEL: [i32; 5] = [6, 8, 12, 16, 22];
const BASE_FEE_PER_VISITOR: f64 = 0.5;
const MAX_ENTRY_FEE: f32 = 5.0;
const BASE_VISITORS_PER_TICK: u32 = 5;
const AMUSEMENT_TOURISM_MULT: f32 = 1.5;
const NATURE_RESERVE_POLLUTION_REDUCTION: u8 = 10;
const ZOO_EDUCATION_BONUS: f32 = 2.0;

// ---------------------------------------------------------------------------
// Park type
// ---------------------------------------------------------------------------

/// The type of park district, each with unique bonuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum ParkType {
    CityPark,
    AmusementPark,
    NatureReserve,
    Zoo,
}

impl Default for ParkType {
    fn default() -> Self {
        Self::CityPark
    }
}

impl ParkType {
    pub fn name(self) -> &'static str {
        match self {
            Self::CityPark => "City Park",
            Self::AmusementPark => "Amusement Park",
            Self::NatureReserve => "Nature Reserve",
            Self::Zoo => "Zoo",
        }
    }
}

// ---------------------------------------------------------------------------
// Park district
// ---------------------------------------------------------------------------

/// A single park district with configuration and tracked state.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ParkDistrict {
    pub id: u32,
    pub park_type: ParkType,
    pub center_x: usize,
    pub center_y: usize,
    pub level: u8,
    pub total_visitors: u32,
    pub attraction_count: u32,
    pub attraction_variety: u32,
    pub entry_fee: f32,
    pub cycle_revenue: f64,
    pub total_revenue: f64,
}

impl ParkDistrict {
    pub fn new(id: u32, park_type: ParkType, cx: usize, cy: usize) -> Self {
        Self {
            id, park_type, center_x: cx, center_y: cy, level: 1,
            total_visitors: 0, attraction_count: 0, attraction_variety: 0,
            entry_fee: 0.0, cycle_revenue: 0.0, total_revenue: 0.0,
        }
    }

    fn level_idx(&self) -> usize {
        (self.level as usize).saturating_sub(1).min(4)
    }

    pub fn happiness_bonus(&self) -> f32 { HAPPINESS_PER_LEVEL[self.level_idx()] }
    pub fn land_value_bonus(&self) -> f32 { LAND_VALUE_PER_LEVEL[self.level_idx()] }
    pub fn radius_cells(&self) -> i32 { RADIUS_PER_LEVEL[self.level_idx()] }

    pub fn tourism_score(&self) -> f32 {
        let base = TOURISM_PER_LEVEL[self.level_idx()];
        if self.park_type == ParkType::AmusementPark { base * AMUSEMENT_TOURISM_MULT } else { base }
    }

    pub fn recalculate_level(&mut self) {
        let mut new_level: u8 = 1;
        for i in 1..5 {
            if self.total_visitors >= LEVEL_VISITOR_THRESHOLDS[i]
                && self.attraction_count >= LEVEL_ATTRACTION_THRESHOLDS[i]
            {
                new_level = (i + 1) as u8;
            }
        }
        self.level = new_level;
    }
}

// ---------------------------------------------------------------------------
// Per-cell effects grid
// ---------------------------------------------------------------------------

/// Precomputed per-cell effects from park districts, updated each slow tick.
#[derive(Resource, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ParkDistrictEffects {
    pub happiness: Vec<f32>,
    pub land_value: Vec<f32>,
    pub pollution_reduction: Vec<u8>,
    pub education_bonus: Vec<f32>,
}

impl Default for ParkDistrictEffects {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            happiness: vec![0.0; n],
            land_value: vec![0.0; n],
            pollution_reduction: vec![0; n],
            education_bonus: vec![0.0; n],
        }
    }
}

impl ParkDistrictEffects {
    #[inline]
    pub fn idx(x: usize, y: usize) -> usize { y * GRID_WIDTH + x }
    pub fn happiness_at(&self, x: usize, y: usize) -> f32 { self.happiness[Self::idx(x, y)] }
    pub fn land_value_at(&self, x: usize, y: usize) -> f32 { self.land_value[Self::idx(x, y)] }

    fn clear(&mut self) {
        self.happiness.fill(0.0);
        self.land_value.fill(0.0);
        self.pollution_reduction.fill(0);
        self.education_bonus.fill(0.0);
    }
}

// ---------------------------------------------------------------------------
// State resource
// ---------------------------------------------------------------------------

/// City-wide park district state: all districts and aggregate statistics.
#[derive(Resource, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ParkDistrictState {
    pub districts: Vec<ParkDistrict>,
    pub next_id: u32,
    pub total_tourism: f32,
    pub cycle_revenue: f64,
}

impl Default for ParkDistrictState {
    fn default() -> Self {
        Self { districts: Vec::new(), next_id: 1, total_tourism: 0.0, cycle_revenue: 0.0 }
    }
}

impl ParkDistrictState {
    pub fn create_district(&mut self, park_type: ParkType, cx: usize, cy: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.districts.push(ParkDistrict::new(id, park_type, cx, cy));
        id
    }

    pub fn remove_district(&mut self, id: u32) {
        self.districts.retain(|d| d.id != id);
    }

    pub fn get_district(&self, id: u32) -> Option<&ParkDistrict> {
        self.districts.iter().find(|d| d.id == id)
    }

    pub fn get_district_mut(&mut self, id: u32) -> Option<&mut ParkDistrict> {
        self.districts.iter_mut().find(|d| d.id == id)
    }
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for ParkDistrictState {
    const SAVE_KEY: &'static str = "park_districts";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.districts.is_empty() { return None; }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_park_attraction(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground
        | ServiceType::SportsField | ServiceType::Plaza | ServiceType::Stadium
    )
}

fn attraction_type_bit(st: ServiceType) -> u8 {
    match st {
        ServiceType::SmallPark => 1,
        ServiceType::LargePark => 2,
        ServiceType::Playground => 4,
        ServiceType::SportsField => 8,
        ServiceType::Plaza => 16,
        ServiceType::Stadium => 32,
        _ => 0,
    }
}

fn count_attractions_for_district(district: &mut ParkDistrict, services: &[&ServiceBuilding]) {
    let radius = district.radius_cells();
    let cx = district.center_x as i32;
    let cy = district.center_y as i32;
    let mut count = 0u32;
    let mut types_seen = 0u8;
    for svc in services {
        if !is_park_attraction(svc.service_type) { continue; }
        let dx = (svc.grid_x as i32 - cx).abs();
        let dy = (svc.grid_y as i32 - cy).abs();
        if dx <= radius && dy <= radius {
            count += 1;
            types_seen |= attraction_type_bit(svc.service_type);
        }
    }
    district.attraction_count = count;
    district.attraction_variety = types_seen.count_ones();
}

fn apply_district_effects(district: &ParkDistrict, effects: &mut ParkDistrictEffects) {
    let radius = district.radius_cells();
    let cx = district.center_x as i32;
    let cy = district.center_y as i32;
    let r2 = (radius as f32 * CELL_SIZE) * (radius as f32 * CELL_SIZE);
    let happiness = district.happiness_bonus();
    let land_val = district.land_value_bonus();
    let max_dist = radius as f32 * CELL_SIZE;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                continue;
            }
            let wx = dx as f32 * CELL_SIZE;
            let wy = dy as f32 * CELL_SIZE;
            let dist_sq = wx * wx + wy * wy;
            if dist_sq > r2 { continue; }

            let idx = ParkDistrictEffects::idx(nx as usize, ny as usize);
            let falloff = 1.0 - (dist_sq.sqrt() / max_dist).clamp(0.0, 1.0) * 0.5;
            effects.happiness[idx] = effects.happiness[idx].max(happiness * falloff);
            effects.land_value[idx] = effects.land_value[idx].max(land_val * falloff);

            match district.park_type {
                ParkType::NatureReserve => {
                    let red = (NATURE_RESERVE_POLLUTION_REDUCTION as f32 * falloff) as u8;
                    effects.pollution_reduction[idx] = effects.pollution_reduction[idx].max(red);
                }
                ParkType::Zoo => {
                    effects.education_bonus[idx] =
                        effects.education_bonus[idx].max(ZOO_EDUCATION_BONUS * falloff);
                }
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Update park districts each slow tick: count attractions, simulate visitors,
/// recalculate levels, compute entry fee revenue, and apply per-cell effects.
pub fn update_park_districts(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<ParkDistrictState>,
    mut effects: ResMut<ParkDistrictEffects>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() { return; }
    effects.clear();
    state.cycle_revenue = 0.0;
    state.total_tourism = 0.0;

    let svc_list: Vec<&ServiceBuilding> = services.iter().collect();
    let pop = stats.population.max(1);

    for district in &mut state.districts {
        count_attractions_for_district(district, &svc_list);
        let visitor_rate = BASE_VISITORS_PER_TICK
            .saturating_mul(district.attraction_count.max(1))
            .min(pop / 4);
        district.total_visitors = district.total_visitors.saturating_add(visitor_rate);
        district.recalculate_level();

        district.cycle_revenue = if district.entry_fee > 0.0 {
            let fee = (district.entry_fee as f64).min(MAX_ENTRY_FEE as f64);
            visitor_rate as f64 * BASE_FEE_PER_VISITOR * fee
        } else { 0.0 };
        district.total_revenue += district.cycle_revenue;

        apply_district_effects(district, &mut effects);
        state.total_tourism += district.tourism_score();
        state.cycle_revenue += district.cycle_revenue;
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ParkDistrictPlugin;

impl Plugin for ParkDistrictPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParkDistrictState>();
        app.init_resource::<ParkDistrictEffects>();
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ParkDistrictState>();
        app.add_systems(
            FixedUpdate,
            update_park_districts
                .after(crate::stats::update_stats)
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
    fn test_create_and_remove_district() {
        let mut state = ParkDistrictState::default();
        let id1 = state.create_district(ParkType::CityPark, 10, 10);
        let id2 = state.create_district(ParkType::Zoo, 20, 20);
        assert_eq!(state.districts.len(), 2);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        state.remove_district(id1);
        assert_eq!(state.districts.len(), 1);
        assert_eq!(state.districts[0].id, id2);
    }

    #[test]
    fn test_level_progression() {
        let mut d = ParkDistrict::new(1, ParkType::CityPark, 10, 10);
        assert_eq!(d.level, 1);
        d.total_visitors = 50; d.attraction_count = 2;
        d.recalculate_level();
        assert_eq!(d.level, 2);
        d.total_visitors = 200; d.attraction_count = 5;
        d.recalculate_level();
        assert_eq!(d.level, 3);
        d.total_visitors = 500; d.attraction_count = 10;
        d.recalculate_level();
        assert_eq!(d.level, 4);
        d.total_visitors = 1500; d.attraction_count = 20;
        d.recalculate_level();
        assert_eq!(d.level, 5);
    }

    #[test]
    fn test_happiness_scales_with_level() {
        let mut d = ParkDistrict::new(1, ParkType::CityPark, 10, 10);
        let h1 = d.happiness_bonus();
        d.level = 3;
        let h3 = d.happiness_bonus();
        d.level = 5;
        let h5 = d.happiness_bonus();
        assert!(h3 > h1);
        assert!(h5 > h3);
    }

    #[test]
    fn test_amusement_tourism_multiplier() {
        let mut cp = ParkDistrict::new(1, ParkType::CityPark, 10, 10);
        cp.level = 3;
        let mut ap = ParkDistrict::new(2, ParkType::AmusementPark, 10, 10);
        ap.level = 3;
        assert!(ap.tourism_score() > cp.tourism_score());
    }

    #[test]
    fn test_radius_increases_with_level() {
        let mut d = ParkDistrict::new(1, ParkType::CityPark, 10, 10);
        d.level = 1;
        let r1 = d.radius_cells();
        d.level = 5;
        assert!(d.radius_cells() > r1);
    }

    #[test]
    fn test_effects_grid_clear() {
        let mut effects = ParkDistrictEffects::default();
        let idx = ParkDistrictEffects::idx(10, 10);
        effects.happiness[idx] = 15.0;
        effects.land_value[idx] = 10.0;
        effects.clear();
        assert!(effects.happiness[idx].abs() < f32::EPSILON);
        assert!(effects.land_value[idx].abs() < f32::EPSILON);
    }
}
