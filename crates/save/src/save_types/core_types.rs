// ---------------------------------------------------------------------------
// Core save structs: grid, roads, clock, budget, demand, buildings, citizens
// ---------------------------------------------------------------------------

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use bevy::prelude::Entity;
use simulation::citizen::{
    CitizenDetails, CitizenState, Family, Needs, PathCache, Personality, Position, Velocity,
};

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveGrid {
    pub cells: Vec<SaveCell>,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveCell {
    pub elevation: f32,
    pub cell_type: u8,
    pub zone: u8,
    pub road_type: u8,
    pub has_power: bool,
    pub has_water: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveRoadNetwork {
    pub road_positions: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveSegmentNode {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub connected_segments: Vec<u32>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveRoadSegment {
    pub id: u32,
    pub start_node: u32,
    pub end_node: u32,
    pub p0_x: f32,
    pub p0_y: f32,
    pub p1_x: f32,
    pub p1_y: f32,
    pub p2_x: f32,
    pub p2_y: f32,
    pub p3_x: f32,
    pub p3_y: f32,
    pub road_type: u8,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveRoadSegmentStore {
    pub nodes: Vec<SaveSegmentNode>,
    pub segments: Vec<SaveRoadSegment>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveClock {
    pub day: u32,
    pub hour: f32,
    pub speed: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBudget {
    pub treasury: f64,
    pub tax_rate: f32,
    pub last_collection_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
    /// Vacancy rates per zone type (added in v5).
    #[serde(default)]
    pub vacancy_residential: f32,
    #[serde(default)]
    pub vacancy_commercial: f32,
    #[serde(default)]
    pub vacancy_industrial: f32,
    #[serde(default)]
    pub vacancy_office: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBuilding {
    pub zone_type: u8,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
    // MixedUse fields (backward-compatible via serde defaults)
    #[serde(default)]
    pub commercial_capacity: u32,
    #[serde(default)]
    pub commercial_occupants: u32,
    #[serde(default)]
    pub residential_capacity: u32,
    #[serde(default)]
    pub residential_occupants: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveCitizen {
    pub age: u8,
    pub happiness: f32,
    pub education: u8,
    pub state: u8,
    pub home_x: usize,
    pub home_y: usize,
    pub work_x: usize,
    pub work_y: usize,
    // V3 fields: PathCache, Velocity, Position (backward-compatible via serde defaults)
    #[serde(default)]
    pub path_waypoints: Vec<(usize, usize)>,
    #[serde(default)]
    pub path_current_index: usize,
    #[serde(default)]
    pub velocity_x: f32,
    #[serde(default)]
    pub velocity_y: f32,
    #[serde(default)]
    pub pos_x: f32,
    #[serde(default)]
    pub pos_y: f32,
    // V4 fields: Full citizen fidelity (backward-compatible via serde defaults)
    /// Gender: 0 = Male, 1 = Female
    #[serde(default)]
    pub gender: u8,
    #[serde(default = "default_citizen_health")]
    pub health: f32,
    #[serde(default)]
    pub salary: f32,
    #[serde(default)]
    pub savings: f32,
    // Personality traits
    #[serde(default = "default_personality_trait")]
    pub ambition: f32,
    #[serde(default = "default_personality_trait")]
    pub sociability: f32,
    #[serde(default = "default_personality_trait")]
    pub materialism: f32,
    #[serde(default = "default_personality_trait")]
    pub resilience: f32,
    // Needs
    #[serde(default = "default_need_hunger")]
    pub need_hunger: f32,
    #[serde(default = "default_need_energy")]
    pub need_energy: f32,
    #[serde(default = "default_need_social")]
    pub need_social: f32,
    #[serde(default = "default_need_fun")]
    pub need_fun: f32,
    #[serde(default = "default_need_comfort")]
    pub need_comfort: f32,
    // Activity timer
    #[serde(default)]
    pub activity_timer: u32,
    // V32 fields: Family graph (backward-compatible via serde defaults)
    /// Index into the citizen array for partner, or u32::MAX for none.
    #[serde(default = "default_no_family_ref")]
    pub family_partner: u32,
    /// Indices into the citizen array for children.
    #[serde(default)]
    pub family_children: Vec<u32>,
    /// Index into the citizen array for parent, or u32::MAX for none.
    #[serde(default = "default_no_family_ref")]
    pub family_parent: u32,
}

fn default_citizen_health() -> f32 {
    80.0
}

fn default_personality_trait() -> f32 {
    0.5
}

fn default_need_hunger() -> f32 {
    80.0
}

fn default_need_energy() -> f32 {
    80.0
}

fn default_need_social() -> f32 {
    70.0
}

fn default_need_fun() -> f32 {
    70.0
}

fn default_need_comfort() -> f32 {
    60.0
}

fn default_no_family_ref() -> u32 {
    u32::MAX
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveUtilitySource {
    pub utility_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub range: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveServiceBuilding {
    pub service_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub radius_cells: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveWaterSource {
    pub source_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity_mgd: f32,
    pub quality: f32,
    pub operating_cost: f64,
    pub stored_gallons: f32,
    pub storage_capacity: f32,
}

/// Input data for serializing a single citizen, collected from ECS queries.
pub struct CitizenSaveInput {
    pub entity: Entity,
    pub details: CitizenDetails,
    pub state: CitizenState,
    pub home_x: usize,
    pub home_y: usize,
    pub work_x: usize,
    pub work_y: usize,
    pub path: PathCache,
    pub velocity: Velocity,
    pub position: Position,
    pub personality: Personality,
    pub needs: Needs,
    pub activity_timer: u32,
    pub family: Family,
}
