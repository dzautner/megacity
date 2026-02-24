//! SERV-008: Industrial Specializations (Forest/Farming/Oil/Ore)
//!
//! District-level industrial specialization system. Industrial zones within a
//! specialized district focus on a particular resource extraction chain, with
//! unique output values, pollution levels, and depletion behaviour.

use std::collections::HashMap;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::districts::DistrictMap;
use crate::grid::ZoneType;
use crate::natural_resources::{ResourceGrid, ResourceType};
use crate::pollution::PollutionGrid;
use crate::production::IndustryBuilding;
use crate::TickCounter;

// =============================================================================
// Types
// =============================================================================

/// The four industrial specialization types a district can adopt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum IndustrialSpecialization {
    Forest,
    Farming,
    Oil,
    Ore,
}

impl IndustrialSpecialization {
    pub const ALL: &'static [Self] = &[Self::Forest, Self::Farming, Self::Oil, Self::Ore];

    pub fn name(self) -> &'static str {
        match self {
            Self::Forest => "Forestry",
            Self::Farming => "Agriculture",
            Self::Oil => "Oil Industry",
            Self::Ore => "Mining",
        }
    }

    pub fn required_resource(self) -> ResourceType {
        match self {
            Self::Forest => ResourceType::Forest,
            Self::Farming => ResourceType::FertileLand,
            Self::Oil => ResourceType::Oil,
            Self::Ore => ResourceType::Ore,
        }
    }

    /// Base output value per worker per production tick.
    pub fn output_value_per_worker(self) -> f32 {
        match self {
            Self::Forest => 1.2,
            Self::Farming => 0.8,
            Self::Oil => 2.5,
            Self::Ore => 1.8,
        }
    }

    /// Pollution emitted per worker per production tick.
    pub fn pollution_per_worker(self) -> f32 {
        match self {
            Self::Forest => 0.2,
            Self::Farming => 0.4,
            Self::Oil => 2.0,
            Self::Ore => 1.5,
        }
    }

    /// Extraction rate multiplier for resource depletion.
    pub fn extraction_rate(self) -> f32 {
        match self {
            Self::Forest => 0.3,
            Self::Farming => 0.1,
            Self::Oil => 0.8,
            Self::Ore => 0.6,
        }
    }

    pub fn is_renewable(self) -> bool {
        matches!(self, Self::Forest | Self::Farming)
    }

    /// Minimum preferred education level for workers (0-3).
    pub fn preferred_education(self) -> u8 {
        match self {
            Self::Forest | Self::Farming => 0,
            Self::Ore => 1,
            Self::Oil => 2,
        }
    }
}

// =============================================================================
// State resource
// =============================================================================

/// Tracks district-level industrial specialization assignments and metrics.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct IndustrialSpecializationState {
    pub assignments: HashMap<usize, IndustrialSpecialization>,
    pub cumulative_output: HashMap<usize, f32>,
    pub cumulative_extracted: HashMap<usize, f32>,
    pub production_rates: HashMap<usize, f32>,
    /// 0.0 = depleted, 1.0 = full.
    pub resource_availability: HashMap<usize, f32>,
}

impl IndustrialSpecializationState {
    pub fn get_specialization(&self, district_idx: usize) -> Option<IndustrialSpecialization> {
        self.assignments.get(&district_idx).copied()
    }

    pub fn assign(&mut self, district_idx: usize, spec: IndustrialSpecialization) {
        self.assignments.insert(district_idx, spec);
        self.cumulative_output.entry(district_idx).or_insert(0.0);
        self.cumulative_extracted.entry(district_idx).or_insert(0.0);
        self.production_rates.entry(district_idx).or_insert(0.0);
        self.resource_availability.entry(district_idx).or_insert(1.0);
    }

    pub fn remove(&mut self, district_idx: usize) {
        self.assignments.remove(&district_idx);
        self.production_rates.remove(&district_idx);
    }
}

// =============================================================================
// Saveable
// =============================================================================

impl crate::Saveable for IndustrialSpecializationState {
    const SAVE_KEY: &'static str = "industrial_specializations";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.assignments.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

const PRODUCTION_INTERVAL: u64 = 20;
const RESOURCE_SEARCH_RADIUS: isize = 6;

/// Updates industrial specialization production, extraction, and pollution.
#[allow(clippy::too_many_arguments)]
pub fn update_industrial_specializations(
    tick: Res<TickCounter>,
    mut state: ResMut<IndustrialSpecializationState>,
    district_map: Res<DistrictMap>,
    buildings: Query<(&Building, Option<&IndustryBuilding>)>,
    mut resource_grid: ResMut<ResourceGrid>,
    mut pollution_grid: ResMut<PollutionGrid>,
) {
    if !tick.0.is_multiple_of(PRODUCTION_INTERVAL) {
        return;
    }

    for rate in state.production_rates.values_mut() {
        *rate = 0.0;
    }

    let assignments: Vec<(usize, IndustrialSpecialization)> =
        state.assignments.iter().map(|(&k, &v)| (k, v)).collect();

    for (district_idx, spec) in &assignments {
        let district_idx = *district_idx;
        let spec = *spec;
        if district_idx >= district_map.districts.len() {
            continue;
        }

        let required_resource = spec.required_resource();
        let mut district_output = 0.0_f32;
        let mut district_extraction = 0.0_f32;
        let mut resource_total = 0.0_f32;
        let mut resource_max = 0.0_f32;

        for (building, industry) in &buildings {
            if building.zone_type != ZoneType::Industrial {
                continue;
            }
            if district_map.get_district_index_at(building.grid_x, building.grid_y)
                != Some(district_idx)
            {
                continue;
            }
            let workers = building.occupants;
            if workers == 0 {
                continue;
            }

            let efficiency = industry
                .map(|ib| ib.efficiency)
                .unwrap_or(workers as f32 / building.capacity.max(1) as f32);

            let (extracted, avail, max_avail) = extract_specialized_resources(
                building.grid_x,
                building.grid_y,
                &mut resource_grid,
                required_resource,
                spec.extraction_rate() * workers as f32 * efficiency,
            );
            resource_total += avail;
            resource_max += max_avail;

            let output =
                spec.output_value_per_worker() * workers as f32 * efficiency * extracted.min(1.0);
            district_output += output;
            district_extraction += extracted * spec.extraction_rate() * workers as f32;

            apply_specialization_pollution(
                &mut pollution_grid,
                building.grid_x,
                building.grid_y,
                spec.pollution_per_worker() * workers as f32,
            );
        }

        *state.production_rates.entry(district_idx).or_insert(0.0) = district_output;
        *state.cumulative_output.entry(district_idx).or_insert(0.0) += district_output;
        *state.cumulative_extracted.entry(district_idx).or_insert(0.0) += district_extraction;

        let availability = if resource_max > 0.0 {
            (resource_total / resource_max).clamp(0.0, 1.0)
        } else {
            0.0
        };
        state.resource_availability.insert(district_idx, availability);
    }
}

/// Extract resources from deposits near (gx, gy) matching the target type.
/// Returns (extraction_factor, current_total, max_total).
fn extract_specialized_resources(
    gx: usize,
    gy: usize,
    resource_grid: &mut ResourceGrid,
    target_resource: ResourceType,
    demand: f32,
) -> (f32, f32, f32) {
    let mut total_available = 0.0_f32;
    let mut total_max = 0.0_f32;
    let mut total_extracted = 0.0_f32;
    let mut remaining_demand = demand;
    let grid_w = resource_grid.width as isize;
    let grid_h = resource_grid.height as isize;

    for dy in -RESOURCE_SEARCH_RADIUS..=RESOURCE_SEARCH_RADIUS {
        for dx in -RESOURCE_SEARCH_RADIUS..=RESOURCE_SEARCH_RADIUS {
            let nx = gx as isize + dx;
            let ny = gy as isize + dy;
            if nx < 0 || ny < 0 || nx >= grid_w || ny >= grid_h {
                continue;
            }
            let deposit = resource_grid.get_mut(nx as usize, ny as usize);
            if let Some(ref mut d) = deposit {
                if d.resource_type != target_resource {
                    continue;
                }
                total_max += d.max_amount as f32;
                total_available += d.amount as f32;
                if d.amount == 0 || remaining_demand <= 0.0 {
                    continue;
                }
                let extract = remaining_demand.min(d.amount as f32 * 0.01);
                total_extracted += extract;
                remaining_demand -= extract;

                if target_resource.is_renewable() {
                    let depletion = (extract * 0.3) as u32;
                    d.amount = d.amount.saturating_sub(depletion);
                    d.amount = (d.amount + 1).min(d.max_amount);
                } else {
                    let depletion = (extract * 0.8).max(1.0) as u32;
                    d.amount = d.amount.saturating_sub(depletion);
                }
            }
        }
    }

    let extraction_factor = if demand > 0.0 {
        (total_extracted / demand).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (extraction_factor, total_available, total_max)
}

/// Apply pollution from a specialized industrial building.
fn apply_specialization_pollution(
    pollution_grid: &mut PollutionGrid,
    gx: usize,
    gy: usize,
    amount: f32,
) {
    let radius: isize = 3;
    let grid_w = pollution_grid.width as isize;
    let grid_h = pollution_grid.height as isize;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = gx as isize + dx;
            let ny = gy as isize + dy;
            if nx < 0 || ny < 0 || nx >= grid_w || ny >= grid_h {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            let falloff = (1.0 - dist / (radius as f32 + 1.0)).max(0.0);
            let added = (amount * falloff) as u8;
            let current = pollution_grid.get(nx as usize, ny as usize);
            pollution_grid.set(nx as usize, ny as usize, current.saturating_add(added));
        }
    }
}

// =============================================================================
// Suggestion helper
// =============================================================================

/// Suggest the best specialization for a district based on natural resources.
pub fn suggest_specialization(
    district_idx: usize,
    district_map: &DistrictMap,
    resource_grid: &ResourceGrid,
) -> Option<IndustrialSpecialization> {
    if district_idx >= district_map.districts.len() {
        return None;
    }
    let district = &district_map.districts[district_idx];
    let mut counts: HashMap<ResourceType, u32> = HashMap::new();
    for &(cx, cy) in &district.cells {
        if let Some(deposit) = resource_grid.get(cx, cy) {
            if deposit.amount > 0 {
                *counts.entry(deposit.resource_type).or_insert(0) += 1;
            }
        }
    }
    counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .and_then(|(&resource, &count)| {
            if count < 3 {
                return None;
            }
            match resource {
                ResourceType::Forest => Some(IndustrialSpecialization::Forest),
                ResourceType::FertileLand => Some(IndustrialSpecialization::Farming),
                ResourceType::Oil => Some(IndustrialSpecialization::Oil),
                ResourceType::Ore => Some(IndustrialSpecialization::Ore),
            }
        })
}

// =============================================================================
// Plugin
// =============================================================================

pub struct IndustrialSpecializationPlugin;

impl Plugin for IndustrialSpecializationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IndustrialSpecializationState>()
            .add_systems(
                FixedUpdate,
                update_industrial_specializations
                    .after(crate::production::update_production_chains)
                    .in_set(crate::SimulationSet::Simulation),
            );
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<IndustrialSpecializationState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::Saveable;
    use super::*;

    #[test]
    fn test_required_resources() {
        assert_eq!(IndustrialSpecialization::Forest.required_resource(), ResourceType::Forest);
        assert_eq!(IndustrialSpecialization::Farming.required_resource(), ResourceType::FertileLand);
        assert_eq!(IndustrialSpecialization::Oil.required_resource(), ResourceType::Oil);
        assert_eq!(IndustrialSpecialization::Ore.required_resource(), ResourceType::Ore);
    }

    #[test]
    fn test_renewability() {
        assert!(IndustrialSpecialization::Forest.is_renewable());
        assert!(IndustrialSpecialization::Farming.is_renewable());
        assert!(!IndustrialSpecialization::Oil.is_renewable());
        assert!(!IndustrialSpecialization::Ore.is_renewable());
    }

    #[test]
    fn test_state_assign_and_remove() {
        let mut state = IndustrialSpecializationState::default();
        assert!(state.get_specialization(0).is_none());
        state.assign(0, IndustrialSpecialization::Forest);
        assert_eq!(state.get_specialization(0), Some(IndustrialSpecialization::Forest));
        state.remove(0);
        assert!(state.get_specialization(0).is_none());
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = IndustrialSpecializationState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = IndustrialSpecializationState::default();
        state.assign(2, IndustrialSpecialization::Oil);
        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = IndustrialSpecializationState::load_from_bytes(&bytes);
        assert_eq!(restored.get_specialization(2), Some(IndustrialSpecialization::Oil));
    }
}
