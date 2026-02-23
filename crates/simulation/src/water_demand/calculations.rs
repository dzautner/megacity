use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};

use super::types::{
    COMMERCIAL_GPB, HOSPITAL_GPD, INDUSTRIAL_GPB, PARK_PER_CELL_GPD, RESIDENTIAL_GPCD,
    SCHOOL_PER_STUDENT_GPD,
};

// =============================================================================
// Per-building demand calculation
// =============================================================================

/// Compute the base water demand for a zoned building based on its type and occupancy.
pub fn base_demand_for_building(building: &Building) -> f32 {
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
pub fn base_demand_for_service(service: &ServiceBuilding) -> f32 {
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
pub fn supply_capacity_for_utility(utility_type: crate::utilities::UtilityType) -> f32 {
    use crate::utilities::UtilityType;
    match utility_type {
        UtilityType::WaterTower => 50_000.0,
        UtilityType::PumpingStation => 30_000.0,
        UtilityType::WaterTreatment => 80_000.0,
        UtilityType::SewagePlant => 20_000.0,
        _ => 0.0, // Power plants don't supply water
    }
}
