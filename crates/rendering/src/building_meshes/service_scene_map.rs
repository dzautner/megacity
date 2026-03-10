//! Mapping from `ServiceType` → GLB asset path for 3D scene models.
//!
//! Each service type is mapped to a GLB file in:
//! - `assets/models/buildings/services/{commercial,suburban,industrial}/`
//! - `assets/models/buildings/services/external/`
//!
//! The external models are open-license assets with source attribution in the
//! corresponding `SOURCES.md`.

use simulation::services::ServiceType;

/// Returns the asset path (relative to `assets/`) for the given service type.
///
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

        // -- Recreation --
        ServiceType::SmallPark => "models/buildings/services/external/small-park.glb",
        ServiceType::LargePark => "models/buildings/services/external/large-park.glb",
        ServiceType::Playground => "models/buildings/services/external/playground.glb",
        ServiceType::Plaza => "models/buildings/services/external/plaza.glb",
        ServiceType::SportsField => "models/buildings/services/external/sports-field.glb",
        ServiceType::Stadium => "models/buildings/services/suburban/stadium.glb",
        ServiceType::TVStation => "models/buildings/services/external/tv-station.glb",
        ServiceType::Cemetery => "models/buildings/services/external/cemetery.glb",

        // -- Transport extras --
        ServiceType::FerryPier => "models/buildings/services/external/ferry-pier.glb",
        ServiceType::SmallAirstrip => "models/buildings/services/external/small-airstrip.glb",
        ServiceType::RegionalAirport => "models/buildings/services/external/regional-airport.glb",
        ServiceType::InternationalAirport => {
            "models/buildings/services/external/international-airport.glb"
        }
    };
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::service_scene_path;
    use simulation::services::ServiceType;
    use std::path::PathBuf;

    fn asset_path(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../app/assets")
            .join(rel)
    }

    #[test]
    fn all_service_types_have_scene_mapping_and_file_exists() {
        let all = [
            ServiceType::FireStation,
            ServiceType::PoliceStation,
            ServiceType::Hospital,
            ServiceType::ElementarySchool,
            ServiceType::HighSchool,
            ServiceType::University,
            ServiceType::Library,
            ServiceType::SmallPark,
            ServiceType::LargePark,
            ServiceType::Playground,
            ServiceType::Plaza,
            ServiceType::SportsField,
            ServiceType::Stadium,
            ServiceType::Landfill,
            ServiceType::RecyclingCenter,
            ServiceType::Incinerator,
            ServiceType::Cemetery,
            ServiceType::Crematorium,
            ServiceType::CityHall,
            ServiceType::Museum,
            ServiceType::Cathedral,
            ServiceType::TVStation,
            ServiceType::BusDepot,
            ServiceType::TrainStation,
            ServiceType::FireHouse,
            ServiceType::FireHQ,
            ServiceType::PoliceKiosk,
            ServiceType::PoliceHQ,
            ServiceType::Prison,
            ServiceType::MedicalClinic,
            ServiceType::MedicalCenter,
            ServiceType::Kindergarten,
            ServiceType::SubwayStation,
            ServiceType::TramDepot,
            ServiceType::FerryPier,
            ServiceType::SmallAirstrip,
            ServiceType::RegionalAirport,
            ServiceType::InternationalAirport,
            ServiceType::TransferStation,
            ServiceType::CellTower,
            ServiceType::DataCenter,
            ServiceType::HomelessShelter,
            ServiceType::WelfareOffice,
            ServiceType::PostOffice,
            ServiceType::MailSortingCenter,
            ServiceType::HeatingBoiler,
            ServiceType::DistrictHeatingPlant,
            ServiceType::GeothermalPlant,
            ServiceType::WaterTreatmentPlant,
            ServiceType::WellPump,
            ServiceType::Daycare,
            ServiceType::Eldercare,
            ServiceType::CommunityCenter,
            ServiceType::SubstanceAbuseTreatmentCenter,
            ServiceType::SeniorCenter,
            ServiceType::YouthCenter,
        ];

        for ty in all {
            let rel = service_scene_path(ty).unwrap_or_else(|| panic!("missing scene path for {ty:?}"));
            let abs = asset_path(rel);
            assert!(
                abs.exists(),
                "service scene path does not exist for {ty:?}: {}",
                abs.display()
            );
        }
    }
}
