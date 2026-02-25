//! Color query functions used by UI, minimap, and overlay systems.

use bevy::prelude::*;
use simulation::grid::ZoneType;
use simulation::services::ServiceType;
use simulation::utilities::UtilityType;

pub fn zone_base_color(zone: ZoneType, _level: u8) -> Color {
    match zone {
        ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
            Color::srgb(0.40, 0.72, 0.35)
        }
        ZoneType::CommercialLow | ZoneType::CommercialHigh => Color::srgb(0.40, 0.52, 0.78),
        ZoneType::Industrial => Color::srgb(0.72, 0.65, 0.25),
        ZoneType::Office => Color::srgb(0.55, 0.50, 0.68),
        ZoneType::MixedUse => Color::srgb(0.60, 0.55, 0.35),
        ZoneType::None => Color::srgb(0.7, 0.7, 0.7),
    }
}

pub fn service_base_color(service_type: ServiceType) -> Color {
    match service_type {
        ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ => {
            Color::srgb(1.0, 0.50, 0.50)
        }
        ServiceType::PoliceStation | ServiceType::PoliceKiosk | ServiceType::PoliceHQ => {
            Color::srgb(0.41, 0.53, 0.66)
        }
        ServiceType::Prison => Color::srgb(0.45, 0.45, 0.45),
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => {
            Color::srgb(0.94, 0.69, 0.75)
        }
        ServiceType::ElementarySchool
        | ServiceType::HighSchool
        | ServiceType::Library
        | ServiceType::Kindergarten => Color::srgb(0.94, 0.78, 0.63),
        ServiceType::University => Color::srgb(0.47, 0.47, 0.72),
        ServiceType::SmallPark
        | ServiceType::LargePark
        | ServiceType::Playground
        | ServiceType::SportsField => Color::srgb(0.44, 0.82, 0.44),
        ServiceType::Plaza | ServiceType::Stadium => Color::srgb(0.55, 0.75, 0.55),
        ServiceType::Landfill
        | ServiceType::RecyclingCenter
        | ServiceType::Incinerator
        | ServiceType::TransferStation => Color::srgb(0.60, 0.55, 0.45),
        ServiceType::Cemetery | ServiceType::Crematorium => Color::srgb(0.55, 0.55, 0.55),
        ServiceType::CityHall
        | ServiceType::Museum
        | ServiceType::Cathedral
        | ServiceType::TVStation => Color::srgb(0.91, 0.82, 0.38),
        ServiceType::BusDepot
        | ServiceType::TrainStation
        | ServiceType::SubwayStation
        | ServiceType::TramDepot
        | ServiceType::FerryPier => Color::srgb(0.50, 0.60, 0.70),
        ServiceType::SmallAirstrip
        | ServiceType::RegionalAirport
        | ServiceType::InternationalAirport => Color::srgb(0.65, 0.65, 0.70),
        ServiceType::CellTower => Color::srgb(0.6, 0.6, 0.6),
        ServiceType::DataCenter => Color::srgb(0.35, 0.40, 0.50),
        ServiceType::HomelessShelter => Color::srgb(0.65, 0.55, 0.45),
        ServiceType::WelfareOffice => Color::srgb(0.45, 0.60, 0.55),
        ServiceType::PostOffice => Color::srgb(0.72, 0.55, 0.38),
        ServiceType::MailSortingCenter => Color::srgb(0.55, 0.50, 0.45),
        ServiceType::HeatingBoiler => Color::srgb(0.85, 0.40, 0.20),
        ServiceType::DistrictHeatingPlant => Color::srgb(0.75, 0.35, 0.15),
        ServiceType::GeothermalPlant => Color::srgb(0.55, 0.40, 0.25),
        ServiceType::WaterTreatmentPlant => Color::srgb(0.30, 0.55, 0.70),
        ServiceType::WellPump => Color::srgb(0.40, 0.60, 0.55),
        ServiceType::Daycare => Color::srgb(0.75, 0.65, 0.85),
        ServiceType::Eldercare => Color::srgb(0.60, 0.75, 0.65),
        ServiceType::CommunityCenter => Color::srgb(0.55, 0.70, 0.80),
        ServiceType::SubstanceAbuseTreatmentCenter => Color::srgb(0.50, 0.60, 0.65),
        ServiceType::SeniorCenter => Color::srgb(0.65, 0.75, 0.60),
        ServiceType::YouthCenter => Color::srgb(0.60, 0.65, 0.85),
    }
}

pub fn utility_base_color(utility_type: UtilityType) -> Color {
    match utility_type {
        UtilityType::PowerPlant => Color::srgb(0.9, 0.5, 0.1),
        UtilityType::SolarFarm => Color::srgb(0.95, 0.85, 0.2),
        UtilityType::WindTurbine => Color::srgb(0.7, 0.85, 0.95),
        UtilityType::WaterTower => Color::srgb(0.2, 0.7, 0.85),
        UtilityType::SewagePlant => Color::srgb(0.45, 0.55, 0.40),
        UtilityType::NuclearPlant => Color::srgb(0.7, 0.7, 0.75),
        UtilityType::Geothermal => Color::srgb(0.65, 0.45, 0.30),
        UtilityType::PumpingStation => Color::srgb(0.3, 0.6, 0.8),
        UtilityType::WaterTreatment => Color::srgb(0.25, 0.55, 0.75),
        UtilityType::HydroDam => Color::srgb(0.15, 0.45, 0.70),
    }
}
