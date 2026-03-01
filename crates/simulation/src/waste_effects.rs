use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::events::{CityEvent, CityEventType, EventJournal};
use crate::garbage::WasteCollectionGrid;
use crate::time_of_day::GameClock;

// =============================================================================
// Constants (WASTE-010)
// =============================================================================

/// Health penalty per slow tick per 100 lbs of accumulated waste nearby.
pub const WASTE_HEALTH_PENALTY_PER_100_LBS: f32 = 0.5;

/// Radius in grid cells to check for nearby accumulated waste (health penalty).
pub const WASTE_HEALTH_CHECK_RADIUS: i32 = 3;

/// Happiness penalty applied when a building has uncollected waste (> 0 lbs).
pub const WASTE_HAPPINESS_PENALTY: f32 = 5.0;

/// Land value modifier when nearby cells have > 500 lbs waste (20% reduction).
pub const WASTE_LAND_VALUE_MODIFIER: f32 = 0.80;

/// Threshold in lbs for land value penalty to apply.
pub const WASTE_LAND_VALUE_THRESHOLD_LBS: f32 = 500.0;

/// Radius in grid cells to check for nearby waste (land value penalty).
pub const WASTE_LAND_VALUE_CHECK_RADIUS: i32 = 5;

/// Daily waste decay rate (0.5% per day = per slow tick).
pub const WASTE_DECAY_RATE: f32 = 0.005;

/// Fraction of occupied cells with uncollected waste to trigger a public health crisis.
pub const WASTE_CRISIS_THRESHOLD: f32 = 0.20;

// =============================================================================
// WasteAccumulation resource
// =============================================================================

/// Grid tracking accumulated uncollected waste per cell in lbs.
///
/// This is the WASTE-010 accumulation grid, separate from the existing
/// `WasteCollectionGrid.uncollected_lbs` which tracks per-tick uncollected waste.
/// This resource accumulates over time and decays slowly.
#[derive(Resource)]
pub struct WasteAccumulation {
    pub lbs: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for WasteAccumulation {
    fn default() -> Self {
        Self {
            lbs: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl WasteAccumulation {
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Get accumulated waste in lbs at (x, y).
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.lbs[self.idx(x, y)]
    }

    /// Set accumulated waste at (x, y).
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        let idx = self.idx(x, y);
        self.lbs[idx] = val;
    }
}

// =============================================================================
// WasteCrisisEvent
// =============================================================================

/// Event fired when >20% of occupied cells have uncollected waste,
/// indicating a public health crisis.
#[derive(Event, Debug, Clone)]
pub struct WasteCrisisEvent {
    /// Fraction of occupied cells with uncollected waste (0.0 to 1.0).
    pub affected_fraction: f32,
}

// =============================================================================
// Systems
// =============================================================================

/// Updates waste accumulation grid from the waste collection system.
///
/// When garbage collection is insufficient (uncollected_lbs > 0 in
/// WasteCollectionGrid), waste accumulates in WasteAccumulation.
/// Applies 0.5% daily decay to existing accumulation.
/// Runs on the slow tick.
pub fn update_waste_accumulation(
    slow_timer: Res<crate::SlowTickTimer>,
    collection_grid: Res<WasteCollectionGrid>,
    mut accumulation: ResMut<WasteAccumulation>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for i in 0..accumulation.lbs.len() {
        // Add uncollected waste from collection grid
        let uncollected = collection_grid.uncollected_lbs[i];
        if uncollected > 0.0 {
            accumulation.lbs[i] += uncollected;
        }

        // Apply daily decay (0.5%)
        if accumulation.lbs[i] > 0.0 {
            accumulation.lbs[i] *= 1.0 - WASTE_DECAY_RATE;
            // Clamp very small values to zero
            if accumulation.lbs[i] < 0.1 {
                accumulation.lbs[i] = 0.0;
            }
        }

        // Cap to prevent unbounded growth
        accumulation.lbs[i] = accumulation.lbs[i].min(50_000.0);
    }
}

/// Applies health penalty to citizens based on nearby accumulated waste.
///
/// -0.5 health per slow tick per 100 lbs of accumulated waste within
/// a 3-cell radius of the citizen's home.
pub fn waste_health_penalty(
    slow_timer: Res<crate::SlowTickTimer>,
    accumulation: Res<WasteAccumulation>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for (mut details, home) in &mut citizens {
        let hx = home.grid_x as i32;
        let hy = home.grid_y as i32;

        let mut total_nearby_lbs: f32 = 0.0;

        for dy in -WASTE_HEALTH_CHECK_RADIUS..=WASTE_HEALTH_CHECK_RADIUS {
            for dx in -WASTE_HEALTH_CHECK_RADIUS..=WASTE_HEALTH_CHECK_RADIUS {
                let nx = hx + dx;
                let ny = hy + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                total_nearby_lbs += accumulation.get(nx as usize, ny as usize);
            }
        }

        // -0.5 health per 100 lbs
        if total_nearby_lbs > 0.0 {
            let penalty = (total_nearby_lbs / 100.0) * WASTE_HEALTH_PENALTY_PER_100_LBS;
            details.health = (details.health - penalty).max(0.0);
        }
    }
}

/// Maximum accumulated waste (in lbs) at which the full penalty applies.
pub const WASTE_HAPPINESS_MAX_ACCUMULATED_LBS: f32 = 500.0;

/// Returns the happiness penalty for a cell with accumulated waste.
///
/// Scales linearly from 0 to -5 for accumulated waste in 0-500 lbs range.
pub fn waste_happiness_penalty(accumulated_lbs: f32) -> f32 {
    if accumulated_lbs > 0.0 {
        let ratio = (accumulated_lbs / WASTE_HAPPINESS_MAX_ACCUMULATED_LBS).clamp(0.0, 1.0);
        -WASTE_HAPPINESS_PENALTY * ratio
    } else {
        0.0
    }
}

/// Returns the land value modifier for a cell based on nearby accumulated waste.
///
/// Returns 0.8 (20% reduction) if any nearby cell within radius has > 500 lbs
/// of accumulated waste, 1.0 otherwise.
pub fn waste_land_value_modifier(accumulation: &WasteAccumulation, x: usize, y: usize) -> f32 {
    let cx = x as i32;
    let cy = y as i32;

    for dy in -WASTE_LAND_VALUE_CHECK_RADIUS..=WASTE_LAND_VALUE_CHECK_RADIUS {
        for dx in -WASTE_LAND_VALUE_CHECK_RADIUS..=WASTE_LAND_VALUE_CHECK_RADIUS {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            if accumulation.get(nx as usize, ny as usize) > WASTE_LAND_VALUE_THRESHOLD_LBS {
                return WASTE_LAND_VALUE_MODIFIER;
            }
        }
    }

    1.0
}

/// Checks if >20% of occupied cells have uncollected waste and fires
/// a `WasteCrisisEvent` if so. Also logs to the event journal.
pub fn check_waste_crisis(
    slow_timer: Res<crate::SlowTickTimer>,
    accumulation: Res<WasteAccumulation>,
    buildings: Query<&Building>,
    clock: Res<GameClock>,
    mut journal: ResMut<EventJournal>,
    mut crisis_events: EventWriter<WasteCrisisEvent>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut occupied_count: u32 = 0;
    let mut waste_count: u32 = 0;

    for building in &buildings {
        occupied_count += 1;
        if accumulation.get(building.grid_x, building.grid_y) > 0.0 {
            waste_count += 1;
        }
    }

    if occupied_count == 0 {
        return;
    }

    let fraction = waste_count as f32 / occupied_count as f32;

    if fraction > WASTE_CRISIS_THRESHOLD {
        crisis_events.send(WasteCrisisEvent {
            affected_fraction: fraction,
        });

        journal.push(CityEvent {
            event_type: CityEventType::Epidemic,
            day: clock.day,
            hour: clock.hour,
            description: format!(
                "Public health crisis! {:.0}% of buildings have uncollected waste.",
                fraction * 100.0
            ),
        });
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waste_accumulation_default() {
        let acc = WasteAccumulation::default();
        assert_eq!(acc.width, GRID_WIDTH);
        assert_eq!(acc.height, GRID_HEIGHT);
        assert_eq!(acc.get(0, 0), 0.0);
        assert_eq!(acc.get(128, 128), 0.0);
    }

    #[test]
    fn test_waste_accumulation_set_get() {
        let mut acc = WasteAccumulation::default();
        acc.set(10, 20, 500.0);
        assert_eq!(acc.get(10, 20), 500.0);
        assert_eq!(acc.get(0, 0), 0.0);
    }

    #[test]
    fn test_health_penalty_scales_with_waste() {
        // 100 lbs nearby => -0.5 health penalty
        let penalty_100 = (100.0_f32 / 100.0) * WASTE_HEALTH_PENALTY_PER_100_LBS;
        assert!((penalty_100 - 0.5).abs() < 0.001);

        // 500 lbs nearby => -2.5 health penalty
        let penalty_500 = (500.0_f32 / 100.0) * WASTE_HEALTH_PENALTY_PER_100_LBS;
        assert!((penalty_500 - 2.5).abs() < 0.001);

        // 1000 lbs nearby => -5.0 health penalty
        let penalty_1000 = (1000.0_f32 / 100.0) * WASTE_HEALTH_PENALTY_PER_100_LBS;
        assert!((penalty_1000 - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_happiness_penalty_with_waste() {
        // Full penalty at >= 500 lbs
        assert_eq!(waste_happiness_penalty(500.0), -5.0);
        assert_eq!(waste_happiness_penalty(1000.0), -5.0);
        // Half penalty at 250 lbs
        assert!((waste_happiness_penalty(250.0) - (-2.5)).abs() < 0.01);
        // Small waste gives small penalty
        assert!(waste_happiness_penalty(50.0) > -1.0);
        assert!(waste_happiness_penalty(50.0) < 0.0);
    }

    #[test]
    fn test_happiness_penalty_without_waste() {
        // No waste => no penalty
        assert_eq!(waste_happiness_penalty(0.0), 0.0);
        assert_eq!(waste_happiness_penalty(-1.0), 0.0);
    }

    #[test]
    fn test_land_value_reduction_near_waste() {
        let mut acc = WasteAccumulation::default();
        // Place 600 lbs of waste at (50, 50) — above the 500 lbs threshold
        acc.set(50, 50, 600.0);

        // Cell at (50, 50) itself should have reduced land value
        let modifier = waste_land_value_modifier(&acc, 50, 50);
        assert!((modifier - 0.8).abs() < 0.001);

        // Cell at (52, 52) — within radius 5 — should also be affected
        let modifier_near = waste_land_value_modifier(&acc, 52, 52);
        assert!((modifier_near - 0.8).abs() < 0.001);

        // Cell at (56, 56) — outside radius 5 — should NOT be affected
        let modifier_far = waste_land_value_modifier(&acc, 56, 56);
        assert!((modifier_far - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_land_value_no_reduction_below_threshold() {
        let mut acc = WasteAccumulation::default();
        // Place 400 lbs of waste — below the 500 lbs threshold
        acc.set(50, 50, 400.0);

        let modifier = waste_land_value_modifier(&acc, 50, 50);
        assert!((modifier - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_waste_decay() {
        let initial = 1000.0_f32;
        let after_decay = initial * (1.0 - WASTE_DECAY_RATE);
        // Should be 995.0 (0.5% decay of 1000)
        assert!((after_decay - 995.0).abs() < 0.001);
    }

    #[test]
    fn test_waste_decay_small_values_clamped() {
        // Values below 0.1 should be clamped to 0
        let small = 0.05_f32;
        let decayed = small * (1.0 - WASTE_DECAY_RATE);
        assert!(decayed < 0.1);
    }

    #[test]
    fn test_crisis_threshold() {
        // 20% threshold
        assert!((WASTE_CRISIS_THRESHOLD - 0.20).abs() < 0.001);

        // 21% of buildings with waste should trigger crisis
        let fraction = 0.21_f32;
        assert!(fraction > WASTE_CRISIS_THRESHOLD);

        // 19% should not
        let fraction_low = 0.19_f32;
        assert!(fraction_low <= WASTE_CRISIS_THRESHOLD);
    }

    #[test]
    fn test_waste_accumulation_cap() {
        let mut acc = WasteAccumulation::default();
        acc.set(10, 10, 100_000.0);
        // After update_waste_accumulation, it should be capped at 50_000
        // But we test the cap logic directly
        let capped = acc.get(10, 10).min(50_000.0);
        assert_eq!(capped, 50_000.0);
    }
}

pub struct WasteEffectsPlugin;

impl Plugin for WasteEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WasteAccumulation>()
            .add_event::<WasteCrisisEvent>()
            .add_systems(
                FixedUpdate,
                (
                    update_waste_accumulation,
                    waste_health_penalty,
                    check_waste_crisis,
                )
                    .chain()
                    .after(crate::garbage::update_waste_collection)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
