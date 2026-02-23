//! Service category and other service group definitions.

use rendering::overlay::OverlayMode;
use simulation::services::{ServiceBuilding, ServiceType};

// =============================================================================
// Service categories
// =============================================================================

/// High-level service categories shown in the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceCategory {
    Health,
    Education,
    Police,
    Fire,
    Parks,
    Entertainment,
    Telecom,
    Transport,
}

impl ServiceCategory {
    /// All categories in display order.
    pub const ALL: [ServiceCategory; 8] = [
        ServiceCategory::Health,
        ServiceCategory::Education,
        ServiceCategory::Police,
        ServiceCategory::Fire,
        ServiceCategory::Parks,
        ServiceCategory::Entertainment,
        ServiceCategory::Telecom,
        ServiceCategory::Transport,
    ];

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::Education => "Education",
            Self::Police => "Police",
            Self::Fire => "Fire",
            Self::Parks => "Parks",
            Self::Entertainment => "Entertainment",
            Self::Telecom => "Telecom",
            Self::Transport => "Transport",
        }
    }

    /// The coverage bitmask corresponding to this category.
    pub fn coverage_bit(self) -> u8 {
        use simulation::happiness::{
            COVERAGE_EDUCATION, COVERAGE_ENTERTAINMENT, COVERAGE_FIRE, COVERAGE_HEALTH,
            COVERAGE_PARK, COVERAGE_POLICE, COVERAGE_TELECOM, COVERAGE_TRANSPORT,
        };
        match self {
            Self::Health => COVERAGE_HEALTH,
            Self::Education => COVERAGE_EDUCATION,
            Self::Police => COVERAGE_POLICE,
            Self::Fire => COVERAGE_FIRE,
            Self::Parks => COVERAGE_PARK,
            Self::Entertainment => COVERAGE_ENTERTAINMENT,
            Self::Telecom => COVERAGE_TELECOM,
            Self::Transport => COVERAGE_TRANSPORT,
        }
    }

    /// The overlay mode activated when clicking this category.
    pub fn overlay_mode(self) -> Option<OverlayMode> {
        match self {
            Self::Education => Some(OverlayMode::Education),
            Self::Police => Some(OverlayMode::Pollution), // closest available
            Self::Fire => Some(OverlayMode::Power),       // closest available
            Self::Transport => Some(OverlayMode::Traffic),
            Self::Parks => Some(OverlayMode::LandValue),
            _ => None,
        }
    }

    /// Returns true if the given `ServiceType` belongs to this category.
    pub fn matches_service(self, st: ServiceType) -> bool {
        match self {
            Self::Health => ServiceBuilding::is_health(st),
            Self::Education => ServiceBuilding::is_education(st),
            Self::Police => ServiceBuilding::is_police(st),
            Self::Fire => ServiceBuilding::is_fire(st),
            Self::Parks => ServiceBuilding::is_park(st),
            Self::Entertainment => matches!(
                st,
                ServiceType::Stadium
                    | ServiceType::Plaza
                    | ServiceType::SportsField
                    | ServiceType::Museum
                    | ServiceType::Cathedral
                    | ServiceType::TVStation
            ),
            Self::Telecom => ServiceBuilding::is_telecom(st),
            Self::Transport => ServiceBuilding::is_transport(st),
        }
    }

    /// Returns the list of service types that belong to this category.
    pub fn service_types(self) -> &'static [ServiceType] {
        match self {
            Self::Health => &[
                ServiceType::MedicalClinic,
                ServiceType::Hospital,
                ServiceType::MedicalCenter,
            ],
            Self::Education => &[
                ServiceType::Kindergarten,
                ServiceType::ElementarySchool,
                ServiceType::HighSchool,
                ServiceType::University,
                ServiceType::Library,
            ],
            Self::Police => &[
                ServiceType::PoliceKiosk,
                ServiceType::PoliceStation,
                ServiceType::PoliceHQ,
                ServiceType::Prison,
            ],
            Self::Fire => &[
                ServiceType::FireHouse,
                ServiceType::FireStation,
                ServiceType::FireHQ,
            ],
            Self::Parks => &[
                ServiceType::SmallPark,
                ServiceType::LargePark,
                ServiceType::Playground,
                ServiceType::SportsField,
                ServiceType::Stadium,
            ],
            Self::Entertainment => &[
                ServiceType::Plaza,
                ServiceType::Museum,
                ServiceType::Cathedral,
                ServiceType::TVStation,
            ],
            Self::Telecom => &[ServiceType::CellTower, ServiceType::DataCenter],
            Self::Transport => &[
                ServiceType::BusDepot,
                ServiceType::TrainStation,
                ServiceType::SubwayStation,
                ServiceType::TramDepot,
                ServiceType::FerryPier,
                ServiceType::SmallAirstrip,
                ServiceType::RegionalAirport,
                ServiceType::InternationalAirport,
            ],
        }
    }
}

/// Service types that do not belong to any coverage-tracked category.
/// These are shown in the "Other Services" section.
pub const OTHER_SERVICE_TYPES: &[ServiceType] = &[
    ServiceType::Landfill,
    ServiceType::RecyclingCenter,
    ServiceType::Incinerator,
    ServiceType::TransferStation,
    ServiceType::Cemetery,
    ServiceType::Crematorium,
    ServiceType::CityHall,
    ServiceType::HomelessShelter,
    ServiceType::WelfareOffice,
    ServiceType::PostOffice,
    ServiceType::MailSortingCenter,
    ServiceType::WaterTreatmentPlant,
    ServiceType::WellPump,
    ServiceType::HeatingBoiler,
    ServiceType::DistrictHeatingPlant,
    ServiceType::GeothermalPlant,
];

/// Groups for "Other Services" display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtherServiceGroup {
    Garbage,
    DeathCare,
    Governance,
    Welfare,
    Postal,
    WaterService,
    Heating,
}

impl OtherServiceGroup {
    pub const ALL: [OtherServiceGroup; 7] = [
        Self::Garbage,
        Self::DeathCare,
        Self::Governance,
        Self::Welfare,
        Self::Postal,
        Self::WaterService,
        Self::Heating,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Garbage => "Garbage",
            Self::DeathCare => "Death Care",
            Self::Governance => "Governance",
            Self::Welfare => "Welfare",
            Self::Postal => "Postal",
            Self::WaterService => "Water Service",
            Self::Heating => "Heating",
        }
    }

    pub fn service_types(self) -> &'static [ServiceType] {
        match self {
            Self::Garbage => &[
                ServiceType::Landfill,
                ServiceType::RecyclingCenter,
                ServiceType::Incinerator,
                ServiceType::TransferStation,
            ],
            Self::DeathCare => &[ServiceType::Cemetery, ServiceType::Crematorium],
            Self::Governance => &[ServiceType::CityHall],
            Self::Welfare => &[ServiceType::HomelessShelter, ServiceType::WelfareOffice],
            Self::Postal => &[ServiceType::PostOffice, ServiceType::MailSortingCenter],
            Self::WaterService => &[ServiceType::WaterTreatmentPlant, ServiceType::WellPump],
            Self::Heating => &[
                ServiceType::HeatingBoiler,
                ServiceType::DistrictHeatingPlant,
                ServiceType::GeothermalPlant,
            ],
        }
    }

    pub fn overlay_mode(self) -> Option<OverlayMode> {
        match self {
            Self::Garbage => Some(OverlayMode::Garbage),
            Self::WaterService => Some(OverlayMode::Water),
            _ => None,
        }
    }
}
