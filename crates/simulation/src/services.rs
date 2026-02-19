use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CELL_SIZE;
use crate::grid::{CellType, WorldGrid};
use crate::utilities::{UtilitySource, UtilityType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    FireStation,
    PoliceStation,
    Hospital,
    ElementarySchool,
    HighSchool,
    University,
    Library,
    SmallPark,
    LargePark,
    Playground,
    Plaza,
    SportsField,
    Stadium,
    Landfill,
    RecyclingCenter,
    Incinerator,
    Cemetery,
    Crematorium,
    CityHall,
    Museum,
    Cathedral,
    TVStation,
    BusDepot,
    TrainStation,
    // New multi-tier variants
    FireHouse,
    FireHQ,
    PoliceKiosk,
    PoliceHQ,
    Prison,
    MedicalClinic,
    MedicalCenter,
    Kindergarten,
    SubwayStation,
    TramDepot,
    FerryPier,
    SmallAirstrip,
    RegionalAirport,
    InternationalAirport,
    TransferStation,
    CellTower,
    DataCenter,
    HomelessShelter,
    WelfareOffice,
    PostOffice,
    MailSortingCenter,
    HeatingBoiler,
    DistrictHeatingPlant,
    GeothermalPlant,
    WaterTreatmentPlant,
    WellPump,
}

impl ServiceType {
    pub fn name(self) -> &'static str {
        match self {
            ServiceType::FireStation => "Fire Station",
            ServiceType::FireHouse => "Fire House",
            ServiceType::FireHQ => "Fire HQ",
            ServiceType::PoliceStation => "Police Station",
            ServiceType::PoliceKiosk => "Police Kiosk",
            ServiceType::PoliceHQ => "Police HQ",
            ServiceType::Prison => "Prison",
            ServiceType::Hospital => "Hospital",
            ServiceType::MedicalClinic => "Medical Clinic",
            ServiceType::MedicalCenter => "Medical Center",
            ServiceType::ElementarySchool => "Elementary School",
            ServiceType::HighSchool => "High School",
            ServiceType::University => "University",
            ServiceType::Library => "Library",
            ServiceType::Kindergarten => "Kindergarten",
            ServiceType::SmallPark => "Small Park",
            ServiceType::LargePark => "Large Park",
            ServiceType::Playground => "Playground",
            ServiceType::Plaza => "Plaza",
            ServiceType::SportsField => "Sports Field",
            ServiceType::Stadium => "Stadium",
            ServiceType::Landfill => "Landfill",
            ServiceType::RecyclingCenter => "Recycling Center",
            ServiceType::Incinerator => "Incinerator",
            ServiceType::TransferStation => "Transfer Station",
            ServiceType::Cemetery => "Cemetery",
            ServiceType::Crematorium => "Crematorium",
            ServiceType::CityHall => "City Hall",
            ServiceType::Museum => "Museum",
            ServiceType::Cathedral => "Cathedral",
            ServiceType::TVStation => "TV Station",
            ServiceType::BusDepot => "Bus Depot",
            ServiceType::TrainStation => "Train Station",
            ServiceType::SubwayStation => "Subway Station",
            ServiceType::TramDepot => "Tram Depot",
            ServiceType::FerryPier => "Ferry Pier",
            ServiceType::SmallAirstrip => "Small Airstrip",
            ServiceType::RegionalAirport => "Regional Airport",
            ServiceType::InternationalAirport => "International Airport",
            ServiceType::CellTower => "Cell Tower",
            ServiceType::DataCenter => "Data Center",
            ServiceType::HomelessShelter => "Homeless Shelter",
            ServiceType::WelfareOffice => "Welfare Office",
            ServiceType::PostOffice => "Post Office",
            ServiceType::MailSortingCenter => "Mail Sorting Center",
            ServiceType::WaterTreatmentPlant => "Water Treatment Plant",
            ServiceType::WellPump => "Well Pump",
            ServiceType::HeatingBoiler => "Heating Boiler",
            ServiceType::DistrictHeatingPlant => "District Heating Plant",
            ServiceType::GeothermalPlant => "Geothermal Heating Plant",
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBuilding {
    pub service_type: ServiceType,
    pub grid_x: usize,
    pub grid_y: usize,
    pub radius: f32,
}

impl ServiceBuilding {
    pub fn coverage_radius(service_type: ServiceType) -> f32 {
        match service_type {
            ServiceType::FireStation => 20.0 * CELL_SIZE,
            ServiceType::FireHouse => 12.0 * CELL_SIZE,
            ServiceType::FireHQ => 35.0 * CELL_SIZE,
            ServiceType::PoliceStation => 20.0 * CELL_SIZE,
            ServiceType::PoliceKiosk => 10.0 * CELL_SIZE,
            ServiceType::PoliceHQ => 35.0 * CELL_SIZE,
            ServiceType::Prison => 0.0, // city-wide effect
            ServiceType::Hospital => 25.0 * CELL_SIZE,
            ServiceType::MedicalClinic => 12.0 * CELL_SIZE,
            ServiceType::MedicalCenter => 40.0 * CELL_SIZE,
            ServiceType::ElementarySchool => 15.0 * CELL_SIZE,
            ServiceType::HighSchool => 18.0 * CELL_SIZE,
            ServiceType::University => 30.0 * CELL_SIZE,
            ServiceType::Library => 12.0 * CELL_SIZE,
            ServiceType::Kindergarten => 10.0 * CELL_SIZE,
            ServiceType::SmallPark => 8.0 * CELL_SIZE,
            ServiceType::LargePark => 15.0 * CELL_SIZE,
            ServiceType::Playground => 10.0 * CELL_SIZE,
            ServiceType::Plaza => 10.0 * CELL_SIZE,
            ServiceType::SportsField => 12.0 * CELL_SIZE,
            ServiceType::Stadium => 25.0 * CELL_SIZE,
            ServiceType::Landfill => 20.0 * CELL_SIZE,
            ServiceType::RecyclingCenter => 25.0 * CELL_SIZE,
            ServiceType::Incinerator => 30.0 * CELL_SIZE,
            ServiceType::TransferStation => 20.0 * CELL_SIZE,
            ServiceType::Cemetery => 120.0,
            ServiceType::Crematorium => 80.0,
            ServiceType::CityHall => 40.0 * CELL_SIZE,
            ServiceType::Museum => 20.0 * CELL_SIZE,
            ServiceType::Cathedral => 25.0 * CELL_SIZE,
            ServiceType::TVStation => 35.0 * CELL_SIZE,
            ServiceType::BusDepot => 20.0 * CELL_SIZE,
            ServiceType::TrainStation => 30.0 * CELL_SIZE,
            ServiceType::SubwayStation => 25.0 * CELL_SIZE,
            ServiceType::TramDepot => 18.0 * CELL_SIZE,
            ServiceType::FerryPier => 15.0 * CELL_SIZE,
            ServiceType::SmallAirstrip => 40.0 * CELL_SIZE,
            ServiceType::RegionalAirport => 50.0 * CELL_SIZE,
            ServiceType::InternationalAirport => 60.0 * CELL_SIZE,
            ServiceType::CellTower => 15.0 * CELL_SIZE,
            ServiceType::DataCenter => 40.0 * CELL_SIZE,
            ServiceType::HomelessShelter => 15.0 * CELL_SIZE,
            ServiceType::WelfareOffice => 20.0 * CELL_SIZE,
            ServiceType::PostOffice => 12.0 * CELL_SIZE,
            ServiceType::MailSortingCenter => 30.0 * CELL_SIZE,
            ServiceType::WaterTreatmentPlant => 25.0 * CELL_SIZE,
            ServiceType::WellPump => 10.0 * CELL_SIZE,
            ServiceType::HeatingBoiler => 15.0 * CELL_SIZE,
            ServiceType::DistrictHeatingPlant => 40.0 * CELL_SIZE,
            ServiceType::GeothermalPlant => 60.0 * CELL_SIZE,
        }
    }

    pub fn cost(service_type: ServiceType) -> f64 {
        match service_type {
            ServiceType::FireStation => 500.0,
            ServiceType::FireHouse => 200.0,
            ServiceType::FireHQ => 1500.0,
            ServiceType::PoliceStation => 500.0,
            ServiceType::PoliceKiosk => 200.0,
            ServiceType::PoliceHQ => 1500.0,
            ServiceType::Prison => 2000.0,
            ServiceType::Hospital => 1000.0,
            ServiceType::MedicalClinic => 300.0,
            ServiceType::MedicalCenter => 3000.0,
            ServiceType::ElementarySchool => 750.0,
            ServiceType::HighSchool => 1000.0,
            ServiceType::University => 2000.0,
            ServiceType::Library => 500.0,
            ServiceType::Kindergarten => 400.0,
            ServiceType::SmallPark => 100.0,
            ServiceType::LargePark => 300.0,
            ServiceType::Playground => 200.0,
            ServiceType::Plaza => 150.0,
            ServiceType::SportsField => 400.0,
            ServiceType::Stadium => 2000.0,
            ServiceType::Landfill => 300.0,
            ServiceType::RecyclingCenter => 800.0,
            ServiceType::Incinerator => 1500.0,
            ServiceType::TransferStation => 500.0,
            ServiceType::Cemetery => 400.0,
            ServiceType::Crematorium => 600.0,
            ServiceType::CityHall => 5000.0,
            ServiceType::Museum => 3000.0,
            ServiceType::Cathedral => 4000.0,
            ServiceType::TVStation => 3500.0,
            ServiceType::BusDepot => 1000.0,
            ServiceType::TrainStation => 2000.0,
            ServiceType::SubwayStation => 3000.0,
            ServiceType::TramDepot => 1500.0,
            ServiceType::FerryPier => 800.0,
            ServiceType::SmallAirstrip => 5000.0,
            ServiceType::RegionalAirport => 10000.0,
            ServiceType::InternationalAirport => 15000.0,
            ServiceType::CellTower => 300.0,
            ServiceType::DataCenter => 2000.0,
            ServiceType::HomelessShelter => 400.0,
            ServiceType::WelfareOffice => 600.0,
            ServiceType::PostOffice => 300.0,
            ServiceType::MailSortingCenter => 1200.0,
            ServiceType::WaterTreatmentPlant => 800.0,
            ServiceType::WellPump => 200.0,
            ServiceType::HeatingBoiler => 400.0,
            ServiceType::DistrictHeatingPlant => 2000.0,
            ServiceType::GeothermalPlant => 5000.0,
        }
    }

    pub fn monthly_maintenance(service_type: ServiceType) -> f64 {
        match service_type {
            ServiceType::FireStation => 20.0,
            ServiceType::FireHouse => 8.0,
            ServiceType::FireHQ => 60.0,
            ServiceType::PoliceStation => 20.0,
            ServiceType::PoliceKiosk => 8.0,
            ServiceType::PoliceHQ => 60.0,
            ServiceType::Prison => 80.0,
            ServiceType::Hospital => 50.0,
            ServiceType::MedicalClinic => 15.0,
            ServiceType::MedicalCenter => 100.0,
            ServiceType::ElementarySchool => 15.0,
            ServiceType::HighSchool => 25.0,
            ServiceType::University => 40.0,
            ServiceType::Library => 10.0,
            ServiceType::Kindergarten => 12.0,
            ServiceType::SmallPark => 5.0,
            ServiceType::LargePark => 10.0,
            ServiceType::Playground => 5.0,
            ServiceType::Plaza => 5.0,
            ServiceType::SportsField => 10.0,
            ServiceType::Stadium => 30.0,
            ServiceType::Landfill => 15.0,
            ServiceType::RecyclingCenter => 20.0,
            ServiceType::Incinerator => 25.0,
            ServiceType::TransferStation => 12.0,
            ServiceType::Cemetery => 200.0,
            ServiceType::Crematorium => 150.0,
            ServiceType::CityHall => 30.0,
            ServiceType::Museum => 20.0,
            ServiceType::Cathedral => 15.0,
            ServiceType::TVStation => 25.0,
            ServiceType::BusDepot => 15.0,
            ServiceType::TrainStation => 25.0,
            ServiceType::SubwayStation => 40.0,
            ServiceType::TramDepot => 20.0,
            ServiceType::FerryPier => 10.0,
            ServiceType::SmallAirstrip => 60.0,
            ServiceType::RegionalAirport => 100.0,
            ServiceType::InternationalAirport => 150.0,
            ServiceType::CellTower => 8.0,
            ServiceType::DataCenter => 40.0,
            ServiceType::HomelessShelter => 15.0,
            ServiceType::WelfareOffice => 20.0,
            ServiceType::PostOffice => 10.0,
            ServiceType::MailSortingCenter => 35.0,
            ServiceType::WaterTreatmentPlant => 25.0,
            ServiceType::WellPump => 8.0,
            ServiceType::HeatingBoiler => 15.0,
            ServiceType::DistrictHeatingPlant => 50.0,
            ServiceType::GeothermalPlant => 80.0,
        }
    }

    pub fn is_park(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::SmallPark
                | ServiceType::LargePark
                | ServiceType::Playground
                | ServiceType::Plaza
                | ServiceType::SportsField
                | ServiceType::Stadium
        )
    }

    pub fn is_education(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::ElementarySchool
                | ServiceType::HighSchool
                | ServiceType::University
                | ServiceType::Library
                | ServiceType::Kindergarten
        )
    }

    pub fn is_garbage(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::Landfill
                | ServiceType::RecyclingCenter
                | ServiceType::Incinerator
                | ServiceType::TransferStation
        )
    }

    pub fn is_telecom(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::CellTower | ServiceType::DataCenter
        )
    }

    pub fn is_transport(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::BusDepot
                | ServiceType::TrainStation
                | ServiceType::SubwayStation
                | ServiceType::TramDepot
                | ServiceType::FerryPier
                | ServiceType::SmallAirstrip
                | ServiceType::RegionalAirport
                | ServiceType::InternationalAirport
        )
    }

    pub fn is_police(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::PoliceStation
                | ServiceType::PoliceKiosk
                | ServiceType::PoliceHQ
                | ServiceType::Prison
        )
    }

    pub fn is_fire(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ
        )
    }

    pub fn is_health(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter
        )
    }

    pub fn is_airport(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::SmallAirstrip
                | ServiceType::RegionalAirport
                | ServiceType::InternationalAirport
        )
    }

    pub fn is_postal(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::PostOffice | ServiceType::MailSortingCenter
        )
    }

    pub fn is_heating(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::HeatingBoiler
                | ServiceType::DistrictHeatingPlant
                | ServiceType::GeothermalPlant
        )
    }

    pub fn is_water_service(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::WaterTreatmentPlant | ServiceType::WellPump
        )
    }

    pub fn is_death_care(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::Cemetery | ServiceType::Crematorium
        )
    }

    pub fn is_welfare(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::WelfareOffice | ServiceType::HomelessShelter
        )
    }

    pub fn education_level(service_type: ServiceType) -> u8 {
        match service_type {
            ServiceType::Kindergarten => 1,
            ServiceType::ElementarySchool | ServiceType::Library => 1,
            ServiceType::HighSchool => 2,
            ServiceType::University => 3,
            _ => 0,
        }
    }

    /// Returns (width, height) footprint in grid cells
    pub fn footprint(service_type: ServiceType) -> (usize, usize) {
        match service_type {
            ServiceType::FireHQ
            | ServiceType::PoliceHQ
            | ServiceType::MedicalCenter
            | ServiceType::SmallAirstrip => (3, 3),
            ServiceType::RegionalAirport => (4, 3),
            ServiceType::Prison | ServiceType::InternationalAirport => (4, 4),
            ServiceType::SubwayStation
            | ServiceType::TramDepot
            | ServiceType::DataCenter
            | ServiceType::DistrictHeatingPlant
            | ServiceType::TransferStation => (2, 2),
            ServiceType::GeothermalPlant => (3, 3),
            _ => (1, 1),
        }
    }
}

pub fn place_service(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    service_type: ServiceType,
    gx: usize,
    gy: usize,
) -> bool {
    let (fw, fh) = ServiceBuilding::footprint(service_type);

    // Check all cells in footprint are valid
    for dy in 0..fh {
        for dx in 0..fw {
            let cx = gx + dx;
            let cy = gy + dy;
            if !grid.in_bounds(cx, cy) {
                return false;
            }
            let cell = grid.get(cx, cy);
            if cell.cell_type != CellType::Grass || cell.building_id.is_some() {
                return false;
            }
        }
    }

    let entity = commands
        .spawn(ServiceBuilding {
            service_type,
            grid_x: gx,
            grid_y: gy,
            radius: ServiceBuilding::coverage_radius(service_type),
        })
        .id();

    // Mark all cells in footprint
    for dy in 0..fh {
        for dx in 0..fw {
            grid.get_mut(gx + dx, gy + dy).building_id = Some(entity);
        }
    }
    true
}

pub fn place_utility_source(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    utility_type: UtilityType,
    gx: usize,
    gy: usize,
) -> bool {
    if !grid.in_bounds(gx, gy) {
        return false;
    }
    let cell = grid.get(gx, gy);
    if cell.cell_type != CellType::Grass || cell.building_id.is_some() {
        return false;
    }

    let range = match utility_type {
        UtilityType::PowerPlant => 30,
        UtilityType::SolarFarm => 25,
        UtilityType::WindTurbine => 20,
        UtilityType::WaterTower => 25,
        UtilityType::SewagePlant => 20,
        UtilityType::NuclearPlant => 50,
        UtilityType::Geothermal => 35,
        UtilityType::PumpingStation => 15,
        UtilityType::WaterTreatment => 35,
    };

    let entity = commands
        .spawn(UtilitySource {
            utility_type,
            grid_x: gx,
            grid_y: gy,
            range,
        })
        .id();

    grid.get_mut(gx, gy).building_id = Some(entity);
    true
}

pub fn utility_cost(utility_type: UtilityType) -> f64 {
    match utility_type {
        UtilityType::PowerPlant => 800.0,
        UtilityType::SolarFarm => 1200.0,
        UtilityType::WindTurbine => 600.0,
        UtilityType::WaterTower => 600.0,
        UtilityType::SewagePlant => 500.0,
        UtilityType::NuclearPlant => 5000.0,
        UtilityType::Geothermal => 3000.0,
        UtilityType::PumpingStation => 400.0,
        UtilityType::WaterTreatment => 1000.0,
    }
}
