# Megacity Architecture Guide

This document is for new contributors. It explains how the codebase is organized, how to add new features without merge conflicts, and how the major systems work.

For build commands and PR requirements, see [CLAUDE.md](./CLAUDE.md).

---

## Table of Contents

1. [Crate Overview](#crate-overview)
2. [Grid and World Model](#grid-and-world-model)
3. [ECS Basics for Megacity](#ecs-basics-for-megacity)
4. [Per-Feature Plugin Pattern](#per-feature-plugin-pattern)
5. [Adding a New Feature (Step-by-Step)](#adding-a-new-feature-step-by-step)
6. [Save System (Saveable Trait)](#save-system-saveable-trait)
7. [Road System Architecture](#road-system-architecture)
8. [Simulation Scheduling](#simulation-scheduling)
9. [Testing Patterns](#testing-patterns)
10. [Key Resources Reference](#key-resources-reference)

---

## Crate Overview

The workspace lives in `crates/` with five crates:

| Crate | Purpose |
|-------|---------|
| **simulation** | All game logic: economy, citizens, roads, weather, services, etc. |
| **rendering** | Bevy rendering: meshes, cameras, overlays, visual effects |
| **ui** | egui-based UI panels, toolbars, dashboards |
| **save** | Save/load serialization, migration, codec |
| **app** | Binary entry point (`main.rs`), wires everything together |

Almost all new features go in the **simulation** crate. Rendering and UI changes are separate and typically consume simulation resources via `Res<T>`.

## Grid and World Model

The world is a **256x256 cell grid** defined in `simulation/src/grid.rs`:

```
Grid:  256 x 256 cells
Cell:  16.0 world units (CELL_SIZE)
Chunk: 8x8 cells (CHUNK_SIZE), used for spatial queries
World: 4096 x 4096 world units
```

Each cell (`Cell`) has:
- `cell_type`: `Grass`, `Water`, or `Road`
- `zone`: `None`, `ResidentialLow`, `CommercialHigh`, `Industrial`, etc.
- `building_id`: optional entity reference to the building on this cell
- `road_type`: which road type occupies the cell (if any)

The grid resource (`WorldGrid`) is the central spatial data structure. Systems read it to find neighbors, check cell types, and locate buildings.

Key types in `grid.rs`:
- `WorldGrid` — the 256x256 array of `Cell`
- `CellType` — grass/water/road
- `ZoneType` — residential/commercial/industrial/office/mixed-use variants
- `RoadType` — local/avenue/boulevard/highway/oneway/path

## ECS Basics for Megacity

Megacity uses [Bevy ECS](https://bevyengine.org/). The three core concepts:

- **Entities**: unique IDs (citizens, buildings, service buildings)
- **Components**: data attached to entities (`Position`, `Citizen`, `Building`)
- **Resources**: global singletons (`WorldGrid`, `CityBudget`, `Weather`)
- **Systems**: functions that query entities/resources and mutate state

Most simulation features are **resource-based** (they operate on a global `Resource` rather than per-entity `Component`s). Citizens and buildings are the main entity-based systems.

## Per-Feature Plugin Pattern

Every feature is a self-contained module with its own `Plugin` struct. This is the key architectural pattern — it means multiple contributors can work on different features without merge conflicts.

A feature module contains:
1. **Resource types** — the state this feature tracks
2. **Systems** — functions that run each tick to update the state
3. **Plugin struct** — registers resources and systems with the Bevy app

Example from `drought.rs`:

```rust
// 1. Resource
#[derive(Resource, Clone, Debug, Default)]
pub struct DroughtState {
    pub current_index: f32,
    pub current_tier: DroughtTier,
    // ...
}

// 2. System
pub fn update_drought_index(
    weather: Res<Weather>,
    mut drought: ResMut<DroughtState>,
    timer: Res<SlowTickTimer>,
) {
    if !timer.should_run() { return; }
    // ... compute drought from weather data
}

// 3. Plugin
pub struct DroughtPlugin;

impl Plugin for DroughtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DroughtState>()
            .add_systems(
                FixedUpdate,
                update_drought_index.in_set(SimulationSet::Simulation),
            );
    }
}
```

The plugin is then added to `SimulationPlugin::build()` in `lib.rs` within the appropriate group:

```rust
// Weather and environment
app.add_plugins((
    weather::WeatherPlugin,
    drought::DroughtPlugin,
    // ...
));
```

### Rules

- Each module registers its **own** resources and systems inside its `Plugin::build()`.
- Do **not** add `init_resource` or `add_systems` calls to `lib.rs` directly — the plugin handles it.
- The only change to `lib.rs` is: (a) `pub mod my_feature;` and (b) adding `my_feature::MyPlugin` to a plugin group.

## Adding a New Feature (Step-by-Step)

### Step 1: Create the Module

Create `crates/simulation/src/my_feature.rs`:

```rust
use bevy::prelude::*;
use crate::SlowTickTimer;

/// State resource for my feature.
#[derive(Resource, Clone, Debug, Default)]
pub struct MyFeatureState {
    pub value: f32,
}

/// System that updates my feature each slow tick.
fn update_my_feature(
    mut state: ResMut<MyFeatureState>,
    timer: Res<SlowTickTimer>,
) {
    if !timer.should_run() { return; }
    state.value += 1.0;
}

pub struct MyFeaturePlugin;

impl Plugin for MyFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyFeatureState>()
            .add_systems(
                FixedUpdate,
                update_my_feature.in_set(crate::SimulationSet::Simulation),
            );
    }
}
```

### Step 2: Register in lib.rs (2 lines)

In `crates/simulation/src/lib.rs`:

```rust
// Add the module declaration (alphabetical order):
pub mod my_feature;

// Add the plugin to the appropriate group in SimulationPlugin::build():
app.add_plugins((
    // ... existing plugins in this group ...
    my_feature::MyFeaturePlugin,
));
```

**Note:** Bevy `add_plugins` tuples have a max of 15 elements. If a group is full, start a new `app.add_plugins((...))` call.

### Step 3: Add Tests

Add unit tests in the same file and integration tests in `integration_tests.rs` (see [Testing Patterns](#testing-patterns)).

### Step 4: (Optional) Add Persistence

If your feature state should survive save/load, implement `Saveable` (see next section).

That is it. You have touched exactly 2 lines in shared files (`pub mod` + plugin registration), so you will not conflict with other contributors.

## Save System (Saveable Trait)

The save system uses an **extension map** pattern so new features can add persistence without modifying any save system files.

### How It Works

1. `SaveData` contains a `BTreeMap<String, Vec<u8>>` for extensions
2. Each saveable resource implements the `Saveable` trait (in `simulation/src/lib.rs`)
3. On save, the `SaveableRegistry` iterates all registered types and serializes them
4. On load, it deserializes matching keys and resets missing keys to defaults

### Implementing Saveable

```rust
use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct MyFeatureState {
    pub enabled: bool,
    pub level: u32,
}

impl crate::Saveable for MyFeatureState {
    const SAVE_KEY: &'static str = "my_feature";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Return None to skip saving default state (saves space)
        if *self == Self::default() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
```

### Registering in Your Plugin

```rust
impl Plugin for MyFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyFeatureState>()
            .add_systems(/* ... */);

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<MyFeatureState>();
    }
}
```

### Validation

Add your `SAVE_KEY` to the `EXPECTED_SAVEABLE_KEYS` array in `lib.rs`. A startup system validates that every expected key is registered, catching forgotten registrations.

### What NOT to Touch

Do **not** modify these files:
- `save/src/save_types.rs`
- `save/src/serialization.rs`
- `save/src/save_restore.rs`
- `save/src/save_helpers.rs`

The extension map pattern exists precisely so new features never need to touch these.

## Road System Architecture

The road system has three layers:

```
Layer 1: RoadSegmentStore  (source of truth)
    |
    v  rasterize
Layer 2: WorldGrid cells   (derived cache for spatial queries)
    |
    v  build graph
Layer 3: CsrGraph          (pathfinding)
```

### Layer 1: Road Segments (`road_segments.rs`)

Roads are stored as **cubic Bezier curves** (`RoadSegment`). Each segment has:
- Start/end nodes (`SegmentNodeId`)
- Four control points (`p0`, `p1`, `p2`, `p3`)
- Road type (local, avenue, highway, etc.)
- Pre-computed arc length and rasterized cells

The `RoadSegmentStore` resource is the authoritative representation. When a player draws a road, a segment is added here first.

### Layer 2: Grid Cache

Segments are **rasterized** onto the `WorldGrid` — cells under a segment get `cell_type = Road` and the appropriate `road_type`. This allows fast spatial queries ("is this cell a road?") without iterating all segments.

Use `RoadType::half_width()` for road widths — never hardcode pixel values.

### Layer 3: CSR Graph (`road_graph_csr.rs`)

The `CsrGraph` is built from the `RoadNetwork` adjacency list (which is itself derived from segments). It stores the graph in Compressed Sparse Row format for cache-friendly A* pathfinding.

Traffic-aware routing: `csr_find_path_with_traffic()` uses `TrafficGrid` congestion data to penalize busy roads, causing citizens to route around traffic jams.

### Key Resources

- `RoadSegmentStore` — Bezier segments (source of truth)
- `RoadNetwork` — adjacency list (`HashMap<RoadNode, Vec<RoadNode>>`)
- `CsrGraph` — CSR arrays for pathfinding
- `TrafficGrid` — per-cell congestion counters

## Simulation Scheduling

### Fixed vs. Update

- `FixedUpdate` — deterministic simulation tick (10 Hz default). All game logic runs here.
- `Update` — render-rate frame. Visual-only systems (LOD, overlays) run here.

### System Sets

Defined in `simulation_sets.rs`:

```
FixedUpdate:  PreSim → Simulation → PostSim  (chained)
Update:       Input → Visual                   (chained)
```

Most systems use `.in_set(SimulationSet::Simulation)`. Use `PreSim` for setup that other systems depend on, and `PostSim` for cleanup/aggregation.

### Slow Tick Timer

Many grid-wide systems (pollution, land value, crime) are expensive. They use `SlowTickTimer` to run every 100 ticks (~10 seconds):

```rust
fn my_expensive_system(timer: Res<SlowTickTimer>, /* ... */) {
    if !timer.should_run() { return; }
    // ... expensive grid scan
}
```

### System Ordering

Use `.after()` and `.before()` to declare dependencies:

```rust
app.add_systems(
    FixedUpdate,
    my_system
        .after(crate::stats::update_stats)
        .in_set(SimulationSet::Simulation),
);
```

## Testing Patterns

### Unit Tests

Add unit tests at the bottom of your module file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = MyFeatureState::default();
        assert_eq!(state.value, 0.0);
    }
}
```

### Integration Tests with TestCity

The `TestCity` harness (`test_harness.rs`) provides a headless Bevy App with the full `SimulationPlugin` for testing emergent behavior across systems.

Integration tests live in `integration_tests.rs`.

```rust
use crate::test_harness::TestCity;

#[test]
fn test_my_feature_activates_after_ticks() {
    let mut city = TestCity::new();

    // Set up initial state
    city.place_road(128, 128, RoadType::Local);
    city.zone_cell(129, 128, ZoneType::ResidentialLow);

    // Run simulation
    city.tick_slow_cycles(3);

    // Assert outcomes
    assert!(city.building_count() > 0);
}
```

**Key TestCity methods:**

| Method | Purpose |
|--------|---------|
| `TestCity::new()` | Empty 256x256 grass grid |
| `TestCity::with_tel_aviv()` | Full map with ~10K citizens |
| `tick()` | Advance one `FixedUpdate` cycle |
| `tick_slow_cycle()` | Advance 100 ticks (one `SlowTickTimer` cycle) |
| `tick_slow_cycles(n)` | Advance n slow cycles |
| `citizen_count()` | Count citizen entities |
| `building_count()` | Count building entities |
| `place_road(x, y, type)` | Place a road cell |
| `zone_cell(x, y, zone)` | Zone a cell |
| `assert_resource_exists::<T>()` | Assert a resource is present |

**Naming convention:** `test_<feature>_<scenario>_<expected_outcome>`

Use `TestCity::with_tel_aviv()` for smoke/regression tests against the full map (slower, but tests real-world conditions).

### Saveable Roundtrip Tests

If your feature implements `Saveable`, add roundtrip tests:

```rust
#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;
    let mut state = MyFeatureState::default();
    state.level = 5;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = MyFeatureState::load_from_bytes(&bytes);
    assert_eq!(restored.level, 5);
}

#[test]
fn test_saveable_skips_default() {
    use crate::Saveable;
    let state = MyFeatureState::default();
    assert!(state.save_to_bytes().is_none());
}
```

## Key Resources Reference

| Resource | File | Purpose |
|----------|------|---------|
| `WorldGrid` | `grid.rs` | 256x256 cell array — the world map |
| `CityBudget` | `economy.rs` | Treasury, tax rate, income/expenses |
| `ExtendedBudget` | `budget.rs` | Per-category budget breakdown |
| `Weather` | `weather.rs` | Temperature, precipitation, wind, season |
| `GameClock` | `time_of_day.rs` | In-game time, day/month/year, pause state |
| `CityStats` | `stats.rs` | Population, happiness, employment aggregates |
| `RoadSegmentStore` | `road_segments.rs` | Bezier road segments (source of truth) |
| `RoadNetwork` | `roads.rs` | Road adjacency list |
| `CsrGraph` | `road_graph_csr.rs` | CSR pathfinding graph |
| `TrafficGrid` | `traffic.rs` | Per-cell traffic congestion |
| `Policies` | `policies.rs` | Active policy toggles |
| `SlowTickTimer` | `lib.rs` | Throttle for expensive systems (every 100 ticks) |
| `TickCounter` | `lib.rs` | Global tick counter |
| `SaveableRegistry` | `lib.rs` | Type-erased save/load registry |
| `SpatialGrid` | `spatial_grid.rs` | Chunk-based spatial index for entities |
| `PollutionGrid` | `pollution.rs` | Per-cell pollution levels |
| `LandValueGrid` | `land_value.rs` | Per-cell land values |
| `ServiceCoverageGrid` | `happiness.rs` | Per-cell service coverage bitmask |

## Common Pitfalls

- **Bevy system parameter limit:** max 16 parameters per system function. Group `Res` params into tuples if needed.
- **Bevy `add_plugins` tuple limit:** max 15 plugins per call. Split into multiple `app.add_plugins((...))`.
- **Clippy `too_many_arguments`:** max 7 args. Add `#[allow(clippy::too_many_arguments)]` above Bevy system functions.
- **Clippy `type_complexity`:** use type aliases for complex `Box<dyn Fn(...)>` types.
- **Road widths:** always use `RoadType::half_width()`, never hardcode.
- **Diagonal neighbors:** use `neighbors8()` for 8-connectivity.
- **Saving:** return `None` from `save_to_bytes()` when state equals default to save space.
