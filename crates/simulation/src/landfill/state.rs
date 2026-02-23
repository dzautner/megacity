//! City-wide landfill state resource, waste distribution, and Bevy system.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::constants::*;
use super::types::*;
use crate::SlowTickTimer;

// =============================================================================
// LandfillState resource
// =============================================================================

/// City-wide landfill tracking resource.
///
/// Contains all landfill sites and aggregate statistics. Updated each slow tick
/// by the `update_landfill_state` system.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct LandfillState {
    /// All landfill sites in the city.
    pub sites: Vec<LandfillSite>,
    /// Next ID to assign to a new landfill site.
    pub next_id: u32,

    // --- Aggregate statistics ---
    /// Total capacity across all active landfill sites in tons.
    pub total_capacity_tons: f64,
    /// Total current fill across all active landfill sites in tons.
    pub total_fill_tons: f64,
    /// Total remaining capacity across all active sites in tons.
    pub total_remaining_tons: f64,
    /// City-wide remaining capacity percentage (0.0-100.0).
    pub remaining_pct: f32,
    /// City-wide estimated years remaining at current input rate.
    pub estimated_years_remaining: f32,
    /// Total daily waste input across all active landfills in tons/day.
    pub total_daily_input_tons: f64,
    /// Total electricity generated from gas collection in MW.
    pub total_gas_electricity_mw: f64,

    // --- Counts ---
    /// Number of active landfill sites.
    pub active_sites: u32,
    /// Number of closed (monitoring) landfill sites.
    pub closed_sites: u32,
    /// Number of sites converted to parks.
    pub park_sites: u32,
}

impl LandfillState {
    /// Add a new landfill site at the given grid position.
    pub fn add_site(&mut self, grid_x: usize, grid_y: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.sites.push(LandfillSite::new(id, grid_x, grid_y));
        id
    }

    /// Add a new landfill site with specified capacity and liner type.
    pub fn add_site_with_options(
        &mut self,
        grid_x: usize,
        grid_y: usize,
        capacity: f64,
        liner_type: LandfillLinerType,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.sites.push(LandfillSite::with_capacity_and_liner(
            id, grid_x, grid_y, capacity, liner_type,
        ));
        id
    }

    /// Get a reference to a landfill site by ID.
    pub fn get_site(&self, id: u32) -> Option<&LandfillSite> {
        self.sites.iter().find(|s| s.id == id)
    }

    /// Get a mutable reference to a landfill site by ID.
    pub fn get_site_mut(&mut self, id: u32) -> Option<&mut LandfillSite> {
        self.sites.iter_mut().find(|s| s.id == id)
    }

    /// Recompute aggregate statistics from individual sites.
    pub fn recompute_aggregates(&mut self) {
        let mut total_capacity = 0.0_f64;
        let mut total_fill = 0.0_f64;
        let mut total_daily_input = 0.0_f64;
        let mut total_gas_mw = 0.0_f64;
        let mut active = 0_u32;
        let mut closed = 0_u32;
        let mut parks = 0_u32;

        for site in &self.sites {
            match site.status {
                LandfillStatus::Active => {
                    active += 1;
                    total_capacity += site.total_capacity_tons;
                    total_fill += site.current_fill_tons;
                    total_daily_input += site.daily_input_tons;
                    total_gas_mw += site.gas_electricity_mw();
                }
                LandfillStatus::Closed { .. } => {
                    closed += 1;
                    // Closed sites still count toward fill but not capacity for new waste
                }
                LandfillStatus::ConvertedToPark => {
                    parks += 1;
                }
            }
        }

        self.total_capacity_tons = total_capacity;
        self.total_fill_tons = total_fill;
        self.total_remaining_tons = (total_capacity - total_fill).max(0.0);

        self.remaining_pct = if total_capacity > 0.0 {
            (self.total_remaining_tons / total_capacity * 100.0) as f32
        } else {
            0.0
        };

        self.estimated_years_remaining = if total_daily_input > 0.0 {
            (self.total_remaining_tons / total_daily_input) as f32 / DAYS_PER_YEAR
        } else {
            f32::INFINITY
        };

        self.total_daily_input_tons = total_daily_input;
        self.total_gas_electricity_mw = total_gas_mw;
        self.active_sites = active;
        self.closed_sites = closed;
        self.park_sites = parks;
    }
}

// =============================================================================
// Pure helper functions
// =============================================================================

/// Calculate the environmental effect radius for a landfill with given liner type.
/// Returns (odor_radius, land_value_penalty, groundwater_pollution_factor).
pub fn environmental_effects(liner_type: LandfillLinerType) -> (u32, f32, f32) {
    (
        liner_type.odor_radius(),
        liner_type.land_value_penalty(),
        liner_type.groundwater_pollution_factor(),
    )
}

/// Calculate electricity output in MW from landfill gas for a given daily waste input.
/// Returns 0.0 if gas collection is not enabled.
pub fn calculate_gas_electricity(daily_input_tons: f64, has_collection: bool) -> f64 {
    if !has_collection {
        return 0.0;
    }
    daily_input_tons * GAS_COLLECTION_MW_PER_1000_TONS_DAY / 1000.0
}

/// Distribute daily waste input across active landfill sites proportionally
/// to their remaining capacity.
pub fn distribute_waste(sites: &mut [LandfillSite], total_daily_input: f64) {
    let total_remaining: f64 = sites
        .iter()
        .filter(|s| s.status.is_active())
        .map(|s| s.remaining_capacity_tons())
        .sum();

    if total_remaining <= 0.0 {
        return;
    }

    for site in sites.iter_mut() {
        if !site.status.is_active() {
            continue;
        }
        let share = site.remaining_capacity_tons() / total_remaining;
        let daily_input = total_daily_input * share;
        site.advance_fill(daily_input);
    }
}

// =============================================================================
// Bevy system
// =============================================================================

/// Updates landfill state each slow tick.
///
/// 1. Reads daily waste generation from WasteSystem.
/// 2. Distributes waste across active landfill sites proportionally.
/// 3. Advances post-closure monitoring for closed sites.
/// 4. Recomputes aggregate statistics.
pub fn update_landfill_state(
    slow_timer: Res<SlowTickTimer>,
    waste_system: Res<crate::garbage::WasteSystem>,
    mut state: ResMut<LandfillState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let daily_input = waste_system.period_generated_tons;

    // Distribute waste to active sites
    distribute_waste(&mut state.sites, daily_input);

    // Advance closure monitoring for closed sites
    for site in &mut state.sites {
        site.advance_closure();
    }

    // Recompute aggregates
    state.recompute_aggregates();
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for LandfillState {
    const SAVE_KEY: &'static str = "landfill_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.sites.is_empty() && self.next_id == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct LandfillPlugin;

impl Plugin for LandfillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LandfillState>().add_systems(
            FixedUpdate,
            update_landfill_state
                .after(crate::garbage::update_waste_generation)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<LandfillState>();
    }
}
