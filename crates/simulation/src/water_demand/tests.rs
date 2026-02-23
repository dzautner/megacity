use super::calculations::{
    base_demand_for_building, base_demand_for_service, supply_capacity_for_utility,
};
use super::types::*;
use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};

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
    use crate::weather::Weather;
    let mut weather = Weather::default();
    weather.season = crate::weather::Season::Summer;
    weather.temperature = 28.0;
    let mult = weather.water_multiplier();
    assert_eq!(mult, 1.3);
}

#[test]
fn test_seasonal_modifier_winter() {
    use crate::weather::Weather;
    let mut weather = Weather::default();
    weather.season = crate::weather::Season::Winter;
    weather.temperature = -2.0;
    let mult = weather.water_multiplier();
    assert_eq!(mult, 0.9);
}

#[test]
fn test_seasonal_modifier_spring() {
    use crate::weather::Weather;
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
