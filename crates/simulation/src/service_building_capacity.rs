//! SVC-002: Service Building Capacity Limits
//!
//! Adds per-building staffing requirements and capacity scaling. Each service
//! building has a staff requirement based on its type and tier. Staff are drawn
//! from the employed citizen pool. Unstaffed buildings provide no service.
//! Partially staffed buildings have reduced effective capacity.
//!
//! This module works alongside `service_capacity.rs` (SERV-001) which tracks
//! `ServiceCapacity` per building. Here we add `ServiceBuildingStaffing` and
//! a system that scales the effective capacity based on staffing level.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails};
use crate::service_capacity::ServiceCapacity;
use crate::services::{ServiceBuilding, ServiceType};
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Per-building tier capacity values (from research doc tables)
// ---------------------------------------------------------------------------

/// Returns the tier-specific unit capacity for a service type:
/// - Hospital beds: Clinic=50, Hospital=200, MedicalCenter=500
/// - School students: Elementary=300, HighSchool=1500, University=5000
/// - Fire trucks: FireHouse=2, FireStation=5, FireHQ=10
/// - Police officers: Kiosk=10, Station=30, HQ=100
pub fn tier_capacity(st: ServiceType) -> u32 {
    use ServiceType::*;
    match st {
        MedicalClinic => 50,     Hospital => 200,        MedicalCenter => 500,
        Kindergarten => 150,     ElementarySchool => 300, HighSchool => 1500,
        University => 5000,      Library => 200,
        FireHouse => 2,          FireStation => 5,        FireHQ => 10,
        PoliceKiosk => 10,       PoliceStation => 30,     PoliceHQ => 100,
        Prison => 50,
        SmallPark => 200,        LargePark => 500,        Playground => 100,
        Plaza => 300,            SportsField => 400,      Stadium => 2000,
        Landfill => 1000,        RecyclingCenter => 800,  Incinerator => 1200,
        TransferStation => 600,  Cemetery => 500,         Crematorium => 300,
        CityHall => 1000,        Museum => 500,           Cathedral => 800,
        TVStation => 1000,       BusDepot => 800,         TrainStation => 1500,
        SubwayStation => 2000,   TramDepot => 1000,       FerryPier => 500,
        SmallAirstrip => 200,    RegionalAirport => 2000, InternationalAirport => 5000,
        CellTower => 500,        DataCenter => 2000,
        HomelessShelter => 100,  WelfareOffice => 300,
        PostOffice => 500,       MailSortingCenter => 2000,
        WaterTreatmentPlant => 1000, WellPump => 300,
        HeatingBoiler => 300,    DistrictHeatingPlant => 1500, GeothermalPlant => 3000,
        Daycare => 200,          Eldercare => 150,
        CommunityCenter => 300,  SubstanceAbuseTreatmentCenter => 100,
        SeniorCenter => 200,     YouthCenter => 250,
    }
}

/// Returns the number of staff required for full-capacity operation.
pub fn staff_required(st: ServiceType) -> u32 {
    use ServiceType::*;
    match st {
        MedicalClinic => 10,     Hospital => 40,          MedicalCenter => 100,
        Kindergarten => 8,       ElementarySchool => 20,  HighSchool => 60,
        University => 200,       Library => 5,
        FireHouse => 6,          FireStation => 15,       FireHQ => 40,
        PoliceKiosk => 5,        PoliceStation => 20,     PoliceHQ => 80,
        Prison => 30,
        SmallPark => 1,          LargePark => 3,          Playground => 1,
        Plaza => 2,              SportsField => 3,        Stadium => 20,
        Landfill => 10,          RecyclingCenter => 15,   Incinerator => 12,
        TransferStation => 8,    Cemetery => 5,           Crematorium => 4,
        CityHall => 30,          Museum => 10,            Cathedral => 5,
        TVStation => 15,         BusDepot => 20,          TrainStation => 25,
        SubwayStation => 15,     TramDepot => 12,         FerryPier => 8,
        SmallAirstrip => 10,     RegionalAirport => 50,   InternationalAirport => 150,
        CellTower => 2,          DataCenter => 20,
        HomelessShelter => 5,    WelfareOffice => 10,
        PostOffice => 8,         MailSortingCenter => 25,
        WaterTreatmentPlant => 15, WellPump => 3,
        HeatingBoiler => 4,      DistrictHeatingPlant => 12, GeothermalPlant => 20,
        Daycare => 10,           Eldercare => 8,
        CommunityCenter => 8,   SubstanceAbuseTreatmentCenter => 12,
        SeniorCenter => 6,      YouthCenter => 6,
    }
}

// ---------------------------------------------------------------------------
// ServiceBuildingStaffing component
// ---------------------------------------------------------------------------

/// Tracks staffing requirements and assignments for a service building.
/// Effective capacity is scaled by the staffing ratio.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBuildingStaffing {
    /// Number of staff needed for full-capacity operation.
    pub staff_required: u32,
    /// Number of staff currently assigned from the employed citizen pool.
    pub staff_assigned: u32,
    /// The base (max) capacity of this building before staffing adjustments.
    pub max_capacity: u32,
}

impl ServiceBuildingStaffing {
    pub fn new(service_type: ServiceType) -> Self {
        Self {
            staff_required: staff_required(service_type),
            staff_assigned: 0,
            max_capacity: tier_capacity(service_type),
        }
    }

    /// Staffing ratio: staff_assigned / staff_required. Clamped to [0, 1].
    pub fn staffing_ratio(&self) -> f32 {
        if self.staff_required == 0 {
            return 1.0;
        }
        (self.staff_assigned as f32 / self.staff_required as f32).clamp(0.0, 1.0)
    }

    /// Effective capacity after staffing adjustment.
    /// If unstaffed (staff_assigned == 0), returns 0.
    pub fn effective_capacity(&self) -> u32 {
        if self.staff_assigned == 0 {
            return 0;
        }
        (self.max_capacity as f32 * self.staffing_ratio()) as u32
    }

    /// Returns true if the building has zero staff assigned.
    pub fn is_unstaffed(&self) -> bool {
        self.staff_assigned == 0
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Attach `ServiceBuildingStaffing` to service buildings that lack one.
fn attach_staffing(
    mut commands: Commands,
    query: Query<(Entity, &ServiceBuilding), Without<ServiceBuildingStaffing>>,
) {
    for (entity, service) in &query {
        commands
            .entity(entity)
            .insert(ServiceBuildingStaffing::new(service.service_type));
    }
}

/// Assign staff to service buildings from the employed citizen pool.
///
/// Strategy: count total employed citizens, then distribute proportionally
/// to each building's `staff_required`. This avoids circular dependency
/// with the employment system.
///
/// Runs every 20 ticks to avoid per-frame overhead.
fn assign_staff(
    tick: Res<TickCounter>,
    citizens: Query<&CitizenDetails, With<Citizen>>,
    mut staffing_query: Query<&mut ServiceBuildingStaffing>,
) {
    if !tick.0.is_multiple_of(20) {
        return;
    }

    let total_employed: u32 = citizens.iter().filter(|c| c.salary > 0.0).count() as u32;
    let total_required: u32 = staffing_query.iter().map(|s| s.staff_required).sum();

    if total_required == 0 {
        return;
    }

    for mut staffing in &mut staffing_query {
        if staffing.staff_required == 0 {
            staffing.staff_assigned = 0;
            continue;
        }
        let share =
            total_employed as f64 * (staffing.staff_required as f64 / total_required as f64);
        staffing.staff_assigned = (share as u32).min(staffing.staff_required);
    }
}

/// Update `ServiceCapacity` based on staffing levels.
/// Unstaffed buildings get capacity 0; partially staffed scale proportionally.
fn apply_staffing_to_capacity(
    tick: Res<TickCounter>,
    mut query: Query<(&ServiceBuildingStaffing, &mut ServiceCapacity)>,
) {
    if !tick.0.is_multiple_of(20) {
        return;
    }

    for (staffing, mut capacity) in &mut query {
        capacity.capacity = staffing.effective_capacity();
    }
}

// ---------------------------------------------------------------------------
// Aggregate resource for save/load and UI
// ---------------------------------------------------------------------------

/// Per-service-category staffing summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct CategoryStaffing {
    pub category: String,
    pub total_staff_required: u32,
    pub total_staff_assigned: u32,
    pub total_max_capacity: u32,
    pub total_effective_capacity: u32,
}

impl CategoryStaffing {
    pub fn staffing_ratio(&self) -> f32 {
        if self.total_staff_required == 0 {
            return 1.0;
        }
        (self.total_staff_assigned as f32 / self.total_staff_required as f32).clamp(0.0, 1.0)
    }
}

/// Aggregate staffing and capacity data across all service buildings.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(default)]
pub struct ServiceBuildingCapacityState {
    pub categories: Vec<CategoryStaffing>,
    pub total_employed_as_staff: u32,
}

fn update_staffing_stats(
    tick: Res<TickCounter>,
    query: Query<(&ServiceBuilding, &ServiceBuildingStaffing)>,
    mut state: ResMut<ServiceBuildingCapacityState>,
) {
    if !tick.0.is_multiple_of(20) {
        return;
    }

    let mut health = CategoryStaffing { category: "Health".into(), ..Default::default() };
    let mut education = CategoryStaffing { category: "Education".into(), ..Default::default() };
    let mut fire = CategoryStaffing { category: "Fire".into(), ..Default::default() };
    let mut police = CategoryStaffing { category: "Police".into(), ..Default::default() };
    let mut total_assigned = 0u32;

    for (service, staffing) in &query {
        let st = service.service_type;
        let target = if ServiceBuilding::is_health(st) {
            &mut health
        } else if ServiceBuilding::is_education(st) {
            &mut education
        } else if ServiceBuilding::is_fire(st) {
            &mut fire
        } else if ServiceBuilding::is_police(st) {
            &mut police
        } else {
            total_assigned = total_assigned.saturating_add(staffing.staff_assigned);
            continue;
        };
        target.total_staff_required += staffing.staff_required;
        target.total_staff_assigned += staffing.staff_assigned;
        target.total_max_capacity += staffing.max_capacity;
        target.total_effective_capacity += staffing.effective_capacity();
        total_assigned = total_assigned.saturating_add(staffing.staff_assigned);
    }

    state.categories = vec![health, education, fire, police];
    state.total_employed_as_staff = total_assigned;
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for ServiceBuildingCapacityState {
    const SAVE_KEY: &'static str = "service_building_capacity";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.categories.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ServiceBuildingCapacityPlugin;

impl Plugin for ServiceBuildingCapacityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceBuildingCapacityState>();

        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<ServiceBuildingCapacityState>();

        app.add_systems(
            FixedUpdate,
            (
                attach_staffing,
                assign_staff,
                apply_staffing_to_capacity,
                update_staffing_stats,
            )
                .chain()
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
    fn test_tier_capacity_hospital_tiers() {
        assert_eq!(tier_capacity(ServiceType::MedicalClinic), 50);
        assert_eq!(tier_capacity(ServiceType::Hospital), 200);
        assert_eq!(tier_capacity(ServiceType::MedicalCenter), 500);
    }

    #[test]
    fn test_tier_capacity_school_tiers() {
        assert_eq!(tier_capacity(ServiceType::ElementarySchool), 300);
        assert_eq!(tier_capacity(ServiceType::HighSchool), 1500);
        assert_eq!(tier_capacity(ServiceType::University), 5000);
    }

    #[test]
    fn test_tier_capacity_fire_tiers() {
        assert_eq!(tier_capacity(ServiceType::FireHouse), 2);
        assert_eq!(tier_capacity(ServiceType::FireStation), 5);
        assert_eq!(tier_capacity(ServiceType::FireHQ), 10);
    }

    #[test]
    fn test_tier_capacity_police_tiers() {
        assert_eq!(tier_capacity(ServiceType::PoliceKiosk), 10);
        assert_eq!(tier_capacity(ServiceType::PoliceStation), 30);
        assert_eq!(tier_capacity(ServiceType::PoliceHQ), 100);
    }

    #[test]
    fn test_staffing_ratio_fully_staffed() {
        let s = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 40, max_capacity: 200,
        };
        assert!((s.staffing_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_staffing_ratio_half_staffed() {
        let s = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 20, max_capacity: 200,
        };
        assert!((s.staffing_ratio() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_staffing_ratio_zero_required() {
        let s = ServiceBuildingStaffing {
            staff_required: 0, staff_assigned: 0, max_capacity: 200,
        };
        assert!((s.staffing_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_capacity_fully_staffed() {
        let s = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 40, max_capacity: 200,
        };
        assert_eq!(s.effective_capacity(), 200);
    }

    #[test]
    fn test_effective_capacity_half_staffed() {
        let s = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 20, max_capacity: 200,
        };
        assert_eq!(s.effective_capacity(), 100);
    }

    #[test]
    fn test_effective_capacity_unstaffed() {
        let s = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 0, max_capacity: 200,
        };
        assert_eq!(s.effective_capacity(), 0);
    }

    #[test]
    fn test_is_unstaffed() {
        let unstaffed = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 0, max_capacity: 200,
        };
        assert!(unstaffed.is_unstaffed());
        let staffed = ServiceBuildingStaffing {
            staff_required: 40, staff_assigned: 10, max_capacity: 200,
        };
        assert!(!staffed.is_unstaffed());
    }

    #[test]
    fn test_overcrowding_penalty_at_capacity() {
        let cap = ServiceCapacity { capacity: 200, current_usage: 200 };
        assert!((cap.effectiveness() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_overcrowding_penalty_double_demand() {
        let cap = ServiceCapacity { capacity: 200, current_usage: 400 };
        assert!((cap.effectiveness() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_staffing_ratio() {
        let cat = CategoryStaffing {
            category: "Health".into(),
            total_staff_required: 100,
            total_staff_assigned: 75,
            total_max_capacity: 500,
            total_effective_capacity: 375,
        };
        assert!((cat.staffing_ratio() - 0.75).abs() < f32::EPSILON);
    }
}
