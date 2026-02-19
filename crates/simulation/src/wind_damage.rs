use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;
use crate::wind::WindState;
use crate::TickCounter;

// =============================================================================
// Wind Damage Tiers (Beaufort-inspired)
// =============================================================================

/// Beaufort-inspired wind damage classification based on normalized wind speed [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WindDamageTier {
    /// Speed 0.0 - 0.15: No damage.
    #[default]
    Calm,
    /// Speed 0.15 - 0.3: No damage, light wind.
    Breezy,
    /// Speed 0.3 - 0.45: Minor effects, no structural damage.
    Strong,
    /// Speed 0.45 - 0.6: Light structural risk begins.
    Gale,
    /// Speed 0.6 - 0.75: Moderate damage, power lines at risk.
    Storm,
    /// Speed 0.75 - 0.9: Significant damage, trees knocked down.
    Severe,
    /// Speed 0.9 - 0.95: Extreme damage to structures.
    HurricaneForce,
    /// Speed > 0.95: Catastrophic damage.
    Extreme,
}

impl WindDamageTier {
    /// Classify a normalized wind speed [0, 1] into a damage tier.
    pub fn from_speed(speed: f32) -> Self {
        if speed < 0.15 {
            WindDamageTier::Calm
        } else if speed < 0.3 {
            WindDamageTier::Breezy
        } else if speed < 0.45 {
            WindDamageTier::Strong
        } else if speed < 0.6 {
            WindDamageTier::Gale
        } else if speed < 0.75 {
            WindDamageTier::Storm
        } else if speed < 0.9 {
            WindDamageTier::Severe
        } else if speed < 0.95 {
            WindDamageTier::HurricaneForce
        } else {
            WindDamageTier::Extreme
        }
    }

    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            WindDamageTier::Calm => "Calm",
            WindDamageTier::Breezy => "Breezy",
            WindDamageTier::Strong => "Strong",
            WindDamageTier::Gale => "Gale",
            WindDamageTier::Storm => "Storm",
            WindDamageTier::Severe => "Severe",
            WindDamageTier::HurricaneForce => "Hurricane Force",
            WindDamageTier::Extreme => "Extreme",
        }
    }
}

// =============================================================================
// Damage formulas
// =============================================================================

/// Wind damage threshold: damage begins above this normalized speed.
const WIND_DAMAGE_THRESHOLD: f32 = 0.4;

/// Power outage threshold: outage probability begins above this speed.
const POWER_OUTAGE_THRESHOLD: f32 = 0.6;

/// Tree knockdown threshold: tree damage begins above this speed.
const TREE_KNOCKDOWN_THRESHOLD: f32 = 0.6;

/// Calculate wind damage amount using cubic formula.
/// Returns 0.0 for speeds <= 0.4, otherwise `(speed - 0.4)^3 * 1000`.
pub fn wind_damage_amount(speed: f32) -> f32 {
    if speed <= WIND_DAMAGE_THRESHOLD {
        return 0.0;
    }
    let excess = speed - WIND_DAMAGE_THRESHOLD;
    excess * excess * excess * 1000.0
}

/// Calculate power outage probability based on wind speed.
/// Returns 0.0 for speeds <= 0.6, scaling up to ~1.0 at extreme speeds.
/// Formula: `((speed - 0.6) / 0.4)^2` clamped to [0, 1].
pub fn power_outage_probability(speed: f32) -> f32 {
    if speed <= POWER_OUTAGE_THRESHOLD {
        return 0.0;
    }
    let factor = (speed - POWER_OUTAGE_THRESHOLD) / 0.4;
    (factor * factor).min(1.0)
}

/// Calculate tree knockdown probability based on wind speed.
/// Returns 0.0 for speeds <= 0.6, scaling up for higher speeds.
/// Formula: `((speed - 0.6) / 0.4)^2 * 0.1` per tree per update tick.
pub fn tree_knockdown_probability(speed: f32) -> f32 {
    if speed <= TREE_KNOCKDOWN_THRESHOLD {
        return 0.0;
    }
    let factor = (speed - TREE_KNOCKDOWN_THRESHOLD) / 0.4;
    (factor * factor * 0.1).min(1.0)
}

// =============================================================================
// Wind Damage State (resource)
// =============================================================================

/// Resource tracking accumulated wind damage during a storm.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WindDamageState {
    /// Current wind damage tier classification.
    #[serde(default)]
    pub current_tier: WindDamageTier,
    /// Accumulated building damage this storm (cumulative damage points).
    #[serde(default)]
    pub accumulated_building_damage: f32,
    /// Number of trees knocked down during this storm.
    #[serde(default)]
    pub trees_knocked_down: u32,
    /// Whether a power outage is currently active due to wind.
    #[serde(default)]
    pub power_outage_active: bool,
}

impl Default for WindDamageState {
    fn default() -> Self {
        Self {
            current_tier: WindDamageTier::Calm,
            accumulated_building_damage: 0.0,
            trees_knocked_down: 0,
            power_outage_active: false,
        }
    }
}

// =============================================================================
// Wind Damage Event
// =============================================================================

/// Event fired when wind damage occurs, for notification to other systems.
#[derive(Event, Debug, Clone)]
pub struct WindDamageEvent {
    /// The damage tier that triggered this event.
    pub tier: WindDamageTier,
    /// Amount of building damage dealt this tick.
    pub building_damage: f32,
    /// Number of trees knocked down this tick.
    pub trees_knocked: u32,
    /// Whether power outage was triggered.
    pub power_outage: bool,
}

// =============================================================================
// Deterministic pseudo-random (splitmix64, matching wind.rs pattern)
// =============================================================================

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
fn rand_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    (hash % 1_000_000) as f32 / 1_000_000.0
}

// =============================================================================
// Systems
// =============================================================================

/// Wind damage update interval in ticks (aligns with wind update interval).
const WIND_DAMAGE_INTERVAL: u64 = 100;

/// Updates wind damage state each interval based on current wind speed.
///
/// - Classifies wind into a damage tier
/// - Accumulates building damage for speeds > 0.4
/// - Probabilistically knocks down trees at high wind speeds
/// - Sets power outage flag based on outage probability
///
/// Resets accumulated counters when wind drops below damage threshold.
pub fn update_wind_damage(
    tick: Res<TickCounter>,
    wind: Res<WindState>,
    mut state: ResMut<WindDamageState>,
    mut tree_grid: ResMut<TreeGrid>,
    mut events: EventWriter<WindDamageEvent>,
) {
    if tick.0 == 0 || !tick.0.is_multiple_of(WIND_DAMAGE_INTERVAL) {
        return;
    }

    let speed = wind.speed;
    let tier = WindDamageTier::from_speed(speed);
    state.current_tier = tier;

    // If below damage threshold, reset storm counters and exit
    if speed <= WIND_DAMAGE_THRESHOLD {
        // Only reset when transitioning from a damaging state
        if state.accumulated_building_damage > 0.0 || state.trees_knocked_down > 0 {
            state.accumulated_building_damage = 0.0;
            state.trees_knocked_down = 0;
        }
        state.power_outage_active = false;
        return;
    }

    // --- Building damage ---
    let damage = wind_damage_amount(speed);
    state.accumulated_building_damage += damage;

    // --- Power outage ---
    let outage_prob = power_outage_probability(speed);
    let outage_seed = tick.0.wrapping_mul(0xdeadbeef_cafebabe);
    let outage_roll = rand_f32(outage_seed);
    state.power_outage_active = outage_roll < outage_prob;

    // --- Tree knockdown ---
    let knockdown_prob = tree_knockdown_probability(speed);
    let mut trees_knocked_this_tick: u32 = 0;

    if knockdown_prob > 0.0 {
        // Iterate over the grid to find trees and probabilistically knock them down.
        // Use deterministic hash based on tick + position for each cell.
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if tree_grid.has_tree(x, y) {
                    let cell_seed = tick
                        .0
                        .wrapping_mul(0x517cc1b727220a95)
                        .wrapping_add((y * GRID_WIDTH + x) as u64);
                    let roll = rand_f32(cell_seed);
                    if roll < knockdown_prob {
                        tree_grid.set(x, y, false);
                        trees_knocked_this_tick += 1;
                    }
                }
            }
        }
    }

    state.trees_knocked_down += trees_knocked_this_tick;

    // Fire event if any damage occurred
    if damage > 0.0 || trees_knocked_this_tick > 0 || state.power_outage_active {
        events.send(WindDamageEvent {
            tier,
            building_damage: damage,
            trees_knocked: trees_knocked_this_tick,
            power_outage: state.power_outage_active,
        });
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Unit tests for damage tier classification
    // -----------------------------------------------------------------------

    #[test]
    fn test_tier_calm() {
        assert_eq!(WindDamageTier::from_speed(0.0), WindDamageTier::Calm);
        assert_eq!(WindDamageTier::from_speed(0.1), WindDamageTier::Calm);
        assert_eq!(WindDamageTier::from_speed(0.14), WindDamageTier::Calm);
    }

    #[test]
    fn test_tier_breezy() {
        assert_eq!(WindDamageTier::from_speed(0.15), WindDamageTier::Breezy);
        assert_eq!(WindDamageTier::from_speed(0.2), WindDamageTier::Breezy);
        assert_eq!(WindDamageTier::from_speed(0.29), WindDamageTier::Breezy);
    }

    #[test]
    fn test_tier_strong() {
        assert_eq!(WindDamageTier::from_speed(0.3), WindDamageTier::Strong);
        assert_eq!(WindDamageTier::from_speed(0.44), WindDamageTier::Strong);
    }

    #[test]
    fn test_tier_gale() {
        assert_eq!(WindDamageTier::from_speed(0.45), WindDamageTier::Gale);
        assert_eq!(WindDamageTier::from_speed(0.59), WindDamageTier::Gale);
    }

    #[test]
    fn test_tier_storm() {
        assert_eq!(WindDamageTier::from_speed(0.6), WindDamageTier::Storm);
        assert_eq!(WindDamageTier::from_speed(0.74), WindDamageTier::Storm);
    }

    #[test]
    fn test_tier_severe() {
        assert_eq!(WindDamageTier::from_speed(0.75), WindDamageTier::Severe);
        assert_eq!(WindDamageTier::from_speed(0.89), WindDamageTier::Severe);
    }

    #[test]
    fn test_tier_hurricane_force() {
        assert_eq!(
            WindDamageTier::from_speed(0.9),
            WindDamageTier::HurricaneForce
        );
        assert_eq!(
            WindDamageTier::from_speed(0.94),
            WindDamageTier::HurricaneForce
        );
    }

    #[test]
    fn test_tier_extreme() {
        assert_eq!(WindDamageTier::from_speed(0.95), WindDamageTier::Extreme);
        assert_eq!(WindDamageTier::from_speed(1.0), WindDamageTier::Extreme);
    }

    // -----------------------------------------------------------------------
    // Unit tests for damage formulas
    // -----------------------------------------------------------------------

    #[test]
    fn test_wind_damage_zero_below_threshold() {
        assert_eq!(wind_damage_amount(0.0), 0.0);
        assert_eq!(wind_damage_amount(0.2), 0.0);
        assert_eq!(wind_damage_amount(0.4), 0.0);
    }

    #[test]
    fn test_wind_damage_cubic_above_threshold() {
        // At speed 0.5: (0.5 - 0.4)^3 * 1000 = 0.1^3 * 1000 = 0.001 * 1000 = 1.0
        let damage = wind_damage_amount(0.5);
        assert!((damage - 1.0).abs() < 0.01, "Expected ~1.0, got {}", damage);

        // At speed 0.6: (0.6 - 0.4)^3 * 1000 = 0.2^3 * 1000 = 0.008 * 1000 = 8.0
        let damage = wind_damage_amount(0.6);
        assert!((damage - 8.0).abs() < 0.01, "Expected ~8.0, got {}", damage);

        // At speed 0.9: (0.9 - 0.4)^3 * 1000 = 0.5^3 * 1000 = 0.125 * 1000 = 125.0
        let damage = wind_damage_amount(0.9);
        assert!(
            (damage - 125.0).abs() < 0.01,
            "Expected ~125.0, got {}",
            damage
        );

        // At speed 1.0: (1.0 - 0.4)^3 * 1000 = 0.6^3 * 1000 = 0.216 * 1000 = 216.0
        let damage = wind_damage_amount(1.0);
        assert!(
            (damage - 216.0).abs() < 0.01,
            "Expected ~216.0, got {}",
            damage
        );
    }

    #[test]
    fn test_wind_damage_monotonically_increasing() {
        let mut prev = wind_damage_amount(0.4);
        for i in 41..=100 {
            let speed = i as f32 / 100.0;
            let damage = wind_damage_amount(speed);
            assert!(
                damage >= prev,
                "Damage should increase: at speed {} got {} < {}",
                speed,
                damage,
                prev
            );
            prev = damage;
        }
    }

    #[test]
    fn test_power_outage_zero_below_threshold() {
        assert_eq!(power_outage_probability(0.0), 0.0);
        assert_eq!(power_outage_probability(0.3), 0.0);
        assert_eq!(power_outage_probability(0.6), 0.0);
    }

    #[test]
    fn test_power_outage_scales_above_threshold() {
        // At speed 0.8: ((0.8 - 0.6) / 0.4)^2 = (0.5)^2 = 0.25
        let prob = power_outage_probability(0.8);
        assert!((prob - 0.25).abs() < 0.01, "Expected ~0.25, got {}", prob);

        // At speed 1.0: ((1.0 - 0.6) / 0.4)^2 = (1.0)^2 = 1.0
        let prob = power_outage_probability(1.0);
        assert!((prob - 1.0).abs() < 0.01, "Expected ~1.0, got {}", prob);
    }

    #[test]
    fn test_tree_knockdown_zero_below_threshold() {
        assert_eq!(tree_knockdown_probability(0.0), 0.0);
        assert_eq!(tree_knockdown_probability(0.5), 0.0);
        assert_eq!(tree_knockdown_probability(0.6), 0.0);
    }

    #[test]
    fn test_tree_knockdown_scales_above_threshold() {
        // At speed 0.8: ((0.8 - 0.6) / 0.4)^2 * 0.1 = 0.25 * 0.1 = 0.025
        let prob = tree_knockdown_probability(0.8);
        assert!(
            (prob - 0.025).abs() < 0.001,
            "Expected ~0.025, got {}",
            prob
        );

        // At speed 1.0: ((1.0 - 0.6) / 0.4)^2 * 0.1 = 1.0 * 0.1 = 0.1
        let prob = tree_knockdown_probability(1.0);
        assert!((prob - 0.1).abs() < 0.001, "Expected ~0.1, got {}", prob);
    }

    // -----------------------------------------------------------------------
    // Unit tests for WindDamageState defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_wind_damage_state_default() {
        let state = WindDamageState::default();
        assert_eq!(state.current_tier, WindDamageTier::Calm);
        assert_eq!(state.accumulated_building_damage, 0.0);
        assert_eq!(state.trees_knocked_down, 0);
        assert!(!state.power_outage_active);
    }

    // -----------------------------------------------------------------------
    // Unit tests for tier labels
    // -----------------------------------------------------------------------

    #[test]
    fn test_tier_labels() {
        assert_eq!(WindDamageTier::Calm.label(), "Calm");
        assert_eq!(WindDamageTier::Breezy.label(), "Breezy");
        assert_eq!(WindDamageTier::Strong.label(), "Strong");
        assert_eq!(WindDamageTier::Gale.label(), "Gale");
        assert_eq!(WindDamageTier::Storm.label(), "Storm");
        assert_eq!(WindDamageTier::Severe.label(), "Severe");
        assert_eq!(WindDamageTier::HurricaneForce.label(), "Hurricane Force");
        assert_eq!(WindDamageTier::Extreme.label(), "Extreme");
    }

    // -----------------------------------------------------------------------
    // Integration tests using Bevy App
    // -----------------------------------------------------------------------

    /// Helper: build a minimal Bevy App with wind damage system.
    fn wind_damage_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<TickCounter>()
            .init_resource::<WindState>()
            .init_resource::<WindDamageState>()
            .init_resource::<TreeGrid>()
            .add_event::<WindDamageEvent>()
            .add_systems(Update, update_wind_damage);
        app
    }

    fn advance(app: &mut App, tick_value: u64) {
        app.world_mut().resource_mut::<TickCounter>().0 = tick_value;
        app.update();
    }

    #[test]
    fn test_system_no_damage_at_low_wind() {
        let mut app = wind_damage_test_app();
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.2; // Breezy, below damage threshold
        }
        advance(&mut app, 100);

        let state = app.world().resource::<WindDamageState>();
        assert_eq!(state.current_tier, WindDamageTier::Breezy);
        assert_eq!(state.accumulated_building_damage, 0.0);
        assert_eq!(state.trees_knocked_down, 0);
        assert!(!state.power_outage_active);
    }

    #[test]
    fn test_system_damage_at_high_wind() {
        let mut app = wind_damage_test_app();
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.8; // Severe tier
        }
        advance(&mut app, 100);

        let state = app.world().resource::<WindDamageState>();
        assert_eq!(state.current_tier, WindDamageTier::Severe);
        assert!(
            state.accumulated_building_damage > 0.0,
            "Should have building damage at speed 0.8"
        );
    }

    #[test]
    fn test_system_trees_knocked_down_at_extreme_wind() {
        let mut app = wind_damage_test_app();

        // Place some trees on the grid
        {
            let mut tree_grid = app.world_mut().resource_mut::<TreeGrid>();
            for x in 0..10 {
                for y in 0..10 {
                    tree_grid.set(x, y, true);
                }
            }
        }

        // Set extreme wind
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 1.0; // Extreme
        }

        // Run several ticks to knock down trees
        for i in 1..=5 {
            advance(&mut app, i * 100);
        }

        let state = app.world().resource::<WindDamageState>();
        assert!(
            state.trees_knocked_down > 0,
            "Should have knocked down some trees at extreme wind"
        );

        // Verify trees were actually removed from the grid
        let tree_grid = app.world().resource::<TreeGrid>();
        let remaining: usize = (0..10)
            .flat_map(|x| (0..10).map(move |y| (x, y)))
            .filter(|&(x, y)| tree_grid.has_tree(x, y))
            .count();
        assert!(
            remaining < 100,
            "Some trees should have been knocked down, {} remaining out of 100",
            remaining
        );
    }

    #[test]
    fn test_system_damage_resets_when_calm() {
        let mut app = wind_damage_test_app();

        // First: apply damage
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.8;
        }
        advance(&mut app, 100);

        let state = app.world().resource::<WindDamageState>();
        assert!(state.accumulated_building_damage > 0.0);

        // Now drop to calm
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.1;
        }
        advance(&mut app, 200);

        let state = app.world().resource::<WindDamageState>();
        assert_eq!(state.accumulated_building_damage, 0.0);
        assert_eq!(state.trees_knocked_down, 0);
        assert!(!state.power_outage_active);
    }

    #[test]
    fn test_system_skips_non_interval_ticks() {
        let mut app = wind_damage_test_app();
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.8;
        }
        // Tick 50 is not a multiple of 100, so no update
        advance(&mut app, 50);

        let state = app.world().resource::<WindDamageState>();
        assert_eq!(state.current_tier, WindDamageTier::Calm); // default, never updated
        assert_eq!(state.accumulated_building_damage, 0.0);
    }

    #[test]
    fn test_system_event_fired_on_damage() {
        let mut app = wind_damage_test_app();
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.8;
        }
        advance(&mut app, 100);

        let events = app.world().resource::<Events<WindDamageEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "WindDamageEvent should fire when damage occurs"
        );
        let evt = &fired[0];
        assert_eq!(evt.tier, WindDamageTier::Severe);
        assert!(evt.building_damage > 0.0);
    }

    #[test]
    fn test_system_no_event_at_calm() {
        let mut app = wind_damage_test_app();
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.1;
        }
        advance(&mut app, 100);

        let events = app.world().resource::<Events<WindDamageEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            fired.is_empty(),
            "No WindDamageEvent should fire at calm wind"
        );
    }

    #[test]
    fn test_damage_accumulates_over_ticks() {
        let mut app = wind_damage_test_app();
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.7; // Storm tier
        }

        advance(&mut app, 100);
        let damage_after_1 = app
            .world()
            .resource::<WindDamageState>()
            .accumulated_building_damage;

        advance(&mut app, 200);
        let damage_after_2 = app
            .world()
            .resource::<WindDamageState>()
            .accumulated_building_damage;

        assert!(
            damage_after_2 > damage_after_1,
            "Damage should accumulate: {} should be > {}",
            damage_after_2,
            damage_after_1
        );

        // Should be roughly 2x
        let expected = wind_damage_amount(0.7) * 2.0;
        assert!(
            (damage_after_2 - expected).abs() < 0.01,
            "Expected ~{}, got {}",
            expected,
            damage_after_2
        );
    }

    #[test]
    fn test_splitmix64_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        assert_ne!(splitmix64(42), splitmix64(43));
    }

    #[test]
    fn test_rand_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_f32(seed);
            assert!(
                (0.0..1.0).contains(&val),
                "rand_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }
}
