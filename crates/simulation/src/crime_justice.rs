//! SERV-005: Crime Types and Justice Pipeline
//!
//! Typed crime events, a justice pipeline (crime -> police response -> arrest
//! -> court -> jail), and per-district crime statistics. Crime frequency
//! depends on poverty, unemployment, and density. Police effectiveness and
//! jail capacity create feedback loops influencing deterrence.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::crime::CrimeGrid;
use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};
use crate::services::{ServiceBuilding, ServiceType};
use crate::Saveable;

// ---------------------------------------------------------------------------
// Crime types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum CrimeType {
    PettyTheft,
    Burglary,
    Assault,
    OrganizedCrime,
}

impl CrimeType {
    pub fn base_weight(self) -> f32 {
        match self {
            Self::PettyTheft => 0.50,
            Self::Burglary => 0.25,
            Self::Assault => 0.15,
            Self::OrganizedCrime => 0.10,
        }
    }
    pub fn poverty_factor(self) -> f32 {
        match self {
            Self::PettyTheft => 1.5,
            Self::Burglary => 1.2,
            Self::Assault => 0.8,
            Self::OrganizedCrime => 1.0,
        }
    }
    pub fn unemployment_factor(self) -> f32 {
        match self {
            Self::PettyTheft => 1.3,
            Self::Burglary => 1.4,
            Self::Assault => 0.6,
            Self::OrganizedCrime => 1.6,
        }
    }
    pub fn density_factor(self) -> f32 {
        match self {
            Self::PettyTheft => 1.0,
            Self::Burglary => 0.8,
            Self::Assault => 1.3,
            Self::OrganizedCrime => 1.5,
        }
    }
    pub fn jail_time(self) -> u32 {
        match self {
            Self::PettyTheft => 1,
            Self::Burglary => 3,
            Self::Assault => 5,
            Self::OrganizedCrime => 10,
        }
    }
}

const ALL_CRIME_TYPES: [CrimeType; 4] = [
    CrimeType::PettyTheft,
    CrimeType::Burglary,
    CrimeType::Assault,
    CrimeType::OrganizedCrime,
];

// ---------------------------------------------------------------------------
// Justice pipeline stages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct CrimeEvent {
    pub crime_type: CrimeType,
    pub district_x: usize,
    pub district_y: usize,
    pub stage: JusticeStage,
    pub stage_timer: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub enum JusticeStage {
    Reported,
    PoliceResponding,
    Arrested,
    InCourt,
    InJail,
    Resolved,
}

// ---------------------------------------------------------------------------
// Per-district crime statistics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct DistrictCrimeStats {
    pub petty_theft_count: u32,
    pub burglary_count: u32,
    pub assault_count: u32,
    pub organized_crime_count: u32,
    pub total_arrests: u32,
    pub total_convictions: u32,
    pub active_cases: u32,
}

impl DistrictCrimeStats {
    pub fn total_crimes(&self) -> u32 {
        self.petty_theft_count
            + self.burglary_count
            + self.assault_count
            + self.organized_crime_count
    }
    fn increment(&mut self, ct: CrimeType) {
        match ct {
            CrimeType::PettyTheft => self.petty_theft_count += 1,
            CrimeType::Burglary => self.burglary_count += 1,
            CrimeType::Assault => self.assault_count += 1,
            CrimeType::OrganizedCrime => self.organized_crime_count += 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Main resource
// ---------------------------------------------------------------------------

pub const PRISON_CAPACITY: u32 = 50;
const BASE_CRIMES_PER_TICK: f32 = 0.5;

#[derive(Resource, Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct CrimeJusticeState {
    pub events: Vec<CrimeEvent>,
    pub district_stats: Vec<DistrictCrimeStats>,
    pub jail_population: u32,
    pub jail_capacity: u32,
    pub police_effectiveness: f32,
    pub deterrence: f32,
    pub rng_state: u64,
}

impl Default for CrimeJusticeState {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            district_stats: vec![DistrictCrimeStats::default(); DISTRICTS_X * DISTRICTS_Y],
            jail_population: 0,
            jail_capacity: 0,
            police_effectiveness: 0.0,
            deterrence: 0.5,
            rng_state: 12345,
        }
    }
}

impl CrimeJusticeState {
    pub fn get_district_stats(&self, dx: usize, dy: usize) -> &DistrictCrimeStats {
        &self.district_stats[dy * DISTRICTS_X + dx]
    }
    pub fn get_district_stats_mut(&mut self, dx: usize, dy: usize) -> &mut DistrictCrimeStats {
        &mut self.district_stats[dy * DISTRICTS_X + dx]
    }
    fn next_random(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x % 10000) as f32 / 10000.0
    }
}

impl Saveable for CrimeJusticeState {
    const SAVE_KEY: &'static str = "crime_justice";
    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }
    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn update_police_effectiveness(
    slow_timer: Res<crate::SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    ext_budget: Res<crate::budget::ExtendedBudget>,
    mut state: ResMut<CrimeJusticeState>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let budget = ext_budget.service_budgets.police;
    let mut score: f32 = 0.0;
    let mut prison_count: u32 = 0;
    for service in &services {
        match service.service_type {
            ServiceType::PoliceKiosk => score += 5.0,
            ServiceType::PoliceStation => score += 15.0,
            ServiceType::PoliceHQ => score += 30.0,
            ServiceType::Prison => prison_count += 1,
            _ => {}
        }
    }
    score *= budget;
    state.police_effectiveness = (1.0 - 1.0 / (1.0 + score * 0.02)).clamp(0.0, 1.0);
    state.jail_capacity = prison_count * PRISON_CAPACITY;
    if state.jail_capacity == 0 {
        state.deterrence = 0.1;
    } else {
        let util = state.jail_population as f32 / state.jail_capacity as f32;
        state.deterrence = (1.0 - util * 0.5).clamp(0.1, 1.0);
    }
}

pub fn generate_crimes(
    slow_timer: Res<crate::SlowTickTimer>,
    districts: Res<Districts>,
    crime_grid: Res<CrimeGrid>,
    mut state: ResMut<CrimeJusticeState>,
) {
    if !slow_timer.should_run() {
        return;
    }
    for dy in 0..DISTRICTS_Y {
        for dx in 0..DISTRICTS_X {
            let dd = districts.get(dx, dy);
            if dd.population == 0 {
                continue;
            }
            let total_jobs = dd.commercial_jobs + dd.industrial_jobs + dd.office_jobs;
            let unemployment_rate = if total_jobs > 0 {
                1.0 - (dd.employed as f32 / dd.population as f32).clamp(0.0, 1.0)
            } else {
                0.8
            };
            let poverty_rate = (1.0 - dd.avg_happiness / 100.0).clamp(0.0, 1.0);
            let density_rate = if dd.residential_capacity > 0 {
                (dd.population as f32 / dd.residential_capacity as f32).clamp(0.0, 2.0) / 2.0
            } else {
                0.0
            };
            // Average crime level across district cells
            let (mut crime_sum, mut cell_count) = (0.0f32, 0u32);
            let (xs, ys) = (dx * DISTRICT_SIZE, dy * DISTRICT_SIZE);
            for cy in ys..(ys + DISTRICT_SIZE).min(crate::config::GRID_HEIGHT) {
                for cx in xs..(xs + DISTRICT_SIZE).min(crate::config::GRID_WIDTH) {
                    let c = crime_grid.get(cx, cy);
                    if c > 0 {
                        crime_sum += c as f32;
                        cell_count += 1;
                    }
                }
            }
            let grid_factor = if cell_count > 0 {
                (crime_sum / cell_count as f32 / 25.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let deterrence_mod = 1.0 - state.deterrence * 0.3;
            let police_mod = 1.0 - state.police_effectiveness * 0.4;

            for &ct in &ALL_CRIME_TYPES {
                let combined = (poverty_rate * ct.poverty_factor()
                    + unemployment_rate * ct.unemployment_factor()
                    + density_rate * ct.density_factor())
                    / 3.0;
                let prob = BASE_CRIMES_PER_TICK
                    * ct.base_weight()
                    * combined
                    * grid_factor
                    * deterrence_mod
                    * police_mod;
                if state.next_random() < prob {
                    state.events.push(CrimeEvent {
                        crime_type: ct,
                        district_x: dx,
                        district_y: dy,
                        stage: JusticeStage::Reported,
                        stage_timer: 0,
                    });
                    state.get_district_stats_mut(dx, dy).increment(ct);
                }
            }
        }
    }
}

pub fn advance_justice_pipeline(
    slow_timer: Res<crate::SlowTickTimer>,
    mut state: ResMut<CrimeJusticeState>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let effectiveness = state.police_effectiveness;
    let jail_cap = state.jail_capacity;
    let mut jail_pop = state.jail_population;
    for s in &mut state.district_stats {
        s.active_cases = 0;
    }
    let events = std::mem::take(&mut state.events);
    let mut kept = Vec::with_capacity(events.len());
    for mut ev in events {
        match ev.stage {
            JusticeStage::Reported => {
                ev.stage = JusticeStage::PoliceResponding;
                ev.stage_timer = 1;
                kept.push(ev);
            }
            JusticeStage::PoliceResponding => {
                if ev.stage_timer > 0 {
                    ev.stage_timer -= 1;
                    kept.push(ev);
                    continue;
                }
                let roll = {
                    let mut x = state.rng_state;
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    state.rng_state = x;
                    (x % 10000) as f32 / 10000.0
                };
                if roll < effectiveness * 0.7 + 0.1 {
                    ev.stage = JusticeStage::Arrested;
                    ev.stage_timer = 1;
                    let di = ev.district_y * DISTRICTS_X + ev.district_x;
                    if di < state.district_stats.len() {
                        state.district_stats[di].total_arrests += 1;
                    }
                    kept.push(ev);
                }
                // else criminal escapes
            }
            JusticeStage::Arrested => {
                if ev.stage_timer > 0 {
                    ev.stage_timer -= 1;
                    kept.push(ev);
                    continue;
                }
                ev.stage = JusticeStage::InCourt;
                ev.stage_timer = 1;
                kept.push(ev);
            }
            JusticeStage::InCourt => {
                if ev.stage_timer > 0 {
                    ev.stage_timer -= 1;
                    kept.push(ev);
                    continue;
                }
                if jail_pop < jail_cap {
                    ev.stage = JusticeStage::InJail;
                    ev.stage_timer = ev.crime_type.jail_time();
                    jail_pop += 1;
                    let di = ev.district_y * DISTRICTS_X + ev.district_x;
                    if di < state.district_stats.len() {
                        state.district_stats[di].total_convictions += 1;
                    }
                    kept.push(ev);
                }
                // else released (overcrowding)
            }
            JusticeStage::InJail => {
                if ev.stage_timer > 0 {
                    ev.stage_timer -= 1;
                    kept.push(ev);
                } else {
                    jail_pop = jail_pop.saturating_sub(1);
                }
            }
            JusticeStage::Resolved => {} // drop
        }
    }
    for ev in &kept {
        let di = ev.district_y * DISTRICTS_X + ev.district_x;
        if di < state.district_stats.len() {
            state.district_stats[di].active_cases += 1;
        }
    }
    state.events = kept;
    state.jail_population = jail_pop;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CrimeJusticePlugin;

impl Plugin for CrimeJusticePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrimeJusticeState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<CrimeJusticeState>();

        app.add_systems(
                FixedUpdate,
                (
                    update_police_effectiveness,
                    generate_crimes,
                    advance_justice_pipeline,
                )
                    .chain()
                    .after(crate::crime::update_crime)
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
    fn test_crime_type_weights_sum_to_one() {
        let sum: f32 = ALL_CRIME_TYPES.iter().map(|c| c.base_weight()).sum();
        assert!((sum - 1.0).abs() < 0.01);
    }
    #[test]
    fn test_crime_type_jail_times_ordered() {
        assert!(CrimeType::PettyTheft.jail_time() < CrimeType::Burglary.jail_time());
        assert!(CrimeType::Burglary.jail_time() < CrimeType::Assault.jail_time());
        assert!(CrimeType::Assault.jail_time() < CrimeType::OrganizedCrime.jail_time());
    }
    #[test]
    fn test_default_state() {
        let s = CrimeJusticeState::default();
        assert!(s.events.is_empty());
        assert_eq!(s.jail_population, 0);
        assert_eq!(s.district_stats.len(), DISTRICTS_X * DISTRICTS_Y);
    }
    #[test]
    fn test_district_crime_stats_increment() {
        let mut s = DistrictCrimeStats::default();
        s.increment(CrimeType::PettyTheft);
        s.increment(CrimeType::PettyTheft);
        s.increment(CrimeType::Assault);
        assert_eq!(s.petty_theft_count, 2);
        assert_eq!(s.assault_count, 1);
        assert_eq!(s.total_crimes(), 3);
    }
    #[test]
    fn test_rng_deterministic() {
        let mut a = CrimeJusticeState::default();
        let mut b = CrimeJusticeState::default();
        let sa: Vec<f32> = (0..10).map(|_| a.next_random()).collect();
        let sb: Vec<f32> = (0..10).map(|_| b.next_random()).collect();
        assert_eq!(sa, sb);
    }
    #[test]
    fn test_rng_values_in_range() {
        let mut s = CrimeJusticeState::default();
        for _ in 0..100 {
            let v = s.next_random();
            assert!(v >= 0.0 && v < 1.0, "value {v} out of range");
        }
    }
    #[test]
    fn test_saveable_roundtrip() {
        let mut s = CrimeJusticeState::default();
        s.jail_population = 42;
        s.police_effectiveness = 0.75;
        s.events.push(CrimeEvent {
            crime_type: CrimeType::Burglary,
            district_x: 3,
            district_y: 5,
            stage: JusticeStage::InJail,
            stage_timer: 2,
        });
        let bytes = s.save_to_bytes().unwrap();
        let r = CrimeJusticeState::load_from_bytes(&bytes);
        assert_eq!(r.jail_population, 42);
        assert!((r.police_effectiveness - 0.75).abs() < 0.001);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].crime_type, CrimeType::Burglary);
    }
}
