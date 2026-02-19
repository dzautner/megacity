use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::weather::Weather;
use crate::SlowTickTimer;

// =============================================================================
// Per-building water demand rates (gallons per day)
// =============================================================================

/// Residential: 150 gallons per capita per day (GPCD).
const RESIDENTIAL_GPCD: f32 = 150.0;

/// Commercial: 100 gallons per building occupant per day (GPB).
const COMMERCIAL_GPB: f32 = 100.0;

/// Industrial: 500 gallons per building occupant per day (GPB).
const INDUSTRIAL_GPB: f32 = 500.0;

/// Hospital: flat 300 gallons per day base.
const HOSPITAL_GPD: f32 = 300.0;

/// School: 25 gallons per student per day.
const SCHOOL_PER_STUDENT_GPD: f32 = 25.0;

/// Park: 500 gallons per cell per day (irrigation).
const PARK_PER_CELL_GPD: f32 = 500.0;

// =============================================================================
// Components and resources
// =============================================================================

/// Component attached to each building entity tracking its freshwater demand.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WaterDemand {
    /// Current water demand in gallons per day, including seasonal modifier.
    pub demand_gpd: f32,
    /// Whether this building is currently receiving water service.
    pub has_water_service: bool,
}

impl Default for WaterDemand {
    fn default() -> Self {
        Self {
            demand_gpd: 0.0,
            has_water_service: false,
        }
    }
}

/// City-wide water supply and demand tracking resource.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaterSupply {
    /// Total city-wide water demand in gallons per day.
    pub total_demand_gpd: f32,
    /// Total city-wide water supply capacity in gallons per day.
    /// Derived from water utility infrastructure (WaterTower, PumpingStation, etc.).
    pub total_supply_gpd: f32,
    /// Number of buildings currently served (have water coverage).
    pub buildings_served: u32,
    /// Number of buildings without water service.
    pub buildings_unserved: u32,
    /// Ratio of supply to demand (>1.0 means surplus).
    pub supply_ratio: f32,
}

// =============================================================================
// Per-building demand calculation
// =============================================================================

/// Compute the base water demand for a zoned building based on its type and occupancy.
fn base_demand_for_building(building: &Building) -> f32 {
    match building.zone_type {
        ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
            building.occupants as f32 * RESIDENTIAL_GPCD
        }
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::Office => {
            building.occupants as f32 * COMMERCIAL_GPB
        }
        ZoneType::Industrial => building.occupants as f32 * INDUSTRIAL_GPB,
        ZoneType::MixedUse => {
            // MixedUse: blend of residential and commercial water demand
            building.occupants as f32 * (RESIDENTIAL_GPCD + COMMERCIAL_GPB) * 0.5
        }
        ZoneType::None => 0.0,
    }
}

/// Compute the base water demand for a service building.
fn base_demand_for_service(service: &ServiceBuilding) -> f32 {
    match service.service_type {
        ServiceType::Hospital | ServiceType::MedicalCenter => HOSPITAL_GPD,
        ServiceType::MedicalClinic => HOSPITAL_GPD * 0.5,

        ServiceType::ElementarySchool
        | ServiceType::HighSchool
        | ServiceType::University
        | ServiceType::Kindergarten => {
            // Approximate student count from coverage radius.
            // Schools with larger radius serve more students.
            let estimated_students = service.radius / crate::config::CELL_SIZE;
            estimated_students * SCHOOL_PER_STUDENT_GPD
        }

        ServiceType::SmallPark | ServiceType::Playground | ServiceType::Plaza => {
            // 1 cell footprint
            PARK_PER_CELL_GPD
        }
        ServiceType::LargePark | ServiceType::SportsField => {
            // Larger parks need more irrigation
            let (fw, fh) = ServiceBuilding::footprint(service.service_type);
            let cells = (fw * fh).max(1) as f32;
            cells * PARK_PER_CELL_GPD
        }
        ServiceType::Stadium => {
            // Large water consumer
            PARK_PER_CELL_GPD * 4.0
        }

        // Fire stations need water reserves
        ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ => 200.0,

        // Water treatment uses water itself
        ServiceType::WaterTreatmentPlant => 100.0,

        // Other services have minimal water needs
        _ => 50.0,
    }
}

/// Supply capacity per water utility type (gallons per day).
fn supply_capacity_for_utility(utility_type: crate::utilities::UtilityType) -> f32 {
    use crate::utilities::UtilityType;
    match utility_type {
        UtilityType::WaterTower => 50_000.0,
        UtilityType::PumpingStation => 30_000.0,
        UtilityType::WaterTreatment => 80_000.0,
        UtilityType::SewagePlant => 20_000.0,
        _ => 0.0, // Power plants don't supply water
    }
}

// =============================================================================
// Systems
// =============================================================================

/// System: Calculate per-building water demand for zoned buildings.
/// Attaches/updates `WaterDemand` components. Runs on the slow tick.
pub fn calculate_building_water_demand(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    grid: Res<crate::grid::WorldGrid>,
    mut commands: Commands,
    buildings_without_demand: Query<(Entity, &Building), Without<WaterDemand>>,
    mut buildings_with_demand: Query<(Entity, &Building, &mut WaterDemand)>,
) {
    if !timer.should_run() {
        return;
    }

    let water_mult = weather.water_multiplier();

    // Attach WaterDemand to buildings that don't have it yet
    for (entity, building) in &buildings_without_demand {
        let base = base_demand_for_building(building);
        let has_water = grid.get(building.grid_x, building.grid_y).has_water;
        commands.entity(entity).insert(WaterDemand {
            demand_gpd: base * water_mult,
            has_water_service: has_water,
        });
    }

    // Update existing WaterDemand components
    for (_entity, building, mut demand) in &mut buildings_with_demand {
        let base = base_demand_for_building(building);
        demand.demand_gpd = base * water_mult;
        demand.has_water_service = grid.get(building.grid_x, building.grid_y).has_water;
    }
}

/// System: Aggregate city-wide water demand and supply totals.
/// Runs on the slow tick.
pub fn aggregate_water_supply(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    mut water_supply: ResMut<WaterSupply>,
    building_demands: Query<&WaterDemand>,
    services: Query<&ServiceBuilding>,
    utilities: Query<&crate::utilities::UtilitySource>,
) {
    if !timer.should_run() {
        return;
    }

    let water_mult = weather.water_multiplier();

    // Sum building demands
    let mut total_demand: f32 = 0.0;
    let mut served: u32 = 0;
    let mut unserved: u32 = 0;

    for demand in &building_demands {
        total_demand += demand.demand_gpd;
        if demand.has_water_service {
            served += 1;
        } else {
            unserved += 1;
        }
    }

    // Add service building demands (hospitals, schools, parks, etc.)
    for service in &services {
        let base = base_demand_for_service(service);
        total_demand += base * water_mult;
    }

    // Compute total supply from water utilities
    let mut total_supply: f32 = 0.0;
    for utility in &utilities {
        total_supply += supply_capacity_for_utility(utility.utility_type);
    }

    water_supply.total_demand_gpd = total_demand;
    water_supply.total_supply_gpd = total_supply;
    water_supply.buildings_served = served;
    water_supply.buildings_unserved = unserved;
    water_supply.supply_ratio = if total_demand > 0.0 {
        total_supply / total_demand
    } else {
        1.0
    };
}

/// Happiness penalty applied to citizens living in buildings without water service.
pub const NO_WATER_SERVICE_PENALTY: f32 = 8.0;

/// System: Citizens in buildings without water service suffer a happiness penalty.
/// Runs on the slow tick.
pub fn water_service_happiness_penalty(
    timer: Res<SlowTickTimer>,
    grid: Res<crate::grid::WorldGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !timer.should_run() {
        return;
    }

    for (mut details, home) in &mut citizens {
        let hx = home.grid_x;
        let hy = home.grid_y;
        if hx >= crate::config::GRID_WIDTH || hy >= crate::config::GRID_HEIGHT {
            continue;
        }

        let cell = grid.get(hx, hy);
        if !cell.has_water {
            // Reduce happiness for citizens without water service
            details.happiness = (details.happiness - NO_WATER_SERVICE_PENALTY).max(0.0);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::ZoneType;

    #[test]
    fn test_residential_demand() {
        let building = Building {
            zone_type: ZoneType::ResidentialLow,
            level: 1,
            grid_x: 10,
            grid_y: 10,
            capacity: 10,
            occupants: 5,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 5.0 * RESIDENTIAL_GPCD);
    }

    #[test]
    fn test_residential_high_demand() {
        let building = Building {
            zone_type: ZoneType::ResidentialHigh,
            level: 2,
            grid_x: 10,
            grid_y: 10,
            capacity: 200,
            occupants: 150,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 150.0 * RESIDENTIAL_GPCD);
    }

    #[test]
    fn test_commercial_demand() {
        let building = Building {
            zone_type: ZoneType::CommercialLow,
            level: 1,
            grid_x: 10,
            grid_y: 10,
            capacity: 8,
            occupants: 4,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 4.0 * COMMERCIAL_GPB);
    }

    #[test]
    fn test_industrial_demand() {
        let building = Building {
            zone_type: ZoneType::Industrial,
            level: 1,
            grid_x: 10,
            grid_y: 10,
            capacity: 20,
            occupants: 10,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 10.0 * INDUSTRIAL_GPB);
    }

    #[test]
    fn test_office_demand() {
        let building = Building {
            zone_type: ZoneType::Office,
            level: 1,
            grid_x: 10,
            grid_y: 10,
            capacity: 30,
            occupants: 20,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 20.0 * COMMERCIAL_GPB);
    }

    #[test]
    fn test_empty_building_zero_demand() {
        let building = Building {
            zone_type: ZoneType::ResidentialLow,
            level: 1,
            grid_x: 10,
            grid_y: 10,
            capacity: 10,
            occupants: 0,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 0.0);
    }

    #[test]
    fn test_none_zone_zero_demand() {
        let building = Building {
            zone_type: ZoneType::None,
            level: 0,
            grid_x: 10,
            grid_y: 10,
            capacity: 0,
            occupants: 0,
        };
        let demand = base_demand_for_building(&building);
        assert_eq!(demand, 0.0);
    }

    #[test]
    fn test_hospital_demand() {
        let service = ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 10,
            grid_y: 10,
            radius: ServiceBuilding::coverage_radius(ServiceType::Hospital),
        };
        let demand = base_demand_for_service(&service);
        assert_eq!(demand, HOSPITAL_GPD);
    }

    #[test]
    fn test_school_demand() {
        let service = ServiceBuilding {
            service_type: ServiceType::ElementarySchool,
            grid_x: 10,
            grid_y: 10,
            radius: ServiceBuilding::coverage_radius(ServiceType::ElementarySchool),
        };
        let demand = base_demand_for_service(&service);
        assert!(demand > 0.0, "school demand should be positive");
        // radius / CELL_SIZE * 25
        let expected = (service.radius / crate::config::CELL_SIZE) * SCHOOL_PER_STUDENT_GPD;
        assert_eq!(demand, expected);
    }

    #[test]
    fn test_park_demand() {
        let service = ServiceBuilding {
            service_type: ServiceType::SmallPark,
            grid_x: 10,
            grid_y: 10,
            radius: ServiceBuilding::coverage_radius(ServiceType::SmallPark),
        };
        let demand = base_demand_for_service(&service);
        assert_eq!(demand, PARK_PER_CELL_GPD);
    }

    #[test]
    fn test_seasonal_modifier_summer() {
        let mut weather = Weather::default();
        weather.season = crate::weather::Season::Summer;
        weather.temperature = 28.0;
        let mult = weather.water_multiplier();
        assert_eq!(mult, 1.3);
    }

    #[test]
    fn test_seasonal_modifier_winter() {
        let mut weather = Weather::default();
        weather.season = crate::weather::Season::Winter;
        weather.temperature = -2.0;
        let mult = weather.water_multiplier();
        assert_eq!(mult, 0.9);
    }

    #[test]
    fn test_seasonal_modifier_spring() {
        let weather = Weather::default();
        let mult = weather.water_multiplier();
        assert_eq!(mult, 1.0);
    }

    #[test]
    fn test_water_supply_default() {
        let supply = WaterSupply::default();
        assert_eq!(supply.total_demand_gpd, 0.0);
        assert_eq!(supply.total_supply_gpd, 0.0);
        assert_eq!(supply.buildings_served, 0);
        assert_eq!(supply.buildings_unserved, 0);
        assert_eq!(supply.supply_ratio, 0.0);
    }

    #[test]
    fn test_water_demand_default() {
        let demand = WaterDemand::default();
        assert_eq!(demand.demand_gpd, 0.0);
        assert!(!demand.has_water_service);
    }

    #[test]
    fn test_supply_capacity_water_types() {
        use crate::utilities::UtilityType;
        assert!(supply_capacity_for_utility(UtilityType::WaterTower) > 0.0);
        assert!(supply_capacity_for_utility(UtilityType::PumpingStation) > 0.0);
        assert!(supply_capacity_for_utility(UtilityType::WaterTreatment) > 0.0);
        assert!(supply_capacity_for_utility(UtilityType::SewagePlant) > 0.0);
    }

    #[test]
    fn test_supply_capacity_power_types_zero() {
        use crate::utilities::UtilityType;
        assert_eq!(supply_capacity_for_utility(UtilityType::PowerPlant), 0.0);
        assert_eq!(supply_capacity_for_utility(UtilityType::SolarFarm), 0.0);
        assert_eq!(supply_capacity_for_utility(UtilityType::WindTurbine), 0.0);
        assert_eq!(supply_capacity_for_utility(UtilityType::NuclearPlant), 0.0);
        assert_eq!(supply_capacity_for_utility(UtilityType::Geothermal), 0.0);
    }

    #[test]
    fn test_industrial_higher_than_commercial() {
        let industrial = Building {
            zone_type: ZoneType::Industrial,
            level: 1,
            grid_x: 0,
            grid_y: 0,
            capacity: 20,
            occupants: 10,
        };
        let commercial = Building {
            zone_type: ZoneType::CommercialLow,
            level: 1,
            grid_x: 0,
            grid_y: 0,
            capacity: 8,
            occupants: 10,
        };
        assert!(
            base_demand_for_building(&industrial) > base_demand_for_building(&commercial),
            "industrial water demand should exceed commercial for same occupancy"
        );
    }

    #[test]
    fn test_demand_scales_with_occupants() {
        let building_half = Building {
            zone_type: ZoneType::ResidentialHigh,
            level: 1,
            grid_x: 0,
            grid_y: 0,
            capacity: 50,
            occupants: 25,
        };
        let building_full = Building {
            zone_type: ZoneType::ResidentialHigh,
            level: 1,
            grid_x: 0,
            grid_y: 0,
            capacity: 50,
            occupants: 50,
        };
        let demand_half = base_demand_for_building(&building_half);
        let demand_full = base_demand_for_building(&building_full);
        assert_eq!(
            demand_full,
            demand_half * 2.0,
            "double occupancy should double demand"
        );
    }
}
