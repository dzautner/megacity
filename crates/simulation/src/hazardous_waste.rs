//! Hazardous waste facility and industrial waste management (WASTE-007).
//!
//! Industrial and medical buildings generate hazardous waste that requires
//! specialized treatment at a HazardousWasteFacility. Without adequate
//! treatment capacity, overflow waste triggers illegal dumping events that
//! cause soil and groundwater contamination, plus federal fines.
//!
//! Key design points:
//! - HazardousWasteFacility: 20 tons/day capacity, $3M build, $5K/day operating
//! - Industrial buildings generate waste based on level (higher level = more waste)
//! - Medical buildings (Hospital, MedicalClinic, MedicalCenter) generate medical waste
//! - Without facility: illegal dumping causes contamination + federal fines
//! - Treatment types: chemical, thermal, biological, stabilization

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::services::{ServiceBuilding, ServiceType};

// =============================================================================
// Constants
// =============================================================================

/// Capacity per hazardous waste facility in tons/day.
pub const FACILITY_CAPACITY_TONS_PER_DAY: f32 = 20.0;

/// Build cost for a single hazardous waste facility ($3M).
pub const FACILITY_BUILD_COST: f64 = 3_000_000.0;

/// Operating cost per facility per day ($5K).
pub const FACILITY_OPERATING_COST_PER_DAY: f64 = 5_000.0;

/// Federal fine per illegal dump event ($50K).
pub const FEDERAL_FINE_PER_EVENT: f64 = 50_000.0;

/// Groundwater quality reduction per unit of illegal dumping overflow.
/// Applied to cells around industrial buildings when dumping occurs.
pub const CONTAMINATION_PER_OVERFLOW_TON: f32 = 2.0;

/// Contamination natural decay rate per slow tick (1% reduction).
pub const CONTAMINATION_DECAY_RATE: f32 = 0.01;

/// Base hazardous waste generation rate per industrial building level (tons/day).
/// Level 1 = 0.5, Level 2 = 1.0, Level 3 = 2.0, Level 4 = 3.5, Level 5 = 5.0.
pub const INDUSTRIAL_WASTE_PER_LEVEL: [f32; 5] = [0.5, 1.0, 2.0, 3.5, 5.0];

/// Base hazardous waste generation for medical buildings (tons/day per facility).
pub const MEDICAL_WASTE_RATE: f32 = 0.8;

/// Radius (in grid cells) around industrial buildings affected by illegal dumping
/// contamination of groundwater.
pub const CONTAMINATION_RADIUS: i32 = 4;

// =============================================================================
// Treatment type enum
// =============================================================================

/// Treatment method used by a hazardous waste facility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TreatmentType {
    /// Chemical treatment: neutralization, oxidation, reduction.
    #[default]
    Chemical,
    /// Thermal treatment: incineration at high temperatures.
    Thermal,
    /// Biological treatment: bioremediation using microorganisms.
    Biological,
    /// Stabilization/solidification: encapsulating waste in concrete or polymers.
    Stabilization,
}

impl TreatmentType {
    /// Efficiency multiplier for treatment capacity.
    /// Higher efficiency means more waste can be treated per unit capacity.
    pub fn efficiency(&self) -> f32 {
        match self {
            TreatmentType::Chemical => 1.0,
            TreatmentType::Thermal => 1.2,
            TreatmentType::Biological => 0.8,
            TreatmentType::Stabilization => 0.9,
        }
    }

    /// Cost multiplier relative to base operating cost.
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            TreatmentType::Chemical => 1.0,
            TreatmentType::Thermal => 1.5,
            TreatmentType::Biological => 0.7,
            TreatmentType::Stabilization => 0.8,
        }
    }
}

// =============================================================================
// HazardousWasteState resource
// =============================================================================

/// City-wide hazardous waste management state.
///
/// Tracks generation, treatment capacity, overflow, contamination, and fines.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct HazardousWasteState {
    /// Total hazardous waste generated per day (tons/day).
    pub total_generation: f32,
    /// Total treatment capacity available (tons/day).
    pub treatment_capacity: f32,
    /// Untreated waste overflow (tons) — resets each tick but accumulates effects.
    pub overflow: f32,
    /// Cumulative count of illegal dump events.
    pub illegal_dump_events: u32,
    /// Accumulated ground contamination level (0.0 = clean).
    pub contamination_level: f32,
    /// Accumulated federal fines in dollars.
    pub federal_fines: f64,
    /// Number of active hazardous waste treatment facilities.
    pub facility_count: u32,
    /// Daily operating cost for all facilities.
    pub daily_operating_cost: f64,
    /// Breakdown of waste by treatment type (tons processed per type).
    pub chemical_treated: f32,
    /// Tons treated via thermal methods.
    pub thermal_treated: f32,
    /// Tons treated via biological methods.
    pub biological_treated: f32,
    /// Tons treated via stabilization methods.
    pub stabilization_treated: f32,
}

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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // TreatmentType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_treatment_type_default() {
        assert_eq!(TreatmentType::default(), TreatmentType::Chemical);
    }

    #[test]
    fn test_treatment_efficiency_values() {
        assert!((TreatmentType::Chemical.efficiency() - 1.0).abs() < f32::EPSILON);
        assert!((TreatmentType::Thermal.efficiency() - 1.2).abs() < f32::EPSILON);
        assert!((TreatmentType::Biological.efficiency() - 0.8).abs() < f32::EPSILON);
        assert!((TreatmentType::Stabilization.efficiency() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_treatment_cost_multiplier_values() {
        assert!((TreatmentType::Chemical.cost_multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((TreatmentType::Thermal.cost_multiplier() - 1.5).abs() < f64::EPSILON);
        assert!((TreatmentType::Biological.cost_multiplier() - 0.7).abs() < f64::EPSILON);
        assert!((TreatmentType::Stabilization.cost_multiplier() - 0.8).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Industrial waste generation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_industrial_waste_generation_per_level() {
        assert!((industrial_waste_generation(1) - 0.5).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(2) - 1.0).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(3) - 2.0).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(4) - 3.5).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(5) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_industrial_waste_generation_level_zero_clamps() {
        // Level 0 should clamp to level 1 rate via saturating_sub
        assert!((industrial_waste_generation(0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_industrial_waste_generation_high_level_clamps() {
        // Level 10 should clamp to level 5 rate
        assert!((industrial_waste_generation(10) - 5.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Medical waste identification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_medical_waste_generators() {
        assert!(is_medical_waste_generator(ServiceType::Hospital));
        assert!(is_medical_waste_generator(ServiceType::MedicalClinic));
        assert!(is_medical_waste_generator(ServiceType::MedicalCenter));
    }

    #[test]
    fn test_non_medical_not_waste_generators() {
        assert!(!is_medical_waste_generator(ServiceType::FireStation));
        assert!(!is_medical_waste_generator(ServiceType::PoliceStation));
        assert!(!is_medical_waste_generator(ServiceType::Landfill));
        assert!(!is_medical_waste_generator(ServiceType::Incinerator));
        assert!(!is_medical_waste_generator(ServiceType::University));
    }

    // -------------------------------------------------------------------------
    // HazardousWasteState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = HazardousWasteState::default();
        assert_eq!(state.total_generation, 0.0);
        assert_eq!(state.treatment_capacity, 0.0);
        assert_eq!(state.overflow, 0.0);
        assert_eq!(state.illegal_dump_events, 0);
        assert_eq!(state.contamination_level, 0.0);
        assert_eq!(state.federal_fines, 0.0);
        assert_eq!(state.facility_count, 0);
        assert_eq!(state.daily_operating_cost, 0.0);
        assert_eq!(state.chemical_treated, 0.0);
        assert_eq!(state.thermal_treated, 0.0);
        assert_eq!(state.biological_treated, 0.0);
        assert_eq!(state.stabilization_treated, 0.0);
    }

    // -------------------------------------------------------------------------
    // Constants tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_facility_constants() {
        assert!((FACILITY_CAPACITY_TONS_PER_DAY - 20.0).abs() < f32::EPSILON);
        assert!((FACILITY_BUILD_COST - 3_000_000.0).abs() < f64::EPSILON);
        assert!((FACILITY_OPERATING_COST_PER_DAY - 5_000.0).abs() < f64::EPSILON);
        assert!((FEDERAL_FINE_PER_EVENT - 50_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Overflow and fines logic tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overflow_when_generation_exceeds_capacity() {
        let generation = 30.0_f32;
        let capacity = 20.0_f32;
        let overflow = (generation - capacity).max(0.0);
        assert!((overflow - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_overflow_when_capacity_sufficient() {
        let generation = 15.0_f32;
        let capacity = 20.0_f32;
        let overflow = (generation - capacity).max(0.0);
        assert!(overflow.abs() < f32::EPSILON);
    }

    #[test]
    fn test_federal_fines_accumulate() {
        let mut fines = 0.0_f64;
        // Three illegal dump events
        fines += FEDERAL_FINE_PER_EVENT;
        fines += FEDERAL_FINE_PER_EVENT;
        fines += FEDERAL_FINE_PER_EVENT;
        assert!((fines - 150_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Contamination logic tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_contamination_increases_with_overflow() {
        let overflow = 5.0_f32;
        let contamination = overflow * CONTAMINATION_PER_OVERFLOW_TON;
        assert!((contamination - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_contamination_decay() {
        let mut contamination = 100.0_f32;
        contamination *= 1.0 - CONTAMINATION_DECAY_RATE;
        assert!((contamination - 99.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_contamination_decay_clamps_to_zero() {
        let mut contamination = 0.005_f32;
        contamination *= 1.0 - CONTAMINATION_DECAY_RATE;
        if contamination < 0.01 {
            contamination = 0.0;
        }
        assert!(contamination.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Treatment capacity scaling tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_capacity_scales_with_facility_count() {
        assert!((0_u32 as f32 * FACILITY_CAPACITY_TONS_PER_DAY).abs() < f32::EPSILON);
        assert!((1_u32 as f32 * FACILITY_CAPACITY_TONS_PER_DAY - 20.0).abs() < f32::EPSILON);
        assert!((3_u32 as f32 * FACILITY_CAPACITY_TONS_PER_DAY - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_operating_cost_scales_with_facility_count() {
        let cost_2 = 2_u32 as f64 * FACILITY_OPERATING_COST_PER_DAY;
        assert!((cost_2 - 10_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (simulating the update logic)
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_cycle_no_overflow() {
        // 2 industrial buildings (level 1 + level 2) = 0.5 + 1.0 = 1.5 tons/day
        // 1 facility = 20 tons/day capacity
        // No overflow expected
        let generation = industrial_waste_generation(1) + industrial_waste_generation(2);
        let capacity = 1.0 * FACILITY_CAPACITY_TONS_PER_DAY;
        let overflow = (generation - capacity).max(0.0);

        assert!((generation - 1.5).abs() < f32::EPSILON);
        assert!((capacity - 20.0).abs() < f32::EPSILON);
        assert!(overflow.abs() < f32::EPSILON);
    }

    #[test]
    fn test_full_cycle_with_overflow() {
        // 5 industrial buildings at level 5 = 5 * 5.0 = 25 tons/day
        // 1 medical facility = 0.8 tons/day
        // Total = 25.8 tons/day
        // 1 facility = 20 tons/day capacity
        // Overflow = 5.8 tons
        let industrial_gen = 5.0 * industrial_waste_generation(5);
        let medical_gen = MEDICAL_WASTE_RATE;
        let total = industrial_gen + medical_gen;
        let capacity = 1.0 * FACILITY_CAPACITY_TONS_PER_DAY;
        let overflow = (total - capacity).max(0.0);

        assert!((total - 25.8).abs() < 0.01);
        assert!((overflow - 5.8).abs() < 0.01);
    }

    #[test]
    fn test_full_cycle_contamination_accumulation() {
        // Simulate 3 ticks of overflow
        let mut state = HazardousWasteState::default();
        let overflow_per_tick = 5.0_f32;

        for _ in 0..3 {
            let contamination_increase = overflow_per_tick * CONTAMINATION_PER_OVERFLOW_TON;
            state.contamination_level += contamination_increase;
            state.illegal_dump_events += 1;
            state.federal_fines += FEDERAL_FINE_PER_EVENT;

            // Apply decay
            state.contamination_level *= 1.0 - CONTAMINATION_DECAY_RATE;
        }

        assert_eq!(state.illegal_dump_events, 3);
        assert!((state.federal_fines - 150_000.0).abs() < f64::EPSILON);
        // Contamination should be positive and accumulated
        assert!(state.contamination_level > 0.0);
        // After 3 ticks of 10.0 increase each with 1% decay:
        // tick 1: 10.0 * 0.99 = 9.9
        // tick 2: (9.9 + 10.0) * 0.99 = 19.701
        // tick 3: (19.701 + 10.0) * 0.99 = 29.40399
        assert!((state.contamination_level - 29.404).abs() < 0.01);
    }

    #[test]
    fn test_no_generation_no_effects() {
        // No industrial or medical buildings => zero everything
        let generation = 0.0_f32;
        let capacity = 0.0_f32;
        let overflow = (generation - capacity).max(0.0);

        assert!(generation.abs() < f32::EPSILON);
        assert!(overflow.abs() < f32::EPSILON);
    }

    #[test]
    fn test_groundwater_contamination_radius() {
        // Verify contamination radius constant
        assert_eq!(CONTAMINATION_RADIUS, 4);

        // Within radius: affected
        let dist_inside = 3_i32;
        assert!(dist_inside <= CONTAMINATION_RADIUS);

        // Outside radius: not affected
        let dist_outside = 5_i32;
        assert!(dist_outside > CONTAMINATION_RADIUS);
    }

    #[test]
    fn test_groundwater_quality_reduction_with_overflow() {
        // Simulate quality reduction logic
        let overflow = 8.0_f32;
        let quality_reduction = (overflow * 0.5).min(10.0) as u8;
        assert_eq!(quality_reduction, 4);

        // Large overflow is capped at 10
        let large_overflow = 100.0_f32;
        let capped_reduction = (large_overflow * 0.5).min(10.0) as u8;
        assert_eq!(capped_reduction, 10);
    }

    #[test]
    fn test_count_hazardous_facilities_helper() {
        // Test the helper function with mock service buildings
        let incinerator = ServiceBuilding {
            service_type: ServiceType::Incinerator,
            grid_x: 10,
            grid_y: 10,
            radius: 480.0,
        };
        let landfill = ServiceBuilding {
            service_type: ServiceType::Landfill,
            grid_x: 20,
            grid_y: 20,
            radius: 320.0,
        };
        let incinerator2 = ServiceBuilding {
            service_type: ServiceType::Incinerator,
            grid_x: 30,
            grid_y: 30,
            radius: 480.0,
        };

        let services: Vec<&ServiceBuilding> = vec![&incinerator, &landfill, &incinerator2];
        let count = count_hazardous_facilities(&services);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_hazardous_facilities_empty() {
        let services: Vec<&ServiceBuilding> = vec![];
        assert_eq!(count_hazardous_facilities(&services), 0);
    }

    #[test]
    fn test_multiple_treatment_types_efficiency() {
        // Verify that different treatment types have distinct efficiencies
        let types = [
            TreatmentType::Chemical,
            TreatmentType::Thermal,
            TreatmentType::Biological,
            TreatmentType::Stabilization,
        ];
        // All efficiencies should be positive
        for t in &types {
            assert!(t.efficiency() > 0.0);
        }
        // Thermal should be the most efficient
        assert!(TreatmentType::Thermal.efficiency() > TreatmentType::Chemical.efficiency());
        // Biological should be least efficient
        assert!(TreatmentType::Biological.efficiency() < TreatmentType::Chemical.efficiency());
    }
}

pub struct HazardousWastePlugin;

impl Plugin for HazardousWastePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HazardousWasteState>().add_systems(
            FixedUpdate,
            update_hazardous_waste.after(crate::imports_exports::process_trade),
        );
    }
}
