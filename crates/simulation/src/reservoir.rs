use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::water_demand::WaterSupply;
use crate::water_sources::{WaterSource, WaterSourceType};
use crate::weather::Weather;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Rainfall intensity (in/hr) to MGD conversion factor for catchment areas.
const CATCHMENT_FACTOR: f32 = 0.001;

/// Base evaporation rate in MGD per reservoir per day.
const BASE_EVAPORATION_RATE: f32 = 0.005;

/// Additional evaporation per degree Celsius above 20C (MGD per reservoir).
const TEMPERATURE_EVAP_FACTOR: f32 = 0.03;

/// Minimum reserve percentage (below this triggers Critical tier).
const MIN_RESERVE_PCT: f32 = 0.20;

/// Gallons per million gallons.
const MGD_TO_GPD: f32 = 1_000_000.0;

// =============================================================================
// Types
// =============================================================================

/// Warning tier based on reservoir fill percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ReservoirWarningTier {
    /// Fill > 50%: operating normally.
    #[default]
    Normal,
    /// Fill 30%-50%: conservation advisories.
    Watch,
    /// Fill 20%-30%: mandatory restrictions.
    Warning,
    /// Fill <= 20% (at or below min reserve): emergency measures.
    Critical,
}

impl ReservoirWarningTier {
    /// Human-readable name for UI display.
    pub fn name(self) -> &'static str {
        match self {
            ReservoirWarningTier::Normal => "Normal",
            ReservoirWarningTier::Watch => "Watch",
            ReservoirWarningTier::Warning => "Warning",
            ReservoirWarningTier::Critical => "Critical",
        }
    }
}

/// Determine the warning tier from a fill percentage (0.0 to 1.0).
pub fn warning_tier_from_fill(fill_pct: f32) -> ReservoirWarningTier {
    if fill_pct > 0.50 {
        ReservoirWarningTier::Normal
    } else if fill_pct > 0.30 {
        ReservoirWarningTier::Watch
    } else if fill_pct > MIN_RESERVE_PCT {
        ReservoirWarningTier::Warning
    } else {
        ReservoirWarningTier::Critical
    }
}

/// Event fired when the reservoir warning tier changes.
#[derive(Event, Debug, Clone)]
pub struct ReservoirWarningEvent {
    /// The previous warning tier.
    pub old_tier: ReservoirWarningTier,
    /// The new warning tier.
    pub new_tier: ReservoirWarningTier,
    /// Current fill percentage when the event was fired (0.0 to 1.0).
    pub fill_pct: f32,
}

/// City-wide reservoir statistics resource.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ReservoirState {
    /// Total storage capacity across all reservoirs (million gallons).
    pub total_storage_capacity_mg: f32,
    /// Current total water level across all reservoirs (million gallons).
    pub current_level_mg: f32,
    /// Inflow rate from rainfall catchment (million gallons per day).
    pub inflow_rate_mgd: f32,
    /// Outflow rate from water demand extraction (million gallons per day).
    pub outflow_rate_mgd: f32,
    /// Evaporation rate (million gallons per day).
    pub evaporation_rate_mgd: f32,
    /// Net change = inflow - outflow - evaporation (million gallons per day).
    pub net_change_mgd: f32,
    /// Days of supply remaining at current demand rate.
    pub storage_days: f32,
    /// Number of active reservoir entities.
    pub reservoir_count: u32,
    /// Current warning tier based on fill percentage.
    pub warning_tier: ReservoirWarningTier,
    /// Minimum reserve percentage threshold (default 0.20 = 20%).
    pub min_reserve_pct: f32,
}

impl Default for ReservoirState {
    fn default() -> Self {
        Self {
            total_storage_capacity_mg: 0.0,
            current_level_mg: 0.0,
            inflow_rate_mgd: 0.0,
            outflow_rate_mgd: 0.0,
            evaporation_rate_mgd: 0.0,
            net_change_mgd: 0.0,
            storage_days: 0.0,
            reservoir_count: 0,
            warning_tier: ReservoirWarningTier::Normal,
            min_reserve_pct: MIN_RESERVE_PCT,
        }
    }
}

impl ReservoirState {
    /// Current fill percentage (0.0 to 1.0). Returns 0.0 if no capacity.
    pub fn fill_pct(&self) -> f32 {
        if self.total_storage_capacity_mg > 0.0 {
            self.current_level_mg / self.total_storage_capacity_mg
        } else {
            0.0
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

/// System: Update reservoir levels based on rainfall inflow, demand outflow,
/// and evaporation. Fires `ReservoirWarningEvent` when the warning tier changes.
///
/// Runs on the SlowTickTimer.
pub fn update_reservoir_levels(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    water_supply: Res<WaterSupply>,
    mut reservoir_state: ResMut<ReservoirState>,
    mut sources: Query<&mut WaterSource>,
    mut warning_events: EventWriter<ReservoirWarningEvent>,
) {
    if !timer.should_run() {
        return;
    }

    // ---- Step 1-2: Find reservoirs, sum capacity and current levels ----
    let mut total_capacity_gallons: f32 = 0.0;
    let mut total_stored_gallons: f32 = 0.0;
    let mut reservoir_count: u32 = 0;

    for source in sources.iter() {
        if source.source_type != WaterSourceType::Reservoir {
            continue;
        }
        total_capacity_gallons += source.storage_capacity;
        total_stored_gallons += source.stored_gallons;
        reservoir_count += 1;
    }

    reservoir_state.reservoir_count = reservoir_count;

    // Convert gallons to million gallons for the resource.
    reservoir_state.total_storage_capacity_mg = total_capacity_gallons / MGD_TO_GPD;
    reservoir_state.current_level_mg = total_stored_gallons / MGD_TO_GPD;

    // If there are no reservoirs, zero everything out and return early.
    if reservoir_count == 0 {
        reservoir_state.inflow_rate_mgd = 0.0;
        reservoir_state.outflow_rate_mgd = 0.0;
        reservoir_state.evaporation_rate_mgd = 0.0;
        reservoir_state.net_change_mgd = 0.0;
        reservoir_state.storage_days = 0.0;
        // Tier stays Normal when there are no reservoirs.
        let old_tier = reservoir_state.warning_tier;
        reservoir_state.warning_tier = ReservoirWarningTier::Normal;
        if old_tier != ReservoirWarningTier::Normal {
            warning_events.send(ReservoirWarningEvent {
                old_tier,
                new_tier: ReservoirWarningTier::Normal,
                fill_pct: 0.0,
            });
        }
        return;
    }

    // ---- Step 3: Calculate inflow from rainfall ----
    // precipitation_intensity is in inches/hour. CATCHMENT_FACTOR converts to MGD.
    let inflow_mgd = weather.precipitation_intensity * CATCHMENT_FACTOR * reservoir_count as f32;

    // ---- Step 4: Calculate outflow from water demand ----
    // WaterSupply.total_demand_gpd is in gallons per day; convert to MGD.
    let outflow_mgd = water_supply.total_demand_gpd / MGD_TO_GPD;

    // ---- Step 5: Calculate evaporation ----
    let temp_above_20 = (weather.temperature - 20.0).max(0.0);
    let evaporation_mgd =
        reservoir_count as f32 * (BASE_EVAPORATION_RATE + temp_above_20 * TEMPERATURE_EVAP_FACTOR);

    // ---- Step 6: Net change ----
    let net_change_mgd = inflow_mgd - outflow_mgd - evaporation_mgd;

    // Store rates on the resource.
    reservoir_state.inflow_rate_mgd = inflow_mgd;
    reservoir_state.outflow_rate_mgd = outflow_mgd;
    reservoir_state.evaporation_rate_mgd = evaporation_mgd;
    reservoir_state.net_change_mgd = net_change_mgd;

    // ---- Step 7: Distribute net change to each reservoir proportionally ----
    let net_change_gallons = net_change_mgd * MGD_TO_GPD;
    let mut new_total_stored: f32 = 0.0;

    for mut source in &mut sources {
        if source.source_type != WaterSourceType::Reservoir {
            continue;
        }
        // Distribute proportionally to each reservoir's share of total capacity.
        let share = if total_capacity_gallons > 0.0 {
            source.storage_capacity / total_capacity_gallons
        } else {
            0.0
        };
        let delta = net_change_gallons * share;
        source.stored_gallons = (source.stored_gallons + delta).clamp(0.0, source.storage_capacity);
        new_total_stored += source.stored_gallons;
    }

    // Update resource with post-distribution totals.
    reservoir_state.current_level_mg = new_total_stored / MGD_TO_GPD;

    // ---- Step 8: Determine warning tier from fill percentage ----
    let fill_pct = reservoir_state.fill_pct();
    let new_tier = warning_tier_from_fill(fill_pct);

    // ---- Step 9: Fire event when tier changes ----
    let old_tier = reservoir_state.warning_tier;
    reservoir_state.warning_tier = new_tier;

    // Calculate storage days: current level / daily demand.
    reservoir_state.storage_days = if outflow_mgd > 0.0 {
        reservoir_state.current_level_mg / outflow_mgd
    } else {
        f32::INFINITY
    };

    if old_tier != new_tier {
        warning_events.send(ReservoirWarningEvent {
            old_tier,
            new_tier,
            fill_pct,
        });
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Warning tier tests
    // =========================================================================

    #[test]
    fn test_tier_normal_above_50_pct() {
        assert_eq!(warning_tier_from_fill(1.0), ReservoirWarningTier::Normal);
        assert_eq!(warning_tier_from_fill(0.75), ReservoirWarningTier::Normal);
        assert_eq!(warning_tier_from_fill(0.51), ReservoirWarningTier::Normal);
    }

    #[test]
    fn test_tier_watch_30_to_50_pct() {
        assert_eq!(warning_tier_from_fill(0.50), ReservoirWarningTier::Watch);
        assert_eq!(warning_tier_from_fill(0.40), ReservoirWarningTier::Watch);
        assert_eq!(warning_tier_from_fill(0.31), ReservoirWarningTier::Watch);
    }

    #[test]
    fn test_tier_warning_20_to_30_pct() {
        assert_eq!(warning_tier_from_fill(0.30), ReservoirWarningTier::Warning);
        assert_eq!(warning_tier_from_fill(0.25), ReservoirWarningTier::Warning);
        assert_eq!(warning_tier_from_fill(0.21), ReservoirWarningTier::Warning);
    }

    #[test]
    fn test_tier_critical_at_or_below_20_pct() {
        assert_eq!(warning_tier_from_fill(0.20), ReservoirWarningTier::Critical);
        assert_eq!(warning_tier_from_fill(0.10), ReservoirWarningTier::Critical);
        assert_eq!(warning_tier_from_fill(0.0), ReservoirWarningTier::Critical);
    }

    #[test]
    fn test_tier_boundary_exactly_50() {
        // 50% is the boundary between Normal and Watch: <= 50% is Watch.
        assert_eq!(warning_tier_from_fill(0.50), ReservoirWarningTier::Watch);
    }

    #[test]
    fn test_tier_boundary_exactly_30() {
        assert_eq!(warning_tier_from_fill(0.30), ReservoirWarningTier::Warning);
    }

    #[test]
    fn test_tier_boundary_exactly_20() {
        assert_eq!(warning_tier_from_fill(0.20), ReservoirWarningTier::Critical);
    }

    // =========================================================================
    // ReservoirWarningTier name tests
    // =========================================================================

    #[test]
    fn test_tier_names() {
        assert_eq!(ReservoirWarningTier::Normal.name(), "Normal");
        assert_eq!(ReservoirWarningTier::Watch.name(), "Watch");
        assert_eq!(ReservoirWarningTier::Warning.name(), "Warning");
        assert_eq!(ReservoirWarningTier::Critical.name(), "Critical");
    }

    // =========================================================================
    // ReservoirState default and fill_pct tests
    // =========================================================================

    #[test]
    fn test_default_state() {
        let state = ReservoirState::default();
        assert!((state.total_storage_capacity_mg - 0.0).abs() < f32::EPSILON);
        assert!((state.current_level_mg - 0.0).abs() < f32::EPSILON);
        assert!((state.inflow_rate_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.outflow_rate_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.evaporation_rate_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.net_change_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.storage_days - 0.0).abs() < f32::EPSILON);
        assert_eq!(state.reservoir_count, 0);
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!((state.min_reserve_pct - MIN_RESERVE_PCT).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_full() {
        let state = ReservoirState {
            total_storage_capacity_mg: 100.0,
            current_level_mg: 100.0,
            ..Default::default()
        };
        assert!((state.fill_pct() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_half() {
        let state = ReservoirState {
            total_storage_capacity_mg: 100.0,
            current_level_mg: 50.0,
            ..Default::default()
        };
        assert!((state.fill_pct() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_empty() {
        let state = ReservoirState {
            total_storage_capacity_mg: 100.0,
            current_level_mg: 0.0,
            ..Default::default()
        };
        assert!((state.fill_pct() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_no_capacity_returns_zero() {
        let state = ReservoirState {
            total_storage_capacity_mg: 0.0,
            current_level_mg: 0.0,
            ..Default::default()
        };
        assert!((state.fill_pct() - 0.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Constants tests
    // =========================================================================

    #[test]
    fn test_constants_are_positive() {
        assert!(CATCHMENT_FACTOR > 0.0);
        assert!(BASE_EVAPORATION_RATE > 0.0);
        assert!(TEMPERATURE_EVAP_FACTOR > 0.0);
        assert!(MIN_RESERVE_PCT > 0.0);
        assert!(MIN_RESERVE_PCT < 1.0);
    }

    #[test]
    fn test_catchment_factor_value() {
        assert!((CATCHMENT_FACTOR - 0.001).abs() < f32::EPSILON);
    }

    #[test]
    fn test_base_evaporation_rate_value() {
        assert!((BASE_EVAPORATION_RATE - 0.005).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temperature_evap_factor_value() {
        assert!((TEMPERATURE_EVAP_FACTOR - 0.03).abs() < f32::EPSILON);
    }

    #[test]
    fn test_min_reserve_pct_value() {
        assert!((MIN_RESERVE_PCT - 0.20).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Inflow calculation tests
    // =========================================================================

    #[test]
    fn test_inflow_zero_when_no_rain() {
        let precipitation_intensity = 0.0;
        let reservoir_count = 3_u32;
        let inflow = precipitation_intensity * CATCHMENT_FACTOR * reservoir_count as f32;
        assert!((inflow - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inflow_scales_with_precipitation() {
        let reservoir_count = 1_u32;
        let inflow_low = 0.5 * CATCHMENT_FACTOR * reservoir_count as f32;
        let inflow_high = 2.0 * CATCHMENT_FACTOR * reservoir_count as f32;
        assert!(inflow_high > inflow_low);
        assert!((inflow_high / inflow_low - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inflow_scales_with_reservoir_count() {
        let precipitation_intensity = 1.0;
        let inflow_1 = precipitation_intensity * CATCHMENT_FACTOR * 1.0;
        let inflow_3 = precipitation_intensity * CATCHMENT_FACTOR * 3.0;
        assert!((inflow_3 / inflow_1 - 3.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Evaporation calculation tests
    // =========================================================================

    #[test]
    fn test_evaporation_at_20c_is_base_rate() {
        let temp = 20.0;
        let reservoir_count = 1_u32;
        let temp_above = (temp - 20.0).max(0.0);
        let evap =
            reservoir_count as f32 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        assert!((evap - BASE_EVAPORATION_RATE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaporation_below_20c_is_base_rate() {
        let temp = 10.0;
        let reservoir_count = 1_u32;
        let temp_above = (temp - 20.0).max(0.0);
        let evap =
            reservoir_count as f32 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        // Below 20C, temp_above is 0.0, so evap == base rate.
        assert!((evap - BASE_EVAPORATION_RATE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaporation_increases_above_20c() {
        let reservoir_count = 1_u32;
        let temp_cool = 20.0;
        let temp_hot = 30.0;
        let evap_cool = reservoir_count as f32
            * (BASE_EVAPORATION_RATE + (temp_cool - 20.0).max(0.0) * TEMPERATURE_EVAP_FACTOR);
        let evap_hot = reservoir_count as f32
            * (BASE_EVAPORATION_RATE + (temp_hot - 20.0).max(0.0) * TEMPERATURE_EVAP_FACTOR);
        assert!(evap_hot > evap_cool);
        // At 30C: 0.005 + 10*0.03 = 0.305
        let expected_hot = BASE_EVAPORATION_RATE + 10.0 * TEMPERATURE_EVAP_FACTOR;
        assert!((evap_hot - expected_hot).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaporation_scales_with_reservoir_count() {
        let temp = 25.0;
        let temp_above = (temp - 20.0).max(0.0);
        let evap_1 = 1.0 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        let evap_4 = 4.0 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        assert!((evap_4 / evap_1 - 4.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Net change calculation tests
    // =========================================================================

    #[test]
    fn test_net_change_positive_with_high_inflow() {
        let inflow = 5.0;
        let outflow = 2.0;
        let evaporation = 0.5;
        let net = inflow - outflow - evaporation;
        assert!(net > 0.0);
        assert!((net - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_net_change_negative_with_high_demand() {
        let inflow = 0.5;
        let outflow = 3.0;
        let evaporation = 0.1;
        let net = inflow - outflow - evaporation;
        assert!(net < 0.0);
        assert!((net - (-2.6)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_net_change_zero_when_balanced() {
        let inflow = 2.5;
        let outflow = 2.0;
        let evaporation = 0.5;
        let net = inflow - outflow - evaporation;
        assert!((net - 0.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Storage days calculation tests
    // =========================================================================

    #[test]
    fn test_storage_days_with_demand() {
        let current_level_mg = 100.0;
        let outflow_mgd = 10.0;
        let days = current_level_mg / outflow_mgd;
        assert!((days - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_storage_days_infinity_with_zero_demand() {
        let current_level_mg = 100.0;
        let outflow_mgd = 0.0;
        let days = if outflow_mgd > 0.0 {
            current_level_mg / outflow_mgd
        } else {
            f32::INFINITY
        };
        assert!(days.is_infinite());
    }

    #[test]
    fn test_storage_days_zero_when_empty() {
        let current_level_mg = 0.0;
        let outflow_mgd = 5.0;
        let days = current_level_mg / outflow_mgd;
        assert!((days - 0.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Clamping tests
    // =========================================================================

    #[test]
    fn test_stored_gallons_clamped_to_capacity() {
        let capacity = 1000.0_f32;
        let stored = 800.0_f32;
        let delta = 500.0_f32; // would exceed capacity
        let new_stored = (stored + delta).clamp(0.0, capacity);
        assert!((new_stored - capacity).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stored_gallons_clamped_to_zero() {
        let capacity = 1000.0_f32;
        let stored = 200.0_f32;
        let delta = -500.0_f32; // would go below zero
        let new_stored = (stored + delta).clamp(0.0, capacity);
        assert!((new_stored - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stored_gallons_normal_delta() {
        let capacity = 1000.0_f32;
        let stored = 500.0_f32;
        let delta = 100.0_f32;
        let new_stored = (stored + delta).clamp(0.0, capacity);
        assert!((new_stored - 600.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Proportional distribution tests
    // =========================================================================

    #[test]
    fn test_proportional_share_equal_reservoirs() {
        let total_capacity = 2000.0_f32;
        let cap_a = 1000.0_f32;
        let cap_b = 1000.0_f32;
        let share_a = cap_a / total_capacity;
        let share_b = cap_b / total_capacity;
        assert!((share_a - 0.5).abs() < f32::EPSILON);
        assert!((share_b - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_proportional_share_unequal_reservoirs() {
        let total_capacity = 3000.0_f32;
        let cap_a = 1000.0_f32;
        let cap_b = 2000.0_f32;
        let share_a = cap_a / total_capacity;
        let share_b = cap_b / total_capacity;
        assert!((share_a - 1.0 / 3.0).abs() < 0.001);
        assert!((share_b - 2.0 / 3.0).abs() < 0.001);
    }

    // =========================================================================
    // ReservoirWarningEvent tests
    // =========================================================================

    #[test]
    fn test_warning_event_fields() {
        let event = ReservoirWarningEvent {
            old_tier: ReservoirWarningTier::Normal,
            new_tier: ReservoirWarningTier::Watch,
            fill_pct: 0.45,
        };
        assert_eq!(event.old_tier, ReservoirWarningTier::Normal);
        assert_eq!(event.new_tier, ReservoirWarningTier::Watch);
        assert!((event.fill_pct - 0.45).abs() < f32::EPSILON);
    }

    // =========================================================================
    // MGD/GPD conversion tests
    // =========================================================================

    #[test]
    fn test_gpd_to_mgd_conversion() {
        let gpd = 5_000_000.0_f32;
        let mgd = gpd / MGD_TO_GPD;
        assert!((mgd - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mgd_to_gpd_conversion() {
        let mgd = 3.0_f32;
        let gpd = mgd * MGD_TO_GPD;
        assert!((gpd - 3_000_000.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Full integration-style unit tests (logic only, no ECS app)
    // =========================================================================

    /// Simulate one tick of the reservoir update logic without a Bevy app.
    /// Returns the updated ReservoirState and whether a warning event would fire.
    fn simulate_tick(
        precipitation_intensity: f32,
        temperature: f32,
        total_demand_gpd: f32,
        reservoirs: &mut [(f32, f32)], // (stored_gallons, storage_capacity)
        state: &mut ReservoirState,
    ) -> Option<ReservoirWarningEvent> {
        let reservoir_count = reservoirs.len() as u32;
        state.reservoir_count = reservoir_count;

        let total_capacity_gallons: f32 = reservoirs.iter().map(|(_, cap)| *cap).sum();
        let total_stored_gallons: f32 = reservoirs.iter().map(|(stored, _)| *stored).sum();

        state.total_storage_capacity_mg = total_capacity_gallons / MGD_TO_GPD;
        state.current_level_mg = total_stored_gallons / MGD_TO_GPD;

        if reservoir_count == 0 {
            state.inflow_rate_mgd = 0.0;
            state.outflow_rate_mgd = 0.0;
            state.evaporation_rate_mgd = 0.0;
            state.net_change_mgd = 0.0;
            state.storage_days = 0.0;
            let old_tier = state.warning_tier;
            state.warning_tier = ReservoirWarningTier::Normal;
            if old_tier != ReservoirWarningTier::Normal {
                return Some(ReservoirWarningEvent {
                    old_tier,
                    new_tier: ReservoirWarningTier::Normal,
                    fill_pct: 0.0,
                });
            }
            return None;
        }

        let inflow_mgd = precipitation_intensity * CATCHMENT_FACTOR * reservoir_count as f32;
        let outflow_mgd = total_demand_gpd / MGD_TO_GPD;
        let temp_above_20 = (temperature - 20.0).max(0.0);
        let evaporation_mgd = reservoir_count as f32
            * (BASE_EVAPORATION_RATE + temp_above_20 * TEMPERATURE_EVAP_FACTOR);
        let net_change_mgd = inflow_mgd - outflow_mgd - evaporation_mgd;

        state.inflow_rate_mgd = inflow_mgd;
        state.outflow_rate_mgd = outflow_mgd;
        state.evaporation_rate_mgd = evaporation_mgd;
        state.net_change_mgd = net_change_mgd;

        let net_change_gallons = net_change_mgd * MGD_TO_GPD;
        let mut new_total_stored: f32 = 0.0;

        for (stored, capacity) in reservoirs.iter_mut() {
            let share = if total_capacity_gallons > 0.0 {
                *capacity / total_capacity_gallons
            } else {
                0.0
            };
            let delta = net_change_gallons * share;
            *stored = (*stored + delta).clamp(0.0, *capacity);
            new_total_stored += *stored;
        }

        state.current_level_mg = new_total_stored / MGD_TO_GPD;

        let fill_pct = state.fill_pct();
        let new_tier = warning_tier_from_fill(fill_pct);
        let old_tier = state.warning_tier;
        state.warning_tier = new_tier;

        state.storage_days = if outflow_mgd > 0.0 {
            state.current_level_mg / outflow_mgd
        } else {
            f32::INFINITY
        };

        if old_tier != new_tier {
            Some(ReservoirWarningEvent {
                old_tier,
                new_tier,
                fill_pct,
            })
        } else {
            None
        }
    }

    #[test]
    fn test_simulate_tick_no_reservoirs() {
        let mut state = ReservoirState::default();
        let event = simulate_tick(1.0, 25.0, 100_000.0, &mut [], &mut state);
        assert_eq!(state.reservoir_count, 0);
        assert!((state.inflow_rate_mgd).abs() < f32::EPSILON);
        assert!(event.is_none());
    }

    #[test]
    fn test_simulate_tick_single_reservoir_no_rain() {
        let mut state = ReservoirState::default();
        // Reservoir starts full at 1,000,000 gallons capacity.
        let mut reservoirs = vec![(1_000_000.0, 1_000_000.0)];
        let event = simulate_tick(
            0.0,  // no rain
            20.0, // 20C (base evap only)
            0.0,  // no demand
            &mut reservoirs,
            &mut state,
        );
        // Evaporation should have reduced stored level.
        // Evap = 1 * (0.005 + 0) = 0.005 MGD = 5000 gallons.
        let expected_stored = 1_000_000.0 - 5000.0;
        assert!((reservoirs[0].0 - expected_stored).abs() < 1.0);
        // Still nearly full, so Normal tier.
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!(event.is_none());
    }

    #[test]
    fn test_simulate_tick_high_demand_drains_reservoir() {
        let mut state = ReservoirState::default();
        // Start half full.
        let mut reservoirs = vec![(500_000.0, 1_000_000.0)];
        // Heavy demand: 400,000 GPD = 0.4 MGD.
        let event = simulate_tick(
            0.0,       // no rain
            20.0,      // 20C
            400_000.0, // demand GPD
            &mut reservoirs,
            &mut state,
        );
        // Outflow = 0.4 MGD = 400,000 gallons. Evap = 5,000. Net = -405,000.
        let expected = 500_000.0 - 400_000.0 - 5_000.0;
        assert!((reservoirs[0].0 - expected).abs() < 1.0);
        // 95,000 / 1,000,000 = 9.5% -> Critical tier.
        assert_eq!(state.warning_tier, ReservoirWarningTier::Critical);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.old_tier, ReservoirWarningTier::Normal);
        assert_eq!(ev.new_tier, ReservoirWarningTier::Critical);
    }

    #[test]
    fn test_simulate_tick_rainfall_replenishes() {
        let mut state = ReservoirState::default();
        let mut reservoirs = vec![(500_000.0, 1_000_000.0)];
        // Heavy rain at 2.0 in/hr, no demand, cool temp.
        let event = simulate_tick(
            2.0,  // heavy rain
            15.0, // below 20C, base evap only
            0.0,  // no demand
            &mut reservoirs,
            &mut state,
        );
        // Inflow = 2.0 * 0.001 * 1 = 0.002 MGD = 2000 gallons.
        // Evap = 0.005 MGD = 5000 gallons.
        // Net = 2000 - 0 - 5000 = -3000 gallons.
        let expected = 500_000.0 - 3_000.0;
        assert!((reservoirs[0].0 - expected).abs() < 1.0);
        // Still Normal (about 49.7%).
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!(event.is_none());
    }

    #[test]
    fn test_simulate_tick_hot_temperature_increases_evaporation() {
        let mut state = ReservoirState::default();
        let mut reservoirs = vec![(1_000_000.0, 1_000_000.0)];
        // No rain, no demand, but 40C.
        simulate_tick(0.0, 40.0, 0.0, &mut reservoirs, &mut state);
        // Evap = 1 * (0.005 + 20*0.03) = 1 * 0.605 = 0.605 MGD = 605,000 gallons.
        let expected = 1_000_000.0 - 605_000.0;
        assert!((reservoirs[0].0 - expected).abs() < 1.0);
    }

    #[test]
    fn test_simulate_tick_stored_does_not_go_below_zero() {
        let mut state = ReservoirState::default();
        // Very small reservoir nearly empty.
        let mut reservoirs = vec![(100.0, 1_000_000.0)];
        // Huge demand.
        simulate_tick(0.0, 20.0, 10_000_000.0, &mut reservoirs, &mut state);
        assert!((reservoirs[0].0 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_simulate_tick_stored_does_not_exceed_capacity() {
        let mut state = ReservoirState::default();
        // Nearly full.
        let mut reservoirs = vec![(999_999.0, 1_000_000.0)];
        // Huge rainfall, no demand, cool.
        simulate_tick(10000.0, 10.0, 0.0, &mut reservoirs, &mut state);
        // Should be clamped to capacity.
        assert!(reservoirs[0].0 <= reservoirs[0].1);
    }

    #[test]
    fn test_simulate_tick_multiple_reservoirs_proportional() {
        let mut state = ReservoirState::default();
        // Two reservoirs: one 1M capacity, one 3M capacity, both start full.
        let mut reservoirs = vec![(1_000_000.0, 1_000_000.0), (3_000_000.0, 3_000_000.0)];
        // Drain with demand, no rain, 20C.
        simulate_tick(0.0, 20.0, 200_000.0, &mut reservoirs, &mut state);
        // Total capacity = 4M. Reservoir A gets 25% of delta, B gets 75%.
        // Evap = 2 * 0.005 = 0.01 MGD = 10,000 gallons.
        // Outflow = 0.2 MGD = 200,000 gallons.
        // Net = -210,000 gallons.
        // A delta = -210,000 * 0.25 = -52,500. B delta = -210,000 * 0.75 = -157,500.
        let expected_a = 1_000_000.0 - 52_500.0;
        let expected_b = 3_000_000.0 - 157_500.0;
        assert!((reservoirs[0].0 - expected_a).abs() < 1.0);
        assert!((reservoirs[1].0 - expected_b).abs() < 1.0);
    }

    #[test]
    fn test_simulate_tick_storage_days_calculation() {
        let mut state = ReservoirState::default();
        // 10M gallons stored, 1M GPD demand = 10 days.
        let mut reservoirs = vec![(10_000_000.0, 20_000_000.0)];
        simulate_tick(0.0, 20.0, 1_000_000.0, &mut reservoirs, &mut state);
        // After draining: stored ~ 10M - 1M - 5000 = ~8.995M
        // storage_days = 8.995 / 1.0 ~ 8.995
        // outflow_mgd = 1.0
        assert!(state.storage_days > 0.0);
        assert!(state.storage_days < 10.0); // reduced from draining
    }

    #[test]
    fn test_simulate_tier_transition_normal_to_watch() {
        let mut state = ReservoirState::default();
        // Start at 51% fill (Normal).
        let mut reservoirs = vec![(510_000.0, 1_000_000.0)];
        // Drain enough to cross 50% boundary.
        // Need to drain ~10,001+ gallons. Outflow 100,000 GPD would do it.
        let event = simulate_tick(0.0, 20.0, 100_000.0, &mut reservoirs, &mut state);
        // After: 510,000 - 100,000 - 5,000 = 405,000 = 40.5% -> Watch.
        assert_eq!(state.warning_tier, ReservoirWarningTier::Watch);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.old_tier, ReservoirWarningTier::Normal);
        assert_eq!(ev.new_tier, ReservoirWarningTier::Watch);
    }

    #[test]
    fn test_simulate_no_event_when_tier_unchanged() {
        let mut state = ReservoirState::default();
        // Start at 90% (Normal). Small drain stays Normal.
        let mut reservoirs = vec![(900_000.0, 1_000_000.0)];
        let event = simulate_tick(0.0, 20.0, 1_000.0, &mut reservoirs, &mut state);
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!(event.is_none());
    }
}
