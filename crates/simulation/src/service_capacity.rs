//! SERV-001: Service Capacity Limits
//!
//! Adds capacity limits to service buildings. Each service building has a maximum
//! capacity (e.g. hospital beds, school students) and tracks current usage based
//! on nearby residential population. When a service is over-capacity, its
//! effectiveness degrades gracefully rather than cutting off abruptly.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::TickCounter;

/// Component attached to service building entities to track capacity and usage.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapacity {
    /// Maximum number of people/units this service can handle.
    pub capacity: u32,
    /// Current usage count (residents within coverage radius).
    pub current_usage: u32,
}

impl ServiceCapacity {
    /// Create a new capacity component with the default capacity for the given
    /// service type.
    pub fn new(service_type: ServiceType) -> Self {
        Self {
            capacity: default_capacity(service_type),
            current_usage: 0,
        }
    }

    /// Utilization ratio: usage / capacity. Returns 0.0 if capacity is 0.
    pub fn utilization(&self) -> f32 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.current_usage as f32 / self.capacity as f32
    }

    /// Effectiveness multiplier (0.0 to 1.0).
    ///
    /// - At or below capacity: 1.0 (full effectiveness)
    /// - Over capacity: degrades smoothly via `effectiveness = 1 / utilization`
    ///   e.g. 200% utilization -> 0.5, 300% -> 0.33
    ///
    /// Minimum effectiveness is clamped at 0.1 (services never fully stop).
    pub fn effectiveness(&self) -> f32 {
        if self.capacity == 0 {
            return 0.1;
        }
        let util = self.utilization();
        if util <= 1.0 {
            1.0
        } else {
            (1.0 / util).max(0.1)
        }
    }

    /// Returns true if the service is over capacity.
    pub fn is_over_capacity(&self) -> bool {
        self.current_usage > self.capacity
    }
}

/// Returns the default capacity for a given service type.
pub fn default_capacity(service_type: ServiceType) -> u32 {
    match service_type {
        // Health services
        ServiceType::Hospital => 200,
        ServiceType::MedicalClinic => 50,
        ServiceType::MedicalCenter => 500,

        // Education
        ServiceType::Kindergarten => 100,
        ServiceType::ElementarySchool => 300,
        ServiceType::HighSchool => 600,
        ServiceType::University => 2000,
        ServiceType::Library => 150,

        // Fire services (population served capacity)
        ServiceType::FireStation => 500,
        ServiceType::FireHouse => 200,
        ServiceType::FireHQ => 1500,

        // Police services (population served capacity)
        ServiceType::PoliceStation => 500,
        ServiceType::PoliceKiosk => 200,
        ServiceType::PoliceHQ => 1500,
        ServiceType::Prison => 300,

        // Parks and recreation
        ServiceType::SmallPark => 200,
        ServiceType::LargePark => 500,
        ServiceType::Playground => 100,
        ServiceType::Plaza => 300,
        ServiceType::SportsField => 400,
        ServiceType::Stadium => 2000,

        // Waste management
        ServiceType::Landfill => 1000,
        ServiceType::RecyclingCenter => 800,
        ServiceType::Incinerator => 1200,
        ServiceType::TransferStation => 600,

        // Death care
        ServiceType::Cemetery => 500,
        ServiceType::Crematorium => 300,

        // Civic / cultural
        ServiceType::CityHall => 1000,
        ServiceType::Museum => 500,
        ServiceType::Cathedral => 800,
        ServiceType::TVStation => 1000,

        // Transport
        ServiceType::BusDepot => 800,
        ServiceType::TrainStation => 1500,
        ServiceType::SubwayStation => 2000,
        ServiceType::TramDepot => 1000,
        ServiceType::FerryPier => 500,
        ServiceType::SmallAirstrip => 200,
        ServiceType::RegionalAirport => 2000,
        ServiceType::InternationalAirport => 5000,

        // Telecom
        ServiceType::CellTower => 500,
        ServiceType::DataCenter => 2000,

        // Social services
        ServiceType::HomelessShelter => 100,
        ServiceType::WelfareOffice => 300,

        // Postal
        ServiceType::PostOffice => 500,
        ServiceType::MailSortingCenter => 2000,

        // Water services
        ServiceType::WaterTreatmentPlant => 1000,
        ServiceType::WellPump => 300,

        // Heating
        ServiceType::HeatingBoiler => 300,
        ServiceType::DistrictHeatingPlant => 1500,
        ServiceType::GeothermalPlant => 3000,
    }
}

/// System that attaches `ServiceCapacity` to any `ServiceBuilding` entity
/// that doesn't have one yet.
fn attach_capacity_to_new_services(
    mut commands: Commands,
    query: Query<(Entity, &ServiceBuilding), Without<ServiceCapacity>>,
) {
    for (entity, service) in &query {
        commands
            .entity(entity)
            .insert(ServiceCapacity::new(service.service_type));
    }
}

/// System that updates `current_usage` for each service building by counting
/// the residential population within its coverage radius.
///
/// Runs every 10 ticks to avoid per-frame overhead.
fn update_service_usage(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    mut services: Query<(&ServiceBuilding, &mut ServiceCapacity)>,
    ext_budget: Res<crate::budget::ExtendedBudget>,
) {
    if !tick.0.is_multiple_of(10) {
        return;
    }

    for (service, mut capacity) in &mut services {
        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let effective_radius = service.radius * budget_level;
        let radius_cells = (effective_radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = effective_radius * effective_radius;

        let mut usage: u32 = 0;

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let cell = grid.get(cx as usize, cy as usize);
                if let Some(building_entity) = cell.building_id {
                    if let Ok(building) = buildings.get(building_entity) {
                        usage = usage.saturating_add(building.occupants);
                    }
                }
            }
        }

        capacity.current_usage = usage;
    }
}

// ---------------------------------------------------------------------------
// Aggregate stats resource
// ---------------------------------------------------------------------------

/// Aggregate capacity data for a category of services.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct CategoryCapacity {
    pub category: String,
    pub total_capacity: u32,
    pub total_usage: u32,
}

impl CategoryCapacity {
    pub fn utilization(&self) -> f32 {
        if self.total_capacity == 0 {
            return 0.0;
        }
        self.total_usage as f32 / self.total_capacity as f32
    }
}

/// Resource that tracks aggregate service capacity data across all services.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(default)]
pub struct ServiceCapacityStats {
    /// Per-category aggregate data.
    pub categories: Vec<CategoryCapacity>,
}

/// System that aggregates capacity stats across all service buildings.
fn update_capacity_stats(
    tick: Res<TickCounter>,
    services: Query<(&ServiceBuilding, &ServiceCapacity)>,
    mut stats: ResMut<ServiceCapacityStats>,
) {
    if !tick.0.is_multiple_of(10) {
        return;
    }

    let mut health = (0u32, 0u32);
    let mut education = (0u32, 0u32);
    let mut fire = (0u32, 0u32);
    let mut police = (0u32, 0u32);
    let mut parks = (0u32, 0u32);
    let mut transport = (0u32, 0u32);

    for (service, capacity) in &services {
        let st = service.service_type;
        let target = if ServiceBuilding::is_health(st) {
            &mut health
        } else if ServiceBuilding::is_education(st) {
            &mut education
        } else if ServiceBuilding::is_fire(st) {
            &mut fire
        } else if ServiceBuilding::is_police(st) {
            &mut police
        } else if ServiceBuilding::is_park(st) {
            &mut parks
        } else if ServiceBuilding::is_transport(st) {
            &mut transport
        } else {
            continue;
        };
        target.0 = target.0.saturating_add(capacity.capacity);
        target.1 = target.1.saturating_add(capacity.current_usage);
    }

    stats.categories = vec![
        CategoryCapacity {
            category: "Health".to_string(),
            total_capacity: health.0,
            total_usage: health.1,
        },
        CategoryCapacity {
            category: "Education".to_string(),
            total_capacity: education.0,
            total_usage: education.1,
        },
        CategoryCapacity {
            category: "Fire".to_string(),
            total_capacity: fire.0,
            total_usage: fire.1,
        },
        CategoryCapacity {
            category: "Police".to_string(),
            total_capacity: police.0,
            total_usage: police.1,
        },
        CategoryCapacity {
            category: "Parks".to_string(),
            total_capacity: parks.0,
            total_usage: parks.1,
        },
        CategoryCapacity {
            category: "Transport".to_string(),
            total_capacity: transport.0,
            total_usage: transport.1,
        },
    ];
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for ServiceCapacityStats {
    const SAVE_KEY: &'static str = "service_capacity";

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

pub struct ServiceCapacityPlugin;

impl Plugin for ServiceCapacityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceCapacityStats>();

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<ServiceCapacityStats>();

        app.add_systems(
            FixedUpdate,
            (
                attach_capacity_to_new_services,
                update_service_usage,
                update_capacity_stats,
            )
                .chain()
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capacity_hospital() {
        assert_eq!(default_capacity(ServiceType::Hospital), 200);
    }

    #[test]
    fn test_default_capacity_elementary_school() {
        assert_eq!(default_capacity(ServiceType::ElementarySchool), 300);
    }

    #[test]
    fn test_utilization_zero_when_empty() {
        let cap = ServiceCapacity {
            capacity: 200,
            current_usage: 0,
        };
        assert!((cap.utilization()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_utilization_full() {
        let cap = ServiceCapacity {
            capacity: 200,
            current_usage: 200,
        };
        assert!((cap.utilization() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_utilization_over_capacity() {
        let cap = ServiceCapacity {
            capacity: 200,
            current_usage: 400,
        };
        assert!((cap.utilization() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effectiveness_at_capacity() {
        let cap = ServiceCapacity {
            capacity: 200,
            current_usage: 200,
        };
        assert!((cap.effectiveness() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effectiveness_under_capacity() {
        let cap = ServiceCapacity {
            capacity: 200,
            current_usage: 100,
        };
        assert!((cap.effectiveness() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effectiveness_double_capacity() {
        let cap = ServiceCapacity {
            capacity: 200,
            current_usage: 400,
        };
        assert!((cap.effectiveness() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effectiveness_minimum_clamp() {
        let cap = ServiceCapacity {
            capacity: 10,
            current_usage: 10000,
        };
        assert!((cap.effectiveness() - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effectiveness_zero_capacity() {
        let cap = ServiceCapacity {
            capacity: 0,
            current_usage: 0,
        };
        assert!((cap.effectiveness() - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_over_capacity() {
        let under = ServiceCapacity {
            capacity: 200,
            current_usage: 100,
        };
        assert!(!under.is_over_capacity());

        let at = ServiceCapacity {
            capacity: 200,
            current_usage: 200,
        };
        assert!(!at.is_over_capacity());

        let over = ServiceCapacity {
            capacity: 200,
            current_usage: 201,
        };
        assert!(over.is_over_capacity());
    }

    #[test]
    fn test_category_capacity_utilization() {
        let cat = CategoryCapacity {
            category: "Health".to_string(),
            total_capacity: 1000,
            total_usage: 500,
        };
        assert!((cat.utilization() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_capacity_zero() {
        let cat = CategoryCapacity {
            category: "Health".to_string(),
            total_capacity: 0,
            total_usage: 0,
        };
        assert!(cat.utilization().abs() < f32::EPSILON);
    }
}
