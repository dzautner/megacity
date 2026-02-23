//! SVC-020: Service Budget Framework (Realistic Proportions)
//!
//! Municipal budget with real-world proportions: Police 25-35%, Fire/EMS 10-15%,
//! Public works 15-25%, Education 15-30%, etc. Revenue from property tax, sales tax.
//! Per capita spending. Over/under-funding effects per department.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::budget::ExtendedBudget;
use crate::economy::CityBudget;
use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::Saveable;

// ---------------------------------------------------------------------------
// Department definitions with real-world budget proportions
// ---------------------------------------------------------------------------

/// Municipal departments that receive budget allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum Department {
    Police,
    FireEms,
    PublicWorks,
    Education,
    Healthcare,
    ParksRecreation,
    Sanitation,
    Transport,
}

impl Department {
    pub const ALL: [Department; 8] = [
        Department::Police,
        Department::FireEms,
        Department::PublicWorks,
        Department::Education,
        Department::Healthcare,
        Department::ParksRecreation,
        Department::Sanitation,
        Department::Transport,
    ];

    /// Recommended proportion of the total municipal budget (sums to ~1.0).
    /// Based on real-world US municipal averages.
    pub fn recommended_proportion(self) -> f32 {
        match self {
            Department::Police => 0.30,         // 25-35%
            Department::FireEms => 0.12,        // 10-15%
            Department::PublicWorks => 0.18,     // 15-25%
            Department::Education => 0.20,       // 15-30%
            Department::Healthcare => 0.08,      // 5-10%
            Department::ParksRecreation => 0.04, // 3-5%
            Department::Sanitation => 0.05,      // 3-7%
            Department::Transport => 0.03,       // 2-5%
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Department::Police => "Police",
            Department::FireEms => "Fire & EMS",
            Department::PublicWorks => "Public Works",
            Department::Education => "Education",
            Department::Healthcare => "Healthcare",
            Department::ParksRecreation => "Parks & Recreation",
            Department::Sanitation => "Sanitation",
            Department::Transport => "Transport",
        }
    }

    /// Map a ServiceType to its owning department.
    pub fn for_service(service_type: ServiceType) -> Option<Department> {
        match service_type {
            ServiceType::PoliceStation
            | ServiceType::PoliceKiosk
            | ServiceType::PoliceHQ
            | ServiceType::Prison => Some(Department::Police),

            ServiceType::FireStation
            | ServiceType::FireHouse
            | ServiceType::FireHQ => Some(Department::FireEms),

            ServiceType::Hospital
            | ServiceType::MedicalClinic
            | ServiceType::MedicalCenter => Some(Department::Healthcare),

            ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::University
            | ServiceType::Library
            | ServiceType::Kindergarten => Some(Department::Education),

            ServiceType::SmallPark
            | ServiceType::LargePark
            | ServiceType::Playground
            | ServiceType::Plaza
            | ServiceType::SportsField
            | ServiceType::Stadium => Some(Department::ParksRecreation),

            ServiceType::Landfill
            | ServiceType::RecyclingCenter
            | ServiceType::Incinerator
            | ServiceType::TransferStation
            | ServiceType::Cemetery
            | ServiceType::Crematorium => Some(Department::Sanitation),

            ServiceType::BusDepot
            | ServiceType::TrainStation
            | ServiceType::SubwayStation
            | ServiceType::TramDepot
            | ServiceType::FerryPier
            | ServiceType::SmallAirstrip
            | ServiceType::RegionalAirport
            | ServiceType::InternationalAirport => Some(Department::Transport),

            // CityHall, Museum, etc. don't belong to a specific department
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-department tracking
// ---------------------------------------------------------------------------

/// Budget and performance data for a single department.
#[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct DepartmentBudget {
    /// Actual spending this period (sum of service maintenance * budget level).
    pub actual_spending: f64,
    /// Recommended spending based on the recommended proportion of total revenue.
    pub recommended_spending: f64,
    /// Funding ratio: actual / recommended. 1.0 = fully funded.
    pub funding_ratio: f32,
    /// Per-capita spending (actual_spending / population).
    pub per_capita_spending: f64,
    /// Number of service buildings in this department.
    pub building_count: u32,
}

// ---------------------------------------------------------------------------
// ServiceBudgetState resource
// ---------------------------------------------------------------------------

/// Tracks per-department budget allocations, funding ratios, and effects.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct ServiceBudgetState {
    pub departments: [DepartmentBudget; 8],
    /// Total municipal revenue (from last tax collection).
    pub total_revenue: f64,
    /// Total spending across all departments.
    pub total_spending: f64,
    /// Overall per-capita spending.
    pub overall_per_capita: f64,
    /// Funding effect multipliers (0.0-2.0) applied to game systems.
    /// Underfunded departments produce penalties, overfunded produce bonuses.
    pub effects: DepartmentEffects,
}

impl ServiceBudgetState {
    fn dept_index(dept: Department) -> usize {
        match dept {
            Department::Police => 0,
            Department::FireEms => 1,
            Department::PublicWorks => 2,
            Department::Education => 3,
            Department::Healthcare => 4,
            Department::ParksRecreation => 5,
            Department::Sanitation => 6,
            Department::Transport => 7,
        }
    }

    pub fn department(&self, dept: Department) -> &DepartmentBudget {
        &self.departments[Self::dept_index(dept)]
    }

    pub fn department_mut(&mut self, dept: Department) -> &mut DepartmentBudget {
        &mut self.departments[Self::dept_index(dept)]
    }
}

// ---------------------------------------------------------------------------
// Funding effects
// ---------------------------------------------------------------------------

/// Multipliers derived from department funding ratios.
/// Values range from 0.0 (completely unfunded) to 2.0 (heavily overfunded).
/// 1.0 = adequately funded, no bonus or penalty.
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct DepartmentEffects {
    /// Police effectiveness multiplier: affects crime reduction.
    pub police_effectiveness: f32,
    /// Fire response multiplier: affects fire spread/damage.
    pub fire_response: f32,
    /// Road quality multiplier: affects road degradation rate.
    pub road_quality: f32,
    /// Education quality multiplier: affects education level gains.
    pub education_quality: f32,
    /// Healthcare quality multiplier: affects disease/health outcomes.
    pub healthcare_quality: f32,
    /// Park quality multiplier: affects happiness from parks.
    pub park_quality: f32,
    /// Sanitation efficiency: affects garbage processing.
    pub sanitation_efficiency: f32,
    /// Transit efficiency: affects transit service quality.
    pub transit_efficiency: f32,
}

impl Default for DepartmentEffects {
    fn default() -> Self {
        Self {
            police_effectiveness: 1.0,
            fire_response: 1.0,
            road_quality: 1.0,
            education_quality: 1.0,
            healthcare_quality: 1.0,
            park_quality: 1.0,
            sanitation_efficiency: 1.0,
            transit_efficiency: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl Saveable for ServiceBudgetState {
    const SAVE_KEY: &'static str = "service_budget";
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

/// Convert a funding ratio (0.0..inf) into an effect multiplier (0.0..2.0).
/// Uses a diminishing-returns curve:
/// - ratio 0.0 => multiplier 0.0 (no funding = no service)
/// - ratio 0.5 => multiplier ~0.65
/// - ratio 1.0 => multiplier 1.0 (baseline)
/// - ratio 1.5 => multiplier ~1.35
/// - ratio 2.0+ => capped at ~1.6 (diminishing returns)
fn funding_ratio_to_effect(ratio: f32) -> f32 {
    if ratio <= 0.0 {
        return 0.0;
    }
    // Diminishing returns: effect = 2.0 * ratio / (ratio + 1.0)
    // At ratio=1.0 this gives 1.0, at ratio=2.0 gives ~1.33, converges to 2.0
    let raw = 2.0 * ratio / (ratio + 1.0);
    raw.clamp(0.0, 2.0)
}

/// Main system: compute per-department spending, funding ratios, and effects.
#[allow(clippy::too_many_arguments)]
pub fn update_service_budget(
    slow_timer: Res<crate::SlowTickTimer>,
    budget: Res<CityBudget>,
    ext_budget: Res<ExtendedBudget>,
    stats: Res<CityStats>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<ServiceBudgetState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let population = stats.population.max(1) as f64;
    let total_revenue = budget.monthly_income;

    // Reset department data
    for dept_budget in state.departments.iter_mut() {
        dept_budget.actual_spending = 0.0;
        dept_budget.recommended_spending = 0.0;
        dept_budget.building_count = 0;
        dept_budget.per_capita_spending = 0.0;
    }

    // Accumulate actual spending per department from service buildings.
    // Actual spending = maintenance_cost * budget_level (from ServiceBudgets).
    for service in &services {
        let Some(dept) = Department::for_service(service.service_type) else {
            continue;
        };
        let base_maintenance = ServiceBuilding::monthly_maintenance(service.service_type);
        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let actual = base_maintenance * budget_level as f64;

        let dept_budget = state.department_mut(dept);
        dept_budget.actual_spending += actual;
        dept_budget.building_count += 1;
    }

    // Compute recommended spending and funding ratios
    let mut total_spending = 0.0;
    for dept in Department::ALL {
        let recommended = total_revenue * dept.recommended_proportion() as f64;
        let dept_budget = state.department_mut(dept);
        dept_budget.recommended_spending = recommended;

        total_spending += dept_budget.actual_spending;
        dept_budget.per_capita_spending = dept_budget.actual_spending / population;

        dept_budget.funding_ratio = if recommended > 0.0 {
            (dept_budget.actual_spending / recommended) as f32
        } else if dept_budget.actual_spending > 0.0 {
            // Revenue is zero but we're still spending (from treasury)
            1.0
        } else {
            0.0
        };
    }

    // Update summary fields
    state.total_revenue = total_revenue;
    state.total_spending = total_spending;
    state.overall_per_capita = total_spending / population;

    // Extract funding ratios before mutating effects (avoids borrow conflict)
    let police_ratio = state.department(Department::Police).funding_ratio;
    let fire_ratio = state.department(Department::FireEms).funding_ratio;
    let works_ratio = state.department(Department::PublicWorks).funding_ratio;
    let edu_ratio = state.department(Department::Education).funding_ratio;
    let health_ratio = state.department(Department::Healthcare).funding_ratio;
    let parks_ratio = state.department(Department::ParksRecreation).funding_ratio;
    let sanit_ratio = state.department(Department::Sanitation).funding_ratio;
    let trans_ratio = state.department(Department::Transport).funding_ratio;

    // Compute effect multipliers from funding ratios
    state.effects.police_effectiveness = funding_ratio_to_effect(police_ratio);
    state.effects.fire_response = funding_ratio_to_effect(fire_ratio);
    state.effects.road_quality = funding_ratio_to_effect(works_ratio);
    state.effects.education_quality = funding_ratio_to_effect(edu_ratio);
    state.effects.healthcare_quality = funding_ratio_to_effect(health_ratio);
    state.effects.park_quality = funding_ratio_to_effect(parks_ratio);
    state.effects.sanitation_efficiency = funding_ratio_to_effect(sanit_ratio);
    state.effects.transit_efficiency = funding_ratio_to_effect(trans_ratio);
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ServiceBudgetPlugin;

impl Plugin for ServiceBudgetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceBudgetState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ServiceBudgetState>();

        app.add_systems(
            FixedUpdate,
            update_service_budget
                .after(crate::economy::collect_taxes)
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
    fn test_recommended_proportions_sum_to_one() {
        let sum: f32 = Department::ALL
            .iter()
            .map(|d| d.recommended_proportion())
            .sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Department proportions should sum to ~1.0, got {sum}"
        );
    }

    #[test]
    fn test_funding_ratio_to_effect_baseline() {
        let effect = funding_ratio_to_effect(1.0);
        assert!(
            (effect - 1.0).abs() < 0.01,
            "funding_ratio 1.0 should give effect ~1.0, got {effect}"
        );
    }

    #[test]
    fn test_funding_ratio_to_effect_zero() {
        let effect = funding_ratio_to_effect(0.0);
        assert!(
            effect.abs() < 0.01,
            "funding_ratio 0.0 should give effect ~0.0, got {effect}"
        );
    }

    #[test]
    fn test_funding_ratio_to_effect_overfunded() {
        let effect = funding_ratio_to_effect(2.0);
        assert!(
            effect > 1.0,
            "funding_ratio 2.0 should give effect > 1.0, got {effect}"
        );
        assert!(
            effect < 2.0,
            "funding_ratio 2.0 should give effect < 2.0 (diminishing returns), got {effect}"
        );
    }

    #[test]
    fn test_funding_ratio_to_effect_underfunded() {
        let effect = funding_ratio_to_effect(0.5);
        assert!(
            effect > 0.0 && effect < 1.0,
            "funding_ratio 0.5 should give effect between 0 and 1, got {effect}"
        );
    }

    #[test]
    fn test_department_for_service_police() {
        assert_eq!(
            Department::for_service(ServiceType::PoliceStation),
            Some(Department::Police)
        );
        assert_eq!(
            Department::for_service(ServiceType::PoliceKiosk),
            Some(Department::Police)
        );
        assert_eq!(
            Department::for_service(ServiceType::PoliceHQ),
            Some(Department::Police)
        );
    }

    #[test]
    fn test_department_for_service_fire() {
        assert_eq!(
            Department::for_service(ServiceType::FireStation),
            Some(Department::FireEms)
        );
    }

    #[test]
    fn test_department_for_service_education() {
        assert_eq!(
            Department::for_service(ServiceType::University),
            Some(Department::Education)
        );
    }

    #[test]
    fn test_department_for_service_none() {
        assert_eq!(
            Department::for_service(ServiceType::CityHall),
            None
        );
    }

    #[test]
    fn test_department_index_roundtrip() {
        let state = ServiceBudgetState::default();
        for dept in Department::ALL {
            let db = state.department(dept);
            assert!(
                db.funding_ratio.abs() < f32::EPSILON,
                "Default funding ratio should be 0.0"
            );
        }
    }

    #[test]
    fn test_default_effects_are_baseline() {
        let effects = DepartmentEffects::default();
        assert!((effects.police_effectiveness - 1.0).abs() < f32::EPSILON);
        assert!((effects.fire_response - 1.0).abs() < f32::EPSILON);
        assert!((effects.road_quality - 1.0).abs() < f32::EPSILON);
    }
}
