//! Walkability category definitions and classification helpers.

use crate::grid::ZoneType;
use crate::services::ServiceType;

// =============================================================================
// Constants
// =============================================================================

/// Category weights (must sum to 1.0).
pub(crate) const WEIGHT_GROCERY: f32 = 0.25;
pub(crate) const WEIGHT_SCHOOL: f32 = 0.15;
pub(crate) const WEIGHT_HEALTHCARE: f32 = 0.20;
pub(crate) const WEIGHT_PARK: f32 = 0.15;
pub(crate) const WEIGHT_TRANSIT: f32 = 0.15;
pub(crate) const WEIGHT_EMPLOYMENT: f32 = 0.10;

// =============================================================================
// Walkability category
// =============================================================================

/// The six service categories used for walkability scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkabilityCategory {
    Grocery,
    School,
    Healthcare,
    Park,
    Transit,
    Employment,
}

impl WalkabilityCategory {
    /// Weight of this category in the composite score.
    pub fn weight(self) -> f32 {
        match self {
            WalkabilityCategory::Grocery => WEIGHT_GROCERY,
            WalkabilityCategory::School => WEIGHT_SCHOOL,
            WalkabilityCategory::Healthcare => WEIGHT_HEALTHCARE,
            WalkabilityCategory::Park => WEIGHT_PARK,
            WalkabilityCategory::Transit => WEIGHT_TRANSIT,
            WalkabilityCategory::Employment => WEIGHT_EMPLOYMENT,
        }
    }
}

// =============================================================================
// Classification helpers
// =============================================================================

/// Classify a `ServiceType` into a walkability category, if applicable.
pub fn classify_service(service_type: ServiceType) -> Option<WalkabilityCategory> {
    match service_type {
        // Healthcare
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => {
            Some(WalkabilityCategory::Healthcare)
        }
        // School/Education
        ServiceType::ElementarySchool
        | ServiceType::HighSchool
        | ServiceType::University
        | ServiceType::Library
        | ServiceType::Kindergarten => Some(WalkabilityCategory::School),
        // Park/Recreation
        ServiceType::SmallPark
        | ServiceType::LargePark
        | ServiceType::Playground
        | ServiceType::Plaza
        | ServiceType::SportsField => Some(WalkabilityCategory::Park),
        // Transit
        ServiceType::BusDepot
        | ServiceType::TrainStation
        | ServiceType::SubwayStation
        | ServiceType::TramDepot
        | ServiceType::FerryPier => Some(WalkabilityCategory::Transit),
        _ => None,
    }
}

/// Classify a `ZoneType` into a walkability category, if applicable.
pub fn classify_zone(zone_type: ZoneType) -> Option<WalkabilityCategory> {
    match zone_type {
        // Grocery/Commercial: commercial zones and mixed-use with commercial ground floors
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::MixedUse => {
            Some(WalkabilityCategory::Grocery)
        }
        // Employment: industrial, office zones
        ZoneType::Industrial | ZoneType::Office => Some(WalkabilityCategory::Employment),
        _ => None,
    }
}
