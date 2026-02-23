use crate::services::ServiceBuilding;
use crate::services::ServiceType;

impl ServiceBuilding {
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

    pub fn is_care_service(service_type: ServiceType) -> bool {
        matches!(
            service_type,
            ServiceType::Daycare | ServiceType::Eldercare
        )
    }
}
