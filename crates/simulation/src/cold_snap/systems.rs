use bevy::prelude::*;

use crate::weather::{ClimateZone, Weather};
use crate::SlowTickTimer;

use super::detection::{
    cold_snap_tier, heating_demand_modifier, homeless_mortality, is_cold_day, seasonal_average_temp,
};
use super::pipe_damage::{
    calculate_pipe_bursts, estimate_water_main_miles, water_service_from_bursts,
};
use super::types::{
    ColdSnapEvent, ColdSnapState, ColdSnapTier, CONSTRUCTION_HALT_THRESHOLD_C,
    SCHOOL_CLOSURE_THRESHOLD_C,
};

// =============================================================================
// System
// =============================================================================

/// System that updates the `ColdSnapState` resource based on current weather.
///
/// Runs on the slow tick timer (every ~100 ticks). Reads the `Weather` resource
/// for temperature and tracks consecutive cold days, pipe bursts, and derived
/// effects (heating demand, traffic capacity, school closures, construction halt,
/// homeless mortality).
pub fn update_cold_snap(
    weather: Res<Weather>,
    climate: Res<ClimateZone>,
    mut state: ResMut<ColdSnapState>,
    timer: Res<SlowTickTimer>,
    mut events: EventWriter<ColdSnapEvent>,
) {
    if !timer.should_run() {
        return;
    }

    let temp = weather.temperature;
    let current_day = weather.last_update_day;
    let seasonal_avg = seasonal_average_temp(weather.season, *climate);

    // --- Day change: update consecutive cold day counter ---
    if current_day != state.last_check_day && current_day > 0 {
        state.last_check_day = current_day;

        if is_cold_day(temp, seasonal_avg) {
            state.consecutive_cold_days += 1;
        } else {
            // Reset streak on a non-cold day; also repair pipes over time
            state.consecutive_cold_days = 0;
            // Pipes get repaired: reduce burst count by 20% per warm day (min 0)
            state.pipe_burst_count = (state.pipe_burst_count as f32 * 0.8) as u32;
        }
    }

    // --- Determine tier ---
    let prev_tier = state.current_tier;
    state.current_tier = cold_snap_tier(state.consecutive_cold_days, temp);
    state.is_active = matches!(
        state.current_tier,
        ColdSnapTier::Warning | ColdSnapTier::Emergency
    );

    // --- Pipe bursts (only when below freezing) ---
    let mut new_bursts = 0u32;
    if temp <= 0.0 {
        // Approximate water main miles from a rough road cell estimate.
        // In a real integration, this would read from the road network resource.
        // For now, use a conservative estimate of 5000 road cells (~15 miles).
        let estimated_road_cells: u32 = 5000;
        let water_main_miles = estimate_water_main_miles(estimated_road_cells);
        let seed = (current_day as u64).wrapping_mul(0xdeadbeef_cafebabe);
        new_bursts = calculate_pipe_bursts(temp, water_main_miles, seed);
        state.pipe_burst_count = state.pipe_burst_count.saturating_add(new_bursts);

        // Update water service modifier
        state.water_service_modifier =
            water_service_from_bursts(state.pipe_burst_count, water_main_miles);
    } else {
        // Above freezing: water service recovers
        state.water_service_modifier = water_service_from_bursts(state.pipe_burst_count, 15.0);
    }

    // --- Heating demand surge ---
    state.heating_demand_modifier = heating_demand_modifier(state.current_tier, temp);

    // --- Traffic capacity: -20% during active cold snap (vehicle failures) ---
    state.traffic_capacity_modifier = if state.is_active { 0.8 } else { 1.0 };

    // --- School closures below -29C ---
    state.schools_closed = temp < SCHOOL_CLOSURE_THRESHOLD_C;

    // --- Construction halted below -9C ---
    state.construction_halted = temp < CONSTRUCTION_HALT_THRESHOLD_C;

    // --- Homeless mortality ---
    state.homeless_mortality_rate = homeless_mortality(temp);

    // --- Fire event on tier change or new pipe bursts ---
    let tier_changed = state.current_tier != prev_tier;
    if tier_changed || new_bursts > 0 {
        events.send(ColdSnapEvent {
            tier: state.current_tier,
            new_pipe_bursts: new_bursts,
            schools_closed: state.schools_closed,
            construction_halted: state.construction_halted,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ColdSnapPlugin;

impl Plugin for ColdSnapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColdSnapState>()
            .add_event::<ColdSnapEvent>()
            .add_systems(
                FixedUpdate,
                update_cold_snap
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal Bevy App with cold snap system.
    fn cold_snap_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<SlowTickTimer>()
            .init_resource::<Weather>()
            .init_resource::<ClimateZone>()
            .init_resource::<ColdSnapState>()
            .add_event::<ColdSnapEvent>()
            .add_systems(Update, update_cold_snap);
        app
    }

    fn advance_with_day(app: &mut App, day: u32) {
        {
            let mut timer = app.world_mut().resource_mut::<SlowTickTimer>();
            // Set counter to a multiple of INTERVAL so should_run() returns true
            timer.counter = SlowTickTimer::INTERVAL;
        }
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.last_update_day = day;
        }
        app.update();
    }

    #[test]
    fn test_system_no_cold_snap_above_threshold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = 5.0;
            weather.season = crate::weather::Season::Winter;
        }
        advance_with_day(&mut app, 1);

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.current_tier, ColdSnapTier::Normal);
        assert!(!state.is_active);
        assert_eq!(state.consecutive_cold_days, 0);
    }

    #[test]
    fn test_system_cold_snap_activates_after_3_days() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0; // Below -12C absolute threshold
            weather.season = crate::weather::Season::Winter;
        }
        // Simulate 2 prior cold days by pre-setting state
        {
            let mut state = app.world_mut().resource_mut::<ColdSnapState>();
            state.consecutive_cold_days = 2;
            state.last_check_day = 2;
        }
        // Day 3: Warning (cold snap active)
        advance_with_day(&mut app, 3);
        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.consecutive_cold_days, 3);
        assert_eq!(state.current_tier, ColdSnapTier::Warning);
        assert!(state.is_active);
    }

    #[test]
    fn test_system_emergency_at_extreme_cold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -25.0; // Below -23C for Emergency
            weather.season = crate::weather::Season::Winter;
        }
        // Simulate 2 prior cold days by pre-setting state
        {
            let mut state = app.world_mut().resource_mut::<ColdSnapState>();
            state.consecutive_cold_days = 2;
            state.last_check_day = 2;
        }
        // Day 3: Emergency (extreme cold + 3 consecutive days)
        advance_with_day(&mut app, 3);

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.current_tier, ColdSnapTier::Emergency);
        assert!(state.is_active);
        assert!((state.heating_demand_modifier - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_system_resets_on_warm_day() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = 5.0; // Warm day
            weather.season = crate::weather::Season::Winter;
        }
        // Pre-set state as if 3 cold days already occurred
        {
            let mut state = app.world_mut().resource_mut::<ColdSnapState>();
            state.consecutive_cold_days = 3;
            state.last_check_day = 3;
            state.is_active = true;
            state.current_tier = ColdSnapTier::Warning;
        }
        // Day 4: Warm day resets
        advance_with_day(&mut app, 4);

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.consecutive_cold_days, 0);
        assert_eq!(state.current_tier, ColdSnapTier::Normal);
        assert!(!state.is_active);
    }

    #[test]
    fn test_system_traffic_capacity_during_cold_snap() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0;
            weather.season = crate::weather::Season::Winter;
        }
        // Simulate 2 prior cold days by pre-setting state
        {
            let mut state = app.world_mut().resource_mut::<ColdSnapState>();
            state.consecutive_cold_days = 2;
            state.last_check_day = 2;
        }
        // Day 3: Active cold snap
        advance_with_day(&mut app, 3);

        let state = app.world().resource::<ColdSnapState>();
        assert!(
            (state.traffic_capacity_modifier - 0.8).abs() < f32::EPSILON,
            "Traffic capacity should be 0.8 during cold snap, got {}",
            state.traffic_capacity_modifier
        );
    }

    #[test]
    fn test_system_school_closure() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -30.0; // Below -29C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(state.schools_closed, "Schools should close below -29C");
    }

    #[test]
    fn test_system_construction_halted() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -10.0; // Below -9C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            state.construction_halted,
            "Construction should halt below -9C"
        );
    }

    #[test]
    fn test_system_construction_not_halted_above_threshold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -8.0; // Above -9C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            !state.construction_halted,
            "Construction should not halt above -9C"
        );
    }

    #[test]
    fn test_system_homeless_mortality_at_extreme_cold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -25.0;
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            state.homeless_mortality_rate > 0.0,
            "Homeless mortality should be positive below -18C"
        );
    }

    #[test]
    fn test_system_no_mortality_above_threshold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -10.0; // Above -18C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            state.homeless_mortality_rate.abs() < f32::EPSILON,
            "Homeless mortality should be zero above -18C"
        );
    }

    #[test]
    fn test_system_event_fired_on_tier_change() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0;
            weather.season = crate::weather::Season::Winter;
        }

        // Day 1: Normal -> Watch fires event
        advance_with_day(&mut app, 1);

        let events = app.world().resource::<Events<ColdSnapEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();
        assert!(
            !fired.is_empty(),
            "ColdSnapEvent should fire on tier change"
        );
        assert_eq!(fired[0].tier, ColdSnapTier::Watch);
    }

    #[test]
    fn test_system_skips_when_timer_not_ready() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -20.0;
            weather.season = crate::weather::Season::Winter;
            weather.last_update_day = 1;
        }
        // Don't set timer to interval - it starts at 0 which is a multiple of 100
        // but SlowTickTimer::should_run checks counter.is_multiple_of(100)
        // 0.is_multiple_of(100) is true in Rust, so set to non-multiple
        {
            let mut timer = app.world_mut().resource_mut::<SlowTickTimer>();
            timer.counter = 1; // Not a multiple of 100
        }
        app.update();

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(
            state.consecutive_cold_days, 0,
            "Should not update when timer not ready"
        );
    }
}
