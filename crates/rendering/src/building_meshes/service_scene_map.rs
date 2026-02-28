//! Mapping from `ServiceType` → GLB asset path for 3D scene models.
//!
//! Each service type is mapped to a GLB file in
//! `assets/models/buildings/services/{commercial,suburban,industrial}/`.
//! The models originate from different Kenney asset packs, grouped by
//! visual style so each subdirectory has the correct shared colormap.

use simulation::services::ServiceType;

/// Returns the asset path (relative to `assets/`) for the given service type,
/// or `None` if no GLB model has been assigned yet (fallback to procedural mesh).
pub fn service_scene_path(service_type: ServiceType) -> Option<&'static str> {
    let path = match service_type {
        // -- Emergency (commercial-style buildings) --
        ServiceType::FireStation => "models/buildings/services/commercial/fire-station.glb",
        ServiceType::FireHouse => "models/buildings/services/commercial/fire-house.glb",
        ServiceType::FireHQ => "models/buildings/services/commercial/fire-hq.glb",
        ServiceType::PoliceStation => "models/buildings/services/commercial/police-station.glb",
        ServiceType::PoliceKiosk => "models/buildings/services/commercial/police-kiosk.glb",
        ServiceType::PoliceHQ => "models/buildings/services/commercial/police-hq.glb",
        ServiceType::Prison => "models/buildings/services/commercial/prison.glb",
        ServiceType::Hospital => "models/buildings/services/commercial/hospital.glb",
        ServiceType::MedicalClinic => "models/buildings/services/commercial/medical-clinic.glb",
        ServiceType::MedicalCenter => "models/buildings/services/commercial/medical-center.glb",

        // -- Education (suburban-style buildings) --
        ServiceType::ElementarySchool => {
            "models/buildings/services/suburban/elementary-school.glb"
        }
        ServiceType::HighSchool => "models/buildings/services/suburban/high-school.glb",
        ServiceType::Kindergarten => "models/buildings/services/suburban/kindergarten.glb",
        ServiceType::Library => "models/buildings/services/suburban/library.glb",
        ServiceType::University => "models/buildings/services/commercial/university.glb",

        // -- Civic / landmarks (commercial-style buildings) --
        ServiceType::CityHall => "models/buildings/services/commercial/city-hall.glb",
        ServiceType::Museum => "models/buildings/services/commercial/museum.glb",
        ServiceType::Cathedral => "models/buildings/services/commercial/cathedral.glb",
        ServiceType::CellTower => "models/buildings/services/commercial/cell-tower.glb",
        ServiceType::Crematorium => "models/buildings/services/commercial/crematorium.glb",

        // -- Transport / waste (industrial-style buildings) --
        ServiceType::TrainStation => "models/buildings/services/industrial/train-station.glb",
        ServiceType::BusDepot => "models/buildings/services/industrial/bus-depot.glb",
        ServiceType::SubwayStation => "models/buildings/services/industrial/subway-station.glb",
        ServiceType::TramDepot => "models/buildings/services/industrial/tram-depot.glb",
        ServiceType::DataCenter => "models/buildings/services/industrial/data-center.glb",
        ServiceType::TransferStation => {
            "models/buildings/services/industrial/transfer-station.glb"
        }
        ServiceType::Incinerator => "models/buildings/services/industrial/incinerator.glb",
        ServiceType::RecyclingCenter => {
            "models/buildings/services/industrial/recycling-center.glb"
        }
        ServiceType::Landfill => "models/buildings/services/industrial/landfill.glb",

        // -- Infrastructure (industrial-style buildings) --
        ServiceType::WaterTreatmentPlant => {
            "models/buildings/services/industrial/water-treatment-plant.glb"
        }
        ServiceType::HeatingBoiler => "models/buildings/services/industrial/heating-boiler.glb",
        ServiceType::DistrictHeatingPlant => {
            "models/buildings/services/industrial/district-heating-plant.glb"
        }
        ServiceType::GeothermalPlant => {
            "models/buildings/services/industrial/geothermal-plant.glb"
        }
        ServiceType::WellPump => "models/buildings/services/industrial/well-pump.glb",

        // -- Welfare / social (suburban-style buildings) --
        ServiceType::PostOffice => "models/buildings/services/suburban/post-office.glb",
        ServiceType::WelfareOffice => "models/buildings/services/suburban/welfare-office.glb",
        ServiceType::HomelessShelter => {
            "models/buildings/services/suburban/homeless-shelter.glb"
        }
        ServiceType::MailSortingCenter => {
            "models/buildings/services/suburban/mail-sorting-center.glb"
        }
        ServiceType::Daycare => "models/buildings/services/suburban/daycare.glb",
        ServiceType::Eldercare => "models/buildings/services/suburban/eldercare.glb",
        ServiceType::SeniorCenter => "models/buildings/services/suburban/senior-center.glb",
        ServiceType::YouthCenter => "models/buildings/services/suburban/youth-center.glb",
        ServiceType::CommunityCenter => {
            "models/buildings/services/suburban/community-center.glb"
        }
        ServiceType::SubstanceAbuseTreatmentCenter => {
            "models/buildings/services/suburban/substance-abuse-treatment-center.glb"
        }

        // -- Recreation (suburban-style buildings) --
        ServiceType::Stadium => "models/buildings/services/suburban/stadium.glb",

        // -- No GLB model yet — keep procedural mesh as fallback --
        ServiceType::SmallPark
        | ServiceType::LargePark
        | ServiceType::Playground
        | ServiceType::Plaza
        | ServiceType::SportsField
        | ServiceType::TVStation
        | ServiceType::Cemetery
        | ServiceType::FerryPier
        | ServiceType::SmallAirstrip
        | ServiceType::RegionalAirport
        | ServiceType::InternationalAirport => return None,
    };
    Some(path)
}
