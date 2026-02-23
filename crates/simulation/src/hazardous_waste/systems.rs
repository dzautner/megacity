//! Systems and helper functions for hazardous waste management.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::services::{ServiceBuilding, ServiceType};

use super::constants::*;
use super::types::HazardousWasteState;

// =============================================================================
// Helper functions
// =============================================================================

/// Calculate hazardous waste generation for an industrial building based on level.
pub fn industrial_waste_generation(level: u8) -> f32 {
    let idx = (level as usize)
        .saturating_sub(1)
        .min(INDUSTRIAL_WASTE_PER_LEVEL.len() - 1);
    INDUSTRIAL_WASTE_PER_LEVEL[idx]
}

/// Calculate the number of hazardous waste treatment facilities from service buildings.
/// Uses Incinerator as the closest existing ServiceType for hazardous waste treatment,
/// since a dedicated HazardousWasteFacility ServiceType doesn't exist yet.
pub fn count_hazardous_facilities(services: &[&ServiceBuilding]) -> u32 {
    services
        .iter()
        .filter(|s| s.service_type == ServiceType::Incinerator)
        .count() as u32
}

/// Determine if a service building qualifies as a medical facility that generates
/// hazardous waste.
pub fn is_medical_waste_generator(service_type: ServiceType) -> bool {
    matches!(
        service_type,
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter
    )
}

// =============================================================================
// System
// =============================================================================

/// Updates hazardous waste state each slow tick.
///
/// 1. Tallies hazardous waste generation from industrial + medical buildings
/// 2. Counts treatment facilities (Incinerators serving as hazardous waste facilities)
/// 3. Computes overflow (generation - capacity)
/// 4. If overflow > 0: triggers illegal dumping, contaminates groundwater, adds fines
/// 5. Applies natural contamination decay
#[allow(clippy::too_many_arguments)]
pub fn update_hazardous_waste(
    slow_timer: Res<crate::SlowTickTimer>,
    mut state: ResMut<HazardousWasteState>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    mut water_quality: Option<ResMut<WaterQualityGrid>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Calculate total hazardous waste generation ---
    let mut total_generation: f32 = 0.0;

    // Industrial buildings generate hazardous waste based on level
    for building in &buildings {
        if building.zone_type == ZoneType::Industrial {
            total_generation += industrial_waste_generation(building.level);
        }
    }

    // Medical facilities generate medical hazardous waste
    for service in &services {
        if is_medical_waste_generator(service.service_type) {
            total_generation += MEDICAL_WASTE_RATE;
        }
    }

    state.total_generation = total_generation;

    // --- Phase 2: Count treatment facilities and compute capacity ---
    // Use Incinerators as proxy for hazardous waste treatment facilities
    let mut facility_count: u32 = 0;
    for service in &services {
        if service.service_type == ServiceType::Incinerator {
            facility_count += 1;
        }
    }

    state.facility_count = facility_count;
    state.treatment_capacity = facility_count as f32 * FACILITY_CAPACITY_TONS_PER_DAY;
    state.daily_operating_cost = facility_count as f64 * FACILITY_OPERATING_COST_PER_DAY;

    // --- Phase 3: Compute overflow ---
    let overflow = (total_generation - state.treatment_capacity).max(0.0);
    state.overflow = overflow;

    // --- Phase 4: Distribute treated waste across treatment types ---
    // For now, all facilities use chemical treatment by default.
    // In a future PR, individual facility treatment types can be tracked.
    let treated = total_generation.min(state.treatment_capacity);
    state.chemical_treated = treated;
    state.thermal_treated = 0.0;
    state.biological_treated = 0.0;
    state.stabilization_treated = 0.0;

    // --- Phase 5: Handle overflow — illegal dumping ---
    if overflow > 0.0 {
        state.illegal_dump_events += 1;
        state.federal_fines += FEDERAL_FINE_PER_EVENT;

        // Contamination accumulates based on overflow amount
        let contamination_increase = overflow * CONTAMINATION_PER_OVERFLOW_TON;
        state.contamination_level += contamination_increase;

        // Apply groundwater contamination around industrial buildings
        if let Some(ref mut wq) = water_quality {
            for building in &buildings {
                if building.zone_type != ZoneType::Industrial {
                    continue;
                }
                let cx = building.grid_x as i32;
                let cy = building.grid_y as i32;
                // Scale contamination by overflow magnitude (capped)
                let quality_reduction = (overflow * 0.5).min(10.0) as u8;
                if quality_reduction == 0 {
                    continue;
                }
                for dy in -CONTAMINATION_RADIUS..=CONTAMINATION_RADIUS {
                    for dx in -CONTAMINATION_RADIUS..=CONTAMINATION_RADIUS {
                        let nx = cx + dx;
                        let ny = cy + dy;
                        if nx < 0
                            || ny < 0
                            || (nx as usize) >= wq.width
                            || (ny as usize) >= wq.height
                        {
                            continue;
                        }
                        let dist = dx.abs() + dy.abs();
                        let falloff = quality_reduction.saturating_sub(dist as u8);
                        if falloff > 0 {
                            let ux = nx as usize;
                            let uy = ny as usize;
                            let idx = uy * wq.width + ux;
                            wq.levels[idx] = wq.levels[idx].saturating_sub(falloff);
                        }
                    }
                }
            }
        }
    }

    // --- Phase 6: Natural contamination decay ---
    if state.contamination_level > 0.0 {
        state.contamination_level *= 1.0 - CONTAMINATION_DECAY_RATE;
        if state.contamination_level < 0.01 {
            state.contamination_level = 0.0;
        }
    }
}
