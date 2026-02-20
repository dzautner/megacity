//! Composting facility and organic waste diversion (WASTE-006).
//!
//! Composting facilities divert organic waste (food + yard waste) from landfill.
//! Multiple composting methods are supported, each with different capacities and costs.
//! Compost products can be sold for revenue, and anaerobic digestion facilities
//! additionally produce biogas that generates electricity.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::garbage::WasteSystem;

// =============================================================================
// Compost method enum
// =============================================================================

/// Composting method with different capacity, cost, and output characteristics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompostMethod {
    /// Open-air windrow composting. Low cost, moderate capacity.
    #[default]
    Windrow,
    /// Aerated static pile composting. Medium cost, higher capacity.
    AeratedStaticPile,
    /// Enclosed in-vessel composting. High cost, highest capacity.
    InVessel,
    /// Anaerobic digestion. Medium-high cost, produces biogas as a bonus.
    AnaerobicDigestion,
}

impl CompostMethod {
    /// Default processing capacity in tons per day.
    pub fn capacity(&self) -> f32 {
        match self {
            CompostMethod::Windrow => 50.0,
            CompostMethod::AeratedStaticPile => 100.0,
            CompostMethod::InVessel => 200.0,
            CompostMethod::AnaerobicDigestion => 100.0,
        }
    }

    /// Default operating cost per ton of waste processed.
    pub fn cost_per_ton(&self) -> f32 {
        match self {
            CompostMethod::Windrow => 30.0,
            CompostMethod::AeratedStaticPile => 45.0,
            CompostMethod::InVessel => 60.0,
            CompostMethod::AnaerobicDigestion => 50.0,
        }
    }

    /// Whether this method produces biogas as a byproduct.
    pub fn produces_biogas(&self) -> bool {
        matches!(self, CompostMethod::AnaerobicDigestion)
    }
}

// =============================================================================
// Compost facility
// =============================================================================

/// A single composting facility with a specific method and capacity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompostFacility {
    /// Composting method used by this facility.
    pub method: CompostMethod,
    /// Maximum processing capacity in tons per day.
    pub capacity_tons_per_day: f32,
    /// Operating cost per ton of waste processed.
    pub cost_per_ton: f32,
    /// Tons of organic waste processed in the current period.
    pub tons_processed_today: f32,
}

impl CompostFacility {
    /// Create a new facility with default capacity and cost for the given method.
    pub fn new(method: CompostMethod) -> Self {
        Self {
            method,
            capacity_tons_per_day: method.capacity(),
            cost_per_ton: method.cost_per_ton(),
            tons_processed_today: 0.0,
        }
    }
}

// =============================================================================
// Composting state resource
// =============================================================================

/// City-wide composting state, tracking all facilities, diversion rates, and revenue.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CompostingState {
    /// Active composting facilities in the city.
    pub facilities: Vec<CompostFacility>,
    /// Fraction of population participating in organic waste separation (0.0..=1.0).
    pub participation_rate: f32,
    /// Fraction of total MSW that is compostable (food + yard waste).
    pub organic_fraction: f32,
    /// Cumulative tons of organic waste diverted from landfill.
    pub total_diverted_tons: f32,
    /// Tons of organic waste diverted in the most recent period.
    pub daily_diversion_tons: f32,
    /// Revenue per ton of finished compost sold.
    pub compost_revenue_per_ton: f32,
    /// Revenue earned from compost sales in the most recent period.
    pub daily_revenue: f32,
    /// Biogas electricity yield in MWh per ton for anaerobic digestion facilities.
    pub biogas_mwh_per_ton: f32,
    /// Biogas electricity generated in the most recent period (MWh).
    pub daily_biogas_mwh: f32,
}

impl Default for CompostingState {
    fn default() -> Self {
        Self {
            facilities: Vec::new(),
            participation_rate: 0.70,
            organic_fraction: 0.34,
            total_diverted_tons: 0.0,
            daily_diversion_tons: 0.0,
            compost_revenue_per_ton: 30.0,
            daily_revenue: 0.0,
            biogas_mwh_per_ton: 0.15,
            daily_biogas_mwh: 0.0,
        }
    }
}

// =============================================================================
// System
// =============================================================================

/// Updates composting facilities each period.
///
/// Reads the city's total waste generation, calculates the organic fraction
/// available for composting (adjusted by participation rate), distributes it
/// among facilities up to their capacity, and computes revenue and biogas output.
pub fn update_composting(waste_system: Res<WasteSystem>, mut composting: ResMut<CompostingState>) {
    // Read config values before mutable iteration
    let total_waste_tons = waste_system.period_generated_tons as f32;
    let organic_available =
        total_waste_tons * composting.organic_fraction * composting.participation_rate;
    let revenue_per_ton = composting.compost_revenue_per_ton;
    let biogas_rate = composting.biogas_mwh_per_ton;

    let mut remaining = organic_available;
    let mut total_diverted = 0.0_f32;
    let mut total_revenue = 0.0_f32;
    let mut total_biogas = 0.0_f32;

    // Distribute organic waste across facilities up to each one's capacity
    for facility in composting.facilities.iter_mut() {
        if remaining <= 0.0 {
            facility.tons_processed_today = 0.0;
            continue;
        }

        let processed = remaining.min(facility.capacity_tons_per_day);
        facility.tons_processed_today = processed;
        remaining -= processed;
        total_diverted += processed;

        // Revenue from selling compost
        total_revenue += processed * revenue_per_ton;

        // Biogas from anaerobic digestion
        if facility.method.produces_biogas() {
            total_biogas += processed * biogas_rate;
        }
    }

    composting.daily_diversion_tons = total_diverted;
    composting.total_diverted_tons += total_diverted;
    composting.daily_revenue = total_revenue;
    composting.daily_biogas_mwh = total_biogas;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // CompostMethod helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_method_capacity_values() {
        assert_eq!(CompostMethod::Windrow.capacity(), 50.0);
        assert_eq!(CompostMethod::AeratedStaticPile.capacity(), 100.0);
        assert_eq!(CompostMethod::InVessel.capacity(), 200.0);
        assert_eq!(CompostMethod::AnaerobicDigestion.capacity(), 100.0);
    }

    #[test]
    fn test_method_cost_per_ton_values() {
        assert_eq!(CompostMethod::Windrow.cost_per_ton(), 30.0);
        assert_eq!(CompostMethod::AeratedStaticPile.cost_per_ton(), 45.0);
        assert_eq!(CompostMethod::InVessel.cost_per_ton(), 60.0);
        assert_eq!(CompostMethod::AnaerobicDigestion.cost_per_ton(), 50.0);
    }

    #[test]
    fn test_method_produces_biogas() {
        assert!(!CompostMethod::Windrow.produces_biogas());
        assert!(!CompostMethod::AeratedStaticPile.produces_biogas());
        assert!(!CompostMethod::InVessel.produces_biogas());
        assert!(CompostMethod::AnaerobicDigestion.produces_biogas());
    }

    #[test]
    fn test_default_method_is_windrow() {
        assert_eq!(CompostMethod::default(), CompostMethod::Windrow);
    }

    // -------------------------------------------------------------------------
    // CompostFacility tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_facility_new_uses_method_defaults() {
        let windrow = CompostFacility::new(CompostMethod::Windrow);
        assert_eq!(windrow.capacity_tons_per_day, 50.0);
        assert_eq!(windrow.cost_per_ton, 30.0);
        assert_eq!(windrow.tons_processed_today, 0.0);

        let vessel = CompostFacility::new(CompostMethod::InVessel);
        assert_eq!(vessel.capacity_tons_per_day, 200.0);
        assert_eq!(vessel.cost_per_ton, 60.0);
    }

    // -------------------------------------------------------------------------
    // CompostingState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_composting_state_default() {
        let state = CompostingState::default();
        assert!(state.facilities.is_empty());
        assert!((state.participation_rate - 0.70).abs() < 0.001);
        assert!((state.organic_fraction - 0.34).abs() < 0.001);
        assert_eq!(state.total_diverted_tons, 0.0);
        assert_eq!(state.daily_diversion_tons, 0.0);
        assert!((state.compost_revenue_per_ton - 30.0).abs() < 0.001);
        assert_eq!(state.daily_revenue, 0.0);
        assert!((state.biogas_mwh_per_ton - 0.15).abs() < 0.001);
        assert_eq!(state.daily_biogas_mwh, 0.0);
    }

    // -------------------------------------------------------------------------
    // Organic waste fraction calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_organic_waste_fraction_calculation() {
        // With 1000 tons total waste, 34% organic, 70% participation:
        // organic available = 1000 * 0.34 * 0.70 = 238 tons
        let total_waste = 1000.0_f32;
        let organic_fraction = 0.34_f32;
        let participation = 0.70_f32;
        let organic_available = total_waste * organic_fraction * participation;
        assert!((organic_available - 238.0).abs() < 0.1);
    }

    // -------------------------------------------------------------------------
    // Facility processing up to capacity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_facility_processes_up_to_capacity() {
        // Windrow facility (50 tons/day) with 100 tons available: should process 50
        let mut state = CompostingState::default();
        state
            .facilities
            .push(CompostFacility::new(CompostMethod::Windrow));

        // Simulate the logic manually
        let organic_available = 100.0_f32;
        let mut remaining = organic_available;

        for facility in state.facilities.iter_mut() {
            let processed = remaining.min(facility.capacity_tons_per_day);
            facility.tons_processed_today = processed;
            remaining -= processed;
        }

        assert!((state.facilities[0].tons_processed_today - 50.0).abs() < 0.001);
        assert!((remaining - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_facility_processes_less_than_capacity() {
        // Windrow facility (50 tons/day) with only 30 tons available: should process 30
        let mut state = CompostingState::default();
        state
            .facilities
            .push(CompostFacility::new(CompostMethod::Windrow));

        let organic_available = 30.0_f32;
        let mut remaining = organic_available;

        for facility in state.facilities.iter_mut() {
            let processed = remaining.min(facility.capacity_tons_per_day);
            facility.tons_processed_today = processed;
            remaining -= processed;
        }

        assert!((state.facilities[0].tons_processed_today - 30.0).abs() < 0.001);
        assert!(remaining.abs() < 0.001);
    }

    #[test]
    fn test_multiple_facilities_distribute_waste() {
        // Windrow (50) + InVessel (200) with 180 tons: first gets 50, second gets 130
        let mut state = CompostingState::default();
        state
            .facilities
            .push(CompostFacility::new(CompostMethod::Windrow));
        state
            .facilities
            .push(CompostFacility::new(CompostMethod::InVessel));

        let organic_available = 180.0_f32;
        let mut remaining = organic_available;

        for facility in state.facilities.iter_mut() {
            let processed = remaining.min(facility.capacity_tons_per_day);
            facility.tons_processed_today = processed;
            remaining -= processed;
        }

        assert!((state.facilities[0].tons_processed_today - 50.0).abs() < 0.001);
        assert!((state.facilities[1].tons_processed_today - 130.0).abs() < 0.001);
        assert!(remaining.abs() < 0.001);
    }

    // -------------------------------------------------------------------------
    // Biogas generation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_biogas_only_from_anaerobic_digestion() {
        let state = CompostingState::default();
        let biogas_rate = state.biogas_mwh_per_ton;

        // Windrow: no biogas
        let windrow = CompostFacility::new(CompostMethod::Windrow);
        let windrow_biogas = if windrow.method.produces_biogas() {
            50.0 * biogas_rate
        } else {
            0.0
        };
        assert_eq!(windrow_biogas, 0.0);

        // AnaerobicDigestion: produces biogas
        let ad = CompostFacility::new(CompostMethod::AnaerobicDigestion);
        let ad_biogas = if ad.method.produces_biogas() {
            50.0 * biogas_rate
        } else {
            0.0
        };
        assert!((ad_biogas - 7.5).abs() < 0.001); // 50 * 0.15 = 7.5 MWh
    }

    #[test]
    fn test_biogas_generation_scales_with_processed_tons() {
        let biogas_rate = 0.15_f32;

        // 100 tons processed -> 15 MWh
        assert!((100.0 * biogas_rate - 15.0).abs() < 0.001);

        // 0 tons processed -> 0 MWh
        assert!((0.0 * biogas_rate).abs() < 0.001);
    }

    // -------------------------------------------------------------------------
    // Revenue calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_revenue_calculation() {
        let revenue_per_ton = 30.0_f32;

        // 100 tons diverted * $30/ton = $3000
        let revenue = 100.0 * revenue_per_ton;
        assert!((revenue - 3000.0).abs() < 0.001);
    }

    #[test]
    fn test_zero_waste_produces_zero_revenue() {
        let revenue_per_ton = 30.0_f32;
        let revenue = 0.0 * revenue_per_ton;
        assert!(revenue.abs() < 0.001);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (simulating the update logic)
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_composting_cycle() {
        // Setup: 500 tons/day total waste, 34% organic, 70% participation
        // => organic = 500 * 0.34 * 0.70 = 119 tons
        // Facilities: Windrow(50) + AnaerobicDigestion(100) = 150 tons capacity
        // Should process all 119 tons (under capacity)
        let total_waste = 500.0_f32;
        let organic_fraction = 0.34_f32;
        let participation = 0.70_f32;
        let compost_revenue = 30.0_f32;
        let biogas_rate = 0.15_f32;

        let organic_available = total_waste * organic_fraction * participation;
        assert!((organic_available - 119.0).abs() < 0.1);

        let mut facilities = vec![
            CompostFacility::new(CompostMethod::Windrow),
            CompostFacility::new(CompostMethod::AnaerobicDigestion),
        ];

        let mut remaining = organic_available;
        let mut total_diverted = 0.0_f32;
        let mut total_revenue = 0.0_f32;
        let mut total_biogas = 0.0_f32;

        for facility in facilities.iter_mut() {
            if remaining <= 0.0 {
                break;
            }
            let processed = remaining.min(facility.capacity_tons_per_day);
            facility.tons_processed_today = processed;
            remaining -= processed;
            total_diverted += processed;
            total_revenue += processed * compost_revenue;
            if facility.method.produces_biogas() {
                total_biogas += processed * biogas_rate;
            }
        }

        // Windrow processes 50, AD processes remaining 69
        assert!((facilities[0].tons_processed_today - 50.0).abs() < 0.001);
        assert!((facilities[1].tons_processed_today - 69.0).abs() < 0.1);
        assert!((total_diverted - organic_available).abs() < 0.1);
        assert!(remaining.abs() < 0.1);

        // Revenue: 119 * 30 = 3570
        assert!((total_revenue - organic_available * compost_revenue).abs() < 1.0);

        // Biogas: only from AD facility, ~69 * 0.15 = ~10.35 MWh
        assert!((total_biogas - facilities[1].tons_processed_today * biogas_rate).abs() < 0.1);
        assert!(total_biogas > 0.0);
    }

    #[test]
    fn test_no_facilities_no_diversion() {
        let organic_available = 100.0_f32;
        let facilities: Vec<CompostFacility> = Vec::new();
        let mut remaining = organic_available;
        let mut total_diverted = 0.0_f32;

        for facility in facilities.iter() {
            let processed = remaining.min(facility.capacity_tons_per_day);
            remaining -= processed;
            total_diverted += processed;
        }

        assert_eq!(total_diverted, 0.0);
        assert!((remaining - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_excess_capacity_does_not_overprocess() {
        // InVessel (200 tons/day) but only 50 tons available: processes only 50
        let organic_available = 50.0_f32;
        let mut facility = CompostFacility::new(CompostMethod::InVessel);
        let processed = organic_available.min(facility.capacity_tons_per_day);
        facility.tons_processed_today = processed;

        assert!((facility.tons_processed_today - 50.0).abs() < 0.001);
    }
}

pub struct CompostingPlugin;

impl Plugin for CompostingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompostingState>().add_systems(
            FixedUpdate,
            update_composting.after(crate::imports_exports::process_trade),
        );
    }
}
