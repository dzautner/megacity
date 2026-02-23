//! Pure helper functions for computing NIMBY/YIMBY opinions.
//!
//! These functions are free of ECS dependencies and can be tested in isolation.

use crate::citizen::Personality;
use crate::grid::ZoneType;
use crate::wealth::WealthTier;

use super::types::{
    CONSTRUCTION_SLOWDOWN_PER_OPPOSITION, HAPPINESS_PENALTY_PER_OPPOSITION,
    MAX_CONSTRUCTION_SLOWDOWN, MAX_NIMBY_HAPPINESS_PENALTY,
};

/// Calculate a density score for a zone type. Higher values = higher impact.
pub fn zone_density_score(zone: ZoneType) -> f32 {
    match zone {
        ZoneType::None => 0.0,
        ZoneType::ResidentialLow => 1.0,
        ZoneType::ResidentialMedium => 2.0,
        ZoneType::ResidentialHigh => 3.0,
        ZoneType::CommercialLow => 1.5,
        ZoneType::CommercialHigh => 2.5,
        ZoneType::Industrial => 3.5,
        ZoneType::Office => 2.0,
        ZoneType::MixedUse => 2.5,
    }
}

/// Determine if a zone type is residential.
pub fn is_residential(zone: ZoneType) -> bool {
    matches!(
        zone,
        ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh
    )
}

/// Calculate the NIMBY opposition score for a zone change, from the perspective
/// of a citizen at a given distance with certain characteristics.
///
/// Returns a value where positive = opposition, negative = support.
#[allow(clippy::too_many_arguments)]
pub fn calculate_opinion(
    old_zone: ZoneType,
    new_zone: ZoneType,
    distance: f32,
    land_value: u8,
    citizen_education: u8,
    personality: &Personality,
    has_park_coverage: bool,
    has_transit_coverage: bool,
    residential_vacancy: f32,
) -> f32 {
    let mut score = 0.0;

    // --- NIMBY factors (positive = opposition) ---

    // 1. Density increase: citizens oppose higher-density development
    let density_change = zone_density_score(new_zone) - zone_density_score(old_zone);
    if density_change > 0.0 {
        score += density_change * 5.0;
    }

    // 2. Industrial adjacency: residential citizens strongly oppose industrial
    if new_zone == ZoneType::Industrial && old_zone != ZoneType::Industrial {
        score += 15.0;
    }

    // 3. Income mismatch: high-income citizens oppose high-density residential
    let wealth = WealthTier::from_education(citizen_education);
    if wealth == WealthTier::HighIncome
        && matches!(new_zone, ZoneType::ResidentialHigh | ZoneType::Industrial)
    {
        score += 8.0;
    }

    // --- YIMBY factors (negative = support) ---

    // 4. Job creation: commercial/office zones create jobs, citizens support this
    if matches!(
        new_zone,
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::Office
    ) && is_residential(old_zone)
    {
        // Only oppose if replacing residential; if replacing None, it's pure support
    } else if matches!(
        new_zone,
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::Office
    ) {
        score -= 5.0;
    }

    // 5. Housing need: if residential vacancy is very low, citizens support new housing
    if is_residential(new_zone) && residential_vacancy < 0.05 {
        score -= 8.0;
    }

    // 6. Amenity proximity bonus: if near parks/transit, development is more welcome
    if has_park_coverage {
        score -= 3.0;
    }
    if has_transit_coverage {
        score -= 2.0;
    }

    // 7. Mixed use is moderately welcomed (creates local services)
    if new_zone == ZoneType::MixedUse && !is_residential(old_zone) {
        score -= 3.0;
    }

    // --- Personality modifiers ---

    // Materialistic citizens care more about property values (oppose more)
    score *= 0.7 + personality.materialism * 0.6;

    // Resilient citizens are less bothered
    score *= 1.3 - personality.resilience * 0.5;

    // --- Land value scaling: wealthier areas oppose more ---
    let land_value_factor = 0.5 + (land_value as f32 / 255.0) * 1.0;
    score *= land_value_factor;

    // --- Distance falloff: opposition weakens with distance ---
    let distance_factor = if distance <= 1.0 {
        1.0
    } else {
        (1.0 / distance).max(0.1)
    };
    score *= distance_factor;

    score
}

/// Calculate the construction slowdown (additional ticks) based on opposition.
pub fn construction_slowdown(opposition: f32) -> u32 {
    if opposition <= 0.0 {
        return 0;
    }
    let extra = (opposition * CONSTRUCTION_SLOWDOWN_PER_OPPOSITION) as u32;
    extra.min(MAX_CONSTRUCTION_SLOWDOWN)
}

/// Calculate the happiness penalty for a citizen based on local opposition.
pub fn nimby_happiness_penalty(opposition: f32) -> f32 {
    if opposition <= 0.0 {
        return 0.0;
    }
    (opposition * HAPPINESS_PENALTY_PER_OPPOSITION).min(MAX_NIMBY_HAPPINESS_PENALTY)
}
