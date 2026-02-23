use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CELL_SIZE;
use crate::services::ServiceType;

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
            ServiceType::Daycare => 20.0 * CELL_SIZE,
            ServiceType::Eldercare => 15.0 * CELL_SIZE,
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
            ServiceType::Daycare => 500.0,
            ServiceType::Eldercare => 600.0,
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
            ServiceType::Daycare => 15.0,
            ServiceType::Eldercare => 20.0,
        }
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
