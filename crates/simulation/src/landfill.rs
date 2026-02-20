//! Landfill Capacity and Environmental Effects (WASTE-005).
//!
//! Models individual landfill sites with finite capacity, environmental effects,
//! and post-closure requirements. Each landfill tracks its fill level, liner type,
//! gas collection status, and closure state.
//!
//! Key features:
//! - **Finite capacity**: Each landfill has `total_capacity_tons` and tracks
//!   `current_fill_tons`. `years_remaining()` estimates lifespan based on
//!   current daily input.
//! - **Environmental effects by liner type**:
//!   - Unlined: high groundwater pollution (0.8), large odor radius (15 cells),
//!     land value penalty -40%.
//!   - Lined: low groundwater pollution (0.2), moderate odor radius (10 cells),
//!     land value penalty -25%.
//!   - LinedWithCollection: minimal groundwater pollution (0.05), small odor
//!     radius (5 cells), land value penalty -15%.
//! - **Landfill gas**: ~1 MW per 1,000 tons/day if gas collection is enabled.
//! - **Post-closure**: When full, landfill must be capped. After capping, a
//!   30-year monitoring period begins. After 30+ years, the site can become a park.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Default capacity for a single landfill site in tons.
pub const DEFAULT_LANDFILL_CAPACITY_TONS: f64 = 500_000.0;

/// Groundwater pollution factor for unlined landfills (0.0-1.0).
pub const GROUNDWATER_POLLUTION_UNLINED: f32 = 0.80;

/// Groundwater pollution factor for lined landfills (0.0-1.0).
pub const GROUNDWATER_POLLUTION_LINED: f32 = 0.20;

/// Groundwater pollution factor for lined landfills with gas collection (0.0-1.0).
pub const GROUNDWATER_POLLUTION_LINED_COLLECTION: f32 = 0.05;

/// Odor radius in grid cells for unlined landfills.
pub const ODOR_RADIUS_UNLINED: u32 = 15;

/// Odor radius in grid cells for lined landfills.
pub const ODOR_RADIUS_LINED: u32 = 10;

/// Odor radius in grid cells for lined landfills with gas collection.
pub const ODOR_RADIUS_LINED_COLLECTION: u32 = 5;

/// Land value penalty fraction for unlined landfills (40%).
pub const LAND_VALUE_PENALTY_UNLINED: f32 = 0.40;

/// Land value penalty fraction for lined landfills (25%).
pub const LAND_VALUE_PENALTY_LINED: f32 = 0.25;

/// Land value penalty fraction for lined landfills with gas collection (15%).
pub const LAND_VALUE_PENALTY_LINED_COLLECTION: f32 = 0.15;

/// Megawatts of electricity generated per 1,000 tons/day of waste with gas collection.
pub const GAS_COLLECTION_MW_PER_1000_TONS_DAY: f64 = 1.0;

/// Number of years of post-closure monitoring required.
pub const POST_CLOSURE_MONITORING_YEARS: u32 = 30;

/// Number of slow ticks per game year (each slow tick ~ 1 game day).
pub const SLOW_TICKS_PER_YEAR: f64 = 365.0;

/// Days per year for years_remaining calculation.
pub const DAYS_PER_YEAR: f32 = 365.0;

// =============================================================================
// Types
// =============================================================================

/// Type of liner installed at a landfill, determining environmental impact.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode,
)]
pub enum LandfillLinerType {
    /// No liner: high groundwater pollution, large odor radius, worst land value impact.
    #[default]
    Unlined,
    /// Synthetic or clay liner: low groundwater pollution, moderate odor radius.
    Lined,
    /// Liner plus active gas collection system: minimal pollution, small odor radius.
    LinedWithCollection,
}

impl LandfillLinerType {
    /// Returns the groundwater pollution factor (0.0-1.0) for this liner type.
    pub fn groundwater_pollution_factor(self) -> f32 {
        match self {
            Self::Unlined => GROUNDWATER_POLLUTION_UNLINED,
            Self::Lined => GROUNDWATER_POLLUTION_LINED,
            Self::LinedWithCollection => GROUNDWATER_POLLUTION_LINED_COLLECTION,
        }
    }

    /// Returns the odor radius in grid cells for this liner type.
    pub fn odor_radius(self) -> u32 {
        match self {
            Self::Unlined => ODOR_RADIUS_UNLINED,
            Self::Lined => ODOR_RADIUS_LINED,
            Self::LinedWithCollection => ODOR_RADIUS_LINED_COLLECTION,
        }
    }

    /// Returns the land value penalty fraction (0.0-1.0) for this liner type.
    pub fn land_value_penalty(self) -> f32 {
        match self {
            Self::Unlined => LAND_VALUE_PENALTY_UNLINED,
            Self::Lined => LAND_VALUE_PENALTY_LINED,
            Self::LinedWithCollection => LAND_VALUE_PENALTY_LINED_COLLECTION,
        }
    }

    /// Returns whether this liner type includes gas collection.
    pub fn has_gas_collection(self) -> bool {
        matches!(self, Self::LinedWithCollection)
    }

    /// Returns a human-readable label for this liner type.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unlined => "Unlined",
            Self::Lined => "Lined",
            Self::LinedWithCollection => "Lined + Gas Collection",
        }
    }
}

/// Status of a landfill site throughout its lifecycle.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode,
)]
pub enum LandfillStatus {
    /// Actively receiving waste.
    #[default]
    Active,
    /// Full and capped, undergoing post-closure monitoring.
    Closed {
        /// Number of slow ticks (game days) since closure.
        days_since_closure: u32,
    },
    /// Post-closure monitoring complete (30+ years). Site converted to park.
    ConvertedToPark,
}

impl LandfillStatus {
    /// Returns a human-readable label for this status.
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Closed { .. } => "Closed (Monitoring)",
            Self::ConvertedToPark => "Converted to Park",
        }
    }

    /// Returns whether this landfill is actively receiving waste.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns the number of years since closure, or 0 if not closed.
    pub fn years_since_closure(self) -> f32 {
        match self {
            Self::Closed { days_since_closure } => days_since_closure as f32 / DAYS_PER_YEAR,
            _ => 0.0,
        }
    }
}

/// Individual landfill site data.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct LandfillSite {
    /// Unique identifier for this landfill site.
    pub id: u32,
    /// Grid X position of this landfill.
    pub grid_x: usize,
    /// Grid Y position of this landfill.
    pub grid_y: usize,
    /// Total capacity of this landfill in tons.
    pub total_capacity_tons: f64,
    /// Current fill level in tons.
    pub current_fill_tons: f64,
    /// Current daily input rate in tons/day (updated each tick).
    pub daily_input_tons: f64,
    /// Type of liner installed.
    pub liner_type: LandfillLinerType,
    /// Current operational status.
    pub status: LandfillStatus,
}

impl LandfillSite {
    /// Create a new active landfill site with default capacity.
    pub fn new(id: u32, grid_x: usize, grid_y: usize) -> Self {
        Self {
            id,
            grid_x,
            grid_y,
            total_capacity_tons: DEFAULT_LANDFILL_CAPACITY_TONS,
            current_fill_tons: 0.0,
            daily_input_tons: 0.0,
            liner_type: LandfillLinerType::default(),
            status: LandfillStatus::Active,
        }
    }

    /// Create a new landfill site with specified capacity and liner type.
    pub fn with_capacity_and_liner(
        id: u32,
        grid_x: usize,
        grid_y: usize,
        capacity: f64,
        liner_type: LandfillLinerType,
    ) -> Self {
        Self {
            id,
            grid_x,
            grid_y,
            total_capacity_tons: capacity,
            current_fill_tons: 0.0,
            daily_input_tons: 0.0,
            liner_type,
            status: LandfillStatus::Active,
        }
    }

    /// Returns remaining capacity in tons.
    pub fn remaining_capacity_tons(&self) -> f64 {
        (self.total_capacity_tons - self.current_fill_tons).max(0.0)
    }

    /// Returns remaining capacity as a percentage (0.0 to 100.0).
    pub fn remaining_capacity_pct(&self) -> f64 {
        if self.total_capacity_tons <= 0.0 {
            return 0.0;
        }
        (self.remaining_capacity_tons() / self.total_capacity_tons * 100.0).clamp(0.0, 100.0)
    }

    /// Returns estimated days remaining at current daily input rate.
    /// Returns `f32::INFINITY` if daily input is zero or negative.
    pub fn days_remaining(&self) -> f32 {
        if self.daily_input_tons <= 0.0 {
            return f32::INFINITY;
        }
        (self.remaining_capacity_tons() / self.daily_input_tons) as f32
    }

    /// Returns estimated years remaining at current daily input rate.
    pub fn years_remaining(&self) -> f32 {
        self.days_remaining() / DAYS_PER_YEAR
    }

    /// Returns whether this landfill is full (no remaining capacity).
    pub fn is_full(&self) -> bool {
        self.current_fill_tons >= self.total_capacity_tons
    }

    /// Returns the electricity generated in MW from landfill gas, if gas collection is enabled.
    /// Conversion: 1 MW per 1,000 tons/day of waste input.
    pub fn gas_electricity_mw(&self) -> f64 {
        if !self.liner_type.has_gas_collection() {
            return 0.0;
        }
        self.daily_input_tons * GAS_COLLECTION_MW_PER_1000_TONS_DAY / 1000.0
    }

    /// Advance fill by one day's input. Clamps at total capacity.
    /// If the landfill becomes full, transitions to Closed status.
    pub fn advance_fill(&mut self, daily_input: f64) {
        if !self.status.is_active() {
            return;
        }
        self.daily_input_tons = daily_input;
        self.current_fill_tons =
            (self.current_fill_tons + daily_input).min(self.total_capacity_tons);
        if self.is_full() {
            self.status = LandfillStatus::Closed {
                days_since_closure: 0,
            };
        }
    }

    /// Advance post-closure monitoring by one day.
    /// After POST_CLOSURE_MONITORING_YEARS * 365 days, transitions to ConvertedToPark.
    pub fn advance_closure(&mut self) {
        if let LandfillStatus::Closed { days_since_closure } = &mut self.status {
            *days_since_closure += 1;
            let monitoring_days = POST_CLOSURE_MONITORING_YEARS as f32 * DAYS_PER_YEAR;
            if *days_since_closure as f32 >= monitoring_days {
                self.status = LandfillStatus::ConvertedToPark;
            }
        }
    }
}

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
        bitcode::decode(bytes).unwrap_or_default()
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
            update_landfill_state.after(crate::garbage::update_waste_generation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<LandfillState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // LandfillLinerType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_liner_type_default_is_unlined() {
        assert_eq!(LandfillLinerType::default(), LandfillLinerType::Unlined);
    }

    #[test]
    fn test_unlined_groundwater_pollution() {
        let factor = LandfillLinerType::Unlined.groundwater_pollution_factor();
        assert!((factor - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_groundwater_pollution() {
        let factor = LandfillLinerType::Lined.groundwater_pollution_factor();
        assert!((factor - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_collection_groundwater_pollution() {
        let factor = LandfillLinerType::LinedWithCollection.groundwater_pollution_factor();
        assert!((factor - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pollution_decreases_with_better_liner() {
        let unlined = LandfillLinerType::Unlined.groundwater_pollution_factor();
        let lined = LandfillLinerType::Lined.groundwater_pollution_factor();
        let collection = LandfillLinerType::LinedWithCollection.groundwater_pollution_factor();
        assert!(unlined > lined);
        assert!(lined > collection);
    }

    #[test]
    fn test_unlined_odor_radius_15() {
        assert_eq!(LandfillLinerType::Unlined.odor_radius(), 15);
    }

    #[test]
    fn test_lined_odor_radius_10() {
        assert_eq!(LandfillLinerType::Lined.odor_radius(), 10);
    }

    #[test]
    fn test_lined_collection_odor_radius_5() {
        assert_eq!(LandfillLinerType::LinedWithCollection.odor_radius(), 5);
    }

    #[test]
    fn test_odor_radius_decreases_with_better_liner() {
        let unlined = LandfillLinerType::Unlined.odor_radius();
        let lined = LandfillLinerType::Lined.odor_radius();
        let collection = LandfillLinerType::LinedWithCollection.odor_radius();
        assert!(unlined > lined);
        assert!(lined > collection);
    }

    #[test]
    fn test_unlined_land_value_penalty_40pct() {
        let penalty = LandfillLinerType::Unlined.land_value_penalty();
        assert!((penalty - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_land_value_penalty_25pct() {
        let penalty = LandfillLinerType::Lined.land_value_penalty();
        assert!((penalty - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_collection_land_value_penalty_15pct() {
        let penalty = LandfillLinerType::LinedWithCollection.land_value_penalty();
        assert!((penalty - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_land_value_penalty_decreases_with_better_liner() {
        let unlined = LandfillLinerType::Unlined.land_value_penalty();
        let lined = LandfillLinerType::Lined.land_value_penalty();
        let collection = LandfillLinerType::LinedWithCollection.land_value_penalty();
        assert!(unlined > lined);
        assert!(lined > collection);
    }

    #[test]
    fn test_gas_collection_only_on_lined_with_collection() {
        assert!(!LandfillLinerType::Unlined.has_gas_collection());
        assert!(!LandfillLinerType::Lined.has_gas_collection());
        assert!(LandfillLinerType::LinedWithCollection.has_gas_collection());
    }

    #[test]
    fn test_liner_labels() {
        assert_eq!(LandfillLinerType::Unlined.label(), "Unlined");
        assert_eq!(LandfillLinerType::Lined.label(), "Lined");
        assert_eq!(
            LandfillLinerType::LinedWithCollection.label(),
            "Lined + Gas Collection"
        );
    }

    // -------------------------------------------------------------------------
    // LandfillStatus tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_status_default_is_active() {
        assert_eq!(LandfillStatus::default(), LandfillStatus::Active);
    }

    #[test]
    fn test_active_is_active() {
        assert!(LandfillStatus::Active.is_active());
    }

    #[test]
    fn test_closed_is_not_active() {
        let status = LandfillStatus::Closed {
            days_since_closure: 100,
        };
        assert!(!status.is_active());
    }

    #[test]
    fn test_park_is_not_active() {
        assert!(!LandfillStatus::ConvertedToPark.is_active());
    }

    #[test]
    fn test_status_labels() {
        assert_eq!(LandfillStatus::Active.label(), "Active");
        assert_eq!(
            LandfillStatus::Closed {
                days_since_closure: 0
            }
            .label(),
            "Closed (Monitoring)"
        );
        assert_eq!(LandfillStatus::ConvertedToPark.label(), "Converted to Park");
    }

    #[test]
    fn test_years_since_closure_active() {
        assert!((LandfillStatus::Active.years_since_closure()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_years_since_closure_closed() {
        let status = LandfillStatus::Closed {
            days_since_closure: 365,
        };
        assert!((status.years_since_closure() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_years_since_closure_park() {
        assert!((LandfillStatus::ConvertedToPark.years_since_closure()).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // LandfillSite tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_new_site_defaults() {
        let site = LandfillSite::new(0, 10, 20);
        assert_eq!(site.id, 0);
        assert_eq!(site.grid_x, 10);
        assert_eq!(site.grid_y, 20);
        assert!((site.total_capacity_tons - DEFAULT_LANDFILL_CAPACITY_TONS).abs() < f64::EPSILON);
        assert!((site.current_fill_tons).abs() < f64::EPSILON);
        assert!((site.daily_input_tons).abs() < f64::EPSILON);
        assert_eq!(site.liner_type, LandfillLinerType::Unlined);
        assert!(site.status.is_active());
    }

    #[test]
    fn test_site_with_capacity_and_liner() {
        let site = LandfillSite::with_capacity_and_liner(
            1,
            5,
            10,
            1_000_000.0,
            LandfillLinerType::LinedWithCollection,
        );
        assert_eq!(site.id, 1);
        assert!((site.total_capacity_tons - 1_000_000.0).abs() < f64::EPSILON);
        assert_eq!(site.liner_type, LandfillLinerType::LinedWithCollection);
    }

    #[test]
    fn test_remaining_capacity_empty() {
        let site = LandfillSite::new(0, 0, 0);
        assert!(
            (site.remaining_capacity_tons() - DEFAULT_LANDFILL_CAPACITY_TONS).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_remaining_capacity_half_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS / 2.0;
        let expected = DEFAULT_LANDFILL_CAPACITY_TONS / 2.0;
        assert!((site.remaining_capacity_tons() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS;
        assert!((site.remaining_capacity_tons()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_pct_empty() {
        let site = LandfillSite::new(0, 0, 0);
        assert!((site.remaining_capacity_pct() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_pct_half() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS / 2.0;
        assert!((site.remaining_capacity_pct() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_remaining_capacity_pct_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS;
        assert!((site.remaining_capacity_pct()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_pct_zero_capacity() {
        let site = LandfillSite::with_capacity_and_liner(0, 0, 0, 0.0, LandfillLinerType::Unlined);
        assert!((site.remaining_capacity_pct()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_days_remaining_with_input() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.daily_input_tons = 1000.0;
        // 500,000 / 1000 = 500 days
        assert!((site.days_remaining() - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_days_remaining_zero_input() {
        let site = LandfillSite::new(0, 0, 0);
        assert!(site.days_remaining().is_infinite());
    }

    #[test]
    fn test_years_remaining_with_input() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.daily_input_tons = 1000.0;
        // 500,000 / 1000 = 500 days / 365 = ~1.37 years
        let expected = 500.0 / 365.0;
        assert!((site.years_remaining() - expected).abs() < 0.01);
    }

    #[test]
    fn test_is_full_false_when_empty() {
        let site = LandfillSite::new(0, 0, 0);
        assert!(!site.is_full());
    }

    #[test]
    fn test_is_full_true_when_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS;
        assert!(site.is_full());
    }

    #[test]
    fn test_gas_electricity_no_collection() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.daily_input_tons = 1000.0;
        assert!((site.gas_electricity_mw()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_with_collection_1000_tons() {
        let mut site = LandfillSite::with_capacity_and_liner(
            0,
            0,
            0,
            1_000_000.0,
            LandfillLinerType::LinedWithCollection,
        );
        site.daily_input_tons = 1000.0;
        // 1000 * 1.0 / 1000 = 1.0 MW
        assert!((site.gas_electricity_mw() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_with_collection_500_tons() {
        let mut site = LandfillSite::with_capacity_and_liner(
            0,
            0,
            0,
            1_000_000.0,
            LandfillLinerType::LinedWithCollection,
        );
        site.daily_input_tons = 500.0;
        // 500 * 1.0 / 1000 = 0.5 MW
        assert!((site.gas_electricity_mw() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_fill_normal() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.advance_fill(1000.0);
        assert!((site.current_fill_tons - 1000.0).abs() < f64::EPSILON);
        assert!((site.daily_input_tons - 1000.0).abs() < f64::EPSILON);
        assert!(site.status.is_active());
    }

    #[test]
    fn test_advance_fill_clamps_at_capacity() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined);
        site.advance_fill(150.0);
        assert!((site.current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_fill_triggers_closure_when_full() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined);
        site.advance_fill(100.0);
        assert!(!site.status.is_active());
        match site.status {
            LandfillStatus::Closed { days_since_closure } => assert_eq!(days_since_closure, 0),
            _ => panic!("Expected Closed status"),
        }
    }

    #[test]
    fn test_advance_fill_noop_when_closed() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined);
        site.current_fill_tons = 100.0;
        site.status = LandfillStatus::Closed {
            days_since_closure: 10,
        };
        site.advance_fill(50.0);
        // Fill should not change
        assert!((site.current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_fill_noop_when_park() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.status = LandfillStatus::ConvertedToPark;
        site.advance_fill(50.0);
        assert!((site.current_fill_tons).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_closure_increments_days() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.status = LandfillStatus::Closed {
            days_since_closure: 0,
        };
        site.advance_closure();
        match site.status {
            LandfillStatus::Closed { days_since_closure } => assert_eq!(days_since_closure, 1),
            _ => panic!("Expected Closed status"),
        }
    }

    #[test]
    fn test_advance_closure_converts_to_park_after_30_years() {
        let mut site = LandfillSite::new(0, 0, 0);
        let monitoring_days = (POST_CLOSURE_MONITORING_YEARS as f32 * DAYS_PER_YEAR) as u32;
        site.status = LandfillStatus::Closed {
            days_since_closure: monitoring_days - 1,
        };
        site.advance_closure();
        assert_eq!(site.status, LandfillStatus::ConvertedToPark);
    }

    #[test]
    fn test_advance_closure_noop_when_active() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.advance_closure();
        assert!(site.status.is_active());
    }

    #[test]
    fn test_advance_closure_noop_when_park() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.status = LandfillStatus::ConvertedToPark;
        site.advance_closure();
        assert_eq!(site.status, LandfillStatus::ConvertedToPark);
    }

    // -------------------------------------------------------------------------
    // LandfillState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = LandfillState::default();
        assert!(state.sites.is_empty());
        assert_eq!(state.next_id, 0);
        assert!((state.total_capacity_tons).abs() < f64::EPSILON);
        assert!((state.total_fill_tons).abs() < f64::EPSILON);
        assert_eq!(state.active_sites, 0);
        assert_eq!(state.closed_sites, 0);
        assert_eq!(state.park_sites, 0);
    }

    #[test]
    fn test_add_site() {
        let mut state = LandfillState::default();
        let id = state.add_site(10, 20);
        assert_eq!(id, 0);
        assert_eq!(state.sites.len(), 1);
        assert_eq!(state.next_id, 1);
        assert_eq!(state.sites[0].grid_x, 10);
        assert_eq!(state.sites[0].grid_y, 20);
    }

    #[test]
    fn test_add_site_increments_id() {
        let mut state = LandfillState::default();
        let id1 = state.add_site(0, 0);
        let id2 = state.add_site(1, 1);
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(state.sites.len(), 2);
    }

    #[test]
    fn test_add_site_with_options() {
        let mut state = LandfillState::default();
        let id =
            state.add_site_with_options(5, 10, 1_000_000.0, LandfillLinerType::LinedWithCollection);
        assert_eq!(id, 0);
        let site = state.get_site(0).unwrap();
        assert!((site.total_capacity_tons - 1_000_000.0).abs() < f64::EPSILON);
        assert_eq!(site.liner_type, LandfillLinerType::LinedWithCollection);
    }

    #[test]
    fn test_get_site() {
        let mut state = LandfillState::default();
        state.add_site(10, 20);
        let site = state.get_site(0);
        assert!(site.is_some());
        assert_eq!(site.unwrap().grid_x, 10);
    }

    #[test]
    fn test_get_site_not_found() {
        let state = LandfillState::default();
        assert!(state.get_site(99).is_none());
    }

    #[test]
    fn test_get_site_mut() {
        let mut state = LandfillState::default();
        state.add_site(10, 20);
        let site = state.get_site_mut(0).unwrap();
        site.current_fill_tons = 1000.0;
        assert!((state.sites[0].current_fill_tons - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_aggregates_empty() {
        let mut state = LandfillState::default();
        state.recompute_aggregates();
        assert!((state.total_capacity_tons).abs() < f64::EPSILON);
        assert_eq!(state.active_sites, 0);
        assert_eq!(state.remaining_pct, 0.0);
    }

    #[test]
    fn test_recompute_aggregates_active_sites() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.add_site(1, 1);
        state.sites[0].daily_input_tons = 100.0;
        state.sites[1].daily_input_tons = 200.0;
        state.sites[0].current_fill_tons = 100_000.0;

        state.recompute_aggregates();

        assert!(
            (state.total_capacity_tons - 2.0 * DEFAULT_LANDFILL_CAPACITY_TONS).abs() < f64::EPSILON
        );
        assert!((state.total_fill_tons - 100_000.0).abs() < f64::EPSILON);
        assert_eq!(state.active_sites, 2);
        assert!((state.total_daily_input_tons - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_aggregates_closed_sites() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].status = LandfillStatus::Closed {
            days_since_closure: 100,
        };

        state.recompute_aggregates();

        assert_eq!(state.active_sites, 0);
        assert_eq!(state.closed_sites, 1);
        // Closed sites don't contribute to active capacity
        assert!((state.total_capacity_tons).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_aggregates_park_sites() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].status = LandfillStatus::ConvertedToPark;

        state.recompute_aggregates();

        assert_eq!(state.park_sites, 1);
        assert_eq!(state.active_sites, 0);
        assert_eq!(state.closed_sites, 0);
    }

    #[test]
    fn test_recompute_gas_electricity() {
        let mut state = LandfillState::default();
        state.add_site_with_options(0, 0, 1_000_000.0, LandfillLinerType::LinedWithCollection);
        state.sites[0].daily_input_tons = 1000.0;

        state.recompute_aggregates();

        // 1000 tons/day * 1.0 MW / 1000 = 1.0 MW
        assert!((state.total_gas_electricity_mw - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_remaining_pct() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS * 0.75;

        state.recompute_aggregates();

        assert!((state.remaining_pct - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_recompute_years_remaining() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].daily_input_tons = 1000.0;

        state.recompute_aggregates();

        // 500,000 tons / 1000 tons/day / 365 days/year = ~1.37 years
        let expected = 500_000.0 / 1000.0 / 365.0;
        assert!((state.estimated_years_remaining - expected as f32).abs() < 0.01);
    }

    #[test]
    fn test_recompute_years_remaining_no_input() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);

        state.recompute_aggregates();

        assert!(state.estimated_years_remaining.is_infinite());
    }

    // -------------------------------------------------------------------------
    // distribute_waste tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_distribute_waste_single_site() {
        let mut sites = vec![LandfillSite::new(0, 0, 0)];
        distribute_waste(&mut sites, 100.0);
        assert!((sites[0].current_fill_tons - 100.0).abs() < f64::EPSILON);
        assert!((sites[0].daily_input_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distribute_waste_two_equal_sites() {
        let mut sites = vec![LandfillSite::new(0, 0, 0), LandfillSite::new(1, 1, 1)];
        distribute_waste(&mut sites, 200.0);
        // Both have equal remaining capacity, so waste should split evenly
        assert!((sites[0].current_fill_tons - 100.0).abs() < 0.01);
        assert!((sites[1].current_fill_tons - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_distribute_waste_proportional() {
        let mut sites = vec![
            LandfillSite::with_capacity_and_liner(0, 0, 0, 300_000.0, LandfillLinerType::Unlined),
            LandfillSite::with_capacity_and_liner(1, 1, 1, 100_000.0, LandfillLinerType::Unlined),
        ];
        distribute_waste(&mut sites, 400.0);
        // 300k / 400k = 75% -> 300 tons
        // 100k / 400k = 25% -> 100 tons
        assert!((sites[0].current_fill_tons - 300.0).abs() < 0.01);
        assert!((sites[1].current_fill_tons - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_distribute_waste_skips_closed_sites() {
        let mut sites = vec![LandfillSite::new(0, 0, 0), LandfillSite::new(1, 1, 1)];
        sites[0].status = LandfillStatus::Closed {
            days_since_closure: 0,
        };
        distribute_waste(&mut sites, 100.0);
        assert!((sites[0].current_fill_tons).abs() < f64::EPSILON);
        assert!((sites[1].current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distribute_waste_no_active_sites() {
        let mut sites = vec![LandfillSite::new(0, 0, 0)];
        sites[0].status = LandfillStatus::ConvertedToPark;
        distribute_waste(&mut sites, 100.0);
        // Nothing should change
        assert!((sites[0].current_fill_tons).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distribute_waste_zero_input() {
        let mut sites = vec![LandfillSite::new(0, 0, 0)];
        distribute_waste(&mut sites, 0.0);
        assert!((sites[0].current_fill_tons).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // environmental_effects tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_environmental_effects_unlined() {
        let (odor, penalty, pollution) = environmental_effects(LandfillLinerType::Unlined);
        assert_eq!(odor, 15);
        assert!((penalty - 0.40).abs() < f32::EPSILON);
        assert!((pollution - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_environmental_effects_lined() {
        let (odor, penalty, pollution) = environmental_effects(LandfillLinerType::Lined);
        assert_eq!(odor, 10);
        assert!((penalty - 0.25).abs() < f32::EPSILON);
        assert!((pollution - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_environmental_effects_lined_collection() {
        let (odor, penalty, pollution) =
            environmental_effects(LandfillLinerType::LinedWithCollection);
        assert_eq!(odor, 5);
        assert!((penalty - 0.15).abs() < f32::EPSILON);
        assert!((pollution - 0.05).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // calculate_gas_electricity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gas_electricity_no_collection_fn() {
        let mw = calculate_gas_electricity(1000.0, false);
        assert!((mw).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_with_collection_fn() {
        let mw = calculate_gas_electricity(1000.0, true);
        assert!((mw - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_2000_tons() {
        let mw = calculate_gas_electricity(2000.0, true);
        assert!((mw - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_zero_input() {
        let mw = calculate_gas_electricity(0.0, true);
        assert!((mw).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(LandfillState::SAVE_KEY, "landfill_state");
    }

    #[test]
    fn test_saveable_skips_empty() {
        use crate::Saveable;
        let state = LandfillState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_with_sites() {
        use crate::Saveable;
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = LandfillState::default();
        state.add_site_with_options(5, 10, 1_000_000.0, LandfillLinerType::LinedWithCollection);
        state.sites[0].current_fill_tons = 500_000.0;
        state.sites[0].daily_input_tons = 100.0;
        state.add_site(20, 30);
        state.sites[1].current_fill_tons = 200_000.0;
        state.sites[1].status = LandfillStatus::Closed {
            days_since_closure: 365,
        };
        state.recompute_aggregates();

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = LandfillState::load_from_bytes(&bytes);

        assert_eq!(restored.sites.len(), 2);
        assert_eq!(restored.next_id, 2);
        assert!((restored.sites[0].current_fill_tons - 500_000.0).abs() < f64::EPSILON);
        assert_eq!(
            restored.sites[0].liner_type,
            LandfillLinerType::LinedWithCollection
        );
        assert!(!restored.sites[1].status.is_active());
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_lifecycle_fill_close_monitor_park() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 1000.0, LandfillLinerType::Lined);

        // Fill the landfill over time
        for _ in 0..9 {
            site.advance_fill(100.0);
            assert!(site.status.is_active());
        }
        assert!((site.current_fill_tons - 900.0).abs() < f64::EPSILON);

        // Fill to capacity - should trigger closure
        site.advance_fill(100.0);
        assert!(!site.status.is_active());
        assert!((site.current_fill_tons - 1000.0).abs() < f64::EPSILON);

        // Advance through 30 years of monitoring
        let monitoring_days = (POST_CLOSURE_MONITORING_YEARS as f32 * DAYS_PER_YEAR) as u32;
        for _ in 0..monitoring_days - 1 {
            site.advance_closure();
            assert!(!site.status.is_active());
            assert_ne!(site.status, LandfillStatus::ConvertedToPark);
        }

        // Last day of monitoring - should convert to park
        site.advance_closure();
        assert_eq!(site.status, LandfillStatus::ConvertedToPark);
    }

    #[test]
    fn test_multiple_sites_one_fills_other_takes_over() {
        let mut sites = vec![
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined),
            LandfillSite::with_capacity_and_liner(1, 1, 1, 1000.0, LandfillLinerType::Lined),
        ];

        // First distribution: proportional to remaining capacity
        distribute_waste(&mut sites, 200.0);

        // Site 0 should get ~100/1100 * 200 = ~18.18 tons
        // Site 1 should get ~1000/1100 * 200 = ~181.82 tons
        let total = sites[0].current_fill_tons + sites[1].current_fill_tons;
        assert!((total - 200.0).abs() < 0.01);

        // Fill site 0 completely
        sites[0].current_fill_tons = 100.0;
        sites[0].status = LandfillStatus::Closed {
            days_since_closure: 0,
        };

        // Now all waste goes to site 1
        distribute_waste(&mut sites, 100.0);
        // Site 0 should not receive more waste
        assert!((sites[0].current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_constant_values_match_spec() {
        // Verify all constants match the issue specification
        assert_eq!(ODOR_RADIUS_UNLINED, 15);
        assert_eq!(ODOR_RADIUS_LINED, 10);
        assert_eq!(ODOR_RADIUS_LINED_COLLECTION, 5);
        assert!((LAND_VALUE_PENALTY_UNLINED - 0.40).abs() < f32::EPSILON);
        assert!((LAND_VALUE_PENALTY_LINED_COLLECTION - 0.15).abs() < f32::EPSILON);
        assert_eq!(POST_CLOSURE_MONITORING_YEARS, 30);
        assert!((GAS_COLLECTION_MW_PER_1000_TONS_DAY - 1.0).abs() < f64::EPSILON);
    }
}
