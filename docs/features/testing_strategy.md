# Megacity Testing Strategy

## Table of Contents

1. [Unit Testing Simulation Formulas](#1-unit-testing-simulation-formulas)
2. [Property-Based Testing](#2-property-based-testing-proptestquickcheck)
3. [Integration Testing](#3-integration-testing-system-interactions)
4. [Deterministic Simulation](#4-deterministic-simulation)
5. [Benchmark and Performance Testing](#5-benchmark-and-performance-testing)
6. [Stress Testing](#6-stress-testing)
7. [Save/Load Round-Trip Testing](#7-saveload-round-trip-testing)
8. [Visual and Rendering Testing](#8-visual-and-rendering-testing)
9. [Gameplay and Balance Testing](#9-gameplay-and-balance-testing)
10. [Regression Testing](#10-regression-testing)
11. [Test Infrastructure](#11-test-infrastructure)
12. [Testing Patterns from Other Simulation Games](#12-testing-patterns-from-other-simulation-games)
13. [Bevy-Specific Testing Patterns](#13-bevy-specific-testing-patterns)

---

## 1. Unit Testing Simulation Formulas

City simulation games are built on layers of interconnected mathematical formulas. Traffic delay, land value, tax revenue, happiness scores, congestion levels, pollution diffusion -- each of these is a pure or near-pure function that can be tested in complete isolation from the ECS world. This section covers how to systematically test every simulation formula in the Megacity codebase.

### 1.1 Testing Pure Functions

The cleanest tests target functions with no side effects: they take inputs and return outputs. Megacity has several of these scattered across its simulation crate.

**Traffic Congestion and Path Cost**

The `TrafficGrid` in `crates/simulation/src/traffic.rs` provides two key formulas:

```rust
// congestion_level: maps raw density (u16) to a 0.0..1.0 range
pub fn congestion_level(&self, x: usize, y: usize) -> f32 {
    let d = self.get(x, y) as f32;
    (d / 20.0).min(1.0)
}

// path_cost: base cost + congestion penalty
pub fn path_cost(&self, x: usize, y: usize) -> u32 {
    let base = 1u32;
    let congestion_penalty = (self.congestion_level(x, y) * 5.0) as u32;
    base + congestion_penalty
}
```

These are already partially tested in the codebase. But thorough unit testing means covering every meaningful input region:

```rust
#[cfg(test)]
mod traffic_formula_tests {
    use super::*;

    #[test]
    fn test_congestion_level_zero_density() {
        let traffic = TrafficGrid::default();
        assert_eq!(traffic.congestion_level(0, 0), 0.0);
    }

    #[test]
    fn test_congestion_level_half() {
        let mut traffic = TrafficGrid::default();
        traffic.set(10, 10, 10);
        let level = traffic.congestion_level(10, 10);
        assert!((level - 0.5).abs() < 0.001,
            "density=10 should give congestion 0.5, got {}", level);
    }

    #[test]
    fn test_congestion_level_saturates_at_one() {
        let mut traffic = TrafficGrid::default();
        traffic.set(5, 5, 100); // well above 20
        assert_eq!(traffic.congestion_level(5, 5), 1.0);

        traffic.set(5, 5, u16::MAX);
        assert_eq!(traffic.congestion_level(5, 5), 1.0);
    }

    #[test]
    fn test_congestion_level_at_boundary() {
        let mut traffic = TrafficGrid::default();
        traffic.set(0, 0, 20); // exactly at threshold
        assert!((traffic.congestion_level(0, 0) - 1.0).abs() < 0.001);

        traffic.set(0, 0, 19); // just below
        assert!(traffic.congestion_level(0, 0) < 1.0);
    }

    #[test]
    fn test_path_cost_monotonically_increases() {
        let mut traffic = TrafficGrid::default();
        let mut prev_cost = traffic.path_cost(0, 0);
        for density in 1..=30 {
            traffic.set(0, 0, density);
            let cost = traffic.path_cost(0, 0);
            assert!(cost >= prev_cost,
                "path_cost should be monotonically non-decreasing: \
                 density={}, cost={}, prev_cost={}", density, cost, prev_cost);
            prev_cost = cost;
        }
    }

    #[test]
    fn test_path_cost_minimum_is_one() {
        let traffic = TrafficGrid::default();
        assert_eq!(traffic.path_cost(0, 0), 1,
            "empty road should have base cost of 1");
    }

    #[test]
    fn test_path_cost_with_road_type_speed_scaling() {
        let mut traffic = TrafficGrid::default();
        traffic.set(10, 10, 0); // no congestion

        let cost_local = traffic.path_cost_with_road(10, 10, RoadType::Local);
        let cost_highway = traffic.path_cost_with_road(10, 10, RoadType::Highway);

        assert!(cost_highway < cost_local,
            "highway (speed=100) should have lower base cost than local (speed=30): \
             highway={}, local={}", cost_highway, cost_local);
    }
}
```

**BPR Traffic Delay Function**

The Bureau of Public Roads (BPR) delay function is a standard traffic engineering formula. If Megacity implements or plans to implement it for segment-level traffic modeling, it should be tested as a pure function:

```rust
/// BPR volume-delay function:
/// delay(v, c) = free_flow_time * (1 + alpha * (v/c)^beta)
///
/// v = volume (vehicles), c = capacity
/// alpha typically 0.15, beta typically 4.0
fn bpr_delay(volume: f32, capacity: f32, alpha: f32, beta: f32) -> f32 {
    if capacity <= 0.0 {
        return f32::MAX; // guard against division by zero
    }
    let ratio = volume / capacity;
    1.0 * (1.0 + alpha * ratio.powf(beta))
}

#[test]
fn test_bpr_delay_empty_road() {
    // Zero volume: delay should be exactly free-flow time (1.0)
    let delay = bpr_delay(0.0, 1.0, 0.15, 4.0);
    assert!((delay - 1.0).abs() < 0.0001);
}

#[test]
fn test_bpr_delay_at_capacity() {
    // v/c = 1.0: delay = 1.0 * (1 + 0.15 * 1^4) = 1.15
    let delay = bpr_delay(1.0, 1.0, 0.15, 4.0);
    assert!((delay - 1.15).abs() < 0.001);
}

#[test]
fn test_bpr_delay_half_capacity() {
    // v/c = 0.5: delay = 1.0 * (1 + 0.15 * 0.5^4) = 1.0 + 0.15 * 0.0625 = 1.009375
    let delay = bpr_delay(0.5, 1.0, 0.15, 4.0);
    assert!((delay - 1.009375).abs() < 0.001,
        "BPR delay at v/c=0.5 should be ~1.009, got {}", delay);
}

#[test]
fn test_bpr_delay_over_capacity() {
    // v/c = 2.0: delay = 1.0 * (1 + 0.15 * 16) = 3.4
    let delay = bpr_delay(2.0, 1.0, 0.15, 4.0);
    assert!((delay - 3.4).abs() < 0.001);
}

#[test]
fn test_bpr_delay_zero_capacity_guard() {
    let delay = bpr_delay(10.0, 0.0, 0.15, 4.0);
    assert_eq!(delay, f32::MAX);
}
```

**Land Value Calculation**

The `LandValueGrid` in `crates/simulation/src/land_value.rs` computes land values based on water proximity, zone type, pollution, and service buildings. The formula itself runs inside the `update_land_value` system, which makes it harder to test as a pure function -- but we can extract the formula and test it:

```rust
#[test]
fn test_land_value_baseline() {
    let grid = LandValueGrid::default();
    // Default baseline is 50 for all cells
    assert_eq!(grid.get(0, 0), 50);
    assert_eq!(grid.get(128, 128), 50);
}

#[test]
fn test_land_value_average_on_uniform() {
    let grid = LandValueGrid::default();
    let avg = grid.average();
    assert!((avg - 50.0).abs() < 0.1,
        "uniform grid should have average 50.0, got {}", avg);
}

#[test]
fn test_land_value_average_empty() {
    let grid = LandValueGrid {
        values: Vec::new(),
        width: 0,
        height: 0,
    };
    assert_eq!(grid.average(), 0.0);
}

#[test]
fn test_land_value_clamped_to_u8_range() {
    let mut grid = LandValueGrid::default();
    // Simulate the clamping that happens in update_land_value:
    // value = (some_computation).clamp(0, 255) as u8
    grid.set(10, 10, 255);
    assert_eq!(grid.get(10, 10), 255);
    grid.set(10, 10, 0);
    assert_eq!(grid.get(10, 10), 0);
}
```

**Tax Revenue Formula**

From `crates/simulation/src/economy.rs`, the tax calculation follows:
- Base: `10.0 * tax_rate * population`
- Commercial: occupants * rate-per-zone-type
- Land value multiplier: `income *= 1.0 + (avg_land_value / 500.0)`

```rust
#[test]
fn test_tax_per_citizen_scales_with_rate() {
    for rate in [0.0_f32, 0.05, 0.1, 0.15, 0.2, 0.5, 1.0] {
        let tax = 10.0 * rate as f64;
        assert!((tax - rate as f64 * 10.0).abs() < 0.001);
        assert!(tax >= 0.0, "tax should never be negative");
    }
}

#[test]
fn test_zero_population_zero_income() {
    let pop = 0.0_f64;
    let tax_per_citizen = 10.0 * 0.1_f64;
    let income = pop * tax_per_citizen;
    assert_eq!(income, 0.0);
}

#[test]
fn test_land_value_multiplier_at_baseline() {
    // avg_land_value = 50 (baseline)
    // multiplier = 1.0 + (50.0 / 500.0) = 1.1
    let income = 1000.0_f64;
    let avg_land_value = 50.0_f64;
    let adjusted = income * (1.0 + avg_land_value / 500.0);
    assert!((adjusted - 1100.0).abs() < 0.01);
}

#[test]
fn test_land_value_multiplier_zero() {
    let income = 1000.0_f64;
    let avg_land_value = 0.0_f64;
    let adjusted = income * (1.0 + avg_land_value / 500.0);
    assert!((adjusted - 1000.0).abs() < 0.01,
        "zero land value should not modify income");
}
```

**Happiness Score Calculation**

The happiness system in `crates/simulation/src/happiness.rs` is particularly important because it drives citizen behavior (emigration, immigration, building upgrades). The formula is a sum of bonuses and penalties clamped to 0.0..100.0:

```rust
#[test]
fn test_happiness_base_is_midrange() {
    // BASE_HAPPINESS = 50.0 -- a citizen with nothing gets 50
    assert_eq!(BASE_HAPPINESS, 50.0);
}

#[test]
fn test_happiness_all_penalties_cannot_go_below_zero() {
    // Worst case: no employment, no power, no water, max pollution,
    // max crime, max congestion, high taxes, homeless
    let mut happiness = BASE_HAPPINESS;
    happiness -= NO_POWER_PENALTY;      // -25
    happiness -= NO_WATER_PENALTY;      // -20
    happiness -= CRIME_PENALTY_MAX;     // -15
    happiness -= CONGESTION_PENALTY;    // -5
    happiness -= GARBAGE_PENALTY;       // -5
    happiness -= HIGH_TAX_PENALTY;      // -8
    happiness -= HOMELESS_PENALTY;      // -30
    // Raw could go very negative
    let clamped = happiness.clamp(0.0, 100.0);
    assert_eq!(clamped, 0.0);
}

#[test]
fn test_happiness_all_bonuses_caps_at_100() {
    let mut happiness = BASE_HAPPINESS;
    happiness += EMPLOYED_BONUS;
    happiness += SHORT_COMMUTE_BONUS;
    happiness += POWER_BONUS;
    happiness += WATER_BONUS;
    happiness += HEALTH_COVERAGE_BONUS;
    happiness += EDUCATION_BONUS;
    happiness += POLICE_BONUS;
    happiness += PARK_BONUS;
    happiness += ENTERTAINMENT_BONUS;
    happiness += TELECOM_BONUS;
    happiness += TRANSPORT_BONUS;
    happiness += 255.0 / 50.0; // max land value bonus
    let clamped = happiness.clamp(0.0, 100.0);
    assert_eq!(clamped, 100.0);
}

#[test]
fn test_tax_penalty_threshold() {
    // No penalty at tax_rate <= 0.15
    let rate_ok = 0.15_f32;
    let penalty_ok = if rate_ok > 0.15 {
        HIGH_TAX_PENALTY * ((rate_ok - 0.15) / 0.10)
    } else {
        0.0
    };
    assert_eq!(penalty_ok, 0.0);

    // Penalty at 0.25: HIGH_TAX_PENALTY * (0.10 / 0.10) = HIGH_TAX_PENALTY
    let rate_high = 0.25_f32;
    let penalty_high = if rate_high > 0.15 {
        HIGH_TAX_PENALTY * ((rate_high - 0.15) / 0.10)
    } else {
        0.0
    };
    assert!((penalty_high - HIGH_TAX_PENALTY).abs() < 0.01);
}
```

**Needs Satisfaction**

The `Needs` component in `crates/simulation/src/citizen.rs` uses a weighted average:

```rust
#[test]
fn test_needs_satisfaction_weights_sum_to_one() {
    // hunger=0.25, energy=0.25, social=0.15, fun=0.15, comfort=0.20
    let sum = 0.25 + 0.25 + 0.15 + 0.15 + 0.20;
    assert!((sum - 1.0).abs() < 0.001,
        "needs weights should sum to 1.0 for proper normalization");
}

#[test]
fn test_needs_satisfaction_all_max() {
    let needs = Needs {
        hunger: 100.0, energy: 100.0, social: 100.0,
        fun: 100.0, comfort: 100.0,
    };
    assert!((needs.overall_satisfaction() - 1.0).abs() < 0.001);
}

#[test]
fn test_needs_satisfaction_all_zero() {
    let needs = Needs {
        hunger: 0.0, energy: 0.0, social: 0.0,
        fun: 0.0, comfort: 0.0,
    };
    assert_eq!(needs.overall_satisfaction(), 0.0);
}

#[test]
fn test_needs_satisfaction_clamps() {
    // Even if individual needs exceed 100, satisfaction caps at 1.0
    let needs = Needs {
        hunger: 200.0, energy: 200.0, social: 200.0,
        fun: 200.0, comfort: 200.0,
    };
    assert!(needs.overall_satisfaction() <= 1.0);
}

#[test]
fn test_most_critical_need() {
    let needs = Needs {
        hunger: 80.0, energy: 50.0, social: 30.0,
        fun: 60.0, comfort: 70.0,
    };
    let (name, value) = needs.most_critical();
    assert_eq!(name, "social");
    assert!((value - 30.0).abs() < 0.001);
}
```

### 1.2 Edge Cases

Every simulation formula has edge cases that can cause bugs in production. The most dangerous are:

**Zero population:**
```rust
#[test]
fn test_city_stats_zero_population() {
    // Average happiness with no citizens must not panic or produce NaN
    let citizens: Vec<f32> = vec![];
    let avg = if citizens.is_empty() {
        0.0
    } else {
        citizens.iter().sum::<f32>() / citizens.len() as f32
    };
    assert_eq!(avg, 0.0);
    assert!(!avg.is_nan());
}
```

**Empty grid:**
```rust
#[test]
fn test_grid_operations_on_minimum_size() {
    let grid = WorldGrid::new(1, 1);
    assert!(grid.in_bounds(0, 0));
    assert!(!grid.in_bounds(1, 0));
    assert_eq!(grid.neighbors4(0, 0).1, 0); // corner has no neighbors in 1x1
}
```

**Maximum capacity:**
```rust
#[test]
fn test_traffic_density_saturating_add() {
    let mut traffic = TrafficGrid::default();
    traffic.set(0, 0, u16::MAX);
    // saturating_add should not overflow
    let val = traffic.get(0, 0).saturating_add(1);
    assert_eq!(val, u16::MAX);
}
```

**Division by zero guards:**
```rust
#[test]
fn test_no_division_by_zero_in_averages() {
    let grid = LandValueGrid {
        values: vec![],
        width: 0,
        height: 0,
    };
    assert_eq!(grid.average(), 0.0);
    assert!(!grid.average().is_nan());
    assert!(!grid.average().is_infinite());
}
```

### 1.3 Floating Point Comparison

Never use `==` for floating point values in tests. Megacity already uses `.abs() < epsilon` patterns throughout the codebase, which is correct. Here is the convention to follow:

```rust
/// Standard epsilon for float comparison in simulation tests.
/// This is large enough to absorb floating point rounding but small
/// enough to catch real formula bugs.
const TEST_EPSILON: f32 = 0.001;
const TEST_EPSILON_F64: f64 = 0.0001;

/// Helper for asserting approximate equality with meaningful error messages.
fn assert_approx(actual: f32, expected: f32, context: &str) {
    assert!(
        (actual - expected).abs() < TEST_EPSILON,
        "{}: expected ~{}, got {} (diff={})",
        context, expected, actual, (actual - expected).abs()
    );
}

// Usage:
#[test]
fn test_congestion_formula() {
    let mut traffic = TrafficGrid::default();
    traffic.set(10, 10, 10);
    assert_approx(
        traffic.congestion_level(10, 10),
        0.5,
        "density=10 congestion level"
    );
}
```

For f64 values (like treasury calculations), use a tighter epsilon because f64 has more precision, but the values involved (money) can be large:

```rust
#[test]
fn test_treasury_arithmetic() {
    let mut treasury = 100_000.0_f64;
    let income = 5_432.10;
    let expenses = 3_210.55;
    treasury += income - expenses;
    assert!((treasury - 102_221.55).abs() < TEST_EPSILON_F64);
}
```

**NaN and Infinity checks:** Every formula that divides should be tested for NaN/Inf behavior:

```rust
#[test]
fn test_no_nan_in_simulation_values() {
    // Treasury should never be NaN
    let budget = CityBudget::default();
    assert!(!budget.treasury.is_nan());
    assert!(!budget.treasury.is_infinite());
    assert!(budget.treasury.is_finite());

    // Tax rate should be in valid range
    assert!(budget.tax_rate >= 0.0);
    assert!(budget.tax_rate <= 1.0);
    assert!(!budget.tax_rate.is_nan());
}
```

### 1.4 Bevy-Specific: Testing Systems in Isolation

Many simulation formulas live inside Bevy systems that take `Res<>`, `ResMut<>`, and `Query<>` parameters. Testing these requires creating a minimal `World` with just enough resources and entities. The key insight: you do NOT need a full `App` for formula testing -- you only need the `World`.

```rust
#[cfg(test)]
mod system_isolation_tests {
    use bevy::ecs::world::World;
    use bevy::ecs::system::SystemState;
    use super::*;

    #[test]
    fn test_traffic_grid_congestion_in_world() {
        let mut world = World::new();
        let mut traffic = TrafficGrid::default();
        traffic.set(10, 10, 15);
        world.insert_resource(traffic);

        // Read back from world
        let traffic = world.resource::<TrafficGrid>();
        assert!((traffic.congestion_level(10, 10) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_budget_resource_in_world() {
        let mut world = World::new();
        world.insert_resource(CityBudget {
            treasury: 50_000.0,
            tax_rate: 0.12,
            ..Default::default()
        });

        let budget = world.resource::<CityBudget>();
        assert!((budget.treasury - 50_000.0).abs() < 0.01);
        assert!((budget.tax_rate - 0.12).abs() < 0.001);
    }
}
```

For testing actual systems (not just resources), use `SystemState`:

```rust
#[test]
fn test_slow_tick_timer_should_run() {
    let mut timer = SlowTickTimer::default();
    // Should not run at counter=0... actually it is: 0 % 100 == 0
    // This is intentional: the first tick runs the slow systems
    assert!(timer.should_run());

    timer.tick();
    assert!(!timer.should_run()); // counter=1, not multiple of 100

    // Fast-forward to 100
    for _ in 1..100 {
        timer.tick();
    }
    assert!(timer.should_run()); // counter=100
}
```

### 1.5 Building Capacity Formula

The `Building::capacity_for_level` function is a lookup table that defines the entire population scaling of the game. Every entry must be tested:

```rust
#[test]
fn test_capacity_increases_with_level() {
    for zone in [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ] {
        let max_level = zone.max_level();
        let mut prev_cap = 0;
        for level in 1..=max_level {
            let cap = Building::capacity_for_level(zone, level);
            assert!(cap > prev_cap,
                "capacity should increase with level: {:?} L{} cap={} <= prev={}",
                zone, level, cap, prev_cap);
            prev_cap = cap;
        }
    }
}

#[test]
fn test_capacity_zero_for_invalid() {
    assert_eq!(Building::capacity_for_level(ZoneType::None, 0), 0);
    assert_eq!(Building::capacity_for_level(ZoneType::None, 1), 0);
}
```

### 1.6 Coordinate System Tests

Grid-to-world and world-to-grid conversions must round-trip perfectly. The existing test in `grid.rs` covers this, but we should extend it:

```rust
#[test]
fn test_coordinate_roundtrip_all_corners() {
    for (gx, gy) in [(0, 0), (0, 255), (255, 0), (255, 255)] {
        let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
        let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
        assert_eq!((rx as usize, ry as usize), (gx, gy),
            "roundtrip failed for grid ({}, {})", gx, gy);
    }
}

#[test]
fn test_world_to_grid_center_of_cell() {
    // grid_to_world places at center: gx * CELL_SIZE + CELL_SIZE/2
    let (wx, wy) = WorldGrid::grid_to_world(10, 20);
    assert_eq!(wx, 10.0 * 16.0 + 8.0); // 168.0
    assert_eq!(wy, 20.0 * 16.0 + 8.0); // 328.0
}

#[test]
fn test_world_to_grid_handles_cell_edges() {
    // Any point within a cell should map to that cell's grid coords
    // Cell (5, 5) spans world x: [80.0, 96.0), y: [80.0, 96.0)
    assert_eq!(WorldGrid::world_to_grid(80.0, 80.0), (5, 5));
    assert_eq!(WorldGrid::world_to_grid(95.99, 95.99), (5, 5));
    assert_eq!(WorldGrid::world_to_grid(96.0, 96.0), (6, 6)); // next cell
}
```

---

## 2. Property-Based Testing (proptest/quickcheck)

Traditional unit tests check specific input-output pairs. Property-based tests check that invariants hold across *thousands* of randomly generated inputs. For a city simulation with vast state spaces, this is invaluable -- it finds the edge cases you would never think to test manually.

### 2.1 Adding proptest to the Project

Add proptest to the simulation crate's dev-dependencies:

```toml
# crates/simulation/Cargo.toml
[dev-dependencies]
proptest = "1.4"
```

### 2.2 Invariants That Must ALWAYS Hold

These are properties that should be true no matter what the game state looks like. If any of these fail, it indicates a bug.

**Population is never negative:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn population_count_is_non_negative(
        occupants in prop::collection::vec(0u32..2000, 0..500)
    ) {
        let total: u32 = occupants.iter().sum();
        // u32 cannot be negative, but the invariant is that we never
        // use i32/i64 for population and accidentally go negative
        prop_assert!(total <= u32::MAX);
    }
}
```

**Money is always finite (no NaN/Inf):**

```rust
proptest! {
    #[test]
    fn treasury_stays_finite(
        initial in -1_000_000.0f64..1_000_000.0,
        income in 0.0f64..100_000.0,
        expenses in 0.0f64..100_000.0,
    ) {
        let result = initial + income - expenses;
        prop_assert!(result.is_finite(),
            "treasury went non-finite: {} + {} - {} = {}",
            initial, income, expenses, result);
        prop_assert!(!result.is_nan());
    }
}

proptest! {
    #[test]
    fn land_value_multiplier_never_nan(
        avg_land_value in 0.0f64..255.0,
        income in 0.0f64..1_000_000.0,
    ) {
        let adjusted = income * (1.0 + avg_land_value / 500.0);
        prop_assert!(adjusted.is_finite());
        prop_assert!(!adjusted.is_nan());
        prop_assert!(adjusted >= 0.0, "adjusted income should be non-negative");
    }
}
```

**Grid indices always in bounds:**

```rust
use crate::config::{GRID_WIDTH, GRID_HEIGHT};

proptest! {
    #[test]
    fn grid_index_never_panics(
        x in 0..GRID_WIDTH,
        y in 0..GRID_HEIGHT,
    ) {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // This should never panic
        let _ = grid.get(x, y);
        let _ = grid.index(x, y);
        prop_assert!(grid.in_bounds(x, y));
    }

    #[test]
    fn traffic_grid_index_never_panics(
        x in 0..GRID_WIDTH,
        y in 0..GRID_HEIGHT,
        density in 0u16..u16::MAX,
    ) {
        let mut traffic = TrafficGrid::default();
        traffic.set(x, y, density);
        let retrieved = traffic.get(x, y);
        prop_assert_eq!(retrieved, density);

        let congestion = traffic.congestion_level(x, y);
        prop_assert!(congestion >= 0.0);
        prop_assert!(congestion <= 1.0);
    }
}
```

**Total occupants across buildings equals citizens with homes:**

This is a critical cross-system invariant. After any simulation tick, the sum of all building occupant counts should equal the number of citizens who have a `HomeLocation` pointing to a residential building (and similarly for work buildings).

```rust
/// This test would run inside a Bevy App test harness (see section 3).
/// Here we show the invariant check as a standalone function.
fn assert_occupant_consistency(
    buildings: &[(Entity, &Building)],
    citizens_with_homes: &[(Entity, &HomeLocation)],
) {
    use std::collections::HashMap;

    // Count citizens per building
    let mut citizen_counts: HashMap<Entity, u32> = HashMap::new();
    for (_, home) in citizens_with_homes {
        *citizen_counts.entry(home.building).or_default() += 1;
    }

    // Compare with building occupant fields
    for (entity, building) in buildings {
        if !building.zone_type.is_residential() {
            continue;
        }
        let actual_citizens = citizen_counts.get(entity).copied().unwrap_or(0);
        assert_eq!(
            building.occupants, actual_citizens,
            "Building {:?} at ({},{}) reports {} occupants but {} citizens claim it as home",
            entity, building.grid_x, building.grid_y,
            building.occupants, actual_citizens
        );
    }
}
```

**Road network graph is consistent with road segment store:**

```rust
fn assert_road_graph_consistency(
    road_network: &RoadNetwork,
    csr: &CsrGraph,
    grid: &WorldGrid,
) {
    // Every road cell in the grid should have a corresponding node in the network
    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x, y).cell_type == CellType::Road {
                let node = RoadNode(x, y);
                assert!(
                    road_network.edges.contains_key(&node),
                    "Road cell ({},{}) has no entry in RoadNetwork", x, y
                );
            }
        }
    }

    // CSR graph node count should match road network
    assert_eq!(
        csr.node_count(),
        road_network.edges.len(),
        "CSR graph has {} nodes but RoadNetwork has {} entries",
        csr.node_count(), road_network.edges.len()
    );

    // Every CSR edge should be bidirectional (roads are undirected)
    for node_idx in 0..csr.node_count() as u32 {
        for &neighbor_idx in csr.neighbors(node_idx) {
            let reverse_neighbors = csr.neighbors(neighbor_idx);
            assert!(
                reverse_neighbors.contains(&node_idx),
                "Edge {} -> {} exists but reverse {} -> {} does not",
                node_idx, neighbor_idx, neighbor_idx, node_idx
            );
        }
    }
}
```

**Budget income minus expenses equals treasury delta:**

```rust
proptest! {
    #[test]
    fn budget_accounting_identity(
        treasury_before in 0.0f64..1_000_000.0,
        income in 0.0f64..100_000.0,
        expenses in 0.0f64..100_000.0,
    ) {
        let delta = income - expenses;
        let treasury_after = treasury_before + delta;

        // Accounting identity: after - before == income - expenses
        let actual_delta = treasury_after - treasury_before;
        prop_assert!(
            (actual_delta - delta).abs() < 0.0001,
            "Accounting identity violated: delta={}, actual={}",
            delta, actual_delta
        );
    }
}
```

**Citizen state machine: only valid transitions:**

The `CitizenState` enum has strict transition rules. A citizen at `AtHome` can transition to `CommutingToWork`, `CommutingToShop`, `CommutingToLeisure`, or `CommutingToSchool` -- but never directly to `Working` or `Shopping` (except for `Abstract` LOD tier citizens who skip commuting).

```rust
fn is_valid_transition(from: CitizenState, to: CitizenState) -> bool {
    matches!(
        (from, to),
        // From AtHome: can start commuting to various destinations
        (CitizenState::AtHome, CitizenState::CommutingToWork)
        | (CitizenState::AtHome, CitizenState::CommutingToShop)
        | (CitizenState::AtHome, CitizenState::CommutingToLeisure)
        | (CitizenState::AtHome, CitizenState::CommutingToSchool)
        // Commuting arrives at destination
        | (CitizenState::CommutingToWork, CitizenState::Working)
        | (CitizenState::CommutingHome, CitizenState::AtHome)
        | (CitizenState::CommutingToShop, CitizenState::Shopping)
        | (CitizenState::CommutingToLeisure, CitizenState::AtLeisure)
        | (CitizenState::CommutingToSchool, CitizenState::AtSchool)
        // From destinations: start commuting home or to next destination
        | (CitizenState::Working, CitizenState::CommutingHome)
        | (CitizenState::Working, CitizenState::CommutingToShop)
        | (CitizenState::Working, CitizenState::CommutingToLeisure)
        | (CitizenState::Shopping, CitizenState::CommutingHome)
        | (CitizenState::AtLeisure, CitizenState::CommutingHome)
        | (CitizenState::AtSchool, CitizenState::CommutingHome)
    )
}
```

A runtime instrumentation approach can catch violations during integration tests:

```rust
/// System that asserts no invalid state transitions occurred this tick.
/// Add to the test App only (not production).
fn assert_valid_transitions(
    query: Query<(Entity, &CitizenStateComp, &PreviousState), With<Citizen>>,
) {
    for (entity, current, previous) in &query {
        if current.0 != previous.0 {
            assert!(
                is_valid_transition(previous.0, current.0),
                "Invalid state transition for {:?}: {:?} -> {:?}",
                entity, previous.0, current.0
            );
        }
    }
}
```

### 2.3 Generating Random Valid Game States

To use proptest effectively with a city simulation, you need custom strategies that generate valid game states. A random `WorldGrid` with random zones and random buildings is useless -- the state must be internally consistent.

```rust
use proptest::prelude::*;
use proptest::strategy::Strategy;

/// Generate a small, valid WorldGrid with connected roads and zones.
fn arb_small_grid() -> impl Strategy<Value = WorldGrid> {
    (8usize..=32, 8usize..=32).prop_flat_map(|(w, h)| {
        let total = w * h;
        prop::collection::vec(
            prop::sample::select(vec![
                CellType::Grass, CellType::Grass, CellType::Grass, CellType::Road,
            ]),
            total,
        )
        .prop_map(move |cell_types| {
            let mut grid = WorldGrid::new(w, h);
            for (i, ct) in cell_types.into_iter().enumerate() {
                grid.cells[i].cell_type = ct;
            }
            grid
        })
    })
}

/// Generate a valid TrafficGrid with random densities.
fn arb_traffic_grid() -> impl Strategy<Value = TrafficGrid> {
    prop::collection::vec(0u16..100, GRID_WIDTH * GRID_HEIGHT)
        .prop_map(|densities| {
            TrafficGrid {
                density: densities,
                width: GRID_WIDTH,
                height: GRID_HEIGHT,
            }
        })
}

/// Generate a valid CityBudget with constrained values.
fn arb_budget() -> impl Strategy<Value = CityBudget> {
    (
        -100_000.0f64..1_000_000.0, // treasury
        0.0f32..0.5,                 // tax_rate
    )
    .prop_map(|(treasury, tax_rate)| CityBudget {
        treasury,
        tax_rate,
        ..Default::default()
    })
}
```

### 2.4 Shrinking: Finding Minimal Reproductions

When proptest finds a failing input, it automatically "shrinks" the input to find the smallest reproducing example. This is critical for debugging simulation bugs.

Example: if a pathfinding bug manifests with a random 50-segment road network, proptest will shrink it down to perhaps 3 segments:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn pathfinding_never_panics(
        road_count in 2usize..50,
        seed in 0u64..10000,
    ) {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(seed);
        let mut grid = WorldGrid::new(64, 64);
        let mut network = RoadNetwork::default();

        let mut x = 32usize;
        let mut y = 32usize;
        for _ in 0..road_count {
            network.place_road(&mut grid, x, y);
            match rng.gen_range(0..4) {
                0 if x > 0 => x -= 1,
                1 if x < 63 => x += 1,
                2 if y > 0 => y -= 1,
                3 if y < 63 => y += 1,
                _ => {}
            }
        }

        let csr = CsrGraph::from_road_network(&network);
        let start = RoadNode(32, 32);
        let goal = RoadNode(x, y);
        // Must not panic. Result can be None (no path found).
        let _result = csr_find_path(&csr, start, goal);
    }
}
```

When this fails, proptest outputs the minimal reproduction:

```
proptest: Minimal failing input:
  road_count = 3, seed = 42
```

The regression file (`.proptest-regressions`) automatically records failures for future runs.

### 2.5 Key Invariants Checklist

| Invariant | Systems Involved | Priority |
|-----------|-----------------|----------|
| Population >= 0 | citizen_spawner, lifecycle | Critical |
| Treasury is finite (no NaN/Inf) | economy, budget, loans | Critical |
| Grid indices in bounds | all grid-based systems | Critical |
| Building occupants <= capacity | citizen_spawner, buildings | Critical |
| Citizens with homes reference valid buildings | citizen_spawner, abandonment | Critical |
| Road network is symmetric (undirected) | road_segments, roads | High |
| CSR graph matches RoadNetwork | road_graph_csr, roads | High |
| Congestion in [0.0, 1.0] | traffic | High |
| Happiness in [0.0, 100.0] | happiness | High |
| Health in [0.0, 100.0] | health, life_simulation | High |
| Land value in [0, 255] | land_value | High |
| Tax rate in [0.0, 1.0] | economy, budget | High |
| Service coverage bitflags are valid | happiness | Medium |
| Zone types match building types | buildings, zones | Medium |
| PathCache index <= waypoints.len() | movement | Medium |
| Needs values in [0.0, 100.0] | life_simulation | Medium |
| Building level <= zone max_level | building_upgrade | Medium |
| Loan remaining_balance >= 0 | loans | Medium |

---

## 3. Integration Testing (System Interactions)

Unit tests prove individual formulas work. Integration tests prove the *systems work together*. In a city simulation, the most important bugs live at system boundaries: road placement triggers zone eligibility which triggers building spawn which triggers citizen move-in which triggers economy changes. If any link in this chain breaks, the city stops growing.

### 3.1 The Causal Chain

Megacity has several critical causal chains where the output of one system becomes the input of the next. The most important chain for city growth is:

```
Road Placement
  -> Grid cells marked as CellType::Road
  -> RoadNetwork updated, CSR graph rebuilt
  -> Adjacent cells become zone-eligible
  -> ZoneDemand > 0 triggers building_spawner
  -> Buildings spawn with UnderConstruction component
  -> progress_construction removes UnderConstruction after 100 ticks
  -> citizen_spawner creates citizens in buildings with capacity
  -> Citizens get HomeLocation, WorkLocation, PathCache
  -> citizen_state_machine starts commute cycles
  -> Traffic density increases
  -> Economy collects taxes based on population
  -> Treasury grows
```

Testing this chain end-to-end requires a Bevy `App` with all simulation systems registered.

### 3.2 Bevy Test Pattern: App-Based Integration Tests

The standard pattern for integration testing Bevy systems:

```rust
#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use simulation::*;

    /// Create a minimal test App with the SimulationPlugin.
    /// This registers all systems, resources, and schedules.
    fn test_app() -> App {
        let mut app = App::new();
        // MinimalPlugins gives us time, scheduling, etc. without rendering
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);
        app
    }

    /// Run N FixedUpdate ticks on the App.
    fn run_ticks(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    #[test]
    fn test_road_to_building_chain() {
        let mut app = test_app();
        app.update(); // initial setup tick

        // Place a road segment
        {
            let world = app.world_mut();
            let mut grid = world.resource_mut::<WorldGrid>();
            let mut roads = world.resource_mut::<RoadNetwork>();
            for x in 10..20 {
                roads.place_road(&mut grid, x, 10);
            }

            // Zone cells adjacent to the road
            for x in 10..20 {
                for dy in [-1i32, 1] {
                    let y = (10i32 + dy) as usize;
                    if grid.in_bounds(x, y) {
                        let cell = grid.get_mut(x, y);
                        if cell.cell_type == CellType::Grass {
                            cell.zone = ZoneType::ResidentialLow;
                            cell.has_power = true;
                            cell.has_water = true;
                        }
                    }
                }
            }
        }

        // Run enough ticks for buildings to spawn and complete construction
        // Building construction takes 100 ticks, spawner runs every 2 ticks
        run_ticks(&mut app, 120);

        // Assert buildings were spawned
        let building_count = app
            .world()
            .query::<&Building>()
            .iter(app.world())
            .count();
        assert!(building_count > 0,
            "No buildings spawned after 120 ticks on zoned road-adjacent cells");
    }

    #[test]
    fn test_citizens_spawn_in_completed_buildings() {
        let mut app = test_app();
        app.update();

        // Set up road, zones, and a completed building
        {
            let world = app.world_mut();
            let mut grid = world.resource_mut::<WorldGrid>();

            // Place road
            let mut roads = world.resource_mut::<RoadNetwork>();
            for x in 10..15 {
                roads.place_road(&mut grid, x, 10);
            }

            // Zone residential and provide utilities
            grid.get_mut(12, 11).zone = ZoneType::ResidentialLow;
            grid.get_mut(12, 11).has_power = true;
            grid.get_mut(12, 11).has_water = true;

            // Spawn a completed residential building directly
            let entity = world.spawn(Building {
                zone_type: ZoneType::ResidentialLow,
                level: 1,
                grid_x: 12,
                grid_y: 11,
                capacity: 10,
                occupants: 0,
            }).id();
            grid.get_mut(12, 11).building_id = Some(entity);

            // Also need a work building for citizens
            grid.get_mut(14, 11).zone = ZoneType::CommercialLow;
            grid.get_mut(14, 11).has_power = true;
            grid.get_mut(14, 11).has_water = true;
            let work_entity = world.spawn(Building {
                zone_type: ZoneType::CommercialLow,
                level: 1,
                grid_x: 14,
                grid_y: 11,
                capacity: 8,
                occupants: 0,
            }).id();
            grid.get_mut(14, 11).building_id = Some(work_entity);

            // Ensure zone demand is positive
            let mut demand = world.resource_mut::<ZoneDemand>();
            demand.residential = 1.0;
        }

        // Run enough ticks for citizen spawning
        run_ticks(&mut app, 50);

        let citizen_count = app
            .world()
            .query::<&Citizen>()
            .iter(app.world())
            .count();
        assert!(citizen_count > 0,
            "No citizens spawned after 50 ticks with available housing");
    }
}
```

### 3.3 Testing the Full Causal Chain End-to-End

The ultimate integration test: start from an empty world, place roads, zone, and verify the entire city growth pipeline works:

```rust
#[test]
fn test_full_city_growth_pipeline() {
    let mut app = test_app();
    app.update(); // init

    // Step 1: Place roads
    {
        let world = app.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let mut roads = world.resource_mut::<RoadNetwork>();

        // Create a cross-shaped road network
        for x in 5..25 {
            roads.place_road(&mut grid, x, 15);
        }
        for y in 5..25 {
            roads.place_road(&mut grid, 15, y);
        }
    }

    // Step 2: Zone cells near roads
    {
        let world = app.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        // Residential on one side
        for x in 6..14 {
            for y in 12..14 {
                let cell = grid.get_mut(x, y);
                cell.zone = ZoneType::ResidentialLow;
                cell.has_power = true;
                cell.has_water = true;
            }
        }
        // Commercial on other side
        for x in 16..24 {
            for y in 12..14 {
                let cell = grid.get_mut(x, y);
                cell.zone = ZoneType::CommercialLow;
                cell.has_power = true;
                cell.has_water = true;
            }
        }
        // Set demand
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = 1.0;
        demand.commercial = 1.0;
    }

    // Step 3: Run simulation for 500 ticks (~50 seconds game time)
    run_ticks(&mut app, 500);

    // Step 4: Assert the full chain worked
    let world = app.world();

    // Buildings should exist
    let buildings: Vec<_> = world.query::<&Building>().iter(world).collect();
    assert!(buildings.len() > 0, "No buildings spawned");

    // Citizens should exist
    let citizens: Vec<_> = world.query::<&Citizen>().iter(world).collect();
    assert!(citizens.len() > 0, "No citizens spawned");

    // Economy should have collected taxes
    let budget = world.resource::<CityBudget>();
    // If 500 ticks have passed and there are citizens,
    // at least one tax collection cycle should have occurred
    if citizens.len() > 0 {
        assert!(budget.monthly_income > 0.0 || budget.last_collection_day > 0,
            "Economy not collecting taxes despite {} citizens", citizens.len());
    }

    // Traffic should show some activity
    let traffic = world.resource::<TrafficGrid>();
    let total_traffic: u64 = traffic.density.iter().map(|&d| d as u64).sum();
    // During commute hours, there should be some traffic
    // (this may be 0 if the clock is not in commute window)

    // City stats should be computed
    let stats = world.resource::<CityStats>();
    assert_eq!(stats.population as usize, citizens.len(),
        "CityStats.population doesn't match actual citizen count");
}
```

### 3.4 Testing System Ordering Dependencies

Megacity's simulation systems are carefully ordered with `.chain()` and `.after()` constraints. If these ordering constraints are wrong, systems read stale data. Test this by verifying that data flows correctly between chained systems:

```rust
#[test]
fn test_traffic_updates_before_happiness() {
    // Traffic density should be computed before happiness reads congestion
    let mut app = test_app();
    app.update();

    // Set up a citizen on a congested road
    {
        let world = app.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, 20); // full congestion
    }

    run_ticks(&mut app, 20);

    // If ordering is correct, happiness should reflect the congestion penalty
    // (The actual test depends on having a citizen at that location)
}

#[test]
fn test_service_coverage_updates_before_happiness() {
    // ServiceCoverageGrid must be recomputed before happiness reads it
    let mut app = test_app();
    app.update();

    // Spawn a hospital
    {
        let world = app.world_mut();
        world.spawn(ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 10,
            grid_y: 10,
            radius: ServiceBuilding::coverage_radius(ServiceType::Hospital),
        });
    }

    run_ticks(&mut app, 20);

    // Coverage grid should now have health coverage near the hospital
    let coverage = app.world().resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(10, 10);
    assert!(coverage.has_health(idx),
        "Hospital at (10,10) should provide health coverage");
}
```

### 3.5 Testing Emergency Scenarios

Integration tests for disaster/emergency chains:

```rust
#[test]
fn test_fire_spread_and_extinguish() {
    let mut app = test_app();
    app.update();

    // Place buildings and a fire station
    // ... (setup code) ...

    // Manually start a fire
    {
        let world = app.world_mut();
        let mut fire_grid = world.resource_mut::<FireGrid>();
        fire_grid.set(50, 50, true);
    }

    // Run ticks and verify fire spreads but eventually gets contained
    // if fire station coverage is adequate
    run_ticks(&mut app, 200);

    let fire_grid = app.world().resource::<FireGrid>();
    // Verify fire was handled (depends on fire station placement)
}
```

### 3.6 Apply-Deferred Flush Points

Megacity uses `bevy::ecs::schedule::apply_deferred` at explicit points in the schedule to flush `Commands` (entity spawns, component insertions/removals). This is critical for testing because:

1. `movement::citizen_state_machine` inserts `PathRequest` components via `Commands`
2. `apply_deferred` flushes those insertions
3. `movement::process_path_requests` reads `PathRequest` components

If you test without apply_deferred, the PathRequests will not exist when process_path_requests runs. The test harness using `App::update()` handles this correctly because it runs the full schedule. But if you manually invoke systems via `world.run_system()`, you must call `world.flush()` between them:

```rust
#[test]
fn test_path_request_flush() {
    let mut world = World::new();
    // ... setup ...

    // Run state machine (inserts PathRequests via Commands)
    // world.run_system(citizen_state_machine);
    world.flush(); // CRITICAL: flush Commands

    // Now PathRequests are visible
    // world.run_system(process_path_requests);
}
```

---

## 4. Deterministic Simulation

### 4.1 Why Determinism Matters

A deterministic simulation produces the exact same output given the same input, every time, on every platform. This enables:

- **Reproducible bug reports**: "Load save X, run for 500 ticks, observe bug Y" always reproduces.
- **Replay systems**: Record only player inputs, replay the entire session by re-running the simulation.
- **Automated regression testing**: Compare state hashes between versions.
- **Multiplayer potential**: Lockstep multiplayer requires deterministic simulation (both clients run the same simulation, only inputs are synced).
- **Automated balance testing**: Run the same scenario 100 times and get the same result, rather than needing statistical analysis of random outcomes.

### 4.2 Sources of Non-Determinism in Rust/Bevy

Megacity faces several sources of non-determinism:

**HashMap iteration order:**
`HashMap` in Rust uses randomized hashing (SipHash with random seed). Iterating over a `HashMap` gives different orderings between runs. The `RoadNetwork` uses a `HashMap<RoadNode, Vec<RoadNode>>` for its edge list, which means iterating edges gives different orderings.

Solution: Use `BTreeMap` for deterministic ordering, or `IndexMap` (from the `indexmap` crate) for insertion-order preservation with O(1) lookup. Alternatively, sort the results after iteration.

```rust
// Non-deterministic (current):
pub struct RoadNetwork {
    pub edges: HashMap<RoadNode, Vec<RoadNode>>,
}

// Deterministic option 1: BTreeMap (ordered by key)
pub struct RoadNetwork {
    pub edges: BTreeMap<RoadNode, Vec<RoadNode>>,
}

// Deterministic option 2: sort after iteration
fn deterministic_node_list(network: &RoadNetwork) -> Vec<RoadNode> {
    let mut nodes: Vec<_> = network.edges.keys().copied().collect();
    nodes.sort_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));
    nodes
}
```

The `CsrGraph::from_road_network` already sorts nodes deterministically: `nodes.sort_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)))`. This is good -- the CSR pathfinding graph is deterministic even if the underlying HashMap is not.

**Thread scheduling and parallel iteration:**
Megacity uses `par_iter_mut()` for citizen processing in `update_happiness` and `move_citizens`. Parallel iteration with mutable state can be non-deterministic if the order of writes matters. In this case, each citizen is independently processed (embarrassingly parallel), so the results should be deterministic per-citizen. But the *order* in which entities are stored after parallel processing may vary.

Solution: For deterministic output ordering, sort query results by entity ID or a stable key after parallel processing. Or use a fixed thread count and partition scheme.

**Random number generation:**
Megacity uses `rand::thread_rng()` in several places, most notably `building_spawner`. `thread_rng()` is seeded from OS entropy and is non-deterministic.

Solution: Use a deterministic RNG seeded from game state:

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Resource)]
pub struct SimRng(pub ChaCha8Rng);

impl SimRng {
    pub fn new(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

// Usage in systems:
pub fn building_spawner(
    mut rng: ResMut<SimRng>,
    // ... other params ...
) {
    if rng.0.gen::<f32>() > spawn_chance {
        continue;
    }
}
```

**Floating point across platforms:**
IEEE 754 guarantees identical results for basic operations (+, -, *, /) but NOT for transcendental functions (sin, cos, sqrt, powf). The `smoothed_waypoint_target` function uses `sqrt()`, and the BPR formula uses `powf()`. Different CPUs may produce slightly different results for these operations.

Solution for strict determinism: Use fixed-point arithmetic or software implementations of transcendental functions. For practical purposes, the differences are usually in the least significant bits and do not affect gameplay. But for lockstep multiplayer, these bits matter.

**System ordering:**
Bevy systems that are not explicitly ordered with `.chain()`, `.before()`, or `.after()` can run in any order. If two systems read and write the same resource without ordering constraints, the result is non-deterministic.

Megacity's `SimulationPlugin` is careful about ordering -- most system groups use `.chain()` to enforce sequential execution. But the systems in the last `.add_systems()` block (weather, crime, health, etc.) all run "after imports_exports" but have no ordering relative to each other.

**Fixed timestep:**
Bevy's `FixedUpdate` runs at a fixed rate (default: 64Hz). Megacity should set this explicitly to ensure the same number of ticks per game-second regardless of frame rate:

```rust
app.insert_resource(Time::<Fixed>::from_hz(10.0)); // 10 ticks per second
```

### 4.3 Achieving Determinism: Implementation Plan

**Step 1: Replace rand::thread_rng with seeded RNG**

```rust
// In SimulationPlugin::build:
app.insert_resource(SimRng::new(42)); // or from save file

// Every system that uses randomness takes &mut SimRng instead of thread_rng
```

**Step 2: Use deterministic collections where iteration order matters**

```rust
// Audit all HashMap usage in simulation crate:
// - RoadNetwork::edges -> BTreeMap or sort after iteration
// - Any temporary HashMaps in systems -> sort results
```

**Step 3: Explicit system ordering for all system groups**

```rust
// Audit the parallel system group in lib.rs:
// weather, crime, health, etc. should have explicit .chain() or be documented
// as order-independent
```

**Step 4: Fixed-seed terrain generation**

```rust
// Already done in init_world: generate_terrain uses seed 42
// Make this configurable from save file / new game settings
```

### 4.4 Replay Testing

Once determinism is achieved, implement replay testing:

```rust
/// Record player inputs each tick.
#[derive(Resource, Default)]
struct InputRecording {
    frames: Vec<FrameInput>,
}

#[derive(Clone, Debug)]
struct FrameInput {
    tick: u64,
    actions: Vec<PlayerAction>,
}

#[derive(Clone, Debug)]
enum PlayerAction {
    PlaceRoad { x: usize, y: usize, road_type: RoadType },
    SetZone { x: usize, y: usize, zone: ZoneType },
    PlaceService { x: usize, y: usize, service_type: ServiceType },
    SetTaxRate(f32),
    TogglePolicy(Policy),
    // ...
}

/// Replay test: run the same inputs, compare state at frame N.
#[test]
fn test_replay_determinism() {
    let inputs = vec![
        FrameInput { tick: 0, actions: vec![
            PlayerAction::PlaceRoad { x: 10, y: 10, road_type: RoadType::Local },
        ]},
        FrameInput { tick: 10, actions: vec![
            PlayerAction::SetZone { x: 11, y: 10, zone: ZoneType::ResidentialLow },
        ]},
    ];

    // Run twice with same seed
    let state1 = run_with_inputs(&inputs, 42);
    let state2 = run_with_inputs(&inputs, 42);

    // States must be identical
    assert_eq!(state1.population, state2.population);
    assert_eq!(state1.treasury_bits, state2.treasury_bits);
    assert_eq!(state1.state_hash, state2.state_hash);
}
```

### 4.5 Detecting Non-Determinism

Run the same scenario twice and hash the full simulation state at each tick. If hashes diverge, there is a non-determinism bug:

```rust
fn compute_state_hash(world: &World) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();

    // Hash budget state
    let budget = world.resource::<CityBudget>();
    budget.treasury.to_bits().hash(&mut hasher);
    budget.tax_rate.to_bits().hash(&mut hasher);

    // Hash all citizen positions and states
    // (must iterate in deterministic order)
    let mut citizens: Vec<_> = world
        .query::<(&CitizenDetails, &CitizenStateComp, &Position)>()
        .iter(world)
        .map(|(d, s, p)| (
            d.age,
            d.happiness.to_bits(),
            s.0 as u8,
            p.x.to_bits(),
            p.y.to_bits(),
        ))
        .collect();
    citizens.sort(); // deterministic order
    citizens.hash(&mut hasher);

    // Hash traffic grid
    let traffic = world.resource::<TrafficGrid>();
    traffic.density.hash(&mut hasher);

    hasher.finish()
}

#[test]
fn test_determinism_across_runs() {
    let hashes_run1 = run_and_collect_hashes(42, 100);
    let hashes_run2 = run_and_collect_hashes(42, 100);

    for (tick, (h1, h2)) in hashes_run1.iter().zip(hashes_run2.iter()).enumerate() {
        assert_eq!(h1, h2,
            "Non-determinism detected at tick {}: hash1={:#x}, hash2={:#x}",
            tick, h1, h2);
    }
}
```

---

## 5. Benchmark and Performance Testing

Megacity was designed for 1M+ citizens with a 17x speedup optimization pass. Performance is a feature, and regressions are bugs. This section covers systematic performance testing.

### 5.1 criterion.rs for Microbenchmarks

Add criterion to the workspace:

```toml
# crates/simulation/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "simulation_benchmarks"
harness = false
```

**Pathfinding benchmark:**

```rust
// crates/simulation/benches/simulation_benchmarks.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use simulation::config::{GRID_WIDTH, GRID_HEIGHT};
use simulation::grid::WorldGrid;
use simulation::road_graph_csr::{CsrGraph, csr_find_path};
use simulation::roads::{RoadNetwork, RoadNode};

fn bench_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");

    // Build a grid road network for benchmarking
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();

    // Create a grid of roads every 8 cells
    for y in (0..GRID_HEIGHT).step_by(8) {
        for x in 0..GRID_WIDTH {
            network.place_road(&mut grid, x, y);
        }
    }
    for x in (0..GRID_WIDTH).step_by(8) {
        for y in 0..GRID_HEIGHT {
            network.place_road(&mut grid, x, y);
        }
    }

    let csr = CsrGraph::from_road_network(&network);

    // Benchmark paths of different lengths
    for &(label, start, goal) in &[
        ("short_10", RoadNode(0, 0), RoadNode(8, 8)),
        ("medium_50", RoadNode(0, 0), RoadNode(40, 40)),
        ("long_200", RoadNode(0, 0), RoadNode(200, 200)),
        ("cross_map", RoadNode(0, 0), RoadNode(248, 248)),
    ] {
        group.bench_with_input(
            BenchmarkId::new("csr_astar", label),
            &(start, goal),
            |b, &(s, g)| {
                b.iter(|| csr_find_path(&csr, s, g));
            },
        );
    }

    group.finish();
}

fn bench_grid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_ops");
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    group.bench_function("neighbors4_center", |b| {
        b.iter(|| grid.neighbors4(128, 128));
    });

    group.bench_function("neighbors4_corner", |b| {
        b.iter(|| grid.neighbors4(0, 0));
    });

    group.bench_function("world_to_grid", |b| {
        b.iter(|| WorldGrid::world_to_grid(2048.0, 2048.0));
    });

    group.bench_function("grid_to_world", |b| {
        b.iter(|| WorldGrid::grid_to_world(128, 128));
    });

    group.finish();
}

fn bench_traffic_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic");

    group.bench_function("full_clear_256x256", |b| {
        let mut traffic = simulation::traffic::TrafficGrid::default();
        b.iter(|| traffic.clear());
    });

    group.bench_function("congestion_lookup", |b| {
        let mut traffic = simulation::traffic::TrafficGrid::default();
        traffic.set(128, 128, 15);
        b.iter(|| traffic.congestion_level(128, 128));
    });

    group.bench_function("path_cost_with_road", |b| {
        let mut traffic = simulation::traffic::TrafficGrid::default();
        traffic.set(128, 128, 10);
        b.iter(|| traffic.path_cost_with_road(
            128, 128, simulation::grid::RoadType::Avenue,
        ));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pathfinding,
    bench_grid_operations,
    bench_traffic_grid,
);
criterion_main!(benches);
```

Run with: `cargo bench -p simulation`

### 5.2 Macro Benchmarks: Full Simulation Tick

Benchmark the full simulation tick at different population scales:

```rust
fn bench_full_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_tick");
    group.sample_size(10); // fewer samples for slow benchmarks
    group.measurement_time(std::time::Duration::from_secs(30));

    for &citizen_count in &[1_000, 10_000, 50_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("simulation_tick", citizen_count),
            &citizen_count,
            |b, &count| {
                // Create app with N citizens
                let mut app = create_benchmark_app(count);
                b.iter(|| {
                    app.update();
                });
            },
        );
    }

    group.finish();
}

/// Create a benchmark App pre-populated with N citizens.
fn create_benchmark_app(citizen_count: usize) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin);
    app.update(); // init

    // Manually spawn citizens to reach target count
    {
        let world = app.world_mut();
        // ... spawn roads, buildings, citizens ...
    }

    app
}
```

### 5.3 Performance Regression Detection in CI

Set up a CI job that runs benchmarks and compares against a baseline:

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks
on:
  schedule:
    - cron: '0 2 * * *'  # nightly at 2 AM
  workflow_dispatch:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench -p simulation -- --output-format bencher | tee output.txt

      - name: Compare with baseline
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          alert-threshold: '110%'  # alert on >10% regression
          fail-on-alert: true
          github-token: ${{ secrets.GITHUB_TOKEN }}
```

### 5.4 Performance Budgets

Define explicit performance budgets and enforce them in CI:

```rust
#[test]
fn test_citizen_tick_performance_budget() {
    use std::time::Instant;

    let mut app = create_benchmark_app(100_000);

    // Warm up
    for _ in 0..10 {
        app.update();
    }

    // Measure
    let start = Instant::now();
    let ticks = 100;
    for _ in 0..ticks {
        app.update();
    }
    let elapsed = start.elapsed();
    let per_tick = elapsed / ticks;

    // Budget: full simulation tick must complete in <16ms for 100K citizens
    // (to maintain 60fps with headroom for rendering)
    assert!(
        per_tick.as_millis() < 16,
        "Performance budget exceeded: {:.1}ms per tick for 100K citizens (budget: 16ms)",
        per_tick.as_secs_f64() * 1000.0
    );
}
```

| System | Budget (100K citizens) | Measurement |
|--------|----------------------|-------------|
| Full simulation tick | < 16ms | `app.update()` |
| Pathfinding (single A*) | < 1ms | `csr_find_path()` |
| Traffic grid update | < 2ms | `update_traffic_density()` |
| Happiness computation | < 5ms | `update_happiness()` (par_iter_mut) |
| Citizen movement | < 3ms | `move_citizens()` (par_iter_mut) |
| Grid-wide pollution | < 1ms | `update_pollution()` (runs every 100 ticks) |
| Save to disk | < 1s | `SaveData::encode()` + file write |
| Load from disk | < 3s | file read + `SaveData::decode()` + restore |

### 5.5 Memory Benchmarks

Track allocation patterns and detect memory leaks:

```rust
#[test]
fn test_no_memory_leak_over_time() {
    let mut app = create_benchmark_app(10_000);

    // Run 1000 ticks and check that entity count stays bounded
    let initial_entity_count = app.world().entities().len();

    run_ticks(&mut app, 1000);

    let final_entity_count = app.world().entities().len();

    // Entity count should not grow unboundedly
    // Some growth is expected (new citizens, buildings), but not 10x
    assert!(
        final_entity_count < initial_entity_count * 3,
        "Entity count grew from {} to {} over 1000 ticks -- possible leak",
        initial_entity_count, final_entity_count
    );
}

#[test]
fn test_grid_memory_footprint() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let cell_size = std::mem::size_of::<Cell>();
    let total_bytes = cell_size * GRID_WIDTH * GRID_HEIGHT;

    // 256x256 grid with ~40-byte cells = ~2.5 MB
    // This should stay under 5 MB
    assert!(
        total_bytes < 5 * 1024 * 1024,
        "Grid memory footprint too large: {} bytes ({} per cell)",
        total_bytes, cell_size
    );
}
```

### 5.6 Bevy Diagnostics and Tracy Integration

For runtime profiling during development:

```rust
// In the app crate, add diagnostic plugins for dev builds:
#[cfg(debug_assertions)]
app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
#[cfg(debug_assertions)]
app.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin);

// For detailed system timing, use bevy_diagnostic::SystemInformationDiagnosticsPlugin
// or integrate tracy for frame-level profiling:
// cargo install tracy-client
// app.add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default());
```

Add `#[cfg(feature = "trace")]` spans to critical systems:

```rust
pub fn update_happiness(/* ... */) {
    #[cfg(feature = "trace")]
    let _span = info_span!("update_happiness").entered();
    // ... system body ...
}
```

---

## 6. Stress Testing

Stress testing pushes the simulation to its limits and beyond. The goal is to find what breaks first and at what scale.

### 6.1 Maximum Load Scenarios

**Fill the entire grid:**

```rust
#[test]
fn stress_full_grid() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Fill every other row with roads
    for y in (0..GRID_HEIGHT).step_by(2) {
        for x in 0..GRID_WIDTH {
            roads.place_road(&mut grid, x, y);
        }
    }

    // Zone all non-road grass cells
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get_mut(x, y);
            if cell.cell_type == CellType::Grass {
                cell.zone = ZoneType::ResidentialHigh;
                cell.has_power = true;
                cell.has_water = true;
            }
        }
    }

    // Build CSR graph
    let csr = CsrGraph::from_road_network(&roads);

    // Verify the graph is valid
    assert!(csr.node_count() > 0);
    assert!(csr.edge_count() > 0);

    // Pathfinding should still work
    let path = csr_find_path(&csr, RoadNode(0, 0), RoadNode(254, 254));
    assert!(path.is_some(), "Should find path across full grid");
}
```

**Maximum citizens:**

```rust
#[test]
#[ignore] // slow test, run with: cargo test -- --ignored
fn stress_max_citizens() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin);
    app.update();

    // Spawn 500K citizens
    {
        let world = app.world_mut();
        // ... setup roads and buildings ...

        for i in 0..500_000 {
            world.spawn((
                Citizen,
                Position { x: (i % 256) as f32 * 16.0, y: (i / 256) as f32 * 16.0 },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation { grid_x: i % 256, grid_y: (i / 256) % 256, building: Entity::PLACEHOLDER },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age: 30, gender: Gender::Male, education: 2,
                    happiness: 60.0, health: 90.0,
                    salary: 3000.0, savings: 5000.0,
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
            ));
        }
    }

    // Run 10 ticks and ensure no panic
    let start = std::time::Instant::now();
    run_ticks(&mut app, 10);
    let elapsed = start.elapsed();

    println!("10 ticks with 500K citizens: {:.1}ms ({:.1}ms/tick)",
        elapsed.as_secs_f64() * 1000.0,
        elapsed.as_secs_f64() * 100.0);
}
```

### 6.2 What Breaks First?

Based on profiling real city builder games, the typical failure order under load:

1. **Pathfinding**: A* on large graphs is O(V log V) per query. With 100K citizens each needing a path, this is the first bottleneck. Megacity mitigates this with `MAX_PATHS_PER_TICK = 64` and batched pathfinding.

2. **Spatial queries**: Finding nearest shop/school/park uses linear scan of `DestinationCache`. At 10K+ destinations, this becomes slow. Consider upgrading to a spatial index (k-d tree or grid-based spatial hash).

3. **Grid-wide operations**: Pollution diffusion, land value update, and service coverage iterate the full 256x256 grid (65,536 cells). These are O(n) and fast, but run on the main thread.

4. **Parallel citizen processing**: `par_iter_mut` scales well with cores but has thread synchronization overhead. At 100K+ citizens, this overhead can become significant.

5. **Memory**: Each citizen entity has ~10 components. At 500K citizens, entity storage alone is ~200MB+ depending on component sizes.

### 6.3 Sustained Simulation

Run the simulation for a very long time and verify stability:

```rust
#[test]
#[ignore]
fn stress_sustained_10k_ticks() {
    let mut app = create_benchmark_app(10_000);

    let start = std::time::Instant::now();
    for tick in 0..10_000 {
        app.update();

        // Periodic sanity checks
        if tick % 1000 == 0 {
            let world = app.world();

            // No NaN in budget
            let budget = world.resource::<CityBudget>();
            assert!(budget.treasury.is_finite(),
                "Treasury went non-finite at tick {}", tick);

            // Entity count is bounded
            let entity_count = world.entities().len();
            assert!(entity_count < 1_000_000,
                "Entity count exploded to {} at tick {}", entity_count, tick);

            // No citizens with invalid state
            // (happiness, health in valid range)
            for details in world.query::<&CitizenDetails>().iter(world) {
                assert!(details.happiness >= 0.0 && details.happiness <= 100.0,
                    "Invalid happiness {} at tick {}", details.happiness, tick);
                assert!(details.health >= 0.0 && details.health <= 100.0,
                    "Invalid health {} at tick {}", details.health, tick);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("10K ticks completed in {:.1}s", elapsed.as_secs_f64());
}
```

### 6.4 Concurrent System Stress

Verify that all parallel systems run without data races:

```rust
#[test]
fn stress_all_systems_active() {
    let mut app = test_app();
    app.update();

    // Activate every subsystem:
    // - Weather events
    // - Active fires
    // - Active disasters
    // - All service types
    // - All zone types
    // - Maximum traffic
    // - Loans active
    // - Policies active
    {
        let world = app.world_mut();

        let mut weather = world.resource_mut::<Weather>();
        weather.current_event = WeatherEvent::Storm;
        weather.disasters_enabled = true;

        let mut fire = world.resource_mut::<FireGrid>();
        fire.set(100, 100, true);

        let mut policies = world.resource_mut::<Policies>();
        for &p in Policy::all() {
            policies.active.push(p);
        }
    }

    // Run 500 ticks -- should not panic, deadlock, or corrupt state
    run_ticks(&mut app, 500);

    // Basic sanity
    let budget = app.world().resource::<CityBudget>();
    assert!(budget.treasury.is_finite());
}
```

---

## 7. Save/Load Round-Trip Testing

The save/load system is one of the most error-prone areas of any simulation game. Megacity uses bitcode encoding with serde for compact binary saves. Every piece of simulation state must survive a round trip.

### 7.1 Full State Round-Trip

```rust
#[test]
fn test_full_save_load_roundtrip() {
    let mut app = test_app();
    app.update();

    // Run simulation to build up some state
    run_ticks(&mut app, 200);

    // Capture state before save
    let world = app.world();
    let budget_before = world.resource::<CityBudget>().clone();
    let citizen_count_before = world.query::<&Citizen>().iter(world).count();
    let building_count_before = world.query::<&Building>().iter(world).count();

    // Save
    let save_data = capture_save_data(world);
    let bytes = save_data.encode();

    // Load into a fresh app
    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins);
    app2.add_plugins(SimulationPlugin);
    let loaded = SaveData::decode(&bytes).expect("decode should succeed");
    restore_from_save_data(app2.world_mut(), &loaded);
    app2.update();

    // Compare
    let world2 = app2.world();
    let budget_after = world2.resource::<CityBudget>();
    assert!(
        (budget_before.treasury - budget_after.treasury).abs() < 0.01,
        "Treasury mismatch: before={}, after={}",
        budget_before.treasury, budget_after.treasury
    );

    let citizen_count_after = world2.query::<&Citizen>().iter(world2).count();
    assert_eq!(citizen_count_before, citizen_count_after,
        "Citizen count mismatch");

    let building_count_after = world2.query::<&Building>().iter(world2).count();
    assert_eq!(building_count_before, building_count_after,
        "Building count mismatch");
}
```

### 7.2 Component-by-Component Verification

Not all state is saved. Verify exactly what IS and IS NOT persisted:

```rust
/// State that MUST survive save/load:
/// - WorldGrid (all cells, zones, road types, power, water)
/// - RoadNetwork (all edges)
/// - RoadSegmentStore (all segments and nodes)
/// - CityBudget (treasury, tax_rate, last_collection_day)
/// - ZoneDemand (residential, commercial, industrial, office)
/// - All Buildings (zone_type, level, grid_x, grid_y, capacity, occupants)
/// - All Citizens (age, happiness, education, state, home, work)
/// - All UtilitySources (type, position, range)
/// - All ServiceBuildings (type, position, radius)
/// - GameClock (day, hour, speed)
/// - Policies (active list)
/// - Weather (season, temperature, event, disasters_enabled)
/// - UnlockState (points, unlocked nodes)
/// - ExtendedBudget (zone taxes, service budgets)
/// - LoanBook (active loans, credit rating)

/// State that is NOT saved (reconstructed from saved state):
/// - TrafficGrid (recomputed each tick from citizen positions)
/// - ServiceCoverageGrid (recomputed from service buildings)
/// - CsrGraph (rebuilt from RoadNetwork)
/// - PathCache (citizens will re-pathfind on load)
/// - Velocity (set to zero, recalculated during movement)
/// - LandValueGrid (recomputed from grid state)
/// - PollutionGrid (recomputed from buildings and zones)
/// - CrimeGrid, HealthGrid, GarbageGrid (recomputed)
/// - CityStats (recomputed from entities)
/// - LifeSimTimer (NOT saved -- known issue, all events fire on load)
```

### 7.3 Fuzzing Save Files

Corrupt save files should produce graceful errors, never panics or undefined behavior:

```rust
#[test]
fn test_truncated_save_file() {
    let save = create_minimal_save();
    let bytes = save.encode();

    // Try decoding truncated data
    for truncate_at in [0, 1, 10, bytes.len() / 2, bytes.len() - 1] {
        let result = SaveData::decode(&bytes[..truncate_at]);
        assert!(result.is_err(),
            "Truncated save at {} bytes should fail to decode", truncate_at);
    }
}

#[test]
fn test_corrupted_save_bytes() {
    let save = create_minimal_save();
    let mut bytes = save.encode();

    // Flip random bits
    for i in (0..bytes.len()).step_by(bytes.len() / 20 + 1) {
        bytes[i] ^= 0xFF;
    }

    let result = SaveData::decode(&bytes);
    // Should either fail to decode or produce a valid but different state
    // Must NOT panic
    match result {
        Ok(_) => {} // corrupted but still valid bitcode -- acceptable
        Err(_) => {} // expected: decode error
    }
}

#[test]
fn test_empty_save_file() {
    let result = SaveData::decode(&[]);
    assert!(result.is_err(), "Empty data should fail to decode");
}

#[test]
fn test_random_bytes_save_file() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..1000).map(|_| rng.gen()).collect();

    let result = SaveData::decode(&random_bytes);
    assert!(result.is_err(), "Random bytes should fail to decode");
}
```

### 7.4 Schema Migration Testing

Megacity already supports backward compatibility via `#[serde(default)]` on V2 fields. Maintain a set of test save files from each schema version:

```
tests/fixtures/saves/
  v1_basic.bin          # V1 save: no policies, weather, unlocks
  v1_large_city.bin     # V1 save: 10K citizens, full grid
  v2_all_features.bin   # V2 save: policies, weather, unlocks, loans
  v2_edge_cases.bin     # V2 save: max loans, extreme weather, all policies
```

```rust
#[test]
fn test_load_v1_save() {
    let bytes = include_bytes!("../tests/fixtures/saves/v1_basic.bin");
    let save = SaveData::decode(bytes).expect("V1 save should decode");

    // V2 fields should be None
    assert!(save.policies.is_none());
    assert!(save.weather.is_none());
    assert!(save.unlock_state.is_none());
    assert!(save.extended_budget.is_none());
    assert!(save.loan_book.is_none());

    // V1 fields should be valid
    assert!(save.grid.width > 0);
    assert!(save.grid.height > 0);
    assert_eq!(save.grid.cells.len(), save.grid.width * save.grid.height);
}

#[test]
fn test_load_v2_save() {
    let bytes = include_bytes!("../tests/fixtures/saves/v2_all_features.bin");
    let save = SaveData::decode(bytes).expect("V2 save should decode");

    // V2 fields should be present
    assert!(save.policies.is_some());
    assert!(save.weather.is_some());
    assert!(save.unlock_state.is_some());
    assert!(save.extended_budget.is_some());
}

#[test]
fn test_v1_to_v2_upgrade_defaults() {
    let bytes = include_bytes!("../tests/fixtures/saves/v1_basic.bin");
    let save = SaveData::decode(bytes).expect("V1 save should decode");

    // When V2 fields are None, restore functions should provide defaults
    let policies = save.policies
        .as_ref()
        .map(|p| restore_policies(p))
        .unwrap_or_default();
    assert!(policies.active.is_empty(), "Default policies should be empty");

    let weather = save.weather
        .as_ref()
        .map(|w| restore_weather(w))
        .unwrap_or_default();
    assert_eq!(weather.season, Season::Spring, "Default season should be Spring");
}
```

### 7.5 Save/Load Performance

```rust
#[test]
fn test_save_performance() {
    let mut app = create_benchmark_app(50_000);
    run_ticks(&mut app, 100);

    let world = app.world();
    let save_data = capture_save_data(world);

    // Measure encode time
    let start = std::time::Instant::now();
    let bytes = save_data.encode();
    let encode_time = start.elapsed();

    // Budget: save should complete in < 1 second
    assert!(
        encode_time.as_secs_f64() < 1.0,
        "Save encoding took {:.2}s (budget: 1.0s)",
        encode_time.as_secs_f64()
    );

    // Measure decode time
    let start = std::time::Instant::now();
    let _loaded = SaveData::decode(&bytes).expect("decode");
    let decode_time = start.elapsed();

    // Budget: load should complete in < 3 seconds
    assert!(
        decode_time.as_secs_f64() < 3.0,
        "Save decoding took {:.2}s (budget: 3.0s)",
        decode_time.as_secs_f64()
    );

    // Report size
    println!(
        "Save file: {} bytes ({:.1} KB) for 50K citizens, encode={:.1}ms, decode={:.1}ms",
        bytes.len(),
        bytes.len() as f64 / 1024.0,
        encode_time.as_secs_f64() * 1000.0,
        decode_time.as_secs_f64() * 1000.0,
    );
}
```

### 7.6 Serialization Round-Trip for Every Type

The existing tests in `serialization.rs` already cover most types. Here is the pattern to ensure every enum variant round-trips:

```rust
#[test]
fn test_all_citizen_states_roundtrip() {
    let states = [
        CitizenState::AtHome,
        CitizenState::CommutingToWork,
        CitizenState::Working,
        CitizenState::CommutingHome,
        CitizenState::CommutingToShop,
        CitizenState::Shopping,
        CitizenState::CommutingToLeisure,
        CitizenState::AtLeisure,
        CitizenState::CommutingToSchool,
        CitizenState::AtSchool,
    ];

    for (i, &state) in states.iter().enumerate() {
        let encoded = i as u8;
        let decoded = match encoded {
            0 => CitizenState::AtHome,
            1 => CitizenState::CommutingToWork,
            2 => CitizenState::Working,
            3 => CitizenState::CommutingHome,
            4 => CitizenState::CommutingToShop,
            5 => CitizenState::Shopping,
            6 => CitizenState::CommutingToLeisure,
            7 => CitizenState::AtLeisure,
            8 => CitizenState::CommutingToSchool,
            9 => CitizenState::AtSchool,
            _ => panic!("unknown state"),
        };
        assert_eq!(state, decoded, "State {:?} failed roundtrip", state);
    }
}
```

---

## 8. Visual and Rendering Testing

Rendering bugs are often the most visible to players but the hardest to test automatically. This section covers approaches from screenshot comparison to headless rendering in CI.

### 8.1 Screenshot Comparison Testing

The gold standard for visual regression testing: render a frame, compare it pixel-by-pixel against a "golden" reference image.

```rust
/// Capture a screenshot from the running application.
/// Requires wgpu with a software backend for headless rendering.
fn capture_screenshot(app: &mut App) -> image::RgbaImage {
    // In Bevy 0.15, you can use RenderApp to capture frames
    // This requires the bevy_render plugin with a software backend
    todo!("Implementation depends on Bevy's screenshot API")
}

/// Compare two images with tolerance for anti-aliasing differences.
fn images_match(actual: &image::RgbaImage, expected: &image::RgbaImage, tolerance: f32) -> bool {
    if actual.dimensions() != expected.dimensions() {
        return false;
    }

    let total_pixels = (actual.width() * actual.height()) as f64;
    let mut diff_count = 0u64;

    for (a, e) in actual.pixels().zip(expected.pixels()) {
        let dr = (a[0] as i32 - e[0] as i32).abs();
        let dg = (a[1] as i32 - e[1] as i32).abs();
        let db = (a[2] as i32 - e[2] as i32).abs();
        if dr > 5 || dg > 5 || db > 5 {
            diff_count += 1;
        }
    }

    let diff_ratio = diff_count as f64 / total_pixels;
    diff_ratio < tolerance as f64
}
```

**Perceptual hash comparison:** For more robust comparison that tolerates minor rendering differences:

```rust
/// Compute a perceptual hash of an image.
/// Two images with similar content will have similar hashes,
/// even with minor rendering differences (anti-aliasing, etc.).
fn perceptual_hash(img: &image::RgbaImage) -> u64 {
    // Downscale to 8x8
    let small = image::imageops::resize(img, 8, 8, image::imageops::FilterType::Lanczos3);

    // Convert to grayscale
    let gray: Vec<f32> = small.pixels()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();

    // Compute average
    let avg = gray.iter().sum::<f32>() / gray.len() as f32;

    // Generate hash: each bit is 1 if pixel > average, 0 otherwise
    let mut hash = 0u64;
    for (i, &val) in gray.iter().enumerate() {
        if val > avg {
            hash |= 1 << i;
        }
    }
    hash
}

/// Hamming distance between two perceptual hashes.
/// Lower = more similar. 0 = identical. < 10 = very similar.
fn hash_distance(h1: u64, h2: u64) -> u32 {
    (h1 ^ h2).count_ones()
}
```

### 8.2 LOD Transition Testing

Megacity uses three LOD tiers: `Full`, `Simplified`, and `Abstract`. Verify that the correct meshes are loaded at each zoom level:

```rust
#[test]
fn test_lod_tier_assignment() {
    // Citizens inside the viewport should be Full or Simplified
    // Citizens outside the viewport should be Abstract

    let viewport = ViewportBounds {
        min_x: 1000.0,
        max_x: 2000.0,
        min_y: 1000.0,
        max_y: 2000.0,
    };

    // Citizen inside viewport
    let pos_inside = Position { x: 1500.0, y: 1500.0 };
    let tier = compute_lod_tier(&pos_inside, &viewport, 1.0);
    assert!(matches!(tier, LodTier::Full | LodTier::Simplified));

    // Citizen far outside viewport
    let pos_outside = Position { x: 100.0, y: 100.0 };
    let tier = compute_lod_tier(&pos_outside, &viewport, 1.0);
    assert_eq!(tier, LodTier::Abstract);
}
```

### 8.3 Overlay Rendering Verification

Each overlay (traffic, pollution, land value, services, etc.) maps data values to colors. Test the color mapping functions:

```rust
#[test]
fn test_traffic_overlay_colors() {
    // Zero congestion: should be green-ish
    let color_free = traffic_overlay_color(0.0);
    assert!(color_free.g() > color_free.r(),
        "Free-flowing traffic should be more green than red");

    // Full congestion: should be red-ish
    let color_congested = traffic_overlay_color(1.0);
    assert!(color_congested.r() > color_congested.g(),
        "Congested traffic should be more red than green");

    // Mid congestion: should be yellow-ish (mix)
    let color_mid = traffic_overlay_color(0.5);
    // Yellow = high R + high G
    assert!(color_mid.r() > 0.3 && color_mid.g() > 0.3,
        "Mid congestion should be yellowish");
}

#[test]
fn test_overlay_color_continuity() {
    // Colors should change smoothly, no sudden jumps
    let mut prev_color = traffic_overlay_color(0.0);
    for i in 1..=100 {
        let t = i as f32 / 100.0;
        let color = traffic_overlay_color(t);
        let dr = (color.r() - prev_color.r()).abs();
        let dg = (color.g() - prev_color.g()).abs();
        let db = (color.b() - prev_color.b()).abs();
        assert!(dr < 0.1 && dg < 0.1 && db < 0.1,
            "Color jump at t={}: prev={:?}, curr={:?}", t, prev_color, color);
        prev_color = color;
    }
}
```

### 8.4 Headless Rendering for CI

Bevy can render headlessly using wgpu's software backend. This enables visual tests in CI without a GPU:

```toml
# Cargo.toml feature for CI rendering
[features]
ci_render = ["bevy/bevy_ci_testing"]
```

```rust
// Test app setup for headless rendering
fn headless_render_app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: None, // no window
        ..default()
    }));
    // Use software renderer
    app.add_plugins(RenderPlugin {
        render_creation: WgpuSettings {
            backends: Some(Backends::VULKAN | Backends::GL),
            ..default()
        }.into(),
        ..default()
    });
    app
}
```

### 8.5 Building Mesh Testing

Verify that building meshes are correctly generated for each zone type and level:

```rust
#[test]
fn test_building_mesh_generation() {
    for zone in [
        ZoneType::ResidentialLow, ZoneType::ResidentialHigh,
        ZoneType::CommercialLow, ZoneType::CommercialHigh,
        ZoneType::Industrial, ZoneType::Office,
    ] {
        for level in 1..=zone.max_level() {
            let mesh = generate_building_mesh(zone, level);
            assert!(mesh.count_vertices() > 0,
                "No vertices for {:?} level {}", zone, level);
            assert!(mesh.indices().is_some(),
                "No indices for {:?} level {}", zone, level);

            // Higher level buildings should be taller
            if level > 1 {
                let prev_mesh = generate_building_mesh(zone, level - 1);
                let height = mesh_max_y(&mesh);
                let prev_height = mesh_max_y(&prev_mesh);
                assert!(height > prev_height,
                    "{:?} L{} should be taller than L{}: {} vs {}",
                    zone, level, level - 1, height, prev_height);
            }
        }
    }
}
```

---

## 9. Gameplay and Balance Testing

Balance testing answers the question: "Is this game actually fun and fair?" It uses automated playthroughs and statistical analysis to detect degenerate strategies, broken economies, and unfair difficulty curves.

### 9.1 Automated Playthroughs

Build a "bot" that plays the game following a scripted strategy and measures outcomes:

```rust
/// A simple bot strategy for automated testing.
struct CityBot {
    strategy: BotStrategy,
    actions_taken: Vec<BotAction>,
}

enum BotStrategy {
    /// Build roads, zone residential, add services as needed
    Balanced,
    /// Only build residential, ignore services
    ResidentialOnly,
    /// Industrial focus with minimal residential
    IndustrialHeavy,
    /// Maximum tax rate, minimal spending
    Austerity,
    /// Zero tax, maximum spending (deficit)
    FreeSpending,
}

impl CityBot {
    fn tick(&mut self, world: &mut World) -> Vec<BotAction> {
        let budget = world.resource::<CityBudget>();
        let stats = world.resource::<CityStats>();

        let mut actions = Vec::new();

        match self.strategy {
            BotStrategy::Balanced => {
                // If we have money and no roads, build roads
                if budget.treasury > 1000.0 && stats.road_cells < 100 {
                    actions.push(BotAction::BuildRoad { /* ... */ });
                }

                // If we have roads but low population, zone residential
                if stats.population < 100 && stats.road_cells > 20 {
                    actions.push(BotAction::Zone {
                        zone: ZoneType::ResidentialLow,
                        // ...
                    });
                }

                // If population > 500, add services
                if stats.population > 500 && !self.has_hospital(world) {
                    actions.push(BotAction::PlaceService {
                        service: ServiceType::Hospital,
                        // ...
                    });
                }
            }
            // ... other strategies ...
            _ => {}
        }

        self.actions_taken.extend(actions.clone());
        actions
    }
}

#[test]
fn test_balanced_bot_grows_city() {
    let mut app = test_app();
    let mut bot = CityBot::new(BotStrategy::Balanced);

    for _ in 0..10_000 {
        let actions = bot.tick(app.world_mut());
        apply_bot_actions(app.world_mut(), &actions);
        app.update();
    }

    let stats = app.world().resource::<CityStats>();
    assert!(stats.population > 100,
        "Balanced bot should grow city to >100 pop, got {}", stats.population);

    let budget = app.world().resource::<CityBudget>();
    assert!(budget.treasury > 0.0,
        "Balanced bot should maintain positive treasury");
}
```

### 9.2 Monte Carlo Balance Testing

Run 1000 random cities and check the distribution of outcomes:

```rust
#[test]
#[ignore] // slow
fn test_monte_carlo_economy_stability() {
    let trials = 100;
    let ticks_per_trial = 5_000;
    let mut results: Vec<TrialResult> = Vec::new();

    for seed in 0..trials {
        let mut app = test_app_with_seed(seed);
        let mut bot = CityBot::new(BotStrategy::Balanced);

        for _ in 0..ticks_per_trial {
            let actions = bot.tick(app.world_mut());
            apply_bot_actions(app.world_mut(), &actions);
            app.update();
        }

        let budget = app.world().resource::<CityBudget>();
        let stats = app.world().resource::<CityStats>();

        results.push(TrialResult {
            seed,
            final_population: stats.population,
            final_treasury: budget.treasury,
            final_happiness: stats.avg_happiness,
            went_bankrupt: budget.treasury < -10_000.0,
        });
    }

    // Analyze results
    let bankrupt_count = results.iter().filter(|r| r.went_bankrupt).count();
    let avg_population = results.iter().map(|r| r.final_population).sum::<u32>() as f64
        / trials as f64;
    let avg_happiness = results.iter().map(|r| r.final_happiness).sum::<f64>()
        / trials as f64;

    // Balance assertions:
    // 1. Balanced bot should not go bankrupt more than 10% of the time
    assert!(
        bankrupt_count <= trials / 10,
        "Balanced bot went bankrupt in {}/{} trials (>10%)",
        bankrupt_count, trials
    );

    // 2. Average population should be positive
    assert!(
        avg_population > 50.0,
        "Average final population is too low: {}", avg_population
    );

    // 3. Average happiness should be in a reasonable range
    assert!(
        avg_happiness > 40.0 && avg_happiness < 90.0,
        "Average happiness out of balanced range: {}", avg_happiness
    );

    println!("Monte Carlo results ({} trials):", trials);
    println!("  Bankrupt: {}/{}", bankrupt_count, trials);
    println!("  Avg population: {:.0}", avg_population);
    println!("  Avg happiness: {:.1}", avg_happiness);
}
```

### 9.3 Economy Convergence Testing

Does the economy converge to a stable equilibrium, or does it diverge (infinite growth or spiral to bankruptcy)?

```rust
#[test]
fn test_economy_convergence() {
    let mut app = create_benchmark_app(5_000);

    let mut treasury_history: Vec<f64> = Vec::new();

    for tick in 0..5_000 {
        app.update();
        if tick % 100 == 0 {
            let budget = app.world().resource::<CityBudget>();
            treasury_history.push(budget.treasury);
        }
    }

    // Check that treasury is not diverging
    let last_10: &[f64] = &treasury_history[treasury_history.len().saturating_sub(10)..];
    let variance = statistical_variance(last_10);
    let mean = last_10.iter().sum::<f64>() / last_10.len() as f64;

    // Coefficient of variation should be reasonable
    // (high variance relative to mean = unstable economy)
    if mean.abs() > 1.0 {
        let cv = variance.sqrt() / mean.abs();
        assert!(cv < 0.5,
            "Economy is diverging: CV={:.3} (mean={:.0}, stddev={:.0})",
            cv, mean, variance.sqrt());
    }

    // Treasury should not be NaN or infinite
    for &t in &treasury_history {
        assert!(t.is_finite(), "Treasury went non-finite: {}", t);
    }
}

fn statistical_variance(data: &[f64]) -> f64 {
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
}
```

### 9.4 Difficulty Curve Measurement

Track player income and population at each game-year milestone:

```rust
#[test]
fn test_difficulty_curve() {
    let mut app = test_app();
    let mut bot = CityBot::new(BotStrategy::Balanced);
    let mut milestones: Vec<Milestone> = Vec::new();

    for tick in 0..50_000 {
        let actions = bot.tick(app.world_mut());
        apply_bot_actions(app.world_mut(), &actions);
        app.update();

        let clock = app.world().resource::<GameClock>();
        let year = clock.day / 365;

        // Record milestone every game year
        if clock.day % 365 == 0 && year > 0 {
            let budget = app.world().resource::<CityBudget>();
            let stats = app.world().resource::<CityStats>();
            milestones.push(Milestone {
                year,
                population: stats.population,
                treasury: budget.treasury,
                income: budget.monthly_income,
                happiness: stats.avg_happiness,
            });
        }
    }

    // Verify difficulty curve:
    // 1. Population should generally increase over time
    for window in milestones.windows(5) {
        let start_pop = window.first().unwrap().population;
        let end_pop = window.last().unwrap().population;
        // Allow some decline but not catastrophic
        assert!(
            end_pop >= start_pop / 2,
            "Population crashed: {} -> {} over 5 years",
            start_pop, end_pop
        );
    }

    // 2. Income should grow with population (roughly)
    if milestones.len() > 2 {
        let early_income = milestones[1].income;
        let late_income = milestones.last().unwrap().income;
        if milestones.last().unwrap().population > milestones[1].population * 2 {
            assert!(late_income > early_income,
                "Income should grow with population: early={}, late={}",
                early_income, late_income);
        }
    }
}
```

### 9.5 Exploit Detection

Automated search for degenerate strategies:

```rust
#[test]
fn test_no_infinite_money_exploit() {
    // Strategy: set tax rate to maximum, provide no services
    let mut app = test_app();
    app.update();

    {
        let world = app.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.tax_rate = 1.0; // 100% tax
    }

    run_ticks(&mut app, 5000);

    let budget = app.world().resource::<CityBudget>();
    let stats = app.world().resource::<CityStats>();

    // With 100% tax, citizens should be unhappy and emigrate
    // Treasury should not grow indefinitely
    if stats.population > 0 {
        assert!(
            stats.avg_happiness < 40.0,
            "100% tax should make citizens unhappy, got happiness={}",
            stats.avg_happiness
        );
    }
}

#[test]
fn test_no_zero_cost_services() {
    // Verify every service type has a non-zero maintenance cost
    for i in 0..=49u8 {
        if let Some(st) = u8_to_service_type(i) {
            let cost = ServiceBuilding::monthly_maintenance(st);
            assert!(cost > 0.0,
                "Service {:?} has zero maintenance cost -- potential exploit",
                st);
        }
    }
}

#[test]
fn test_loans_cannot_be_exploited() {
    // Taking a loan and immediately taking another should not
    // create infinite money
    let mut loan_book = LoanBook::default();
    let mut treasury = 0.0;

    for _ in 0..100 {
        if loan_book.can_take_loan() {
            loan_book.take_loan(LoanTier::Small, &mut treasury);
        }
    }

    // Should hit max loan limit
    assert!(loan_book.active_loans.len() <= loan_book.max_loans,
        "Exceeded max loan limit: {} loans", loan_book.active_loans.len());

    // Total debt should not exceed a reasonable amount
    let total_debt: f64 = loan_book.active_loans.iter()
        .map(|l| l.remaining_balance)
        .sum();
    assert!(total_debt < 1_000_000.0,
        "Loan exploit: accumulated {} debt", total_debt);
}
```

---

## 10. Regression Testing

Every bug fix should come with a regression test that prevents the bug from returning. This is the most practical form of testing: each test encodes hard-won knowledge about a real failure mode.

### 10.1 Naming Convention

Use descriptive names that reference the bug:

```rust
#[test]
fn test_bug_citizens_stuck_at_intersection() {
    // Bug: Citizens with paths through T-intersections would stop
    // moving because the next waypoint was unreachable.
    // Fix: Added nearest_road_grid fallback in compute_route_csr.

    let mut grid = WorldGrid::new(64, 64);
    let mut network = RoadNetwork::default();

    // Create a T-intersection
    for x in 5..=15 { network.place_road(&mut grid, x, 10); }
    for y in 5..=10 { network.place_road(&mut grid, 10, y); }

    let csr = CsrGraph::from_road_network(&network);
    let path = csr_find_path(&csr, RoadNode(5, 10), RoadNode(10, 5));
    assert!(path.is_some(), "Should find path through T-intersection");
}

#[test]
fn test_bug_nan_treasury_at_zero_population() {
    // Bug: Division by zero when computing average tax per citizen
    // with zero population produced NaN treasury.
    let budget = CityBudget::default();
    let pop = 0.0_f64;
    let tax_per_citizen = 10.0 * budget.tax_rate as f64;
    let income = pop * tax_per_citizen;
    assert!(!income.is_nan());
    assert!(income.is_finite());
    assert_eq!(income, 0.0);
}

#[test]
fn test_bug_building_spawns_on_water() {
    // Bug: Building spawner did not check cell_type before spawning.
    // Buildings would appear on water cells.
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Water {
                assert!(cell.building_id.is_none(),
                    "Water cell ({},{}) has a building", x, y);
            }
        }
    }
}

#[test]
fn test_bug_path_cache_overflow() {
    // Bug: PathCache::advance() would panic when called on an
    // already-completed path.
    let mut path = PathCache::new(vec![RoadNode(0, 0)]);
    path.advance(); // complete the path
    assert!(path.is_complete());

    // This should not panic
    let _target = path.current_target();
    assert!(path.current_target().is_none());
}

#[test]
fn test_bug_negative_road_cost() {
    // Bug: path_cost_with_road returned 0 for very high-speed roads
    // because (30.0 / 100.0) as u32 = 0, then base = 0 + 1 = 1
    // but with negative congestion penalty it could underflow.
    let traffic = TrafficGrid::default();
    let cost = traffic.path_cost_with_road(0, 0, RoadType::Highway);
    assert!(cost >= 1, "Road cost should never be less than 1");
}

#[test]
fn test_bug_service_coverage_not_recalculated() {
    // Bug: ServiceCoverageGrid.dirty was never set to true when
    // service budgets changed, so coverage would be stale.
    let mut coverage = ServiceCoverageGrid::default();
    assert!(coverage.dirty, "Should start dirty");

    coverage.dirty = false;
    // Simulating budget change should re-dirty the grid
    // (in the actual system, this is triggered by ext_budget.is_changed())
    coverage.dirty = true;
    assert!(coverage.dirty);
}
```

### 10.2 Snapshot Testing

For complex state comparisons, serialize expected state and compare:

```rust
#[test]
fn test_snapshot_initial_world_state() {
    let mut app = test_app();
    app.update();

    let world = app.world();
    let stats = world.resource::<CityStats>();

    // Snapshot of expected initial state
    // (update this snapshot when intentional changes are made)
    assert_eq!(stats.population, 10_000, "Initial population changed");

    let budget = world.resource::<CityBudget>();
    assert!((budget.treasury - 100_000.0).abs() < 1.0,
        "Initial treasury changed: {}", budget.treasury);
}
```

### 10.3 Known Bug Database Pattern

Maintain a mapping from bug IDs to test functions:

```rust
/// Known bug registry. Each entry maps a bug ID to its regression test.
/// When fixing a bug, add an entry here and create the corresponding test.
///
/// Format: (bug_id, description, test_function_name)
///
/// BUGS:
/// - #001: Citizens stuck at T-intersections -> test_bug_citizens_stuck_at_intersection
/// - #002: NaN treasury at zero population -> test_bug_nan_treasury_at_zero_population
/// - #003: Buildings spawn on water cells -> test_bug_building_spawns_on_water
/// - #004: PathCache panic on completed path -> test_bug_path_cache_overflow
/// - #005: Negative road cost for highways -> test_bug_negative_road_cost
/// - #006: Stale service coverage after budget change -> test_bug_service_coverage_not_recalculated
```

---

## 11. Test Infrastructure

Good test infrastructure makes writing tests easy and fast. Bad infrastructure makes tests flaky and slow. This section covers the shared utilities, fixtures, and CI pipeline that support all testing levels.

### 11.1 Test Fixtures: Pre-Built World States

Create a library of reusable test worlds:

```rust
// crates/simulation/src/test_utils.rs (only compiled in test mode)

#[cfg(test)]
pub mod fixtures {
    use super::*;
    use crate::config::{GRID_WIDTH, GRID_HEIGHT};
    use crate::grid::{WorldGrid, CellType, ZoneType};
    use crate::roads::RoadNetwork;

    /// An empty grid with no roads, zones, or buildings.
    pub fn empty_grid() -> WorldGrid {
        WorldGrid::new(GRID_WIDTH, GRID_HEIGHT)
    }

    /// A small test grid (32x32) for fast unit tests.
    pub fn small_grid() -> WorldGrid {
        WorldGrid::new(32, 32)
    }

    /// A grid with a simple cross-shaped road network.
    /// Roads at x=15 (vertical) and y=15 (horizontal).
    pub fn cross_road_grid() -> (WorldGrid, RoadNetwork) {
        let mut grid = WorldGrid::new(32, 32);
        let mut roads = RoadNetwork::default();

        for x in 0..32 {
            roads.place_road(&mut grid, x, 15);
        }
        for y in 0..32 {
            roads.place_road(&mut grid, 15, y);
        }

        (grid, roads)
    }

    /// A grid with roads, residential zones, and utilities --
    /// ready for building and citizen spawning.
    pub fn small_city() -> (WorldGrid, RoadNetwork) {
        let (mut grid, roads) = cross_road_grid();

        // Zone residential near horizontal road
        for x in 5..15 {
            for dy in [-1i32, -2, 1, 2] {
                let y = (15 + dy) as usize;
                if grid.in_bounds(x, y) {
                    let cell = grid.get_mut(x, y);
                    if cell.cell_type == CellType::Grass {
                        cell.zone = ZoneType::ResidentialLow;
                        cell.has_power = true;
                        cell.has_water = true;
                    }
                }
            }
        }

        // Zone commercial near vertical road
        for y in 5..15 {
            for dx in [-1i32, -2, 1, 2] {
                let x = (15 + dx) as usize;
                if grid.in_bounds(x, y) {
                    let cell = grid.get_mut(x, y);
                    if cell.cell_type == CellType::Grass {
                        cell.zone = ZoneType::CommercialLow;
                        cell.has_power = true;
                        cell.has_water = true;
                    }
                }
            }
        }

        (grid, roads)
    }

    /// Stress-test grid: roads every 4 cells, all grass zoned residential.
    pub fn stress_grid() -> (WorldGrid, RoadNetwork) {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        // Dense road grid
        for y in (0..GRID_HEIGHT).step_by(4) {
            for x in 0..GRID_WIDTH {
                roads.place_road(&mut grid, x, y);
            }
        }
        for x in (0..GRID_WIDTH).step_by(4) {
            for y in 0..GRID_HEIGHT {
                roads.place_road(&mut grid, x, y);
            }
        }

        // Zone everything
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get_mut(x, y);
                if cell.cell_type == CellType::Grass {
                    cell.zone = ZoneType::ResidentialHigh;
                    cell.has_power = true;
                    cell.has_water = true;
                }
            }
        }

        (grid, roads)
    }
}
```

### 11.2 Test Helper Functions

```rust
#[cfg(test)]
pub mod helpers {
    use bevy::prelude::*;
    use crate::*;

    /// Create a minimal test App with SimulationPlugin.
    pub fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);
        app
    }

    /// Run N update cycles on the App.
    pub fn run_ticks(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    /// Spawn a road segment and update all derived data structures.
    pub fn spawn_road(
        world: &mut World,
        x1: usize, y1: usize,
        x2: usize, y2: usize,
    ) {
        let mut grid = world.resource_mut::<WorldGrid>();
        let mut roads = world.resource_mut::<RoadNetwork>();

        // Horizontal or vertical road
        if y1 == y2 {
            let (min_x, max_x) = (x1.min(x2), x1.max(x2));
            for x in min_x..=max_x {
                roads.place_road(&mut grid, x, y1);
            }
        } else if x1 == x2 {
            let (min_y, max_y) = (y1.min(y2), y1.max(y2));
            for y in min_y..=max_y {
                roads.place_road(&mut grid, x1, y);
            }
        }
    }

    /// Spawn a building at the given grid position.
    pub fn spawn_building(
        world: &mut World,
        zone: ZoneType,
        level: u8,
        x: usize,
        y: usize,
    ) -> Entity {
        let capacity = Building::capacity_for_level(zone, level);
        let entity = world.spawn(Building {
            zone_type: zone,
            level,
            grid_x: x,
            grid_y: y,
            capacity,
            occupants: 0,
        }).id();

        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(x, y).building_id = Some(entity);
        grid.get_mut(x, y).zone = zone;

        entity
    }

    /// Spawn a citizen with home and work locations.
    pub fn spawn_citizen(
        world: &mut World,
        home_x: usize, home_y: usize, home_building: Entity,
        work_x: usize, work_y: usize, work_building: Entity,
        age: u8,
    ) -> Entity {
        let (wx, wy) = WorldGrid::grid_to_world(home_x, home_y);
        world.spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation { grid_x: home_x, grid_y: home_y, building: home_building },
            WorkLocation { grid_x: work_x, grid_y: work_y, building: work_building },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
                health: 90.0,
                salary: 3000.0,
                savings: 5000.0,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        )).id()
    }

    /// Count entities matching a query.
    pub fn count<Q: bevy::ecs::query::QueryData>(world: &World) -> usize {
        world.query::<Q>().iter(world).count()
    }

    /// Assert that a resource value satisfies a predicate.
    pub fn assert_resource<R: Resource, F: FnOnce(&R) -> bool>(
        world: &World,
        predicate: F,
        message: &str,
    ) {
        let resource = world.resource::<R>();
        assert!(predicate(resource), "{}", message);
    }
}
```

### 11.3 CI Pipeline Configuration

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  # ==== FAST: runs on every PR (~2 min) ====
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run unit tests
        run: cargo test --workspace --lib --bins

      - name: Run doc tests
        run: cargo test --workspace --doc

  # ==== MEDIUM: runs on every PR (~5 min) ====
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run integration tests
        run: cargo test --workspace --test '*'

  # ==== LINT: runs on every PR (~1 min) ====
  lint:
    name: Clippy & Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

  # ==== SLOW: nightly only (~30 min) ====
  benchmarks:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run benchmarks
        run: cargo bench -p simulation -- --output-format bencher | tee output.txt

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          alert-threshold: '110%'
          fail-on-alert: true

  # ==== SLOW: weekly (~60 min) ====
  stress-tests:
    name: Stress Tests
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run stress tests
        run: cargo test --workspace -- --ignored
        timeout-minutes: 60

  # ==== SLOW: weekly ====
  mutation-testing:
    name: Mutation Testing
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation testing on simulation crate
        run: cargo mutants -p simulation --timeout 60
```

### 11.4 Code Coverage Targets

| Crate | Target | Rationale |
|-------|--------|-----------|
| simulation | 80%+ | Core game logic; bugs here break the game |
| save | 90%+ | Data integrity; corruption is catastrophic |
| rendering | 60%+ | Hard to test visuals; focus on data transforms |
| ui | 50%+ | UI layout is best tested manually; logic can be tested |
| app | 40%+ | Mostly glue code; tested via integration tests |

Measure coverage with `cargo-tarpaulin` or `cargo-llvm-cov`:

```bash
# Install
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --workspace --html --output-dir coverage/

# Check specific crate coverage
cargo llvm-cov --package simulation --summary-only
```

### 11.5 Mutation Testing

Mutation testing (cargo-mutants) modifies your source code and checks if your tests catch the change. If tests pass after a mutation, you have a testing gap.

Critical functions to mutation-test in Megacity:

```bash
# Test all pure functions in simulation
cargo mutants -p simulation \
  --file crates/simulation/src/traffic.rs \
  --file crates/simulation/src/land_value.rs \
  --file crates/simulation/src/economy.rs \
  --file crates/simulation/src/happiness.rs \
  --file crates/simulation/src/citizen.rs \
  --timeout 120
```

Common mutations cargo-mutants applies:
- Replace `<` with `<=` (off-by-one errors)
- Replace `+` with `-` (sign errors)
- Replace `0` with `1` (boundary errors)
- Remove function calls (dead code detection)
- Replace `true` with `false` (logic inversions)

Example output showing a missed mutation:

```
MISSED: crates/simulation/src/happiness.rs:288: replace `happiness -= (pollution / 25.0)` with `happiness -= 0.0`
  No test catches this change! Pollution has no effect on happiness.
```

This would tell you that you need a test verifying pollution actually reduces happiness.

---

## 12. Testing Patterns from Other Simulation Games

City builders and simulation games have decades of testing wisdom. Here is what we can learn from the most successful ones.

### 12.1 Factorio: Deterministic Lockstep

Factorio is the gold standard for deterministic simulation. Their entire game state must match bit-for-bit between client and server in multiplayer. Key lessons:

**How they achieve determinism:**
- Custom deterministic math library (no stdlib sin/cos/sqrt).
- All random number generation uses a single, deterministic PRNG seeded from the map seed.
- Entity iteration order is guaranteed by using dense arrays with stable indices, not hash maps.
- Fixed-point arithmetic for positions and physics (they use 1/256th of a tile as the base unit).
- No floating-point comparison for game logic decisions -- all comparisons use integer math.

**What Megacity can adopt:**
- Replace `rand::thread_rng()` with a seeded `ChaCha8Rng` stored as a Resource.
- Consider using `BTreeMap` instead of `HashMap` for `RoadNetwork::edges`.
- Use `.to_bits()` for comparing floating point values in state hashes.
- The CSR graph already achieves deterministic node ordering via sorting.

**Testing approach:**
- Factorio runs the same scenario on two independent instances and compares CRC32 hashes of the game state every tick. If they diverge, it is a bug.
- They record player inputs and replay them in automated tests. The test suite includes hundreds of replay files from real multiplayer sessions.

### 12.2 Dwarf Fortress: Assertion-Heavy Development

Dwarf Fortress uses extensive runtime assertions that crash immediately on invariant violations. This "crash early" philosophy means bugs are caught the moment they occur, not 1000 ticks later when the effects become visible.

**Key pattern:**
```rust
// Dwarf Fortress style: assert invariants after every state change
fn assign_citizen_to_building(citizen: &mut Citizen, building: &mut Building) {
    assert!(building.occupants < building.capacity,
        "Cannot assign citizen to full building");
    building.occupants += 1;
    assert!(building.occupants <= building.capacity,
        "Building over capacity after assignment");
}
```

**What Megacity can adopt:**
- Add `debug_assert!` checks to critical functions in simulation code.
- In debug builds, validate invariants after each system runs.
- Create a `SimulationValidator` system that runs last in `FixedUpdate` and checks all invariants:

```rust
#[cfg(debug_assertions)]
pub fn validate_simulation_state(
    buildings: Query<&Building>,
    citizens: Query<(&Citizen, &HomeLocation, &CitizenDetails)>,
    budget: Res<CityBudget>,
    grid: Res<WorldGrid>,
) {
    // Budget sanity
    debug_assert!(budget.treasury.is_finite(), "Treasury is non-finite");
    debug_assert!(budget.tax_rate >= 0.0 && budget.tax_rate <= 1.0,
        "Tax rate out of range: {}", budget.tax_rate);

    // Building sanity
    for building in &buildings {
        debug_assert!(building.occupants <= building.capacity,
            "Building ({},{}) over capacity: {}/{}",
            building.grid_x, building.grid_y,
            building.occupants, building.capacity);
        debug_assert!(building.level <= building.zone_type.max_level(),
            "Building ({},{}) level {} exceeds max {} for {:?}",
            building.grid_x, building.grid_y,
            building.level, building.zone_type.max_level(),
            building.zone_type);
    }

    // Citizen sanity
    for (_, home, details) in &citizens {
        debug_assert!(grid.in_bounds(home.grid_x, home.grid_y),
            "Citizen home ({},{}) out of bounds",
            home.grid_x, home.grid_y);
        debug_assert!(details.happiness >= 0.0 && details.happiness <= 100.0,
            "Citizen happiness out of range: {}", details.happiness);
        debug_assert!(details.health >= 0.0 && details.health <= 100.0,
            "Citizen health out of range: {}", details.health);
    }
}
```

### 12.3 Paradox Games: Automated Test Campaigns

Paradox Interactive (EU4, CK3, Stellaris) runs automated "test campaigns" that play through entire game sessions:

**How it works:**
- Bot AI plays the game for 400 in-game years.
- At regular intervals, record metrics: population, economy, army size, stability.
- Compare metrics against expected ranges.
- If any metric falls outside the expected range, the test fails.
- Tests run overnight on a build server and results are reviewed each morning.

**What Megacity can adopt:**
- Create bot strategies (as described in section 9) that play through different city-building approaches.
- Run overnight and check that the city reaches expected milestones.
- Track metrics over time and flag sudden regressions.

### 12.4 RimWorld: Storyteller AI Testing

RimWorld's storyteller AI dynamically adjusts difficulty based on player performance. Testing this requires:

**Approach:**
- Run the same colony with different storyteller settings.
- Verify that harder settings produce more threats.
- Verify that the AI doesn't produce impossible scenarios (no threats for 10 years, then 50 raids at once).

**What Megacity can adopt for its events system:**
- The `events::random_city_events` system generates events based on city state. Test that:
  - Events fire at reasonable frequencies (not every tick, not never).
  - Event effects are bounded (no single event can bankrupt a healthy city).
  - Multiple simultaneous events don't stack to impossible levels.

```rust
#[test]
fn test_event_frequency_is_reasonable() {
    let mut app = create_benchmark_app(5_000);
    let mut event_count = 0;

    for _ in 0..10_000 {
        app.update();
        let journal = app.world().resource::<EventJournal>();
        event_count = journal.events.len();
    }

    // Over 10K ticks, we should see some events but not every tick
    assert!(event_count > 0, "No events in 10K ticks");
    assert!(event_count < 1000, "Too many events: {} in 10K ticks", event_count);
}
```

### 12.5 OpenTTD: Pathfinding Test Suite

OpenTTD has one of the most extensive pathfinding test suites in any open-source game. Their approach:

**Test categories:**
1. **Correctness tests**: Known start-goal pairs with pre-computed optimal paths.
2. **Performance tests**: Pathfinding must complete in < X ms for map sizes up to 4096x4096.
3. **Edge case tests**: Single-cell maps, disconnected graphs, one-way roads, loops.
4. **Regression tests**: Every pathfinding bug gets a regression test with the exact map configuration that triggered it.

**What Megacity can adopt:**

```rust
#[test]
fn test_pathfinding_correctness_straight_line() {
    let mut grid = WorldGrid::new(64, 64);
    let mut network = RoadNetwork::default();

    // Straight horizontal road
    for x in 0..64 {
        network.place_road(&mut grid, x, 32);
    }

    let csr = CsrGraph::from_road_network(&network);
    let path = csr_find_path(&csr, RoadNode(0, 32), RoadNode(63, 32))
        .expect("straight line should have path");

    // Path should be 64 nodes (one per cell)
    assert_eq!(path.len(), 64);

    // Path should be monotonically increasing in x
    for (i, node) in path.iter().enumerate() {
        assert_eq!(node.0, i, "Path should follow straight line");
        assert_eq!(node.1, 32, "Path should stay on y=32");
    }
}

#[test]
fn test_pathfinding_disconnected_graph() {
    let mut grid = WorldGrid::new(64, 64);
    let mut network = RoadNetwork::default();

    // Two disconnected road segments
    for x in 0..10 {
        network.place_road(&mut grid, x, 10);
    }
    for x in 50..60 {
        network.place_road(&mut grid, x, 10);
    }

    let csr = CsrGraph::from_road_network(&network);
    let path = csr_find_path(&csr, RoadNode(0, 10), RoadNode(55, 10));
    assert!(path.is_none(), "Should not find path between disconnected segments");
}

#[test]
fn test_pathfinding_loop() {
    let mut grid = WorldGrid::new(64, 64);
    let mut network = RoadNetwork::default();

    // Create a square loop
    for x in 10..=20 { network.place_road(&mut grid, x, 10); }
    for y in 10..=20 { network.place_road(&mut grid, 20, y); }
    for x in 10..=20 { network.place_road(&mut grid, x, 20); }
    for y in 10..=20 { network.place_road(&mut grid, 10, y); }

    let csr = CsrGraph::from_road_network(&network);

    // Path from one corner to the opposite should exist via either direction
    let path = csr_find_path(&csr, RoadNode(10, 10), RoadNode(20, 20))
        .expect("loop should have path");

    // Optimal path is 20 steps (10 right + 10 down or 10 down + 10 right)
    assert_eq!(path.len(), 21, "Optimal path around square should be 21 nodes");
}
```

---

## 13. Bevy-Specific Testing Patterns

Bevy's ECS architecture introduces unique testing challenges and opportunities. This section covers patterns specific to testing Bevy applications.

### 13.1 Minimal App Setup for Fast Tests

The biggest performance mistake in Bevy testing is using `DefaultPlugins` when you only need `MinimalPlugins`. DefaultPlugins initializes windowing, rendering, audio, and asset loading -- all of which are unnecessary for simulation tests and add seconds of startup time.

```rust
/// FAST: ~1ms startup, no window, no rendering
fn test_app_minimal() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Only add the specific plugin/systems you need
    app.add_plugins(SimulationPlugin);
    app
}

/// SLOW: ~500ms startup, creates window
fn test_app_full() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    // Only use this for visual/rendering tests
    app
}
```

**What MinimalPlugins provides:**
- `TaskPoolPlugin`: Thread pools for par_iter
- `TypeRegistrationPlugin`: Type registration
- `FrameCountPlugin`: Frame counting
- `TimePlugin`: Time resources
- `ScheduleRunnerPlugin`: App update loop (for headless)

**What MinimalPlugins does NOT provide:**
- Windowing (no window created)
- Rendering (no GPU access)
- Asset loading (no file system access)
- Audio (no sound system)
- Input handling (no keyboard/mouse)

### 13.2 Using SystemState for One-Shot System Testing

When you want to test a single system function without running the full App schedule, use `SystemState`:

```rust
use bevy::ecs::system::SystemState;

#[test]
fn test_congestion_level_via_system_state() {
    let mut world = World::new();
    world.insert_resource(TrafficGrid::default());

    // Modify the traffic grid
    {
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 10, 15);
    }

    // Use SystemState to create a "one-shot" system call
    let mut system_state: SystemState<Res<TrafficGrid>> =
        SystemState::new(&mut world);

    let traffic = system_state.get(&world);
    assert!((traffic.congestion_level(10, 10) - 0.75).abs() < 0.001);
}

#[test]
fn test_building_spawner_preconditions() {
    let mut world = World::new();
    world.insert_resource(WorldGrid::new(32, 32));
    world.insert_resource(ZoneDemand::default());
    world.insert_resource(BuildingSpawnTimer::default());

    // Create a SystemState matching the building_spawner parameters
    let mut system_state: SystemState<(
        Commands,
        ResMut<WorldGrid>,
        Res<ZoneDemand>,
        ResMut<BuildingSpawnTimer>,
    )> = SystemState::new(&mut world);

    let (commands, grid, demand, timer) = system_state.get_mut(&mut world);

    // Verify preconditions
    assert!(demand.demand_for(ZoneType::ResidentialLow) < 0.1,
        "Default demand should be low");
    assert_eq!(timer.0, 0, "Timer should start at 0");
}
```

### 13.3 Testing with Update Schedule vs Manual world.run_system()

There are two approaches to running systems in tests:

**Approach 1: App::update() (recommended for integration tests)**
- Runs the full schedule with all system ordering constraints
- Handles `apply_deferred` automatically
- Closest to production behavior
- But slower and tests multiple systems at once

```rust
#[test]
fn test_via_app_update() {
    let mut app = test_app();
    app.update(); // runs ALL systems in order

    // Check results
    let world = app.world();
    let stats = world.resource::<CityStats>();
    assert_eq!(stats.population, 10_000); // from init_world
}
```

**Approach 2: world.run_system_once() (for isolated system tests)**
- Runs a single system function
- Must manually register the system first
- Must handle `world.flush()` for deferred operations
- Faster and more focused

```rust
#[test]
fn test_via_run_system() {
    let mut world = World::new();
    // Insert all resources this system needs
    world.insert_resource(TickCounter(0));
    world.insert_resource(TrafficGrid::default());
    // ... insert other required resources ...

    // Register and run the system
    let system_id = world.register_system(traffic::update_traffic_density);
    world.run_system(system_id).expect("system should run");

    // Check results
    let traffic = world.resource::<TrafficGrid>();
    // ... assertions ...
}
```

### 13.4 Resource Initialization Order in Tests

Bevy resources must be initialized before systems that use them run. In production, `SimulationPlugin::build()` handles this with `init_resource::<T>()`. In tests, you must either:

1. Use the full `SimulationPlugin` (recommended)
2. Manually insert all required resources

The second approach is error-prone because systems may depend on resources you forgot to add. The error message is usually a panic: "Resource X does not exist."

```rust
// Pattern: wrap resource insertion in a helper
fn insert_simulation_resources(world: &mut World) {
    world.insert_resource(TickCounter::default());
    world.insert_resource(SlowTickTimer::default());
    world.insert_resource(TrafficGrid::default());
    world.insert_resource(CityBudget::default());
    world.insert_resource(GameClock::default());
    world.insert_resource(ZoneDemand::default());
    world.insert_resource(CityStats::default());
    world.insert_resource(PollutionGrid::default());
    world.insert_resource(LandValueGrid::default());
    world.insert_resource(GarbageGrid::default());
    world.insert_resource(CrimeGrid::default());
    world.insert_resource(NoisePollutionGrid::default());
    world.insert_resource(Policies::default());
    world.insert_resource(Weather::default());
    world.insert_resource(ServiceCoverageGrid::default());
    world.insert_resource(CsrGraph::default());
    world.insert_resource(RoadSegmentStore::default());
    world.insert_resource(DestinationCache::default());
    world.insert_resource(ExtendedBudget::default());
    world.insert_resource(LoanBook::default());
    // ... all other resources from SimulationPlugin::build ...
}
```

### 13.5 Mocking External Dependencies

Megacity's simulation has few external dependencies, but there are some:

**Time mocking:**
Bevy's `Time` resource can be overridden in tests:

```rust
#[test]
fn test_with_fixed_time() {
    let mut app = test_app();

    // Override the fixed timestep
    app.insert_resource(Time::<Fixed>::from_hz(10.0));

    // Now FixedUpdate runs at exactly 10Hz
    app.update();
}
```

**Random number mocking:**
Replace `rand::thread_rng()` with a seeded RNG resource:

```rust
#[derive(Resource)]
pub struct MockRng {
    values: Vec<f32>,
    index: usize,
}

impl MockRng {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values, index: 0 }
    }

    pub fn next(&mut self) -> f32 {
        let val = self.values[self.index % self.values.len()];
        self.index += 1;
        val
    }
}

#[test]
fn test_building_spawner_with_mock_rng() {
    // Mock RNG that always returns 0.0 (below any spawn_chance)
    // This means every eligible cell will get a building
    let mock = MockRng::new(vec![0.0]);
    // ... use in test to control building spawn behavior
}
```

**File system mocking (for save/load):**
Use `tempfile` for save/load tests:

```rust
#[test]
fn test_save_to_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("test_save.bin");

    let save_data = create_minimal_save();
    let bytes = save_data.encode();

    std::fs::write(&path, &bytes).expect("write save file");
    let loaded_bytes = std::fs::read(&path).expect("read save file");
    let loaded = SaveData::decode(&loaded_bytes).expect("decode save file");

    assert_eq!(loaded.grid.width, save_data.grid.width);
}
```

### 13.6 Testing Parallel Systems

Megacity uses `par_iter_mut()` in `update_happiness` and `move_citizens`. Testing parallel systems requires attention to:

1. **Correctness**: Results should be identical whether running in parallel or serial.
2. **No data races**: Rust's type system prevents this at compile time, but logical races (order-dependent behavior) can still occur.

```rust
#[test]
fn test_parallel_happiness_matches_serial() {
    // Run happiness calculation in parallel and serial, compare results

    let mut app_parallel = test_app();
    let mut app_serial = test_app();
    // Both start with identical state
    // ... setup identical worlds ...

    run_ticks(&mut app_parallel, 100);
    run_ticks(&mut app_serial, 100);

    // Compare all citizen happiness values
    let happiness_par: Vec<f32> = app_parallel.world()
        .query::<&CitizenDetails>()
        .iter(app_parallel.world())
        .map(|d| d.happiness)
        .collect();

    let happiness_ser: Vec<f32> = app_serial.world()
        .query::<&CitizenDetails>()
        .iter(app_serial.world())
        .map(|d| d.happiness)
        .collect();

    assert_eq!(happiness_par.len(), happiness_ser.len());
    // Note: order may differ, so sort both
    let mut par_sorted = happiness_par.clone();
    let mut ser_sorted = happiness_ser.clone();
    par_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ser_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for (p, s) in par_sorted.iter().zip(ser_sorted.iter()) {
        assert!((p - s).abs() < 0.01,
            "Happiness mismatch: parallel={}, serial={}", p, s);
    }
}
```

### 13.7 Testing Change Detection

Bevy's change detection (`is_changed()`, `is_added()`) is heavily used in Megacity. For example, `rebuild_csr_on_road_change` only rebuilds the CSR graph when `roads.is_changed()` returns true.

In tests, change detection works within `App::update()` but NOT across manual resource mutations. After calling `world.resource_mut()`, the resource is marked as changed for the next system run.

```rust
#[test]
fn test_csr_rebuilds_on_road_change() {
    let mut app = test_app();
    app.update(); // init

    let initial_node_count = app.world()
        .resource::<CsrGraph>()
        .node_count();

    // Add a road -- this marks RoadNetwork as changed
    {
        let world = app.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let mut roads = world.resource_mut::<RoadNetwork>();
        roads.place_road(&mut grid, 100, 100);
    }

    app.update(); // rebuild_csr_on_road_change should fire

    let new_node_count = app.world()
        .resource::<CsrGraph>()
        .node_count();

    assert!(new_node_count > initial_node_count,
        "CSR should have more nodes after adding a road: {} -> {}",
        initial_node_count, new_node_count);
}
```

### 13.8 Testing Events

Bevy events (like `BankruptcyEvent`) have a two-frame lifetime. Tests must read events in the same or next update cycle:

```rust
#[test]
fn test_bankruptcy_event_fires() {
    let mut app = test_app();
    app.update();

    // Set treasury to deeply negative
    {
        let world = app.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = -100_000.0;
    }

    app.update();

    // Check for bankruptcy event
    let events = app.world().resource::<Events<BankruptcyEvent>>();
    let reader = events.get_reader();
    // Note: events have a 2-frame lifetime, so check in the same update
}
```

### 13.9 Testing with #[should_panic]

For testing that invalid operations correctly panic:

```rust
#[test]
#[should_panic(expected = "index out of bounds")]
fn test_grid_panics_on_out_of_bounds_access() {
    let grid = WorldGrid::new(10, 10);
    let _ = grid.get(10, 10); // x=10 is out of bounds for width=10
}

#[test]
#[should_panic]
fn test_traffic_grid_panics_on_out_of_bounds() {
    let traffic = TrafficGrid::default();
    let _ = traffic.get(GRID_WIDTH, 0); // out of bounds
}
```

### 13.10 Test Organization in the Workspace

```
crates/
  simulation/
    src/
      traffic.rs          # contains #[cfg(test)] mod tests { ... }
      happiness.rs        # contains #[cfg(test)] mod tests { ... }
      economy.rs          # contains #[cfg(test)] mod tests { ... }
      ...
    tests/                # integration tests
      city_growth.rs      # end-to-end city growth tests
      economy_balance.rs  # economy balance tests
      pathfinding.rs      # pathfinding edge cases
      save_load.rs        # save/load round-trip tests
    benches/              # benchmarks
      simulation_benchmarks.rs
  save/
    src/
      serialization.rs    # contains #[cfg(test)] mod tests { ... }
    tests/
      roundtrip.rs        # save/load integration tests
      fuzzing.rs          # save file fuzzing
  rendering/
    tests/
      overlay_colors.rs   # overlay color mapping tests
      lod_transitions.rs  # LOD tier assignment tests
```

---

## Appendix: Quick Reference

### Running Tests

```bash
# All unit tests (fast, ~30s)
cargo test --workspace --lib

# All tests including integration (medium, ~2min)
cargo test --workspace

# Stress tests (slow, ~30min)
cargo test --workspace -- --ignored

# Specific test
cargo test -p simulation test_congestion_level

# Tests matching a pattern
cargo test -p simulation traffic

# With output
cargo test -p simulation -- --nocapture

# Benchmarks
cargo bench -p simulation

# Coverage
cargo llvm-cov --workspace --html

# Mutation testing
cargo mutants -p simulation --timeout 120
```

### Test Categories at a Glance

| Category | Location | Speed | CI Frequency | Purpose |
|----------|----------|-------|-------------|---------|
| Unit tests | `#[cfg(test)] mod tests` | Fast (<1s each) | Every PR | Formula correctness |
| Property tests | `#[cfg(test)]` with proptest | Medium (1-10s) | Every PR | Invariant verification |
| Integration tests | `tests/` directory | Medium (5-30s) | Every PR | System interaction |
| Benchmarks | `benches/` directory | Slow (1-5min) | Nightly | Performance regression |
| Stress tests | `#[ignore]` tests | Very slow (5-60min) | Weekly | Stability under load |
| Mutation tests | cargo-mutants | Very slow (30-60min) | Weekly | Test quality |
| Visual tests | Screenshot comparison | Slow (1-5min) | Nightly | Rendering regression |
| Balance tests | Monte Carlo simulation | Very slow (30min+) | Weekly | Gameplay balance |

### Key Invariants to Test

1. **Treasury is always finite** (never NaN, never Inf)
2. **Population is always non-negative** (u32 handles this by type)
3. **Building occupants never exceed capacity**
4. **Grid indices are always in bounds**
5. **Happiness is clamped to [0.0, 100.0]**
6. **Health is clamped to [0.0, 100.0]**
7. **Tax rate is in [0.0, 1.0]**
8. **Congestion level is in [0.0, 1.0]**
9. **Road network is symmetric (undirected graph)**
10. **CSR graph node count matches RoadNetwork**
11. **Citizens reference valid buildings**
12. **PathCache index does not exceed waypoint count**
13. **Building level does not exceed zone max_level**
14. **Loan remaining_balance is non-negative**
15. **Save/load round-trips preserve all state**
