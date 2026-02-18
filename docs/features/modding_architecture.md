# Modding Architecture for Megacity

## Executive Summary

Cities: Skylines 1's modding ecosystem is the single greatest factor behind its decade-long commercial success. Traffic Manager: President Edition (TM:PE, 8M+ subscribers), Move It (5M+), and Ploppable RICO (3M+) each solved problems that the base game could not, creating a positive feedback loop where mods attracted players who attracted more modders. CS2's launch failure was partly attributed to breaking this ecosystem.

Megacity must be moddable from day one. This is not a post-launch feature; it is an architectural requirement that shapes every system in the game. This document provides a complete technical blueprint for modding support, covering plugin architecture, scripting integration, asset pipelines, sandboxing, data-driven design, distribution, and backward compatibility.

The core thesis is this: **the more data-driven the base game is, the less scripting mods need, and the less scripting mods need, the fewer things can break between versions.** The ideal modding architecture is a pyramid: 80% of mods are pure data (building definitions, road parameters, policy tweaks), 15% use lightweight scripting (Lua/WASM for custom logic), and only 5% need native plugins (deep system overhauls like TM:PE).

---

## Table of Contents

1. [Plugin Architecture for Bevy](#1-plugin-architecture-for-bevy)
   1. [How Bevy's Plugin System Already Works](#11-how-bevys-plugin-system-already-works)
   2. [The Mod SDK Crate: Stable API Without Exposing Internals](#12-the-mod-sdk-crate-stable-api-without-exposing-internals)
   3. [Hot-Reloading Native Plugins](#13-hot-reloading-native-plugins)
   4. [Mod Load Ordering and Dependency Resolution](#14-mod-load-ordering-and-dependency-resolution)
   5. [Mod Conflict Detection and Resolution](#15-mod-conflict-detection-and-resolution)
2. [Scripting Language Integration](#2-scripting-language-integration)
   1. [Lua via mlua](#21-lua-via-mlua)
   2. [WASM via wasmtime](#22-wasm-via-wasmtime)
   3. [Rhai: Rust-Native Scripting](#23-rhai-rust-native-scripting)
   4. [Comparison Matrix and Recommendation](#24-comparison-matrix-and-recommendation)
   5. [API Surface Design for Scripts](#25-api-surface-design-for-scripts)
3. [Asset Pipeline for Custom Content](#3-asset-pipeline-for-custom-content)
   1. [Custom Buildings](#31-custom-buildings)
   2. [Custom Vehicles](#32-custom-vehicles)
   3. [Custom Props and Trees](#33-custom-props-and-trees)
   4. [Asset Packaging Format](#34-asset-packaging-format)
   5. [Asset Validation and Loading](#35-asset-validation-and-loading)
   6. [Hot-Reloading Assets During Development](#36-hot-reloading-assets-during-development)
4. [What Modders Actually Need: CS1 Mod Analysis](#4-what-modders-actually-need-cs1-mod-analysis)
   1. [TM:PE Equivalent: Traffic System API](#41-tmpe-equivalent-traffic-system-api)
   2. [RICO Equivalent: Building Spawn Hooks](#42-rico-equivalent-building-spawn-hooks)
   3. [Move It Equivalent: Entity Transform Access](#43-move-it-equivalent-entity-transform-access)
   4. [81 Tiles Equivalent: Map Size Configuration](#44-81-tiles-equivalent-map-size-configuration)
   5. [Network Extensions Equivalent: Data-Driven Road Types](#45-network-extensions-equivalent-data-driven-road-types)
   6. [Asset Editor Equivalent: In-Game Creation Tools](#46-asset-editor-equivalent-in-game-creation-tools)
5. [Mod Distribution](#5-mod-distribution)
   1. [Steam Workshop Integration](#51-steam-workshop-integration)
   2. [Self-Hosted Mod Repository](#52-self-hosted-mod-repository)
   3. [Mod Manager UI](#53-mod-manager-ui)
6. [Sandboxing and Security](#6-sandboxing-and-security)
   1. [Threat Model](#61-threat-model)
   2. [WASM Sandboxing](#62-wasm-sandboxing)
   3. [Lua Sandboxing](#63-lua-sandboxing)
   4. [Native Plugin Risks](#64-native-plugin-risks)
   5. [Resource Limits](#65-resource-limits)
7. [Data-Driven Architecture](#7-data-driven-architecture)
   1. [Making Game Data Moddable Without Code](#71-making-game-data-moddable-without-code)
   2. [Bevy's Asset System for Data Files](#72-bevys-asset-system-for-data-files)
   3. [Override Hierarchy](#73-override-hierarchy)
   4. [Current Hardcoded Values That Must Become Data](#74-current-hardcoded-values-that-must-become-data)
8. [Backward Compatibility](#8-backward-compatibility)
   1. [Stable Mod API Versioning Strategy](#81-stable-mod-api-versioning-strategy)
   2. [Evolving Internal Systems Without Breaking Mods](#82-evolving-internal-systems-without-breaking-mods)
   3. [Save File Compatibility with Mods](#83-save-file-compatibility-with-mods)
9. [Implementation Roadmap](#9-implementation-roadmap)
10. [Appendix: Reference Implementations](#10-appendix-reference-implementations)

---

## 1. Plugin Architecture for Bevy

### 1.1 How Bevy's Plugin System Already Works

Bevy's architecture is fundamentally plugin-based. Every feature in Bevy -- rendering, audio, input, windowing -- is a plugin. Our own game already uses this pattern:

```rust
// crates/app/src/main.rs - current architecture
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin { ... }))
       .add_plugins((
           simulation::SimulationPlugin,
           rendering::RenderingPlugin,
           ui::UiPlugin,
           save::SavePlugin,
       ));
    app.run();
}
```

The `Plugin` trait is straightforward:

```rust
pub trait Plugin: Send + Sync + 'static {
    fn build(&self, app: &mut App);

    // Optional: called after all plugins have been built
    fn finish(&self, app: &mut App) { }

    // Optional: called after finish, for final cleanup
    fn cleanup(&self, app: &mut App) { }

    // Plugin name for debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
```

The `build()` method receives `&mut App`, which gives full access to:
- **Resources**: `app.init_resource::<T>()`, `app.insert_resource(value)`
- **Systems**: `app.add_systems(schedule, system)` with ordering constraints
- **Events**: `app.add_event::<T>()`
- **States**: `app.init_state::<T>()`
- **Sub-apps**: `app.add_sub_app(label, sub_app)`
- **World**: `app.world_mut()` for direct ECS manipulation

This is both a strength and a risk. Strength: mods can do anything the game itself can do. Risk: mods can break anything the game itself depends on. The solution is the SDK crate pattern described next.

**Plugin Groups** allow bundling related plugins:

```rust
pub struct MegacityDefaultPlugins;

impl PluginGroup for MegacityDefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SimulationPlugin)
            .add(RenderingPlugin)
            .add(UiPlugin)
            .add(SavePlugin)
    }
}
```

The `finish()` method is critical for modding: it runs after all plugins' `build()` methods have completed. This means a mod plugin can register systems in `build()` that depend on resources another mod registers, and both will be available by the time `finish()` runs. We use this for mod dependency resolution.

### 1.2 The Mod SDK Crate: Stable API Without Exposing Internals

The fundamental tension in modding is: modders need access to game systems, but the game needs freedom to refactor internals. The solution is the **facade pattern** via a dedicated `megacity-mod-sdk` crate.

**Current crate hierarchy:**
```
cities/
  crates/
    app/          -- binary entry point
    simulation/   -- all game logic (grid, citizens, economy, traffic, ...)
    rendering/    -- Bevy rendering (meshes, cameras, overlays)
    ui/           -- egui UI panels
    save/         -- serialization
```

**Proposed crate hierarchy with modding:**
```
cities/
  crates/
    app/              -- binary entry point, mod loader
    simulation/       -- INTERNAL: all game logic
    rendering/        -- INTERNAL: rendering systems
    ui/               -- INTERNAL: UI systems
    save/             -- INTERNAL: serialization
    mod-sdk/          -- PUBLIC: stable modding API
    mod-sdk-derive/   -- PUBLIC: proc macros for mod authors
    mod-host/         -- INTERNAL: mod loading, sandboxing, lifecycle
```

The `mod-sdk` crate re-exports carefully curated types and traits:

```rust
// crates/mod-sdk/src/lib.rs

//! Megacity Mod SDK v1.0
//!
//! This crate provides the stable public API for Megacity mods.
//! Types and traits in this crate follow semantic versioning.
//! Internal game systems are NOT re-exported -- use the provided
//! abstractions instead.

// Re-export Bevy types mods will need
pub use bevy::prelude::{
    App, Plugin, Resource, Component, Entity, Query, Res, ResMut,
    Commands, EventReader, EventWriter, Transform, Vec2, Vec3,
    With, Without, Added, Changed, In, Update, FixedUpdate, Startup,
};

// === Stable API types (versioned, backward-compatible) ===

/// Grid position (abstraction over internal WorldGrid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

/// World-space position (abstraction over internal coordinate system)
#[derive(Debug, Clone, Copy)]
pub struct WorldPos {
    pub x: f32,
    pub y: f32,
}

/// Building information exposed to mods
#[derive(Debug, Clone)]
pub struct BuildingInfo {
    pub entity: Entity,
    pub zone: ZoneKind,
    pub level: u8,
    pub position: GridPos,
    pub capacity: u32,
    pub occupants: u32,
}

/// Zone types (mirrors internal ZoneType but is versioned separately)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZoneKind {
    ResidentialLow,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
    Custom(u32),  // <-- Mods can define new zone types!
}

/// Road type definition (data-driven, not an enum)
#[derive(Debug, Clone)]
pub struct RoadDefinition {
    pub id: String,
    pub display_name: String,
    pub speed: f32,
    pub lane_count: u8,
    pub cost: f64,
    pub allows_zoning: bool,
    pub allows_vehicles: bool,
    pub noise_radius: u8,
    pub width_cells: usize,
    pub mesh_path: Option<String>,  // custom mesh for the road surface
}

// === Traits that mods implement ===

/// Main trait for Megacity mods
pub trait MegacityMod: Plugin {
    /// Mod metadata
    fn metadata(&self) -> ModMetadata;

    /// Called when the mod should register its data definitions
    fn register_data(&self, registry: &mut DataRegistry) { }

    /// Called when a new city is created (opportunity to add initial state)
    fn on_new_city(&self, ctx: &mut CityContext) { }

    /// Called when a city is loaded from save
    fn on_city_loaded(&self, ctx: &mut CityContext) { }

    /// Called when the mod is about to be unloaded
    fn on_unload(&self) { }
}

#[derive(Debug, Clone)]
pub struct ModMetadata {
    pub id: String,           // "com.author.traffic-overhaul"
    pub name: String,         // "Traffic Overhaul"
    pub version: semver::Version,
    pub sdk_version: semver::VersionReq,  // "^1.0"
    pub dependencies: Vec<ModDependency>,
    pub conflicts: Vec<String>,           // mod IDs this conflicts with
    pub load_order: LoadOrder,
}

#[derive(Debug, Clone)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_req: semver::VersionReq,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOrder {
    /// Load before the specified mods
    Before(/* ... */),
    /// Load after the specified mods
    After(/* ... */),
    /// No preference
    Any,
}
```

**Key design principle: SDK types wrap internal types, never expose them.** The conversion happens in `mod-host`:

```rust
// crates/mod-host/src/bridge.rs

impl From<simulation::grid::ZoneType> for mod_sdk::ZoneKind {
    fn from(zt: simulation::grid::ZoneType) -> Self {
        match zt {
            ZoneType::ResidentialLow => ZoneKind::ResidentialLow,
            ZoneType::ResidentialHigh => ZoneKind::ResidentialHigh,
            ZoneType::CommercialLow => ZoneKind::CommercialLow,
            ZoneType::CommercialHigh => ZoneKind::CommercialHigh,
            ZoneType::Industrial => ZoneKind::Industrial,
            ZoneType::Office => ZoneKind::Office,
            ZoneType::None => panic!("Cannot convert None zone to SDK type"),
        }
    }
}

impl From<mod_sdk::ZoneKind> for simulation::grid::ZoneType {
    fn from(zk: mod_sdk::ZoneKind) -> Self {
        match zk {
            ZoneKind::ResidentialLow => ZoneType::ResidentialLow,
            // ... etc
            ZoneKind::Custom(_) => ZoneType::None, // handled by custom zone registry
        }
    }
}
```

**Why not just `pub use simulation::*`?** Because:

1. We currently have `RoadType` as a Rust enum. If we add a variant (`RoadType::Tram`), every mod doing `match road_type { ... }` breaks. The SDK's `RoadDefinition` struct is data-driven and can grow fields without breaking.

2. Internal types like `WorldGrid` expose raw cell arrays. If we change the grid from a flat `Vec<Cell>` to a chunked structure for performance, every mod directly accessing cells breaks. The SDK provides `grid.get_cell(pos)` which we can reimplement under the hood.

3. The simulation crate has 70+ modules (we counted in `lib.rs`). Exposing all of that as stable API is unmaintainable. The SDK cherry-picks the 20% that covers 95% of mod needs.

### 1.3 Hot-Reloading Native Plugins

For mod development, hot-reloading is essential. Waiting for a full game restart after every code change kills modder productivity. Bevy has experimental hot-reloading support, but for production use we need our own implementation.

**Dynamic library loading with `libloading`:**

```rust
// crates/mod-host/src/native_loader.rs

use libloading::{Library, Symbol};
use std::path::PathBuf;

/// A loaded native mod (shared library)
pub struct NativeModInstance {
    /// The loaded dynamic library -- must be kept alive
    _library: Library,
    /// The mod's plugin, extracted via the entry point
    plugin: Box<dyn MegacityMod>,
    /// Path to the .dylib/.so/.dll file
    path: PathBuf,
    /// Last modification time (for hot-reload detection)
    last_modified: std::time::SystemTime,
}

/// Entry point function signature that every native mod must export
type ModEntryFn = unsafe fn() -> Box<dyn MegacityMod>;

impl NativeModInstance {
    pub fn load(path: &Path) -> Result<Self, ModLoadError> {
        // SAFETY: We trust the mod author to export a valid function.
        // This is inherently unsafe -- native mods run with full privileges.
        let library = unsafe { Library::new(path) }
            .map_err(|e| ModLoadError::LibraryLoad(e.to_string()))?;

        let entry: Symbol<ModEntryFn> = unsafe { library.get(b"megacity_mod_entry") }
            .map_err(|e| ModLoadError::EntryPointMissing(e.to_string()))?;

        let plugin = unsafe { entry() };

        let metadata = std::fs::metadata(path)
            .map_err(|e| ModLoadError::Io(e))?;
        let last_modified = metadata.modified()
            .map_err(|e| ModLoadError::Io(e))?;

        Ok(Self {
            _library: library,
            plugin,
            path: path.to_owned(),
            last_modified,
        })
    }

    /// Check if the file has been modified since we loaded it
    pub fn needs_reload(&self) -> bool {
        std::fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .map(|t| t > self.last_modified)
            .unwrap_or(false)
    }
}
```

**The mod author's side:**

```rust
// In a mod's lib.rs

use megacity_mod_sdk::prelude::*;

pub struct MyTrafficMod;

impl Plugin for MyTrafficMod {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, my_custom_traffic_system);
    }
}

impl MegacityMod for MyTrafficMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "com.myname.traffic-mod".to_string(),
            name: "Better Traffic AI".to_string(),
            version: semver::Version::new(1, 2, 0),
            sdk_version: "^1.0".parse().unwrap(),
            dependencies: vec![],
            conflicts: vec![],
            load_order: LoadOrder::Any,
        }
    }
}

// Entry point that the mod host calls
#[no_mangle]
pub fn megacity_mod_entry() -> Box<dyn MegacityMod> {
    Box::new(MyTrafficMod)
}
```

**The mod Cargo.toml:**

```toml
[package]
name = "my-traffic-mod"
version = "1.2.0"

[lib]
crate-type = ["cdylib"]  # Produces .dylib/.so/.dll

[dependencies]
megacity-mod-sdk = "1.0"
```

**Hot-reload system (runs every few seconds in dev mode):**

```rust
// crates/mod-host/src/hot_reload.rs

pub fn check_mod_hot_reloads(
    mut mod_manager: ResMut<ModManager>,
    mut commands: Commands,
) {
    if !mod_manager.dev_mode {
        return;
    }

    for mod_instance in &mod_manager.native_mods {
        if mod_instance.needs_reload() {
            info!("Hot-reloading mod: {}", mod_instance.plugin.metadata().name);

            // 1. Call the mod's on_unload() to let it clean up
            mod_instance.plugin.on_unload();

            // 2. Remove all systems this mod registered
            //    (requires tracking which systems came from which mod)
            mod_manager.unregister_mod_systems(&mod_instance.plugin.metadata().id);

            // 3. Drop the old library (releases the file handle)
            //    This is where it gets tricky on Windows (file locking)
            let path = mod_instance.path.clone();

            // 4. Copy the new file to a temp location (avoids Windows lock issues)
            let temp_path = path.with_extension("hot.dll");
            std::fs::copy(&path, &temp_path).ok();

            // 5. Load the new version
            match NativeModInstance::load(&temp_path) {
                Ok(new_instance) => {
                    // 6. Re-run build() with the new plugin
                    // This is the hard part -- Bevy doesn't natively support
                    // removing systems. We need a custom schedule approach.
                    *mod_instance = new_instance;
                }
                Err(e) => {
                    error!("Failed to hot-reload mod: {}", e);
                }
            }
        }
    }
}
```

**The hard part: Bevy system removal.** Bevy's `Schedule` doesn't support removing systems after initialization. There are two solutions:

1. **Mod-specific sub-schedules**: Each mod's systems run in a sub-schedule that can be rebuilt entirely:

```rust
// Each mod gets its own schedule
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModSchedule(pub String);  // mod ID

// In the main FixedUpdate, we run all mod schedules
fn run_mod_schedules(world: &mut World) {
    let mod_ids: Vec<String> = world.resource::<ModManager>()
        .loaded_mods()
        .map(|m| m.id.clone())
        .collect();

    for id in mod_ids {
        world.run_schedule(ModSchedule(id));
    }
}
```

2. **Run-condition gates**: Systems are never removed, but gated behind a run condition that the mod manager controls:

```rust
fn mod_is_active(mod_id: &str) -> impl Fn(Res<ModManager>) -> bool {
    let id = mod_id.to_string();
    move |manager: Res<ModManager>| manager.is_mod_active(&id)
}

// When registering a mod's systems:
app.add_systems(
    FixedUpdate,
    my_system.run_if(mod_is_active("com.author.my-mod"))
);
```

**Recommendation**: Use approach 2 (run conditions) for production, approach 1 (sub-schedules) for dev-mode hot-reload where the performance overhead of rebuilding schedules is acceptable.

### 1.4 Mod Load Ordering and Dependency Resolution

Mod ordering matters. A traffic mod needs the base traffic system to exist before it can modify behavior. A building pack needs the zone registry before it can register custom zones. We need a robust dependency resolution system.

**The manifest file (`mod.toml`):**

```toml
[mod]
id = "com.author.traffic-overhaul"
name = "Traffic Overhaul"
version = "2.1.0"
sdk_version = "^1.0"
description = "Complete traffic AI replacement with lane management"
authors = ["Author Name <author@example.com>"]
license = "MIT"

[dependencies]
# Required: this mod needs the road extensions mod
"com.other.road-extensions" = ">=1.0, <3.0"

[optional-dependencies]
# Optional: enhanced features if this mod is also loaded
"com.other.public-transit" = "^2.0"

[conflicts]
# Cannot coexist with this mod
incompatible = ["com.another.old-traffic-mod"]

[load-order]
# Must load after these (they set up data we depend on)
after = ["com.other.road-extensions"]
# Must load before these (they expect our traffic API)
before = ["com.someone.traffic-analytics"]
```

**Topological sort for load ordering:**

```rust
// crates/mod-host/src/dependency.rs

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct ModGraph {
    /// mod_id -> (metadata, set of mod_ids this depends on)
    nodes: HashMap<String, (ModMetadata, HashSet<String>)>,
}

impl ModGraph {
    /// Perform topological sort. Returns ordered list or cycle error.
    pub fn resolve_load_order(&self) -> Result<Vec<String>, DependencyError> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize in-degrees
        for (id, _) in &self.nodes {
            in_degree.entry(id.as_str()).or_insert(0);
        }

        // Build edges: dependency -> dependent
        for (id, (meta, deps)) in &self.nodes {
            for dep_id in deps {
                if !self.nodes.contains_key(dep_id) {
                    // Check if it's optional
                    let is_optional = meta.dependencies.iter()
                        .any(|d| d.mod_id == *dep_id && d.optional);
                    if !is_optional {
                        return Err(DependencyError::MissingDependency {
                            mod_id: id.clone(),
                            dependency: dep_id.clone(),
                        });
                    }
                    continue;
                }
                dependents.entry(dep_id.as_str())
                    .or_default()
                    .push(id.as_str());
                *in_degree.entry(id.as_str()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order: Vec<String> = Vec::new();

        while let Some(node) = queue.pop_front() {
            order.push(node.to_string());

            if let Some(deps) = dependents.get(node) {
                for &dep in deps {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            // Find cycle
            let in_cycle: Vec<String> = self.nodes.keys()
                .filter(|id| !order.contains(id))
                .cloned()
                .collect();
            return Err(DependencyError::CyclicDependency(in_cycle));
        }

        Ok(order)
    }

    /// Check version compatibility for all dependencies
    pub fn check_version_constraints(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        for (id, (meta, _)) in &self.nodes {
            for dep in &meta.dependencies {
                if let Some((dep_meta, _)) = self.nodes.get(&dep.mod_id) {
                    if !dep.version_req.matches(&dep_meta.version) {
                        conflicts.push(VersionConflict {
                            mod_id: id.clone(),
                            dependency: dep.mod_id.clone(),
                            required: dep.version_req.clone(),
                            found: dep_meta.version.clone(),
                        });
                    }
                }
            }
        }

        conflicts
    }

    /// Check for declared conflicts between loaded mods
    pub fn check_conflicts(&self) -> Vec<ModConflict> {
        let mut conflicts = Vec::new();

        for (id, (meta, _)) in &self.nodes {
            for conflict_id in &meta.conflicts {
                if self.nodes.contains_key(conflict_id) {
                    conflicts.push(ModConflict {
                        mod_a: id.clone(),
                        mod_b: conflict_id.clone(),
                        reason: format!("{} declares conflict with {}", id, conflict_id),
                    });
                }
            }
        }

        conflicts
    }
}

#[derive(Debug)]
pub enum DependencyError {
    MissingDependency { mod_id: String, dependency: String },
    CyclicDependency(Vec<String>),
    VersionMismatch(Vec<VersionConflict>),
    DeclaredConflict(Vec<ModConflict>),
}

#[derive(Debug)]
pub struct VersionConflict {
    pub mod_id: String,
    pub dependency: String,
    pub required: semver::VersionReq,
    pub found: semver::Version,
}

#[derive(Debug)]
pub struct ModConflict {
    pub mod_a: String,
    pub mod_b: String,
    pub reason: String,
}
```

**Handling soft dependencies and load-order hints:**

Beyond hard dependencies (mod A requires mod B), we need soft ordering constraints. The `after` and `before` fields in `mod.toml` express preferences that don't create dependencies. The topological sort handles these by adding edges with a lower priority -- if the referenced mod isn't loaded, the hint is ignored.

```rust
impl ModGraph {
    pub fn add_load_order_hints(&mut self, manifests: &[ModManifest]) {
        for manifest in manifests {
            let id = &manifest.mod_section.id;

            // "after" hints: add edge from target to us
            for after_id in &manifest.load_order.after {
                if self.nodes.contains_key(after_id) {
                    self.nodes.get_mut(id).unwrap().1.insert(after_id.clone());
                }
                // If the target mod isn't loaded, silently ignore
            }

            // "before" hints: add edge from us to target
            for before_id in &manifest.load_order.before {
                if let Some(node) = self.nodes.get_mut(before_id) {
                    node.1.insert(id.clone());
                }
            }
        }
    }
}
```

### 1.5 Mod Conflict Detection and Resolution

Conflicts fall into several categories, each requiring different detection strategies.

**Category 1: System ordering conflicts**

Two mods both want to run a system `after(traffic::update_traffic_density)`. This is fine -- Bevy handles parallel execution. But if mod A writes to `TrafficGrid` and mod B reads from it in the same schedule phase, the order matters.

Detection strategy: Track which resources each mod's systems access (via Bevy's system parameter introspection) and flag potential data races:

```rust
// crates/mod-host/src/conflict_detector.rs

use bevy::ecs::system::SystemMeta;

pub struct SystemAccessTracker {
    /// mod_id -> list of (system_name, reads, writes)
    mod_systems: HashMap<String, Vec<SystemAccess>>,
}

#[derive(Debug)]
pub struct SystemAccess {
    pub system_name: String,
    pub reads: HashSet<TypeId>,    // Resources/Components read
    pub writes: HashSet<TypeId>,   // Resources/Components written
    pub schedule: ScheduleLabel,
}

impl SystemAccessTracker {
    /// Find pairs of systems from different mods that write to the same resource
    /// in the same schedule without explicit ordering
    pub fn detect_write_conflicts(&self) -> Vec<WriteConflict> {
        let mut conflicts = Vec::new();

        let mod_ids: Vec<&String> = self.mod_systems.keys().collect();
        for i in 0..mod_ids.len() {
            for j in (i+1)..mod_ids.len() {
                let systems_a = &self.mod_systems[mod_ids[i]];
                let systems_b = &self.mod_systems[mod_ids[j]];

                for sa in systems_a {
                    for sb in systems_b {
                        if sa.schedule != sb.schedule {
                            continue;
                        }
                        // Write-write conflict
                        let ww: HashSet<_> = sa.writes.intersection(&sb.writes).collect();
                        if !ww.is_empty() {
                            conflicts.push(WriteConflict {
                                mod_a: mod_ids[i].clone(),
                                system_a: sa.system_name.clone(),
                                mod_b: mod_ids[j].clone(),
                                system_b: sb.system_name.clone(),
                                conflicting_types: ww.into_iter().cloned().collect(),
                            });
                        }
                        // Read-write conflict (one reads, other writes)
                        let rw: HashSet<_> = sa.reads.intersection(&sb.writes).collect();
                        let wr: HashSet<_> = sa.writes.intersection(&sb.reads).collect();
                        if !rw.is_empty() || !wr.is_empty() {
                            conflicts.push(WriteConflict {
                                mod_a: mod_ids[i].clone(),
                                system_a: sa.system_name.clone(),
                                mod_b: mod_ids[j].clone(),
                                system_b: sb.system_name.clone(),
                                conflicting_types: rw.into_iter().chain(wr.into_iter())
                                    .cloned().collect(),
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }
}
```

**Category 2: Resource overwrites**

Two mods both try to `insert_resource::<TrafficConfig>()`. The last one wins, silently overwriting the first. Detection:

```rust
pub fn detect_resource_overwrites(
    mod_resources: &HashMap<String, Vec<TypeId>>, // mod_id -> resources inserted
) -> Vec<ResourceOverwrite> {
    let mut resource_owners: HashMap<TypeId, Vec<String>> = HashMap::new();

    for (mod_id, resources) in mod_resources {
        for &type_id in resources {
            resource_owners.entry(type_id)
                .or_default()
                .push(mod_id.clone());
        }
    }

    resource_owners.into_iter()
        .filter(|(_, owners)| owners.len() > 1)
        .map(|(type_id, owners)| ResourceOverwrite { type_id, owners })
        .collect()
}
```

**Category 3: Data definition conflicts**

Two mods define a building with the same ID, or a road type with the same name. The data registry tracks origins:

```rust
pub struct DataRegistry {
    /// type_name -> (entry_id -> (data, source_mod_id))
    entries: HashMap<String, HashMap<String, (Box<dyn Any>, String)>>,
}

impl DataRegistry {
    pub fn register<T: 'static>(
        &mut self,
        category: &str,
        id: &str,
        data: T,
        mod_id: &str,
    ) -> Result<(), DataConflict> {
        let category_map = self.entries
            .entry(category.to_string())
            .or_default();

        if let Some((_, existing_mod)) = category_map.get(id) {
            return Err(DataConflict {
                category: category.to_string(),
                id: id.to_string(),
                existing_mod: existing_mod.clone(),
                new_mod: mod_id.to_string(),
            });
        }

        category_map.insert(id.to_string(), (Box::new(data), mod_id.to_string()));
        Ok(())
    }
}
```

**Resolution strategies** exposed to users via the mod manager UI:

1. **Load order priority**: Later mods override earlier mods (user-configurable order)
2. **Merge**: For compatible changes (e.g., two mods both add buildings -- no conflict, just merge)
3. **Patch stacking**: Mods that modify existing data apply patches in order (like git commits)
4. **User choice**: For irreconcilable conflicts, present the user with a choice

---

## 2. Scripting Language Integration

Native Rust plugins are powerful but have a high barrier to entry: modders need to install the Rust toolchain, understand Bevy's ECS, compile against the exact same Bevy version, and produce platform-specific binaries. For the 90% of modders who want to tweak building stats, add custom events, or write simple AI behaviors, a scripting language is essential.

### 2.1 Lua via mlua

**Heritage and ecosystem.** Lua is the gold standard for game modding scripting. World of Warcraft (2004), Garry's Mod (2006), Factorio (2012), and Roblox all use Lua. Modders know it, documentation is abundant, and the language is deliberately small (the entire reference manual fits on a few pages).

**Rust integration via `mlua`:**

`mlua` (successor to `rlua`) provides safe, ergonomic Lua 5.4 bindings for Rust. Key features:

- **Sandboxing**: Can restrict the Lua standard library (remove `os`, `io`, `debug`, `loadfile`)
- **Async support**: Lua coroutines map to Rust async tasks
- **Serde integration**: Rust structs with `#[derive(Serialize, Deserialize)]` convert to/from Lua tables automatically
- **Error propagation**: Lua errors become Rust `Result` types

```rust
// crates/mod-host/src/lua_runtime.rs

use mlua::prelude::*;

pub struct LuaModRuntime {
    lua: Lua,
    mod_id: String,
    /// Cached references to mod-defined callback functions
    callbacks: LuaCallbacks,
}

struct LuaCallbacks {
    on_tick: Option<LuaFunction>,
    on_building_placed: Option<LuaFunction>,
    on_citizen_spawned: Option<LuaFunction>,
    on_road_built: Option<LuaFunction>,
    on_policy_changed: Option<LuaFunction>,
    on_disaster: Option<LuaFunction>,
    on_milestone: Option<LuaFunction>,
}

impl LuaModRuntime {
    pub fn new(mod_id: &str, script_source: &str) -> Result<Self, LuaError> {
        let lua = Lua::new();

        // === Sandbox: remove dangerous standard library modules ===
        lua.globals().set("os", LuaNil)?;
        lua.globals().set("io", LuaNil)?;
        lua.globals().set("debug", LuaNil)?;
        lua.globals().set("loadfile", LuaNil)?;
        lua.globals().set("dofile", LuaNil)?;
        lua.globals().set("require", LuaNil)?;  // We provide our own import system

        // === Register Megacity API functions ===
        Self::register_city_api(&lua)?;
        Self::register_building_api(&lua)?;
        Self::register_traffic_api(&lua)?;
        Self::register_citizen_api(&lua)?;
        Self::register_ui_api(&lua)?;
        Self::register_event_api(&lua)?;

        // === Load the mod script ===
        lua.load(script_source).exec()?;

        // === Extract callback functions ===
        let callbacks = LuaCallbacks {
            on_tick: lua.globals().get::<Option<LuaFunction>>("on_tick")?,
            on_building_placed: lua.globals().get("on_building_placed")?,
            on_citizen_spawned: lua.globals().get("on_citizen_spawned")?,
            on_road_built: lua.globals().get("on_road_built")?,
            on_policy_changed: lua.globals().get("on_policy_changed")?,
            on_disaster: lua.globals().get("on_disaster")?,
            on_milestone: lua.globals().get("on_milestone")?,
        };

        Ok(Self {
            lua,
            mod_id: mod_id.to_string(),
            callbacks,
        })
    }

    fn register_city_api(lua: &Lua) -> Result<(), LuaError> {
        let city = lua.create_table()?;

        // city.get_population() -> number
        city.set("get_population", lua.create_function(|_, ()| {
            // This will be filled in by the bridge layer with actual game data
            Ok(0u32)  // placeholder
        })?)?;

        // city.get_treasury() -> number
        city.set("get_treasury", lua.create_function(|_, ()| {
            Ok(0.0f64)
        })?)?;

        // city.get_happiness() -> number (0-100)
        city.set("get_happiness", lua.create_function(|_, ()| {
            Ok(50.0f32)
        })?)?;

        // city.get_traffic_flow() -> number (0-1, average congestion)
        city.set("get_traffic_flow", lua.create_function(|_, ()| {
            Ok(0.5f32)
        })?)?;

        // city.get_day() -> number
        city.set("get_day", lua.create_function(|_, ()| {
            Ok(1u32)
        })?)?;

        // city.get_hour() -> number (0.0 - 24.0)
        city.set("get_hour", lua.create_function(|_, ()| {
            Ok(12.0f32)
        })?)?;

        lua.globals().set("city", city)?;
        Ok(())
    }

    fn register_building_api(lua: &Lua) -> Result<(), LuaError> {
        let buildings = lua.create_table()?;

        // buildings.get_at(x, y) -> table or nil
        buildings.set("get_at", lua.create_function(|_, (x, y): (usize, usize)| {
            // Returns building info as a Lua table
            Ok(LuaNil) // placeholder
        })?)?;

        // buildings.get_all_of_zone(zone_name) -> table of buildings
        buildings.set("get_all_of_zone", lua.create_function(|_, zone: String| {
            Ok(LuaNil)
        })?)?;

        // buildings.set_capacity(x, y, new_capacity) -> bool
        buildings.set("set_capacity", lua.create_function(|_, (x, y, cap): (usize, usize, u32)| {
            Ok(false)
        })?)?;

        // buildings.spawn(zone, x, y, level) -> entity_id or nil
        buildings.set("spawn", lua.create_function(|_, (zone, x, y, level): (String, usize, usize, u8)| {
            Ok(LuaNil)
        })?)?;

        lua.globals().set("buildings", buildings)?;
        Ok(())
    }

    fn register_traffic_api(lua: &Lua) -> Result<(), LuaError> {
        let traffic = lua.create_table()?;

        // traffic.get_density(x, y) -> number (0-65535)
        traffic.set("get_density", lua.create_function(|_, (x, y): (usize, usize)| {
            Ok(0u16)
        })?)?;

        // traffic.get_congestion(x, y) -> number (0.0-1.0)
        traffic.set("get_congestion", lua.create_function(|_, (x, y): (usize, usize)| {
            Ok(0.0f32)
        })?)?;

        // traffic.set_speed_limit(x, y, speed) -> bool
        traffic.set("set_speed_limit", lua.create_function(|_, (x, y, speed): (usize, usize, f32)| {
            Ok(false)
        })?)?;

        lua.globals().set("traffic", traffic)?;
        Ok(())
    }

    fn register_citizen_api(lua: &Lua) -> Result<(), LuaError> {
        let citizens = lua.create_table()?;

        // citizens.count() -> number
        citizens.set("count", lua.create_function(|_, ()| {
            Ok(0u32)
        })?)?;

        // citizens.get_average_happiness() -> number
        citizens.set("get_average_happiness", lua.create_function(|_, ()| {
            Ok(50.0f32)
        })?)?;

        lua.globals().set("citizens", citizens)?;
        Ok(())
    }

    fn register_ui_api(lua: &Lua) -> Result<(), LuaError> {
        let ui = lua.create_table()?;

        // ui.show_notification(title, message, icon?)
        ui.set("show_notification", lua.create_function(|_, (title, msg): (String, String)| {
            Ok(())
        })?)?;

        // ui.add_toolbar_button(id, label, icon, callback_name)
        ui.set("add_toolbar_button", lua.create_function(|_, (id, label, icon, cb): (String, String, String, String)| {
            Ok(())
        })?)?;

        lua.globals().set("ui", ui)?;
        Ok(())
    }

    fn register_event_api(lua: &Lua) -> Result<(), LuaError> {
        let events = lua.create_table()?;

        // events.on(event_name, callback_name)
        events.set("on", lua.create_function(|_, (event, callback): (String, String)| {
            Ok(())
        })?)?;

        // events.emit(event_name, data_table)
        events.set("emit", lua.create_function(|_, (event, data): (String, LuaTable)| {
            Ok(())
        })?)?;

        lua.globals().set("events", events)?;
        Ok(())
    }
}
```

**What a Lua mod looks like from the modder's perspective:**

```lua
-- my_tourism_mod/main.lua

-- Metadata
MOD_INFO = {
    id = "com.author.tourism-boost",
    name = "Tourism Boost",
    version = "1.0.0",
    description = "Adds seasonal tourism events and beach bonuses",
}

-- Called every simulation tick
function on_tick()
    local hour = city.get_hour()
    local day = city.get_day()

    -- Summer tourism boost (days 90-270 = April through September)
    if day >= 90 and day <= 270 then
        -- Increase commercial demand near coast
        for x = 55, 70 do
            for y = 30, 180 do
                local bldg = buildings.get_at(x, y)
                if bldg and bldg.zone == "CommercialHigh" then
                    -- 20% capacity bonus during summer
                    local base_cap = bldg.capacity
                    local boosted = math.floor(base_cap * 1.2)
                    buildings.set_capacity(x, y, boosted)
                end
            end
        end
    end

    -- Beach party event on weekends during summer
    if day >= 90 and day <= 270 and day % 7 <= 1 and hour >= 16 then
        ui.show_notification("Beach Party!", "Citizens are heading to the beach!")
    end
end

-- React to building placement
function on_building_placed(building)
    if building.zone == "CommercialHigh" and building.x < 70 then
        ui.show_notification("Beachfront Business",
            "Great location! Tourism bonus applies here.")
    end
end
```

**Performance tradeoffs:**

| Operation | Lua (mlua) | Native Rust |
|-----------|-----------|-------------|
| Function call overhead | ~100ns per call | ~0ns (inlined) |
| Table access | ~50ns | ~5ns (struct field) |
| Iteration over 1000 entities | ~200us | ~10us |
| Complex pathfinding (1000 nodes) | ~5ms | ~200us |
| String manipulation | ~500ns per op | ~50ns per op |

**Verdict for Lua**: Excellent for event handlers, policy tweaks, and UI customization. Not suitable for per-tick systems that touch every entity (traffic AI, pathfinding). The ~20x overhead is fine when you are calling `on_building_placed` a few times per second, but deadly when iterating 100K citizens per tick.

### 2.2 WASM via wasmtime

WebAssembly (WASM) is the modern alternative to Lua for game modding. Figma, Fastly, and Cloudflare all use WASM for plugin sandboxing. For games, it offers a compelling combination of safety, performance, and language flexibility.

**Key advantages:**

1. **Sandboxed by design**: WASM modules execute in a linear memory space with no access to the host filesystem, network, or OS. This is not bolted on (like Lua sandboxing) -- it is fundamental to the specification.

2. **Near-native performance**: WASM JIT compilers (Cranelift in wasmtime) produce machine code within 10-30% of native speed. For hot loops iterating over entities, this is dramatically better than Lua.

3. **Multi-language support**: Modders can write in Rust (compiles to WASM natively), C/C++ (via Emscripten/wasi-sdk), AssemblyScript (TypeScript-like, compiles to WASM), Go (via TinyGo), or even Python (via wasm-enabled interpreters).

4. **Deterministic execution**: WASM execution is deterministic (no undefined behavior, no uninitialized memory). This matters for multiplayer synchronization if we ever add that.

**Rust integration via `wasmtime`:**

```rust
// crates/mod-host/src/wasm_runtime.rs

use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

pub struct WasmModRuntime {
    engine: Engine,
    store: Store<ModState>,
    instance: Instance,
    mod_id: String,
}

/// State accessible to the WASM module via host functions
struct ModState {
    /// Queued commands from the WASM module (applied after execution)
    command_queue: Vec<ModCommand>,
    /// Read-only snapshot of game state (refreshed each tick)
    game_snapshot: GameSnapshot,
    /// Memory/CPU limits
    fuel_remaining: u64,
}

/// Snapshot of game state that WASM modules can read
#[derive(Clone)]
pub struct GameSnapshot {
    pub population: u32,
    pub treasury: f64,
    pub day: u32,
    pub hour: f32,
    pub happiness: f32,
    pub traffic_congestion: f32,
    // Grid data as flat arrays (copied from Resources)
    pub traffic_density: Vec<u16>,
    pub zone_types: Vec<u8>,
    pub building_levels: Vec<u8>,
    pub grid_width: usize,
    pub grid_height: usize,
}

/// Commands that WASM modules can enqueue
#[derive(Debug)]
pub enum ModCommand {
    SetBuildingCapacity { x: usize, y: usize, capacity: u32 },
    SpawnBuilding { zone: u8, x: usize, y: usize, level: u8 },
    ShowNotification { title: String, message: String },
    SetSpeedLimit { x: usize, y: usize, speed: f32 },
    EmitEvent { name: String, data: Vec<u8> },
}

impl WasmModRuntime {
    pub fn new(
        mod_id: &str,
        wasm_bytes: &[u8],
        memory_limit_pages: u64,  // 1 page = 64KB
        fuel_per_tick: u64,       // CPU budget per tick
    ) -> Result<Self, WasmLoadError> {
        let mut config = Config::new();
        config.consume_fuel(true);          // Enable fuel metering
        config.epoch_interruption(true);    // Enable timeout interruption
        config.wasm_memory64(false);        // 32-bit memory only
        config.max_wasm_stack(1024 * 1024); // 1MB stack limit

        let engine = Engine::new(&config)
            .map_err(WasmLoadError::Engine)?;

        let module = Module::new(&engine, wasm_bytes)
            .map_err(WasmLoadError::Compilation)?;

        let mut store = Store::new(&engine, ModState {
            command_queue: Vec::new(),
            game_snapshot: GameSnapshot::default(),
            fuel_remaining: fuel_per_tick,
        });

        // Set memory limits
        store.limiter(|state| {
            wasmtime::ResourceLimiter {
                memory_growing: move |current, desired, maximum| {
                    desired <= memory_limit_pages * 65536
                },
                table_growing: |_, _, _| true,
                ..Default::default()
            }
        });

        // Add initial fuel
        store.set_fuel(fuel_per_tick)
            .map_err(WasmLoadError::Fuel)?;

        // === Register host functions (imports) ===
        let mut linker = Linker::new(&engine);

        // city_get_population() -> i32
        linker.func_wrap("megacity", "city_get_population",
            |caller: Caller<'_, ModState>| -> i32 {
                caller.data().game_snapshot.population as i32
            }
        )?;

        // city_get_treasury() -> f64
        linker.func_wrap("megacity", "city_get_treasury",
            |caller: Caller<'_, ModState>| -> f64 {
                caller.data().game_snapshot.treasury
            }
        )?;

        // city_get_hour() -> f32
        linker.func_wrap("megacity", "city_get_hour",
            |caller: Caller<'_, ModState>| -> f32 {
                caller.data().game_snapshot.hour
            }
        )?;

        // traffic_get_density(x: i32, y: i32) -> i32
        linker.func_wrap("megacity", "traffic_get_density",
            |caller: Caller<'_, ModState>, x: i32, y: i32| -> i32 {
                let snap = &caller.data().game_snapshot;
                let idx = y as usize * snap.grid_width + x as usize;
                if idx < snap.traffic_density.len() {
                    snap.traffic_density[idx] as i32
                } else {
                    0
                }
            }
        )?;

        // building_set_capacity(x: i32, y: i32, capacity: i32)
        linker.func_wrap("megacity", "building_set_capacity",
            |mut caller: Caller<'_, ModState>, x: i32, y: i32, cap: i32| {
                caller.data_mut().command_queue.push(
                    ModCommand::SetBuildingCapacity {
                        x: x as usize,
                        y: y as usize,
                        capacity: cap as u32,
                    }
                );
            }
        )?;

        // show_notification(title_ptr: i32, title_len: i32, msg_ptr: i32, msg_len: i32)
        linker.func_wrap("megacity", "show_notification",
            |mut caller: Caller<'_, ModState>,
             title_ptr: i32, title_len: i32,
             msg_ptr: i32, msg_len: i32| {
                let memory = caller.get_export("memory")
                    .and_then(|e| e.into_memory())
                    .expect("WASM module must export memory");

                let title = read_wasm_string(&memory, &caller, title_ptr, title_len);
                let msg = read_wasm_string(&memory, &caller, msg_ptr, msg_len);

                caller.data_mut().command_queue.push(
                    ModCommand::ShowNotification { title, message: msg }
                );
            }
        )?;

        let instance = linker.instantiate(&mut store, &module)
            .map_err(WasmLoadError::Instantiation)?;

        Ok(Self {
            engine,
            store,
            instance,
            mod_id: mod_id.to_string(),
        })
    }

    /// Call the mod's on_tick export and return queued commands
    pub fn tick(&mut self, snapshot: &GameSnapshot) -> Result<Vec<ModCommand>, WasmError> {
        // Refresh game state snapshot
        self.store.data_mut().game_snapshot = snapshot.clone();

        // Refuel
        self.store.set_fuel(self.store.data().fuel_remaining)?;

        // Clear command queue
        self.store.data_mut().command_queue.clear();

        // Call the on_tick export
        let on_tick = self.instance.get_typed_func::<(), ()>(&mut self.store, "on_tick")
            .map_err(WasmError::MissingExport)?;

        match on_tick.call(&mut self.store, ()) {
            Ok(()) => {},
            Err(e) if e.downcast_ref::<wasmtime::Trap>()
                .map_or(false, |t| *t == Trap::OutOfFuel) => {
                warn!("WASM mod {} ran out of fuel (CPU budget exceeded)", self.mod_id);
                return Err(WasmError::OutOfFuel);
            }
            Err(e) => return Err(WasmError::Execution(e)),
        }

        Ok(std::mem::take(&mut self.store.data_mut().command_queue))
    }
}

fn read_wasm_string(
    memory: &Memory,
    caller: &Caller<'_, ModState>,
    ptr: i32,
    len: i32,
) -> String {
    let data = memory.data(caller);
    let start = ptr as usize;
    let end = start + len as usize;
    if end <= data.len() {
        String::from_utf8_lossy(&data[start..end]).to_string()
    } else {
        String::new()
    }
}
```

**What a WASM mod looks like (Rust source that compiles to WASM):**

```rust
// A mod written in Rust, compiled with:
// cargo build --target wasm32-wasi --release

#[link(wasm_import_module = "megacity")]
extern "C" {
    fn city_get_population() -> i32;
    fn city_get_hour() -> f32;
    fn traffic_get_density(x: i32, y: i32) -> i32;
    fn building_set_capacity(x: i32, y: i32, capacity: i32);
    fn show_notification(title_ptr: *const u8, title_len: i32,
                         msg_ptr: *const u8, msg_len: i32);
}

#[no_mangle]
pub extern "C" fn on_tick() {
    unsafe {
        let hour = city_get_hour();
        let pop = city_get_population();

        // Rush hour traffic management: increase road capacity during peak hours
        if (hour >= 7.0 && hour <= 9.0) || (hour >= 17.0 && hour <= 19.0) {
            for x in 60..190 {
                for y in 30..250 {
                    let density = traffic_get_density(x, y);
                    if density > 15 {
                        // This road is congested during rush hour
                        // Signal to the game that it needs attention
                    }
                }
            }
        }
    }
}
```

**Or in AssemblyScript (TypeScript-like, more accessible to modders):**

```typescript
// A mod written in AssemblyScript
// Compiled with: asc main.ts -o main.wasm

@external("megacity", "city_get_hour")
declare function city_get_hour(): f32;

@external("megacity", "traffic_get_density")
declare function traffic_get_density(x: i32, y: i32): i32;

@external("megacity", "show_notification")
declare function show_notification(
    titlePtr: usize, titleLen: i32,
    msgPtr: usize, msgLen: i32
): void;

export function on_tick(): void {
    const hour = city_get_hour();
    if (hour >= 12.0 && hour < 13.0) {
        // Lunch rush notification
        const title = "Lunch Rush";
        const msg = "Commercial areas are busy!";
        show_notification(
            changetype<usize>(title), title.length * 2,
            changetype<usize>(msg), msg.length * 2
        );
    }
}
```

**Performance comparison with Lua:**

| Operation | WASM (wasmtime) | Lua (mlua) | Native Rust |
|-----------|----------------|-----------|-------------|
| Function call overhead | ~20ns | ~100ns | ~0ns |
| Array iteration (1000 elements) | ~15us | ~200us | ~5us |
| Pathfinding (1000 nodes) | ~250us | ~5ms | ~200us |
| String manipulation | ~100ns | ~500ns | ~50ns |
| Memory allocation | ~30ns | ~100ns | ~15ns |

WASM is 5-10x faster than Lua and within 2-3x of native Rust for computational workloads. The gap to native is mostly JIT warmup and the cost of crossing the host/guest boundary.

### 2.3 Rhai: Rust-Native Scripting

Rhai is a scripting language designed specifically for embedding in Rust applications. It has no FFI layer -- scripts run directly within the Rust process using Rust's own type system.

**Advantages:**
- Zero FFI overhead: Rhai values are Rust values
- Familiar syntax: looks like a mix of Rust and JavaScript
- Built-in operator overloading, custom types, closures
- Thread-safe by design (Send + Sync)
- Built-in script optimization (constant folding, dead code elimination)

**Disadvantages:**
- Small ecosystem: very few existing Rhai scripts in the wild
- No modding heritage: modders won't know it already
- Performance: slower than WASM for computational tasks (interpreted, not JIT-compiled)
- Limited debugging tools compared to Lua (which has ZeroBrane Studio, etc.)

```rust
// crates/mod-host/src/rhai_runtime.rs

use rhai::{Engine, Scope, AST, Dynamic, Map};

pub struct RhaiModRuntime {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
    mod_id: String,
}

impl RhaiModRuntime {
    pub fn new(mod_id: &str, script_source: &str) -> Result<Self, RhaiError> {
        let mut engine = Engine::new();

        // Sandbox: disable dangerous operations
        engine.set_max_call_levels(64);        // prevent stack overflow
        engine.set_max_operations(1_000_000);  // CPU limit per call
        engine.set_max_string_size(10_000);    // prevent memory bombs
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(1_000);

        // Register Megacity API
        engine.register_fn("get_population", || -> i64 { 0 });
        engine.register_fn("get_treasury", || -> f64 { 0.0 });
        engine.register_fn("get_hour", || -> f64 { 12.0 });
        engine.register_fn("get_traffic_density", |x: i64, y: i64| -> i64 { 0 });
        engine.register_fn("set_building_capacity", |x: i64, y: i64, cap: i64| {});
        engine.register_fn("show_notification", |title: &str, msg: &str| {});

        // Register custom types
        engine.register_type_with_name::<BuildingInfo>("Building")
            .register_get("zone", |b: &mut BuildingInfo| b.zone.clone())
            .register_get("level", |b: &mut BuildingInfo| b.level as i64)
            .register_get("capacity", |b: &mut BuildingInfo| b.capacity as i64)
            .register_get("x", |b: &mut BuildingInfo| b.x as i64)
            .register_get("y", |b: &mut BuildingInfo| b.y as i64);

        // Compile script (catches syntax errors early)
        let ast = engine.compile(script_source)?;

        let scope = Scope::new();

        Ok(Self {
            engine,
            ast,
            scope,
            mod_id: mod_id.to_string(),
        })
    }

    pub fn call_on_tick(&mut self) -> Result<(), RhaiError> {
        self.engine.call_fn::<()>(&mut self.scope, &self.ast, "on_tick", ())?;
        Ok(())
    }
}
```

**What a Rhai mod looks like:**

```rhai
// my_mod/main.rhai

fn on_tick() {
    let hour = get_hour();
    let pop = get_population();

    if hour >= 7.0 && hour <= 9.0 {
        // Morning rush hour
        show_notification("Rush Hour", "Traffic is building up!");
    }

    // Dynamic tax adjustment based on population
    if pop > 50000 {
        let new_rate = 0.12;
        set_tax_rate(new_rate);
    }
}

fn on_building_placed(building) {
    if building.zone == "Industrial" && building.x < 70 {
        show_notification("Warning",
            "Industrial buildings near the coast may cause water pollution!");
    }
}
```

### 2.4 Comparison Matrix and Recommendation

| Criterion | Lua (mlua) | WASM (wasmtime) | Rhai | Native Plugin |
|-----------|-----------|-----------------|------|---------------|
| **Modder familiarity** | Excellent (WoW, Factorio) | Low (new paradigm) | Very low | Medium (Rust devs) |
| **Learning curve** | Very low | Medium (toolchain) | Low | High (Bevy ECS) |
| **Performance** | ~20x slower than native | ~2-3x slower | ~30x slower | Native |
| **Sandboxing** | Good (library removal) | Excellent (by design) | Good (limits) | None |
| **Multi-language** | Lua only | Rust/C/C++/AS/Go | Rhai only | Rust only |
| **Debugging** | Good (print, ZeroBrane) | Moderate (WASM tools) | Basic (print) | Excellent (rust-gdb) |
| **Hot-reload** | Instant (text) | Fast (recompile WASM) | Instant (text) | Slow (recompile dll) |
| **Ecosystem size** | Huge | Growing | Tiny | Large (Rust crates) |
| **Memory overhead** | ~2MB per VM | ~5MB per instance | ~1MB per engine | 0 (shared process) |
| **Suitable for** | Events, UI, tweaks | AI, per-entity logic | Simple tweaks | System overhauls |

**Recommendation: Dual-track approach -- Lua primary, WASM for power users.**

The rationale:

1. **Lua first** because modder adoption is everything. If modders cannot start writing useful mods within 30 minutes of opening the documentation, the modding ecosystem will not take off. Lua achieves this. Every experienced gamer has encountered Lua modding; many have written WoW addons or Factorio mods. The syntax is trivial. The tooling exists.

2. **WASM second** because some mods genuinely need near-native performance. A TM:PE-equivalent traffic AI replacement needs to iterate over thousands of road segments every tick. Lua cannot do this. WASM can, and it provides the sandboxing guarantees that native plugins cannot.

3. **Skip Rhai** because it offers no unique advantage. It is slower than WASM, less familiar than Lua, and its "no FFI overhead" advantage is marginal compared to the ecosystem disadvantage.

4. **Native plugins as escape hatch** for the handful of mods that need deep engine access (custom renderers, new ECS systems, etc.). These are opt-in and come with warnings about stability.

**What modders actually want to do vs. what each enables:**

| Mod type | Percentage of mods | Best runtime |
|----------|-------------------|-------------|
| Building packs (new meshes + stats) | 40% | Data-only (no scripting needed) |
| Gameplay tweaks (tax rates, growth speeds) | 20% | Data-only or Lua |
| Custom events/scenarios | 10% | Lua |
| UI modifications | 10% | Lua |
| Traffic AI overhaul | 5% | WASM or Native |
| New zone/road types | 5% | Lua + Data |
| Map tools (Move It, 81 Tiles) | 5% | Native |
| Deep system mods (TM:PE) | 3% | Native |
| Custom rendering | 2% | Native |

This means **70-80% of mods need no scripting at all or only simple Lua.** The data-driven architecture (Section 7) is therefore more important than the scripting system.

### 2.5 API Surface Design for Scripts

The API surface is what makes or breaks a modding ecosystem. Too restrictive and modders cannot achieve their goals. Too permissive and mods break constantly, or worse, compromise security. Here is the complete API surface organized by domain.

**Events that scripts can hook into:**

```
Lifecycle Events:
  on_game_start()              -- Called when a new game starts
  on_game_load()               -- Called when a save is loaded
  on_tick()                    -- Called every simulation tick (~10Hz)
  on_slow_tick()               -- Called every 100 ticks (~0.1Hz)
  on_day_change(day)           -- Called when the day counter advances
  on_season_change(season)     -- Called when the season changes

Entity Events:
  on_building_placed(building_info)     -- After a building spawns
  on_building_destroyed(building_info)  -- Before a building is removed
  on_building_upgraded(old_level, new_level, building_info)
  on_citizen_spawned(citizen_info)
  on_citizen_died(citizen_info)
  on_citizen_moved(citizen_info, from_pos, to_pos)
  on_road_built(segment_info)
  on_road_demolished(segment_info)

City Events:
  on_policy_enabled(policy_name)
  on_policy_disabled(policy_name)
  on_disaster_start(disaster_type, center_pos, radius)
  on_disaster_end(disaster_type)
  on_milestone_reached(milestone_name, population)
  on_budget_collected(revenue, expenses, balance)
  on_zone_demand_changed(residential, commercial, industrial, office)

UI Events:
  on_cell_clicked(x, y, button)
  on_cell_hovered(x, y)
  on_tool_selected(tool_name)
  on_overlay_changed(overlay_name)
```

**Functions that scripts can call:**

```
City Queries (read-only):
  city.get_population() -> number
  city.get_treasury() -> number
  city.get_tax_rate() -> number
  city.get_happiness() -> number
  city.get_day() -> number
  city.get_hour() -> number
  city.get_season() -> string
  city.get_weather() -> table { type, intensity, temperature }

Grid Queries (read-only):
  grid.get_cell(x, y) -> table { cell_type, zone, road_type, elevation, has_power, has_water }
  grid.get_zone(x, y) -> string
  grid.get_elevation(x, y) -> number
  grid.is_road(x, y) -> bool
  grid.is_water(x, y) -> bool
  grid.width() -> number
  grid.height() -> number

Building Queries (read-only):
  buildings.get_at(x, y) -> building_info or nil
  buildings.get_all() -> array of building_info
  buildings.get_by_zone(zone_name) -> array of building_info
  buildings.count() -> number
  buildings.count_by_zone(zone_name) -> number

Traffic Queries (read-only):
  traffic.get_density(x, y) -> number
  traffic.get_congestion(x, y) -> number (0-1)
  traffic.get_average_congestion() -> number

Citizen Queries (read-only):
  citizens.count() -> number
  citizens.get_average_happiness() -> number
  citizens.get_average_health() -> number
  citizens.get_unemployment_rate() -> number
  citizens.count_by_state(state_name) -> number

Service Queries (read-only):
  services.get_coverage(service_type, x, y) -> bool
  services.get_all(service_type) -> array of service_info

City Mutations (write, queued):
  city.set_tax_rate(rate)                    -- Clamped to 0.0-0.50
  city.add_treasury(amount)                  -- Can be negative (clamped to not go below 0)
  city.enable_policy(policy_name) -> bool
  city.disable_policy(policy_name) -> bool

Building Mutations (write, queued):
  buildings.set_capacity(x, y, capacity)     -- Override building capacity
  buildings.spawn(zone, x, y, level) -> id   -- Spawn a building (RICO-style)
  buildings.demolish(x, y) -> bool           -- Remove a building
  buildings.upgrade(x, y) -> bool            -- Force upgrade

Grid Mutations (write, queued):
  grid.set_zone(x, y, zone_name)             -- Change zone type
  grid.set_elevation(x, y, height)           -- Terraform

UI Functions:
  ui.show_notification(title, message)
  ui.show_tooltip(x, y, text)
  ui.add_info_panel_section(title, content_callback)
  ui.add_toolbar_button(id, label, icon, on_click)

Logging:
  log.info(message)
  log.warn(message)
  log.error(message)

Timer Functions:
  timer.after(seconds, callback_name)        -- Call function after delay
  timer.every(seconds, callback_name)        -- Call function repeatedly
  timer.cancel(timer_id)                     -- Cancel a timer
```

**Critical design principle: read operations are synchronous, write operations are queued.** Scripts never directly mutate game state. They enqueue commands (like Bevy's `Commands` pattern) that are applied by the mod host after the script returns. This prevents partial state corruption if a script errors mid-execution.

---

## 3. Asset Pipeline for Custom Content

Custom buildings, vehicles, props, and trees are the bread and butter of city builder modding. CS1's Workshop has over 200,000 custom assets. The asset pipeline must be simple enough for a 3D modeler who has never written code, yet robust enough to prevent broken assets from crashing the game.

### 3.1 Custom Buildings

Buildings are the most important moddable asset class. A building asset in Megacity consists of:

**Mesh requirements:**
- **Format**: glTF 2.0 (`.glb` binary preferred, `.gltf` + separate files supported). glTF is the "JPEG of 3D" -- widely supported by Blender, Maya, 3ds Max, and every game engine. Bevy uses it natively.
- **Polycount budget**: Varies by LOD tier and building level.
  - LOD0 (close-up): up to 5,000 triangles for level 1, up to 15,000 for level 5
  - LOD1 (medium distance): 50% of LOD0
  - LOD2 (far/abstract): 10% of LOD0 or a simple box/billboard
- **Coordinate system**: Y-up, meters scale. Building origin at ground center.
- **UV mapping**: Single UV set, atlas-friendly (all textures in one material).

**LOD levels:**

Our rendering system already has LOD tiers (`crates/simulation/src/lod.rs`): Full, Simplified, Abstract. Custom buildings must provide meshes for each:

```
my_building/
  mesh_lod0.glb    -- Full detail (close camera)
  mesh_lod1.glb    -- Simplified (medium distance)
  mesh_lod2.glb    -- Abstract (far distance, optional - auto-generated if missing)
```

If LOD1 or LOD2 meshes are not provided, the loader auto-generates them using mesh decimation (quadric edge collapse). This lowers the barrier for modders who just want to ship one high-quality mesh.

**Texture atlas:**

```
my_building/
  textures/
    diffuse.png     -- Base color (required, max 2048x2048)
    normal.png      -- Normal map (optional, max 2048x2048)
    orm.png         -- Occlusion/Roughness/Metallic packed (optional)
    emissive.png    -- Emissive map for lit windows at night (optional)
```

Night-time emissive maps are important for city builders. Buildings with lit windows at night create atmosphere. The emissive map is sampled based on the `GameClock` hour -- windows light up at dusk and turn off at late night.

**Collision bounds:**

Buildings need collision volumes for:
1. Preventing overlapping placement (occupancy grid cells)
2. Cursor hover detection (ray casting)
3. Preventing roads from being built through buildings

```toml
# In the building's manifest:
[collision]
# Option 1: Axis-aligned bounding box (simplest)
type = "aabb"
size = [2.0, 8.0, 2.0]  # width, height, depth in grid cells

# Option 2: Grid footprint (for L-shaped or irregular buildings)
type = "grid"
footprint = [
    [1, 1, 0],
    [1, 1, 1],
    [0, 1, 1],
]
```

**Building metadata (`building.toml`):**

```toml
[building]
id = "com.author.bauhaus-apartment"
name = "Bauhaus Apartment Block"
description = "A White City-style residential building with characteristic balconies"
author = "ModAuthor"
version = "1.0.0"

[placement]
zone_type = "ResidentialHigh"    # Which zone this building grows in
level = 2                        # Building level (1-5)
capacity = 150                   # Max occupants
footprint_width = 2              # Grid cells wide
footprint_depth = 2              # Grid cells deep
min_land_value = 30              # Only spawns where land value >= this
max_land_value = 80              # Only spawns where land value <= this

[growth]
# How this building participates in the growth simulation
growth_style = "organic"         # "organic" (zone demand) or "plop" (RICO-style manual)
construction_time = 100          # Ticks to build (0 = instant)
can_upgrade_to = "com.author.bauhaus-tower"  # Optional upgrade path

[visuals]
mesh_lod0 = "mesh_lod0.glb"
mesh_lod1 = "mesh_lod1.glb"
# mesh_lod2 omitted = auto-generated
diffuse = "textures/diffuse.png"
normal = "textures/normal.png"
emissive = "textures/emissive.png"
color_variations = 3             # Number of random color tints
height = 24.0                    # Height in world units (for shadow/occlusion)

[effects]
noise_radius = 1                 # Noise pollution radius (grid cells)
pollution = 0                    # Pollution output (0-100)
fire_hazard = 0.02               # Base fire probability per tick
crime_attractiveness = 0.1       # How much crime this building attracts

[cost]
build_cost = 5000.0              # Treasury cost for RICO-style placement
upkeep = 50.0                    # Monthly upkeep cost
demolish_refund = 0.3            # Fraction of build cost returned on demolition

[tags]
style = "bauhaus"
era = "1930s"
district = "white-city"
```

**Building registration in the data registry:**

```rust
// crates/mod-host/src/asset_loader.rs

pub fn load_building_asset(
    path: &Path,
    asset_server: &AssetServer,
    registry: &mut DataRegistry,
    mod_id: &str,
) -> Result<(), AssetLoadError> {
    // Parse building.toml
    let manifest_path = path.join("building.toml");
    let manifest: BuildingManifest = toml::from_str(
        &std::fs::read_to_string(&manifest_path)?
    )?;

    // Validate required fields
    validate_building_manifest(&manifest)?;

    // Load meshes via Bevy's asset server
    let mesh_lod0: Handle<Scene> = asset_server.load(
        path.join(&manifest.visuals.mesh_lod0).to_str().unwrap()
    );

    let mesh_lod1: Option<Handle<Scene>> = manifest.visuals.mesh_lod1.as_ref()
        .map(|p| asset_server.load(path.join(p).to_str().unwrap()));

    // Register in data registry
    let building_def = CustomBuildingDefinition {
        id: manifest.building.id.clone(),
        metadata: manifest,
        mesh_handles: BuildingMeshHandles {
            lod0: mesh_lod0,
            lod1: mesh_lod1,
            lod2: None,  // Generated later if needed
        },
    };

    registry.register("buildings", &building_def.id, building_def, mod_id)?;

    Ok(())
}
```

### 3.2 Custom Vehicles

Vehicles in Megacity currently are represented as citizen sprites moving along paths. For a more detailed vehicle system:

**Vehicle asset structure:**

```
my_vehicle/
  vehicle.toml     -- Metadata
  model.glb        -- 3D mesh
  textures/
    diffuse.png
    emissive.png   -- Headlights, taillights
```

**Vehicle metadata (`vehicle.toml`):**

```toml
[vehicle]
id = "com.author.city-bus"
name = "City Bus"
category = "public_transit"      # car, truck, bus, tram, train, bicycle, emergency
version = "1.0.0"

[specs]
max_speed = 40.0                 # km/h
acceleration = 2.0               # m/s^2
braking = 5.0                    # m/s^2
length = 12.0                    # meters (affects traffic density)
capacity = 50                    # passengers
fuel_type = "diesel"             # diesel, electric, hybrid

[pathfinding]
allowed_road_types = ["Local", "Avenue", "Boulevard"]  # Not Highway, not Path
turn_radius = 8.0                # minimum turn radius in meters
can_use_bus_lanes = true
priority = 2                     # Higher = right of way (emergency = 10)

[animation]
# States that the mesh can display
states = ["idle", "moving", "doors_open", "turning_left", "turning_right"]
# Animation clips in the glTF file
idle_clip = "Idle"
moving_clip = "Moving"
doors_clip = "DoorsOpen"

[visuals]
model = "model.glb"
scale = 1.0
color_variations = ["#FF0000", "#0000FF", "#00FF00"]  # Random color per instance
```

**Integration with the pathfinding system:**

Vehicles use the same CSR graph pathfinding (`road_graph_csr.rs`) as citizens, but with different edge weights based on vehicle type. The vehicle definition contributes to edge cost calculation:

```rust
pub fn vehicle_path_cost(
    road_type: RoadType,
    vehicle_def: &VehicleDefinition,
    congestion: f32,
) -> f32 {
    // Can this vehicle use this road?
    if !vehicle_def.allowed_road_types.contains(&road_type) {
        return f32::INFINITY;  // Cannot traverse
    }

    let effective_speed = road_type.speed().min(vehicle_def.max_speed);
    let base_cost = 1.0 / effective_speed;
    let congestion_penalty = congestion * 3.0 * vehicle_def.length / 5.0;

    base_cost + congestion_penalty
}
```

### 3.3 Custom Props and Trees

Props are decorative objects placed in the world: benches, street lamps, mailboxes, bus stops, fountains, statues. Trees are a special prop category with seasonal behavior.

**Prop asset structure:**

```
my_prop/
  prop.toml
  model.glb
  textures/
    diffuse.png
```

**Prop metadata (`prop.toml`):**

```toml
[prop]
id = "com.author.modern-bench"
name = "Modern Bench"
category = "street_furniture"    # street_furniture, decoration, infrastructure, nature
version = "1.0.0"

[placement]
surface = "ground"               # ground, water, rooftop, wall
snap_to_road = true              # Automatically align to nearest road edge
rotation = "any"                 # any, 90deg, road_aligned
footprint = [1, 1]               # Grid cells (usually 1x1 for props)

[visuals]
model = "model.glb"
scale = 0.8
lod_distance = 100.0             # Distance at which to hide this prop

[effects]
happiness_bonus = 0.5            # Adds to nearby building happiness
land_value_bonus = 1.0           # Adds to nearby land value
```

**Tree assets with seasonal variants:**

```toml
[prop]
id = "com.author.jacaranda-tree"
name = "Jacaranda Tree"
category = "nature"

[seasons]
spring = "model_spring.glb"       # Purple blossoms
summer = "model_summer.glb"       # Full green
autumn = "model_autumn.glb"       # Yellowing
winter = "model_winter.glb"       # Bare branches

[growth]
growth_stages = 3                  # Sapling -> young -> mature
growth_time = 365                  # Days to fully mature
mature_height = 8.0                # World units

[effects]
pollution_absorption = 2.0        # Reduces nearby pollution
noise_reduction = 1.0             # Reduces nearby noise
happiness_bonus = 1.0
shade_bonus = true                 # Reduces heat in summer
fire_risk = 0.001                  # Can catch fire
```

### 3.4 Asset Packaging Format

Mods are distributed as zip files with a standardized directory structure:

```
my-building-pack.megamod          # Actually a .zip file with .megamod extension
  mod.toml                        # Top-level manifest
  assets/
    buildings/
      bauhaus-apartment/
        building.toml
        mesh_lod0.glb
        mesh_lod1.glb
        textures/
          diffuse.png
          normal.png
          emissive.png
      bauhaus-tower/
        building.toml
        mesh_lod0.glb
        textures/
          diffuse.png
    props/
      modern-bench/
        prop.toml
        model.glb
        textures/
          diffuse.png
    vehicles/
      city-bus/
        vehicle.toml
        model.glb
        textures/
          diffuse.png
  scripts/                        # Optional Lua/WASM scripts
    main.lua
  data/                           # Optional data overrides
    road_types.toml               # Custom road type definitions
    zone_params.toml              # Zone parameter overrides
  preview.png                     # Workshop preview image (512x512)
  changelog.md                    # Version history
```

**Top-level manifest (`mod.toml`):**

```toml
[mod]
id = "com.author.tel-aviv-buildings"
name = "Tel Aviv Building Pack"
version = "2.0.0"
sdk_version = "^1.0"
description = "50+ authentic Tel Aviv buildings: Bauhaus, Ottoman, Modern"
authors = ["Author Name"]
license = "CC-BY-4.0"
preview = "preview.png"

[content]
buildings = 52
props = 12
vehicles = 3
scripts = ["scripts/main.lua"]
data_overrides = ["data/zone_params.toml"]

[requirements]
min_game_version = "1.0.0"
max_game_version = "2.0.0"         # Optional upper bound

[dependencies]
# No required dependencies for a pure asset pack

[tags]
categories = ["buildings", "props", "vehicles"]
themes = ["mediterranean", "bauhaus", "modern"]
region = "middle-east"
```

**Why `.megamod` extension?** Several reasons:
1. File association: double-clicking a `.megamod` file can launch the game's mod installer
2. Prevents confusion with regular zip files
3. Allows OS-level icons for mod packages
4. Steam Workshop can filter by extension

### 3.5 Asset Validation and Loading

Asset validation prevents broken mods from crashing the game. Validation happens in two phases:

**Phase 1: Static validation (at mod install time)**

```rust
// crates/mod-host/src/asset_validator.rs

pub struct AssetValidator;

impl AssetValidator {
    pub fn validate_mod_package(path: &Path) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // 1. Check that mod.toml exists and parses
        let manifest_path = path.join("mod.toml");
        let manifest = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => match toml::from_str::<ModManifest>(&s) {
                Ok(m) => m,
                Err(e) => {
                    issues.push(ValidationIssue::Error(
                        format!("Invalid mod.toml: {}", e)
                    ));
                    return issues;
                }
            },
            Err(_) => {
                issues.push(ValidationIssue::Error(
                    "Missing mod.toml".to_string()
                ));
                return issues;
            }
        };

        // 2. Check SDK version compatibility
        let our_sdk_version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        if !manifest.mod_section.sdk_version.matches(&our_sdk_version) {
            issues.push(ValidationIssue::Error(format!(
                "SDK version mismatch: mod requires {}, game provides {}",
                manifest.mod_section.sdk_version, our_sdk_version
            )));
        }

        // 3. Validate each building asset
        for building_dir in find_building_dirs(path) {
            issues.extend(validate_building(&building_dir));
        }

        // 4. Validate texture sizes
        for texture_path in find_textures(path) {
            issues.extend(validate_texture(&texture_path));
        }

        // 5. Validate mesh polycount
        for mesh_path in find_meshes(path) {
            issues.extend(validate_mesh(&mesh_path));
        }

        issues
    }

    fn validate_building(dir: &Path) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check building.toml
        let manifest = dir.join("building.toml");
        if !manifest.exists() {
            issues.push(ValidationIssue::Error(
                format!("Missing building.toml in {}", dir.display())
            ));
            return issues;
        }

        let building: BuildingManifest = match toml::from_str(
            &std::fs::read_to_string(&manifest).unwrap_or_default()
        ) {
            Ok(b) => b,
            Err(e) => {
                issues.push(ValidationIssue::Error(
                    format!("Invalid building.toml: {}", e)
                ));
                return issues;
            }
        };

        // Check required mesh exists
        let mesh_path = dir.join(&building.visuals.mesh_lod0);
        if !mesh_path.exists() {
            issues.push(ValidationIssue::Error(
                format!("Missing LOD0 mesh: {}", mesh_path.display())
            ));
        }

        // Check capacity is reasonable
        if building.placement.capacity == 0 {
            issues.push(ValidationIssue::Warning(
                "Building capacity is 0 -- it will never accept occupants".to_string()
            ));
        }
        if building.placement.capacity > 10_000 {
            issues.push(ValidationIssue::Warning(
                format!("Very high capacity ({}). This may unbalance the simulation.",
                    building.placement.capacity)
            ));
        }

        // Check footprint fits in grid
        if building.placement.footprint_width > 16 || building.placement.footprint_depth > 16 {
            issues.push(ValidationIssue::Error(
                "Building footprint exceeds 16x16 cell maximum".to_string()
            ));
        }

        issues
    }

    fn validate_texture(path: &Path) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check file size (max 16MB per texture)
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > 16 * 1024 * 1024 {
                issues.push(ValidationIssue::Error(format!(
                    "Texture {} exceeds 16MB limit ({:.1}MB)",
                    path.display(),
                    metadata.len() as f64 / (1024.0 * 1024.0)
                )));
            }
        }

        // Check dimensions (would need an image library like `image` crate)
        // Max 4096x4096 for any single texture
        // Warn if not power-of-two dimensions

        issues
    }

    fn validate_mesh(path: &Path) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check file size (max 50MB per mesh)
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > 50 * 1024 * 1024 {
                issues.push(ValidationIssue::Error(format!(
                    "Mesh {} exceeds 50MB limit ({:.1}MB)",
                    path.display(),
                    metadata.len() as f64 / (1024.0 * 1024.0)
                )));
            }
        }

        // TODO: Parse glTF header to check triangle count
        // LOD0: max 15,000 triangles
        // LOD1: max 7,500 triangles
        // LOD2: max 1,500 triangles

        issues
    }
}

#[derive(Debug)]
pub enum ValidationIssue {
    Error(String),    // Prevents loading
    Warning(String),  // Displayed but doesn't prevent loading
    Info(String),     // Informational
}
```

**Phase 2: Runtime loading (async, via Bevy's asset system)**

```rust
// crates/mod-host/src/runtime_loader.rs

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};

/// Custom asset loader for .megamod files
pub struct MegamodAssetLoader;

impl AssetLoader for MegamodAssetLoader {
    type Asset = LoadedMod;
    type Settings = ();
    type Error = ModLoadError;

    fn load(
        &self,
        reader: &mut dyn std::io::Read,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> Result<LoadedMod, Self::Error> {
        // 1. Read zip contents
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;

        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)?;

        // 2. Extract to temp directory
        let temp_dir = std::env::temp_dir().join("megacity_mods")
            .join(load_context.path().file_stem().unwrap().to_str().unwrap());
        std::fs::create_dir_all(&temp_dir)?;

        archive.extract(&temp_dir)?;

        // 3. Parse mod.toml
        let manifest: ModManifest = toml::from_str(
            &std::fs::read_to_string(temp_dir.join("mod.toml"))?
        )?;

        // 4. Load sub-assets (meshes, textures) via Bevy's asset server
        // These are loaded asynchronously by Bevy
        let mut building_handles = Vec::new();
        for building_dir in find_building_dirs(&temp_dir) {
            let mesh_path = building_dir.join("mesh_lod0.glb");
            let handle = load_context.load(mesh_path);
            building_handles.push(handle);
        }

        Ok(LoadedMod {
            manifest,
            building_handles,
            temp_dir,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["megamod"]
    }
}
```

### 3.6 Hot-Reloading Assets During Development

For mod developers, hot-reloading assets is as important as hot-reloading code. Bevy already has built-in file-watching for assets in debug mode. We extend this to mod asset directories.

**How it works:**

1. When a mod is loaded in dev mode, its asset directory is added to Bevy's `AssetServer` watch list.
2. When a `.glb`, `.png`, or `.toml` file changes, Bevy automatically reloads the associated `Handle<T>`.
3. For meshes: the old mesh is replaced by the new mesh. Entities using the old mesh handle automatically display the new geometry.
4. For textures: same automatic replacement via handle indirection.
5. For `.toml` metadata: we listen for `AssetEvent::Modified` and re-parse the manifest, then update the data registry.

```rust
// crates/mod-host/src/asset_hot_reload.rs

pub fn watch_mod_assets(
    mut events: EventReader<AssetEvent<Scene>>,
    mut data_registry: ResMut<DataRegistry>,
    mod_manager: Res<ModManager>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                // Find which mod this asset belongs to
                if let Some(mod_id) = mod_manager.asset_to_mod(id) {
                    info!("Hot-reloading asset for mod {}: {:?}", mod_id, id);
                    // Bevy handles mesh/texture reload automatically via handles.
                    // We just need to log it and potentially update metadata.
                }
            }
            _ => {}
        }
    }
}

pub fn watch_mod_toml_files(
    // Use a filesystem watcher (notify crate) for .toml files
    // since they aren't Bevy assets by default
    mut watcher_events: EventReader<FileChangeEvent>,
    mut data_registry: ResMut<DataRegistry>,
) {
    for event in watcher_events.read() {
        if event.path.extension() == Some("toml".as_ref()) {
            info!("Reloading mod data file: {}", event.path.display());
            // Re-parse and update the registry
            if let Ok(content) = std::fs::read_to_string(&event.path) {
                if event.path.file_name() == Some("building.toml".as_ref()) {
                    if let Ok(manifest) = toml::from_str::<BuildingManifest>(&content) {
                        data_registry.update("buildings", &manifest.building.id, manifest);
                    }
                }
            }
        }
    }
}
```

**Workflow for a mod developer:**

1. Create a mod directory in `mods/dev/my-building-pack/`
2. Start the game with `--mod-dev mods/dev/my-building-pack/`
3. Edit `building.toml` in a text editor -- stats update live in-game
4. Edit `mesh_lod0.glb` in Blender and re-export -- mesh updates live in-game
5. Edit `diffuse.png` in Photoshop and save -- texture updates live in-game

No restart required at any point. This is the level of developer experience that attracts modders.

---

## 4. What Modders Actually Need: CS1 Mod Analysis

The most successful CS1 mods each solved a specific deficiency in the base game. Understanding exactly what they needed from the engine tells us precisely which hooks and APIs Megacity must expose. This section maps each iconic CS1 mod to concrete Megacity architecture requirements.

### 4.1 TM:PE Equivalent: Traffic System API

**What TM:PE does in CS1 (8M subscribers):**
- Per-lane traffic rule assignment (no trucks, bus only, etc.)
- Custom signal timing at intersections (green wave coordination)
- Speed limit overrides per road segment
- Lane connection editing (which lanes can go where at intersections)
- Priority signs (yield, stop, right-of-way)
- Vehicle restriction zones (heavy traffic ban areas)
- Roundabout setup tools
- Parking AI overrides
- Timed traffic lights with custom phases

**What this requires from Megacity:**

Our traffic system is currently grid-based (`TrafficGrid` with `density: Vec<u16>`) and segment-based (`RoadSegmentStore` with Bezier curves). TM:PE-equivalent functionality requires exposing:

1. **Road segment query and modification API:**

```rust
// In the mod SDK:
pub trait TrafficApi {
    /// Get all road segments
    fn get_segments(&self) -> Vec<SegmentInfo>;

    /// Get segment by ID
    fn get_segment(&self, id: u32) -> Option<SegmentInfo>;

    /// Get segments connected to a node (intersection)
    fn get_intersection_segments(&self, node_id: u32) -> Vec<SegmentInfo>;

    /// Set speed limit override for a segment
    fn set_speed_limit(&mut self, segment_id: u32, speed: f32) -> Result<(), ModError>;

    /// Set vehicle restrictions for a segment
    fn set_vehicle_restriction(
        &mut self,
        segment_id: u32,
        restriction: VehicleRestriction,
    ) -> Result<(), ModError>;

    /// Configure intersection signal timing
    fn set_signal_timing(
        &mut self,
        node_id: u32,
        timing: SignalTiming,
    ) -> Result<(), ModError>;

    /// Set lane routing rules at an intersection
    fn set_lane_routing(
        &mut self,
        node_id: u32,
        from_segment: u32,
        to_segment: u32,
        allowed_lanes: Vec<u8>,
    ) -> Result<(), ModError>;

    /// Get current traffic density on a segment
    fn get_segment_density(&self, segment_id: u32) -> f32;

    /// Get traffic flow rate (vehicles per hour) through a segment
    fn get_segment_flow(&self, segment_id: u32) -> f32;
}

#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub id: u32,
    pub start_node: u32,
    pub end_node: u32,
    pub road_type: String,
    pub speed_limit: f32,        // Current effective speed
    pub base_speed: f32,         // Default speed for this road type
    pub lane_count: u8,
    pub length: f32,             // Arc length in world units
    pub start_pos: WorldPos,
    pub end_pos: WorldPos,
    pub control_point_1: WorldPos,  // Bezier control points
    pub control_point_2: WorldPos,
    pub current_density: f32,
    pub vehicle_restriction: VehicleRestriction,
}

#[derive(Debug, Clone)]
pub struct VehicleRestriction {
    pub allow_cars: bool,
    pub allow_trucks: bool,
    pub allow_buses: bool,
    pub allow_emergency: bool,
    pub allow_bicycles: bool,
}

#[derive(Debug, Clone)]
pub struct SignalTiming {
    pub phases: Vec<SignalPhase>,
    pub cycle_time: f32,         // Total cycle in seconds
    pub offset: f32,             // Phase offset for green wave coordination
}

#[derive(Debug, Clone)]
pub struct SignalPhase {
    pub green_segments: Vec<u32>,  // Segments that have green during this phase
    pub duration: f32,             // Phase duration in seconds
    pub yellow_duration: f32,      // Yellow transition time
}
```

2. **Pathfinding customization hooks:**

```rust
pub trait PathfindingApi {
    /// Register a custom edge cost function that modifies pathfinding weights
    /// This is called for every edge evaluation during A* search
    fn register_cost_modifier(
        &mut self,
        mod_id: &str,
        modifier: Box<dyn Fn(EdgeInfo) -> f32 + Send + Sync>,
    );

    /// Register a custom path post-processor that can modify found paths
    fn register_path_postprocessor(
        &mut self,
        mod_id: &str,
        processor: Box<dyn Fn(Vec<PathNode>) -> Vec<PathNode> + Send + Sync>,
    );
}
```

**Current architecture gaps for TM:PE-level modding:**

Looking at our current code:
- `RoadType` is an enum with hardcoded `speed()`, `lane_count()`, `cost()` methods. This needs to become data-driven (see Section 4.5).
- `TrafficGrid` only tracks density per cell, not per segment or per lane. TM:PE needs per-lane data.
- `CsrGraph` is rebuilt from `RoadNetwork` on road changes but does not support per-edge metadata (speed overrides, restrictions). It needs a metadata layer.
- No intersection concept exists -- segments share nodes in `RoadSegmentStore`, but there is no intersection data structure (signal state, phase timing, turn restrictions).

**Required refactoring before TM:PE-level modding is possible:**
1. Add `IntersectionNode` struct with signal state, phase list, turn restriction matrix
2. Add per-segment metadata storage (speed override, vehicle restriction, custom tags)
3. Extend `CsrGraph` edges with a `metadata_index` pointing into a separate metadata array
4. Add lane-level simulation (not just segment-level density)

### 4.2 RICO Equivalent: Building Spawn Hooks

**What Ploppable RICO does in CS1 (3M subscribers):**
- Bypass the organic zone-growth system to manually place specific buildings
- Set any asset as "ploppable" regardless of its original zone type
- Override building properties (capacity, level, effects) per instance
- Convert decorative/unique buildings into functional zone buildings
- Custom building categories and search/filter in a placement panel

**What this requires from Megacity:**

Our building spawn system (`crates/simulation/src/buildings.rs`) currently works like this:
1. `building_spawner` runs every 2 ticks
2. It picks a random zone type with positive demand
3. It scans the grid for eligible cells (correct zone, adjacent to road, no existing building)
4. It spawns a `Building` component with capacity based on zone and level

RICO needs to bypass steps 1-3 and let mods call step 4 directly, with custom parameters.

```rust
// In the mod SDK:
pub trait BuildingSpawnApi {
    /// Spawn a building at a specific location, bypassing zone demand
    /// Returns the Entity if successful, or an error explaining why not
    fn plop_building(
        &mut self,
        definition_id: &str,     // Building definition from data registry
        grid_x: usize,
        grid_y: usize,
        rotation: f32,           // Rotation in radians
    ) -> Result<Entity, PlacementError>;

    /// Spawn a building with custom overrides (RICO-style)
    fn plop_building_custom(
        &mut self,
        definition_id: &str,
        grid_x: usize,
        grid_y: usize,
        rotation: f32,
        overrides: BuildingOverrides,
    ) -> Result<Entity, PlacementError>;

    /// Query what can be placed at a location
    fn can_place_building(
        &self,
        definition_id: &str,
        grid_x: usize,
        grid_y: usize,
    ) -> PlacementCheck;

    /// Get all registered building definitions
    fn get_building_definitions(&self) -> Vec<BuildingDefinitionInfo>;

    /// Override properties of an existing building
    fn override_building(
        &mut self,
        entity: Entity,
        overrides: BuildingOverrides,
    ) -> Result<(), ModError>;
}

#[derive(Debug, Clone, Default)]
pub struct BuildingOverrides {
    pub capacity: Option<u32>,
    pub level: Option<u8>,
    pub zone_type: Option<ZoneKind>,
    pub construction_time: Option<u32>,
    pub upkeep: Option<f64>,
    pub custom_tags: HashMap<String, String>,
}

#[derive(Debug)]
pub enum PlacementError {
    OutOfBounds,
    CellOccupied,
    OnWater,
    OnRoad,
    InsufficientFunds,
    UnknownDefinition(String),
    FootprintConflict,
    CustomRejection(String),  // A mod's validation hook rejected it
}

#[derive(Debug)]
pub struct PlacementCheck {
    pub can_place: bool,
    pub reason: Option<String>,
    pub estimated_cost: f64,
    pub affected_cells: Vec<GridPos>,
}
```

**Building spawn event hooks (so other mods can react or modify):**

```rust
/// Event fired BEFORE a building is spawned (mods can cancel or modify)
pub struct PreBuildingSpawnEvent {
    pub definition_id: String,
    pub grid_x: usize,
    pub grid_y: usize,
    pub zone_type: ZoneKind,
    pub level: u8,
    pub capacity: u32,
    /// Set to true to cancel this spawn
    pub cancelled: bool,
    /// Mods can modify these fields before spawn happens
    pub modified_capacity: Option<u32>,
    pub modified_level: Option<u8>,
}

/// Event fired AFTER a building has been spawned
pub struct PostBuildingSpawnEvent {
    pub entity: Entity,
    pub definition_id: String,
    pub grid_x: usize,
    pub grid_y: usize,
    pub zone_type: ZoneKind,
    pub level: u8,
    pub capacity: u32,
    pub source: SpawnSource,  // Organic | Plopped | Script | Load
}
```

### 4.3 Move It Equivalent: Entity Transform Access

**What Move It does in CS1 (5M subscribers):**
- Select individual or groups of entities (buildings, trees, props, road nodes)
- Move entities freely (including off-grid for precise placement)
- Rotate entities to any angle
- Copy/paste groups of entities
- Undo/redo for all transformations
- Bulldoze with selection (area delete)
- Align entities to lines, grids, curves
- Snap to road edges, building edges, other props

**What this requires from Megacity:**

Move It is fundamentally an entity manipulation tool. In Bevy ECS terms, it needs:

```rust
// In the mod SDK:
pub trait EntityManipulationApi {
    /// Query entities in a world-space bounding box
    fn query_entities_in_rect(
        &self,
        min: WorldPos,
        max: WorldPos,
        filter: EntityFilter,
    ) -> Vec<EntityInfo>;

    /// Query entities within a radius of a point
    fn query_entities_in_radius(
        &self,
        center: WorldPos,
        radius: f32,
        filter: EntityFilter,
    ) -> Vec<EntityInfo>;

    /// Get full info about a specific entity
    fn get_entity_info(&self, entity: Entity) -> Option<EntityInfo>;

    /// Move an entity to a new position
    fn move_entity(
        &mut self,
        entity: Entity,
        new_pos: WorldPos,
    ) -> Result<(), ModError>;

    /// Move an entity to a new grid position (snaps to grid)
    fn move_entity_to_grid(
        &mut self,
        entity: Entity,
        grid_pos: GridPos,
    ) -> Result<(), ModError>;

    /// Rotate an entity
    fn rotate_entity(
        &mut self,
        entity: Entity,
        angle_radians: f32,
    ) -> Result<(), ModError>;

    /// Set entity scale
    fn scale_entity(
        &mut self,
        entity: Entity,
        scale: f32,
    ) -> Result<(), ModError>;

    /// Despawn an entity and clean up its grid references
    fn despawn_entity(
        &mut self,
        entity: Entity,
    ) -> Result<(), ModError>;

    /// Clone an entity (deep copy all components)
    fn clone_entity(
        &mut self,
        entity: Entity,
        new_pos: WorldPos,
    ) -> Result<Entity, ModError>;

    /// Batch operations (for undo/redo groups)
    fn begin_batch(&mut self) -> BatchId;
    fn commit_batch(&mut self, batch: BatchId);
    fn rollback_batch(&mut self, batch: BatchId);
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub entity: Entity,
    pub entity_type: EntityType,
    pub position: WorldPos,
    pub grid_pos: Option<GridPos>,
    pub rotation: f32,
    pub scale: f32,
    /// Type-specific data
    pub building: Option<BuildingInfo>,
    pub service: Option<ServiceInfo>,
    pub utility: Option<UtilityInfo>,
    pub citizen: Option<CitizenInfo>,
}

#[derive(Debug, Clone, Copy)]
pub enum EntityType {
    Building,
    Service,
    Utility,
    Citizen,
    Prop,
    Tree,
    RoadNode,
    Vehicle,
}

#[derive(Debug, Clone)]
pub struct EntityFilter {
    pub entity_types: Option<Vec<EntityType>>,  // None = all types
    pub zone_types: Option<Vec<ZoneKind>>,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    pub custom_tag: Option<String>,
}
```

**Undo/redo system:**

Move It needs a full undo/redo stack. This is architecturally significant because it requires snapshotting entity state before modifications:

```rust
// crates/mod-host/src/undo.rs

pub struct UndoStack {
    history: Vec<UndoGroup>,
    redo_stack: Vec<UndoGroup>,
    max_history: usize,
}

pub struct UndoGroup {
    pub description: String,
    pub operations: Vec<UndoOperation>,
}

pub enum UndoOperation {
    MoveEntity {
        entity: Entity,
        old_pos: WorldPos,
        new_pos: WorldPos,
        old_grid: Option<GridPos>,
        new_grid: Option<GridPos>,
    },
    RotateEntity {
        entity: Entity,
        old_angle: f32,
        new_angle: f32,
    },
    SpawnEntity {
        entity: Entity,
        // Store enough info to re-create if undone then redone
        snapshot: EntitySnapshot,
    },
    DespawnEntity {
        entity: Entity,
        snapshot: EntitySnapshot,
    },
    ModifyBuilding {
        entity: Entity,
        old_state: BuildingSnapshot,
        new_state: BuildingSnapshot,
    },
}

impl UndoStack {
    pub fn undo(&mut self, world: &mut World) -> Option<String> {
        let group = self.history.pop()?;
        let description = group.description.clone();

        // Apply operations in reverse order
        for op in group.operations.iter().rev() {
            match op {
                UndoOperation::MoveEntity { entity, old_pos, old_grid, .. } => {
                    // Move entity back to old position
                    if let Some(mut transform) = world.get_mut::<Transform>(*entity) {
                        transform.translation = Vec3::new(old_pos.x, 0.0, old_pos.y);
                    }
                    // Update grid references
                    if let Some(grid_pos) = old_grid {
                        // ... update grid cell's building_id
                    }
                }
                // ... handle other operation types
                _ => {}
            }
        }

        self.redo_stack.push(group);
        Some(description)
    }
}
```

### 4.4 81 Tiles Equivalent: Map Size Configuration

**What 81 Tiles does in CS1 (3M+ subscribers):**
- Unlocks all 81 map tiles (CS1 limits to 9 out of 25 tiles, or 25 out of 81 with DLC)
- Each tile is 1920x1920 units (240 cells at 8 units/cell)
- Effectively increases playable area 9x

**What this requires from Megacity:**

Our current grid dimensions are hardcoded constants:

```rust
// crates/simulation/src/config.rs (current)
pub const GRID_WIDTH: usize = 256;
pub const GRID_HEIGHT: usize = 256;
pub const CELL_SIZE: f32 = 16.0;
```

These constants propagate everywhere: `TrafficGrid`, `PollutionGrid`, `CrimeGrid`, `LandValueGrid`, `NoisePollutionGrid`, and every other grid resource uses `GRID_WIDTH * GRID_HEIGHT` for array sizing.

**The fix: make grid dimensions runtime-configurable.**

```rust
// Proposed config.rs
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub chunk_size: usize,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            cell_size: 16.0,
            chunk_size: 8,
        }
    }
}

impl GridConfig {
    pub fn large() -> Self {
        Self {
            width: 512,
            height: 512,
            cell_size: 16.0,
            chunk_size: 16,
        }
    }

    pub fn huge() -> Self {
        Self {
            width: 1024,
            height: 1024,
            cell_size: 16.0,
            chunk_size: 32,
        }
    }

    pub fn chunks_x(&self) -> usize { self.width / self.chunk_size }
    pub fn chunks_y(&self) -> usize { self.height / self.chunk_size }
    pub fn world_width(&self) -> f32 { self.width as f32 * self.cell_size }
    pub fn world_height(&self) -> f32 { self.height as f32 * self.cell_size }
    pub fn total_cells(&self) -> usize { self.width * self.height }
}
```

**Refactoring scope:** Every grid resource (`TrafficGrid`, `PollutionGrid`, etc.) currently does `vec![0; GRID_WIDTH * GRID_HEIGHT]` in its `Default` impl. These all need to read from `GridConfig` instead. This is a significant refactor but a mechanical one -- find-and-replace `GRID_WIDTH` with `config.width`, pass `GridConfig` as a constructor parameter.

**Performance implications of larger maps:**
- 256x256 = 65,536 cells (current, runs at 60fps with 1M virtual citizens)
- 512x512 = 262,144 cells (4x memory for grids, 4x iteration cost for grid scans)
- 1024x1024 = 1,048,576 cells (16x memory, 16x iteration cost)

The SlowTickTimer (Section 1 of lib.rs) already throttles expensive grid operations to every 100 ticks. For larger maps, this interval needs to scale proportionally. The LOD system also needs scaling -- the viewport covers a smaller fraction of a larger map, so more entities can be in Abstract tier.

**Mod API for map size:**

```rust
// In the mod SDK:
pub trait MapApi {
    /// Get current map dimensions
    fn get_map_size(&self) -> (usize, usize);

    /// Get cell size in world units
    fn get_cell_size(&self) -> f32;

    /// Convert grid position to world position
    fn grid_to_world(&self, grid: GridPos) -> WorldPos;

    /// Convert world position to grid position
    fn world_to_grid(&self, world: WorldPos) -> Option<GridPos>;

    /// Check if a grid position is in bounds
    fn is_in_bounds(&self, pos: GridPos) -> bool;
}
```

The map size must be set before the game world is initialized. This means it is a **startup-time configuration**, not a runtime modification. A mod that changes map size would specify it in `mod.toml`:

```toml
[map]
width = 512
height = 512
# Only one mod can set this. Conflicts are errors, not warnings.
```

### 4.5 Network Extensions Equivalent: Data-Driven Road Types

**What Network Extensions does in CS1:**
- Adds dozens of new road types (2-lane highway, 4-lane road with parking, asymmetric roads, etc.)
- Custom road meshes, textures, lane configurations
- Custom median types (grass, concrete, trees)
- Customized node meshes at intersections
- Different road types for bridges vs ground level

**What this requires from Megacity:**

Our current `RoadType` is a Rust enum:

```rust
// Current: crates/simulation/src/grid.rs
pub enum RoadType {
    Local,
    Avenue,
    Boulevard,
    Highway,
    OneWay,
    Path,
}
```

Every method on `RoadType` (`speed()`, `cost()`, `lane_count()`, `allows_zoning()`, etc.) is a `match` statement. Adding a new road type means modifying this enum and every match expression -- impossible for mods.

**The fix: replace the enum with a data-driven registry.**

```rust
// Proposed: data-driven road type system

/// Unique identifier for a road type (replaces the enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoadTypeId(pub u32);

/// Road type definition loaded from data files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadTypeDefinition {
    pub id: RoadTypeId,
    pub string_id: String,          // "local", "avenue", "com.author.parking-road"
    pub display_name: String,       // "Local Road", "Avenue", "Parking Road"
    pub speed: f32,                 // Speed limit
    pub lane_count: u8,             // Number of lanes
    pub cost: f64,                  // Construction cost per cell
    pub upkeep: f64,                // Monthly upkeep per cell
    pub allows_zoning: bool,        // Whether adjacent cells can be zoned
    pub allows_vehicles: bool,      // Whether vehicles can use this
    pub noise_radius: u8,           // Noise pollution radius
    pub width_cells: usize,         // How many cells wide
    pub capacity: u16,              // Max vehicles per cell before congestion
    pub one_way: bool,              // Is this a one-way road?
    pub has_median: bool,           // Divided road?
    pub median_type: Option<String>, // "grass", "concrete", "trees"
    pub has_parking: bool,          // On-street parking?
    pub has_sidewalk: bool,         // Sidewalk on edges?
    pub has_bike_lane: bool,        // Dedicated bike lane?
    pub has_bus_lane: bool,         // Dedicated bus lane?
    pub bridge_variant: Option<String>, // Different mesh for bridges
    pub tunnel_variant: Option<String>, // Different mesh for tunnels
    pub mesh_path: Option<String>,  // Custom mesh for road surface
    pub texture_path: Option<String>, // Custom texture
    pub tags: Vec<String>,          // Custom tags for filtering
}

/// Registry that replaces the hardcoded enum
#[derive(Resource)]
pub struct RoadTypeRegistry {
    types: HashMap<RoadTypeId, RoadTypeDefinition>,
    string_to_id: HashMap<String, RoadTypeId>,
    next_id: u32,
}

impl RoadTypeRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            types: HashMap::new(),
            string_to_id: HashMap::new(),
            next_id: 0,
        };

        // Register base game road types
        reg.register(RoadTypeDefinition {
            id: RoadTypeId(0),
            string_id: "local".to_string(),
            display_name: "Local Road".to_string(),
            speed: 30.0,
            lane_count: 2,
            cost: 10.0,
            upkeep: 1.0,
            allows_zoning: true,
            allows_vehicles: true,
            noise_radius: 2,
            width_cells: 1,
            capacity: 20,
            one_way: false,
            has_median: false,
            median_type: None,
            has_parking: false,
            has_sidewalk: true,
            has_bike_lane: false,
            has_bus_lane: false,
            bridge_variant: None,
            tunnel_variant: None,
            mesh_path: None,
            texture_path: None,
            tags: vec![],
        });

        // ... register Avenue, Boulevard, Highway, OneWay, Path ...

        reg
    }

    pub fn register(&mut self, def: RoadTypeDefinition) -> RoadTypeId {
        let id = if def.id.0 < self.next_id {
            // Mod-registered type, assign new ID
            let id = RoadTypeId(self.next_id);
            self.next_id += 1;
            id
        } else {
            self.next_id = def.id.0 + 1;
            def.id
        };

        self.string_to_id.insert(def.string_id.clone(), id);
        self.types.insert(id, def);
        id
    }

    pub fn get(&self, id: RoadTypeId) -> Option<&RoadTypeDefinition> {
        self.types.get(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&RoadTypeDefinition> {
        self.string_to_id.get(name)
            .and_then(|id| self.types.get(id))
    }

    pub fn speed(&self, id: RoadTypeId) -> f32 {
        self.types.get(&id).map(|d| d.speed).unwrap_or(30.0)
    }

    pub fn all_types(&self) -> impl Iterator<Item = &RoadTypeDefinition> {
        self.types.values()
    }
}
```

**Mod-defined road types in data files:**

```toml
# In a mod's data/road_types.toml

[[road_type]]
string_id = "com.author.parking-road"
display_name = "Road with Parking"
speed = 25.0
lane_count = 2
cost = 15.0
upkeep = 2.0
allows_zoning = true
allows_vehicles = true
noise_radius = 2
width_cells = 2
capacity = 15
has_parking = true
has_sidewalk = true
mesh_path = "assets/roads/parking_road.glb"
texture_path = "assets/roads/parking_road_diffuse.png"
tags = ["residential", "low-speed"]

[[road_type]]
string_id = "com.author.tree-boulevard"
display_name = "Tree-Lined Boulevard"
speed = 50.0
lane_count = 4
cost = 35.0
upkeep = 5.0
allows_zoning = true
allows_vehicles = true
noise_radius = 3
width_cells = 2
has_median = true
median_type = "trees"
has_sidewalk = true
capacity = 30
tags = ["scenic", "high-value"]
```

**Migration path:** The transition from `RoadType` enum to `RoadTypeRegistry` is a significant refactor. Every place that currently does `match road_type { RoadType::Local => ..., }` needs to change to `registry.get(road_type_id).speed`. This can be done incrementally:

1. Create `RoadTypeRegistry` that maps old enum variants to `RoadTypeDefinition` structs
2. Add `RoadTypeId` to cells alongside the existing `RoadType` enum
3. Migrate systems one at a time from `road_type.speed()` to `registry.speed(id)`
4. Remove the old enum once all systems are migrated

### 4.6 Asset Editor Equivalent: In-Game Creation Tools

**What CS1 asset editors do:**
- Visual building editor: import mesh, set properties, test in-game
- Road editor: configure lanes, medians, visual properties
- Map editor: terrain painting, water placement, starting infrastructure

**What this requires from Megacity:**

An in-game asset editor is a "nice to have" for launch but becomes critical for ecosystem growth. Key components:

1. **Property inspector panel**: When a building/road/prop is selected, show all its data-driven properties in an editable egui panel. Since our UI is already egui-based (`crates/ui/`), this is natural:

```rust
// In a mod or in the base game UI:
fn asset_inspector_ui(
    egui_ctx: &egui::Context,
    selected: &SelectedBuilding,
    registry: &mut DataRegistry,
) {
    egui::Window::new("Asset Inspector").show(egui_ctx, |ui| {
        if let Some(building_def) = registry.get_mut::<BuildingDefinition>(
            "buildings", &selected.definition_id
        ) {
            ui.heading(&building_def.metadata.name);
            ui.separator();

            ui.label("Placement");
            ui.horizontal(|ui| {
                ui.label("Capacity:");
                ui.add(egui::DragValue::new(&mut building_def.metadata.placement.capacity)
                    .clamp_range(1..=10000));
            });
            ui.horizontal(|ui| {
                ui.label("Level:");
                ui.add(egui::Slider::new(&mut building_def.metadata.placement.level, 1..=5));
            });
            ui.horizontal(|ui| {
                ui.label("Zone:");
                egui::ComboBox::from_label("")
                    .selected_text(&building_def.metadata.placement.zone_type)
                    .show_ui(ui, |ui| {
                        for zone in &["ResidentialLow", "ResidentialHigh",
                                      "CommercialLow", "CommercialHigh",
                                      "Industrial", "Office"] {
                            ui.selectable_value(
                                &mut building_def.metadata.placement.zone_type,
                                zone.to_string(),
                                *zone,
                            );
                        }
                    });
            });

            ui.separator();
            ui.label("Effects");
            ui.add(egui::Slider::new(
                &mut building_def.metadata.effects.pollution, 0.0..=100.0
            ).text("Pollution"));
            ui.add(egui::Slider::new(
                &mut building_def.metadata.effects.noise_radius, 0..=10
            ).text("Noise Radius"));

            if ui.button("Export to TOML").clicked() {
                let toml_string = toml::to_string_pretty(&building_def.metadata).unwrap();
                // Save to mod directory
            }
        }
    });
}
```

2. **Mesh preview**: Render the building mesh in a viewport within the inspector, with orbit camera controls. Bevy supports render-to-texture via `RenderTarget`, which can feed into an egui image widget.

3. **Test placement**: Place the custom building in the game world to see how it looks at different zoom levels, different times of day, and with neighboring buildings.

4. **Export workflow**: Save the building definition (TOML) and reference the mesh/textures. Optionally package into a `.megamod` for distribution.

---

## 5. Mod Distribution

### 5.1 Steam Workshop Integration

Steam Workshop is the dominant mod distribution platform for PC games. It handles hosting, versioning, automatic updates, and social features (ratings, comments, screenshots). Integration is non-negotiable for a Steam release.

**Steamworks SDK integration via `steamworks-rs`:**

```rust
// crates/mod-host/src/steam_workshop.rs

use steamworks::{Client, PublishedFileId, UGCType, UserList, UserListOrder};

pub struct SteamWorkshopManager {
    client: Client,
    subscribed_mods: Vec<WorkshopModInfo>,
}

#[derive(Debug, Clone)]
pub struct WorkshopModInfo {
    pub file_id: PublishedFileId,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author_steam_id: u64,
    pub file_size: u64,
    pub time_updated: u32,
    pub install_path: PathBuf,
    pub is_installed: bool,
    pub needs_update: bool,
}

impl SteamWorkshopManager {
    pub fn new() -> Result<Self, SteamError> {
        let client = Client::init()?;
        Ok(Self {
            client,
            subscribed_mods: Vec::new(),
        })
    }

    /// Fetch all subscribed mods from Steam Workshop
    pub fn refresh_subscribed_mods(&mut self) -> Result<(), SteamError> {
        let ugc = self.client.ugc();
        let user = self.client.user();

        // Get list of subscribed items
        let subscribed = ugc.subscribed_items();
        self.subscribed_mods.clear();

        for item_id in subscribed {
            // Get item install info
            let install_info = ugc.item_install_info(item_id);

            if let Some(info) = install_info {
                self.subscribed_mods.push(WorkshopModInfo {
                    file_id: item_id,
                    title: String::new(),  // Fetched asynchronously
                    description: String::new(),
                    tags: Vec::new(),
                    author_steam_id: 0,
                    file_size: info.size_on_disk,
                    time_updated: 0,
                    install_path: PathBuf::from(&info.folder),
                    is_installed: true,
                    needs_update: ugc.item_state(item_id)
                        .contains(steamworks::ItemState::NEEDS_UPDATE),
                });
            }
        }

        Ok(())
    }

    /// Upload a mod to Steam Workshop
    pub fn upload_mod(
        &self,
        mod_path: &Path,
        title: &str,
        description: &str,
        tags: &[&str],
        preview_image: &Path,
    ) -> Result<PublishedFileId, SteamError> {
        let ugc = self.client.ugc();

        // Create new Workshop item
        let app_id = self.client.utils().app_id();
        let create_result = ugc.create_item(app_id, UGCType::Items)?;

        // Update item content
        let update = ugc.start_item_update(app_id, create_result.published_file_id);
        update.set_title(title);
        update.set_description(description);
        update.set_tags(tags);
        update.set_content(mod_path);
        update.set_preview(preview_image);

        let submit_result = update.submit("Initial upload")?;

        Ok(create_result.published_file_id)
    }

    /// Get installed mod paths (for the mod loader to pick up)
    pub fn get_installed_mod_paths(&self) -> Vec<PathBuf> {
        self.subscribed_mods.iter()
            .filter(|m| m.is_installed)
            .map(|m| m.install_path.clone())
            .collect()
    }
}
```

**Workshop categories and tags:**

```
Categories:
  - Buildings         (custom building assets)
  - Roads             (custom road types and networks)
  - Props             (decorative objects)
  - Vehicles          (custom vehicle models)
  - Maps              (complete map presets)
  - Gameplay          (simulation tweaks, new mechanics)
  - UI                (interface modifications)
  - Traffic           (traffic AI, signal timing)
  - Tools             (Move It-style manipulation tools)
  - Total Conversion  (complete game overhauls)

Tags (user-selected, multiple allowed):
  - Style: Modern, Historical, Futuristic, Fantasy, Realistic
  - Region: North America, Europe, Asia, Middle East, etc.
  - Scale: Small, Medium, Large, Pack
  - Complexity: Simple, Intermediate, Advanced
```

### 5.2 Self-Hosted Mod Repository

Not all players use Steam. GOG, Epic, and direct purchases need mod access too. A self-hosted mod repository serves as the platform-agnostic alternative.

**REST API design:**

```
Base URL: https://mods.megacity-game.com/api/v1

GET  /mods                       -- List mods (paginated, filterable)
GET  /mods/{mod_id}              -- Get mod details
GET  /mods/{mod_id}/versions     -- List versions
GET  /mods/{mod_id}/versions/{v} -- Get specific version
GET  /mods/{mod_id}/download     -- Download latest version
GET  /mods/{mod_id}/versions/{v}/download  -- Download specific version
POST /mods                       -- Upload new mod (authenticated)
PUT  /mods/{mod_id}/versions     -- Upload new version (authenticated)
GET  /mods/search?q=traffic&tags=gameplay  -- Search
GET  /mods/popular               -- Popular mods (by downloads)
GET  /mods/trending              -- Trending mods (by recent growth)
GET  /mods/featured              -- Curated featured mods
POST /mods/{mod_id}/ratings      -- Submit rating (1-5 stars)
GET  /mods/{mod_id}/dependencies -- Dependency tree
GET  /compatibility/{game_version}  -- Mods compatible with a game version
```

**Response format example:**

```json
{
    "id": "com.author.traffic-overhaul",
    "name": "Traffic Overhaul",
    "version": "2.1.0",
    "author": {
        "name": "ModAuthor",
        "profile_url": "https://mods.megacity-game.com/users/modauthor"
    },
    "description": "Complete traffic AI replacement...",
    "sdk_version": "^1.0",
    "game_version_min": "1.0.0",
    "game_version_max": null,
    "download_count": 152340,
    "rating": 4.7,
    "rating_count": 8432,
    "file_size": 2457600,
    "sha256": "a3f2b8c9d1e5...",
    "dependencies": [
        {
            "mod_id": "com.other.road-extensions",
            "version_req": ">=1.0, <3.0"
        }
    ],
    "tags": ["gameplay", "traffic", "advanced"],
    "created_at": "2025-03-15T10:30:00Z",
    "updated_at": "2025-11-20T14:22:00Z",
    "download_url": "https://cdn.megacity-game.com/mods/traffic-overhaul-2.1.0.megamod",
    "preview_url": "https://cdn.megacity-game.com/mods/traffic-overhaul-preview.png"
}
```

**In-game mod browser:**

```rust
// crates/mod-host/src/mod_browser.rs

/// Async HTTP client for the mod repository
pub struct ModBrowser {
    client: reqwest::Client,
    base_url: String,
    cache: HashMap<String, CachedResponse>,
}

impl ModBrowser {
    pub async fn search(
        &self,
        query: &str,
        tags: &[&str],
        page: u32,
    ) -> Result<ModSearchResults, BrowserError> {
        let url = format!(
            "{}/mods/search?q={}&tags={}&page={}&per_page=20",
            self.base_url,
            urlencoding::encode(query),
            tags.join(","),
            page,
        );

        let response = self.client.get(&url).send().await?;
        let results: ModSearchResults = response.json().await?;
        Ok(results)
    }

    pub async fn download_mod(
        &self,
        mod_id: &str,
        version: Option<&str>,
        target_dir: &Path,
    ) -> Result<PathBuf, BrowserError> {
        let url = match version {
            Some(v) => format!("{}/mods/{}/versions/{}/download", self.base_url, mod_id, v),
            None => format!("{}/mods/{}/download", self.base_url, mod_id),
        };

        let response = self.client.get(&url).send().await?;
        let bytes = response.bytes().await?;

        // Verify checksum
        let hash = sha256::digest(&bytes);
        // Compare with expected hash from metadata...

        let file_path = target_dir.join(format!("{}.megamod", mod_id));
        std::fs::write(&file_path, &bytes)?;

        Ok(file_path)
    }
}
```

### 5.3 Mod Manager UI

The mod manager is an egui panel in the main menu (before entering a city) and accessible in-game via a menu option.

**Key features:**

1. **Mod list with enable/disable toggles**: Shows all installed mods with checkboxes. Disabled mods are skipped during loading.

2. **Load order management**: Drag-and-drop reordering. The mod manager shows the resolved load order after topological sort and highlights any conflicts.

3. **Conflict warnings**: Red indicators for hard conflicts (incompatible mods both enabled), yellow for soft warnings (potential resource overwrites).

4. **Dependency visualization**: Expand a mod to see its dependency tree. Missing dependencies are highlighted with a "Subscribe" button for Workshop mods or "Download" for repository mods.

5. **Version management**: Show current version vs available update. One-click update for Workshop mods, manual download for repository mods.

6. **Performance impact indicator**: Badge showing estimated performance impact based on mod type (data-only = green, script = yellow, native = red).

7. **Profiles**: Save and load mod configuration profiles. "Vanilla," "Light Modding," "Full Modding," custom profiles. Profiles store which mods are enabled and their load order.

```rust
// crates/ui/src/mod_manager.rs

pub fn mod_manager_ui(
    egui_ctx: &egui::Context,
    mod_manager: &mut ModManager,
    workshop: &SteamWorkshopManager,
) {
    egui::Window::new("Mod Manager")
        .default_size([800.0, 600.0])
        .show(egui_ctx, |ui| {
            // Profile selector
            ui.horizontal(|ui| {
                ui.label("Profile:");
                egui::ComboBox::from_id_source("mod_profile")
                    .selected_text(&mod_manager.active_profile)
                    .show_ui(ui, |ui| {
                        for profile in &mod_manager.profiles {
                            ui.selectable_value(
                                &mut mod_manager.active_profile,
                                profile.name.clone(),
                                &profile.name,
                            );
                        }
                    });
                if ui.button("Save Profile").clicked() {
                    mod_manager.save_current_profile();
                }
            });

            ui.separator();

            // Two-panel layout: mod list on left, details on right
            ui.columns(2, |columns| {
                // Left panel: mod list
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, mod_info) in mod_manager.installed_mods.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            // Enable/disable checkbox
                            ui.checkbox(&mut mod_info.enabled, "");

                            // Conflict indicator
                            if mod_info.has_conflicts {
                                ui.colored_label(egui::Color32::RED, "!");
                            } else if mod_info.has_warnings {
                                ui.colored_label(egui::Color32::YELLOW, "!");
                            }

                            // Mod name and version
                            let text = format!("{} v{}",
                                mod_info.metadata.name,
                                mod_info.metadata.version);

                            if ui.selectable_label(
                                mod_manager.selected_mod == Some(i),
                                &text,
                            ).clicked() {
                                mod_manager.selected_mod = Some(i);
                            }

                            // Update available indicator
                            if mod_info.update_available {
                                if ui.small_button("Update").clicked() {
                                    // Trigger update
                                }
                            }
                        });
                    }
                });

                // Right panel: selected mod details
                if let Some(idx) = mod_manager.selected_mod {
                    let mod_info = &mod_manager.installed_mods[idx];
                    columns[1].heading(&mod_info.metadata.name);
                    columns[1].label(format!("Version: {}", mod_info.metadata.version));
                    columns[1].label(format!("Author: {}", mod_info.metadata.id));
                    columns[1].separator();
                    columns[1].label(&mod_info.metadata.description);
                    columns[1].separator();

                    // Dependencies
                    columns[1].label("Dependencies:");
                    for dep in &mod_info.metadata.dependencies {
                        let status = if mod_manager.is_mod_loaded(&dep.mod_id) {
                            "Installed"
                        } else if dep.optional {
                            "Optional (not installed)"
                        } else {
                            "MISSING"
                        };
                        columns[1].label(format!("  {} {} [{}]",
                            dep.mod_id, dep.version_req, status));
                    }
                }
            });

            ui.separator();

            // Conflict summary
            let conflicts = mod_manager.check_all_conflicts();
            if !conflicts.is_empty() {
                ui.colored_label(egui::Color32::RED,
                    format!("{} conflict(s) detected:", conflicts.len()));
                for conflict in &conflicts {
                    ui.label(format!("  {} <-> {}: {}",
                        conflict.mod_a, conflict.mod_b, conflict.reason));
                }
            }

            // Apply button
            ui.horizontal(|ui| {
                if ui.button("Apply Changes").clicked() {
                    mod_manager.apply_pending_changes();
                }
                if ui.button("Open Mod Folder").clicked() {
                    opener::open(mod_manager.mods_directory()).ok();
                }
            });
        });
}
```

---

## 6. Sandboxing and Security

### 6.1 Threat Model

Mods are user-generated code running on the player's machine. The threat model includes:

1. **Malicious mods**: A mod disguised as a building pack that exfiltrates save files, installs keyloggers, or mines cryptocurrency. This is the primary threat.

2. **Accidental damage**: A well-intentioned mod with an infinite loop that freezes the game, or a memory leak that crashes after an hour.

3. **Data corruption**: A mod that writes invalid data to game state, corrupting save files.

4. **Resource exhaustion**: A mod that allocates unbounded memory, spawns millions of entities, or runs expensive computations every tick.

5. **Privacy leakage**: A mod that reads local files (save games, screenshots) and phones home to a server.

The severity varies by mod type:

| Mod Type | Can access filesystem? | Can access network? | Can crash game? | Can corrupt saves? |
|----------|----------------------|--------------------|-----------------|--------------------|
| Data-only | No | No | No (validated) | No (schema-checked) |
| Lua script | No (sandboxed) | No (sandboxed) | Yes (infinite loop) | Yes (bad mutations) |
| WASM module | No (by design) | No (by design) | Yes (fuel exhaustion) | Yes (bad commands) |
| Native plugin | YES | YES | YES | YES |

### 6.2 WASM Sandboxing

WASM provides the strongest sandboxing guarantees because isolation is baked into the specification, not bolted on.

**What WASM modules cannot do (by specification):**
- Access the host filesystem (no `open()`, `read()`, `write()`)
- Make network calls (no `connect()`, `send()`, `recv()`)
- Access other processes or threads on the host
- Execute arbitrary system calls
- Access memory outside their linear memory space
- Call host functions that aren't explicitly imported

**What we additionally restrict:**
- No WASI imports (we don't provide `wasi_snapshot_preview1` functions)
- No threading (no `SharedArrayBuffer`, no `Atomics`)
- No custom sections above 1MB (prevents data exfiltration via bloated WASM files)

**Resource limits via wasmtime:**

```rust
// crates/mod-host/src/wasm_sandbox.rs

pub struct WasmSandboxConfig {
    /// Maximum memory pages (1 page = 64KB)
    /// 256 pages = 16MB, which is generous for a mod
    pub max_memory_pages: u64,

    /// Fuel per tick (1 fuel ~ 1 CPU instruction)
    /// 10,000,000 fuel = roughly 10ms of CPU time on a modern processor
    pub fuel_per_tick: u64,

    /// Maximum execution time per tick (hard timeout)
    /// Even if fuel hasn't run out, kill after this many milliseconds
    pub max_tick_duration_ms: u64,

    /// Maximum stack depth (prevents recursive stack overflow)
    pub max_stack_size: usize,

    /// Maximum number of WASM instances this mod can spawn
    pub max_instances: u32,

    /// Maximum size of the command queue (prevents memory bomb via commands)
    pub max_commands_per_tick: usize,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 256,        // 16MB
            fuel_per_tick: 10_000_000,    // ~10ms
            max_tick_duration_ms: 50,     // 50ms hard limit
            max_stack_size: 1024 * 1024,  // 1MB
            max_instances: 1,
            max_commands_per_tick: 1000,
        }
    }
}

impl WasmSandboxConfig {
    /// Permissive config for trusted mods (e.g., official DLC)
    pub fn trusted() -> Self {
        Self {
            max_memory_pages: 1024,       // 64MB
            fuel_per_tick: 100_000_000,   // ~100ms
            max_tick_duration_ms: 200,
            max_stack_size: 4 * 1024 * 1024,
            max_instances: 4,
            max_commands_per_tick: 10_000,
        }
    }

    /// Restrictive config for untrusted mods
    pub fn untrusted() -> Self {
        Self {
            max_memory_pages: 64,         // 4MB
            fuel_per_tick: 1_000_000,     // ~1ms
            max_tick_duration_ms: 10,
            max_stack_size: 256 * 1024,   // 256KB
            max_instances: 1,
            max_commands_per_tick: 100,
        }
    }
}
```

**Epoch-based interruption** (the nuclear option):

If a WASM module gets stuck in an infinite loop and fuel metering fails to catch it (extremely unlikely but possible with certain WASM constructs), wasmtime's epoch interruption provides a hard kill:

```rust
// In the mod host's main loop:
fn tick_wasm_mods(
    engine: &Engine,
    mods: &mut [WasmModRuntime],
    snapshot: &GameSnapshot,
) {
    // Increment epoch (allows background thread to interrupt stuck mods)
    engine.increment_epoch();

    for wasm_mod in mods.iter_mut() {
        let start = std::time::Instant::now();

        match wasm_mod.tick(snapshot) {
            Ok(commands) => {
                // Validate commands before applying
                let valid_commands = validate_commands(commands, &wasm_mod.mod_id);
                apply_commands(valid_commands);
            }
            Err(WasmError::OutOfFuel) => {
                warn!("Mod {} exceeded CPU budget, skipping tick", wasm_mod.mod_id);
                // Don't disable the mod -- just skip this tick
                // If it happens repeatedly, warn the user
            }
            Err(WasmError::Execution(e)) => {
                error!("Mod {} execution error: {}", wasm_mod.mod_id, e);
                // Disable the mod after 3 consecutive errors
            }
        }

        let elapsed = start.elapsed();
        if elapsed > std::time::Duration::from_millis(50) {
            warn!("Mod {} took {}ms this tick (budget: 50ms)",
                wasm_mod.mod_id, elapsed.as_millis());
        }
    }
}
```

### 6.3 Lua Sandboxing

Lua sandboxing is weaker than WASM because it relies on removing dangerous functions rather than having a fundamentally isolated execution model. However, it is well-understood and battle-tested (WoW has used sandboxed Lua since 2004).

**Standard library restrictions:**

```rust
fn create_sandboxed_lua() -> Lua {
    let lua = Lua::new();

    // Remove dangerous modules entirely
    let globals = lua.globals();
    globals.set("os", LuaNil).unwrap();        // No OS access
    globals.set("io", LuaNil).unwrap();        // No file I/O
    globals.set("debug", LuaNil).unwrap();     // No debug hooks (can escape sandbox)
    globals.set("loadfile", LuaNil).unwrap();  // No loading files
    globals.set("dofile", LuaNil).unwrap();    // No executing files
    globals.set("load", LuaNil).unwrap();      // No loading bytecode (can contain exploits)
    globals.set("rawget", LuaNil).unwrap();    // No bypassing metatables
    globals.set("rawset", LuaNil).unwrap();
    globals.set("rawequal", LuaNil).unwrap();
    globals.set("rawlen", LuaNil).unwrap();
    globals.set("collectgarbage", LuaNil).unwrap(); // No GC manipulation
    globals.set("newproxy", LuaNil).unwrap();

    // Replace `require` with a safe module loader that only loads
    // approved modules from the mod's own directory
    globals.set("require", lua.create_function(|lua, module_name: String| {
        // Only allow loading from a whitelist of safe modules
        match module_name.as_str() {
            "math" | "string" | "table" | "utf8" => {
                // These are safe standard modules
                lua.globals().get::<LuaValue>(&module_name)
            }
            _ => {
                Err(LuaError::RuntimeError(
                    format!("Module '{}' is not available in sandboxed mode", module_name)
                ))
            }
        }
    }).unwrap()).unwrap();

    // Replace `print` with a logging function that goes to our log system
    globals.set("print", lua.create_function(|_, args: LuaMultiValue| {
        let message: Vec<String> = args.iter()
            .map(|v| format!("{:?}", v))
            .collect();
        info!("[Lua] {}", message.join("\t"));
        Ok(())
    }).unwrap()).unwrap();

    lua
}
```

**Instruction counting (CPU budget):**

mlua supports hook-based instruction counting:

```rust
fn set_lua_cpu_budget(lua: &Lua, max_instructions: u32) {
    lua.set_hook(
        mlua::HookTriggers::every_nth_instruction(1000),  // Check every 1000 instructions
        move |_lua, _debug| {
            // This is called every 1000 Lua instructions
            // Use a thread-local counter to track total instructions
            INSTRUCTION_COUNT.with(|count| {
                let current = count.get() + 1000;
                count.set(current);
                if current > max_instructions {
                    Err(LuaError::RuntimeError(
                        "Script exceeded instruction limit".to_string()
                    ))
                } else {
                    Ok(())
                }
            })
        },
    ).unwrap();
}
```

**Memory limits:**

```rust
fn set_lua_memory_limit(lua: &Lua, max_bytes: usize) {
    lua.set_memory_limit(max_bytes);
    // mlua will return LuaError::MemoryError if the limit is exceeded
}
```

**Known sandbox escape vectors in Lua (and how we prevent them):**

1. **`debug.getinfo` / `debug.sethook`**: Can inspect and modify the Lua VM internals. **Mitigation**: Remove the `debug` library entirely.

2. **`load(bytecode)`**: Lua bytecode can contain crafted sequences that exploit VM bugs. **Mitigation**: Remove `load`, `loadfile`, `dofile`.

3. **`string.rep(huge_string, huge_number)`**: Memory bomb. **Mitigation**: Set memory limit, set max string size.

4. **Infinite coroutine resume**: Create a coroutine that never yields and never returns. **Mitigation**: Instruction counting catches this.

5. **Metatable abuse**: Override `__index` / `__newindex` on shared tables to intercept internal values. **Mitigation**: Remove `rawget`/`rawset`, freeze all API tables with `__metatable = false`.

### 6.4 Native Plugin Risks

Native plugins (.dll/.so/.dylib) run with full process privileges. There is no sandbox. A native plugin can:

- Read/write any file the process can access
- Make network connections
- Spawn child processes
- Access arbitrary memory in the game process
- Call any OS API

**Mitigations (none are perfect):**

1. **Code signing**: Require native plugins to be digitally signed. This does not prevent malicious code but creates accountability.

2. **Community review**: Prominent native plugins get community code review (like TM:PE's open-source development).

3. **Capability warnings**: When enabling a native plugin, show a warning dialog: "This mod has full system access. Only install mods from trusted sources."

4. **Antivirus integration**: On Windows, submit new native plugins to Windows Defender SmartScreen for reputation-based blocking.

5. **OS-level sandboxing** (future): On macOS, use App Sandbox entitlements to restrict file access. On Linux, use seccomp-bpf to block dangerous syscalls. On Windows, use process isolation with restricted tokens. This is complex to implement but provides real security.

**Recommendation**: For launch, native plugins are an opt-in feature with prominent security warnings. Encourage mod developers to use WASM for performance-sensitive mods and Lua for everything else. Reserve native plugins for the TM:PE-class mods that genuinely need deep engine access, and feature community-audited versions prominently.

### 6.5 Resource Limits

Every mod type needs resource limits to prevent denial-of-service (intentional or accidental).

**Per-mod resource budget:**

```rust
// crates/mod-host/src/resource_budget.rs

#[derive(Debug, Clone)]
pub struct ModResourceBudget {
    /// Maximum CPU time per tick (microseconds)
    pub cpu_budget_us: u64,

    /// Maximum memory allocation (bytes)
    pub memory_budget_bytes: usize,

    /// Maximum number of entities this mod can spawn per tick
    pub max_entity_spawns_per_tick: u32,

    /// Maximum number of entities this mod can have alive total
    pub max_total_entities: u32,

    /// Maximum number of commands (mutations) per tick
    pub max_commands_per_tick: u32,

    /// Maximum number of events this mod can emit per tick
    pub max_events_per_tick: u32,

    /// Maximum log output per tick (bytes) -- prevents log spam
    pub max_log_bytes_per_tick: usize,
}

impl Default for ModResourceBudget {
    fn default() -> Self {
        Self {
            cpu_budget_us: 5_000,           // 5ms per tick
            memory_budget_bytes: 16 * 1024 * 1024,  // 16MB
            max_entity_spawns_per_tick: 10,
            max_total_entities: 10_000,
            max_commands_per_tick: 100,
            max_events_per_tick: 50,
            max_log_bytes_per_tick: 4096,
        }
    }
}

/// Tracks actual resource usage per mod
pub struct ModResourceTracker {
    pub mod_id: String,
    pub budget: ModResourceBudget,

    // Current tick usage
    pub cpu_used_us: u64,
    pub memory_used_bytes: usize,
    pub entities_spawned_this_tick: u32,
    pub total_entities: u32,
    pub commands_this_tick: u32,
    pub events_this_tick: u32,
    pub log_bytes_this_tick: usize,

    // Historical data for performance monitoring
    pub cpu_history: VecDeque<u64>,     // Last 100 ticks
    pub peak_memory: usize,
    pub violation_count: u32,           // Cumulative budget violations
}

impl ModResourceTracker {
    pub fn begin_tick(&mut self) {
        self.cpu_used_us = 0;
        self.entities_spawned_this_tick = 0;
        self.commands_this_tick = 0;
        self.events_this_tick = 0;
        self.log_bytes_this_tick = 0;
    }

    pub fn check_cpu_budget(&self) -> bool {
        self.cpu_used_us < self.budget.cpu_budget_us
    }

    pub fn check_spawn_budget(&self) -> bool {
        self.entities_spawned_this_tick < self.budget.max_entity_spawns_per_tick
            && self.total_entities < self.budget.max_total_entities
    }

    pub fn record_violation(&mut self, violation_type: &str) {
        self.violation_count += 1;
        warn!(
            "Mod {} resource violation #{}: {}",
            self.mod_id, self.violation_count, violation_type
        );

        // Auto-disable mod after 100 violations
        if self.violation_count >= 100 {
            error!(
                "Mod {} disabled after {} resource violations",
                self.mod_id, self.violation_count
            );
        }
    }
}
```

**Aggregate budget across all mods:**

Even if each mod stays within its budget, 50 mods each using 5ms per tick = 250ms, which would tank the frame rate. We need a global budget:

```rust
pub struct GlobalModBudget {
    /// Total CPU time available for all mods per tick
    /// At 10Hz FixedUpdate, we have 100ms per tick
    /// Reserve 80ms for game systems, leave 20ms for mods
    pub total_cpu_budget_us: u64,

    /// Total memory for all mods combined
    pub total_memory_budget_bytes: usize,

    /// If total mod CPU exceeds this, start throttling (skipping ticks)
    pub throttle_threshold_us: u64,
}

impl Default for GlobalModBudget {
    fn default() -> Self {
        Self {
            total_cpu_budget_us: 20_000,              // 20ms total for all mods
            total_memory_budget_bytes: 256 * 1024 * 1024,  // 256MB total
            throttle_threshold_us: 15_000,            // Start throttling at 15ms
        }
    }
}
```

When the global budget is exceeded, the mod host reduces per-mod budgets proportionally and starts skipping low-priority mods every other tick (data-only mods are never skipped; script mods are throttled first; native mods cannot be throttled).

---

## 7. Data-Driven Architecture

### 7.1 Making Game Data Moddable Without Code

The most important insight for modding architecture is: **every hardcoded value is a missed modding opportunity.** The more game behavior is driven by data files rather than compiled code, the easier it is to mod, the fewer compatibility issues arise, and the more accessible modding becomes to non-programmers.

Here is an inventory of values currently hardcoded in the Megacity codebase that should become data-driven:

**Building definitions** (currently in `crates/simulation/src/buildings.rs`):

```rust
// CURRENT: hardcoded match statement
pub fn capacity_for_level(zone: ZoneType, level: u8) -> u32 {
    match (zone, level) {
        (ZoneType::ResidentialLow, 1) => 10,
        (ZoneType::ResidentialLow, 2) => 30,
        (ZoneType::ResidentialLow, 3) => 80,
        (ZoneType::ResidentialHigh, 1) => 50,
        // ... 20 more entries
    }
}
```

```toml
# PROPOSED: data file (data/buildings/base_game.toml)
[[building_template]]
zone = "ResidentialLow"
level = 1
capacity = 10
mesh = "buildings/res_low_1.glb"
height = 8.0
construction_time = 50

[[building_template]]
zone = "ResidentialLow"
level = 2
capacity = 30
mesh = "buildings/res_low_2.glb"
height = 12.0
construction_time = 80

[[building_template]]
zone = "ResidentialHigh"
level = 1
capacity = 50
mesh = "buildings/res_high_1.glb"
height = 20.0
construction_time = 100
```

**Road type definitions** (currently in `crates/simulation/src/grid.rs`):

```rust
// CURRENT: hardcoded enum methods
impl RoadType {
    pub fn speed(self) -> f32 {
        match self {
            RoadType::Local => 30.0,
            RoadType::Avenue => 50.0,
            RoadType::Boulevard => 60.0,
            RoadType::Highway => 100.0,
            RoadType::OneWay => 40.0,
            RoadType::Path => 5.0,
        }
    }
}
```

Already addressed in Section 4.5. The `RoadTypeRegistry` loads from TOML.

**Policy definitions** (currently in `crates/simulation/src/policies.rs`):

```rust
// CURRENT: hardcoded enum
pub enum Policy {
    FreePublicTransport,
    HeavyIndustryTaxBreak,
    // ... 15 variants
}

impl Policy {
    pub fn monthly_cost(self) -> f64 {
        match self {
            Policy::FreePublicTransport => 50.0,
            // ...
        }
    }
}
```

```toml
# PROPOSED: data file (data/policies/base_game.toml)
[[policy]]
id = "free_public_transport"
name = "Free Public Transport"
description = "Public transit is free for all citizens"
monthly_cost = 50.0
category = "economy"

[policy.effects]
happiness_bonus = 5.0
traffic_reduction = 0.15
commercial_demand_bonus = 0.05
tourism_bonus = 0.10

[policy.requirements]
min_population = 5000
requires_service = "BusDepot"
```

**Service building definitions** (currently in `crates/simulation/src/services.rs`):

```toml
# PROPOSED: data/services/base_game.toml
[[service]]
id = "fire_station"
name = "Fire Station"
category = "safety"
coverage_radius = 30
build_cost = 15000.0
monthly_upkeep = 100.0
workers = 20
effectiveness = 1.0

[service.effects]
fire_protection = 0.8
land_value_bonus = 2.0
noise = 3

[[service]]
id = "elementary_school"
name = "Elementary School"
category = "education"
coverage_radius = 25
build_cost = 10000.0
monthly_upkeep = 80.0
workers = 15
student_capacity = 200

[service.effects]
education_coverage = 1.0
land_value_bonus = 5.0
```

**Zone parameters** (currently scattered across multiple files):

```toml
# PROPOSED: data/zones/base_game.toml
[[zone]]
id = "residential_low"
name = "Low Density Residential"
color = "#00AA00"
max_level = 3
growth_speed = 1.0
demand_factor = 1.0
tax_rate_base = 0.08

[zone.requirements]
adjacent_to_road = true
min_road_type = "Local"
needs_power = true
needs_water = true

[zone.growth]
# Land value thresholds for leveling up
level_2_land_value = 30
level_3_land_value = 60
# Service requirements for leveling up
level_2_services = ["ElementarySchool"]
level_3_services = ["HighSchool", "SmallPark"]
```

**Citizen simulation parameters** (currently hardcoded in movement, lifecycle, happiness):

```toml
# PROPOSED: data/simulation/citizen_params.toml
[commuting]
commute_start_hour = 7.0
commute_end_hour = 9.0
return_start_hour = 17.0
return_end_hour = 19.0
walk_speed = 1.5       # cells per second
drive_speed_factor = 1.0  # multiplier on road speed

[lifecycle]
min_working_age = 18
retirement_age = 65
max_age = 95
base_emigration_rate = 0.001
base_birth_rate = 0.002

[happiness]
weight_commute = 0.15
weight_services = 0.20
weight_pollution = 0.15
weight_crime = 0.10
weight_employment = 0.15
weight_land_value = 0.10
weight_noise = 0.05
weight_education = 0.10

[needs]
hunger_rate = 0.01       # per tick
energy_rate = 0.008
social_rate = 0.005
shopping_rate = 0.003
```

### 7.2 Bevy's Asset System for Data Files

Bevy's asset system (`AssetServer`, `Handle<T>`, `AssetLoader`) is designed for exactly this use case. We can define custom asset types for our data definitions and load them like any other asset.

```rust
// crates/simulation/src/data_assets.rs

use bevy::asset::{AssetLoader, LoadContext};
use serde::Deserialize;

/// A collection of building templates loaded from a TOML file
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct BuildingTemplateAsset {
    pub building_template: Vec<BuildingTemplate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildingTemplate {
    pub zone: String,
    pub level: u8,
    pub capacity: u32,
    pub mesh: String,
    pub height: f32,
    pub construction_time: u32,
}

/// Asset loader for TOML data files
pub struct TomlAssetLoader;

impl AssetLoader for TomlAssetLoader {
    type Asset = BuildingTemplateAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn std::io::Read,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let asset: BuildingTemplateAsset = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["buildings.toml"]
    }
}

/// System that loads data assets on startup
pub fn load_data_assets(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // Load base game data
    let buildings: Handle<BuildingTemplateAsset> = asset_server.load("data/buildings/base_game.buildings.toml");
    commands.insert_resource(BuildingTemplateHandle(buildings));

    // Load mod data (overrides base)
    // The mod host will call this for each enabled mod's data files
}
```

**Benefits of using Bevy's asset system:**

1. **Automatic hot-reloading**: In dev mode, changing a `.toml` file automatically reloads it
2. **Async loading**: Data files load in the background without blocking the main thread
3. **Handle-based indirection**: Systems reference data via `Handle<T>`, so reloading is transparent
4. **Dependency tracking**: Bevy tracks which assets depend on which files
5. **Error handling**: Failed loads are reported without crashing

### 7.3 Override Hierarchy

When multiple mods modify the same data, we need a clear override hierarchy:

```
Layer 0: Base game data (data/*)
Layer 1: First mod's data (mods/mod_a/data/*)
Layer 2: Second mod's data (mods/mod_b/data/*)
...
Layer N: Last mod's data (mods/mod_n/data/*)
```

**Override semantics:**

The override system must support three operations:

1. **Add**: A mod adds new entries (new building types, new road types). This is always safe -- no conflict with base game or other mods (unless IDs collide).

2. **Replace**: A mod replaces an existing entry entirely. The mod provides a complete definition that replaces the base game's definition. Later mods in load order win.

3. **Patch**: A mod modifies specific fields of an existing entry without replacing the whole thing. This is the most flexible and least conflict-prone approach.

```toml
# Mod's data/patches/building_patches.toml

# Patch operation: modify specific fields of existing entries
[[patch]]
target = "buildings/residential_high_3"
field = "capacity"
value = 600          # Was 500 in base game

[[patch]]
target = "buildings/residential_high_3"
field = "construction_time"
value = 150          # Was 100 in base game

# Add operation: entirely new entry
[[add]]
category = "buildings"
id = "com.author.eco_tower"
[add.data]
zone = "ResidentialHigh"
level = 4
capacity = 800
mesh = "buildings/eco_tower.glb"
height = 50.0
construction_time = 200

# Replace operation: complete replacement
[[replace]]
target = "zones/residential_low"
[replace.data]
name = "Suburban Residential"
color = "#88CC88"
max_level = 5           # Was 3
growth_speed = 0.8      # Slower growth
```

**Patch application engine:**

```rust
// crates/mod-host/src/data_patcher.rs

pub struct DataPatcher;

impl DataPatcher {
    /// Apply patches from all mods in load order
    pub fn apply_patches(
        base_data: &mut DataRegistry,
        mod_patches: &[(String, Vec<DataPatch>)],  // (mod_id, patches)
    ) -> Vec<PatchResult> {
        let mut results = Vec::new();

        for (mod_id, patches) in mod_patches {
            for patch in patches {
                match patch {
                    DataPatch::Add { category, id, data } => {
                        match base_data.register_raw(category, id, data.clone(), mod_id) {
                            Ok(()) => results.push(PatchResult::Applied {
                                mod_id: mod_id.clone(),
                                operation: format!("add {}/{}", category, id),
                            }),
                            Err(e) => results.push(PatchResult::Conflict {
                                mod_id: mod_id.clone(),
                                operation: format!("add {}/{}", category, id),
                                reason: format!("{:?}", e),
                            }),
                        }
                    }
                    DataPatch::Replace { target, data } => {
                        if base_data.replace_raw(target, data.clone(), mod_id) {
                            results.push(PatchResult::Applied {
                                mod_id: mod_id.clone(),
                                operation: format!("replace {}", target),
                            });
                        } else {
                            results.push(PatchResult::Warning {
                                mod_id: mod_id.clone(),
                                operation: format!("replace {}", target),
                                reason: "Target not found".to_string(),
                            });
                        }
                    }
                    DataPatch::Patch { target, field, value } => {
                        if base_data.patch_field(target, field, value.clone(), mod_id) {
                            results.push(PatchResult::Applied {
                                mod_id: mod_id.clone(),
                                operation: format!("patch {}.{}", target, field),
                            });
                        } else {
                            results.push(PatchResult::Warning {
                                mod_id: mod_id.clone(),
                                operation: format!("patch {}.{}", target, field),
                                reason: "Target or field not found".to_string(),
                            });
                        }
                    }
                }
            }
        }

        results
    }
}
```

### 7.4 Current Hardcoded Values That Must Become Data

A comprehensive audit of the Megacity codebase reveals these hardcoded values that block modding. This is prioritized by impact:

**Priority 1 (blocks the most common mods):**

| File | Value | Current | Should Be |
|------|-------|---------|-----------|
| `config.rs` | `GRID_WIDTH`, `GRID_HEIGHT` | `256` const | Runtime `GridConfig` |
| `grid.rs` | `RoadType` enum | 6 variants | `RoadTypeRegistry` |
| `grid.rs` | `ZoneType` enum | 7 variants | `ZoneTypeRegistry` |
| `buildings.rs` | `capacity_for_level()` | Match statement | Data file |
| `buildings.rs` | `SPAWN_INTERVAL` | `2` const | Configurable |
| `policies.rs` | `Policy` enum | 15 variants | `PolicyRegistry` |
| `services.rs` | `ServiceType` enum | Hardcoded | `ServiceRegistry` |
| `services.rs` | `coverage_radius()` | Match statement | Data field |

**Priority 2 (blocks gameplay mods):**

| File | Value | Current | Should Be |
|------|-------|---------|-----------|
| `traffic.rs` | Congestion thresholds | `20.0` divisor | Config |
| `road_graph_csr.rs` | Edge cost calculation | Hardcoded formula | Pluggable function |
| `happiness.rs` | Happiness weights | Hardcoded floats | Config file |
| `economy.rs` | Tax rates, income formulas | Hardcoded | Config file |
| `lifecycle.rs` | Age thresholds, emigration rates | Hardcoded | Config file |
| `citizen_spawner.rs` | Spawn rates and caps | Hardcoded | Config file |
| `land_value.rs` | Land value factors | Hardcoded | Config file |
| `pollution.rs` | Pollution diffusion rates | Hardcoded | Config file |
| `crime.rs` | Crime rates, police effectiveness | Hardcoded | Config file |

**Priority 3 (blocks cosmetic/niche mods):**

| File | Value | Current | Should Be |
|------|-------|---------|-----------|
| `time_of_day.rs` | Day length, season timing | Hardcoded | Config |
| `weather.rs` | Weather probabilities | Hardcoded | Config |
| `terrain.rs` | Terrain generation params | Hardcoded | Config |
| `tourism.rs` | Tourism factors | Hardcoded | Config |
| `natural_resources.rs` | Resource generation | Hardcoded | Config |

The total refactoring scope is substantial but can be done incrementally. Priority 1 items should be completed before modding support launches. Priority 2 can come in the first modding update. Priority 3 can be community-driven (modders request what they need).

---

## 8. Backward Compatibility

### 8.1 Stable Mod API Versioning Strategy

The mod SDK follows **semantic versioning (SemVer)** strictly:

- **Major version** (2.0.0): Breaking changes to the mod API. Mods compiled against SDK 1.x will not work with SDK 2.x. This should happen rarely (every 2-3 years at most).
- **Minor version** (1.2.0): New features added to the mod API. Mods compiled against SDK 1.1 will work with SDK 1.2. New functions, new events, new data fields.
- **Patch version** (1.1.3): Bug fixes only. No API changes.

**Version compatibility checking:**

```rust
// At mod load time:
fn check_sdk_compatibility(
    mod_metadata: &ModMetadata,
    game_sdk_version: &semver::Version,
) -> CompatibilityResult {
    if !mod_metadata.sdk_version.matches(game_sdk_version) {
        return CompatibilityResult::Incompatible {
            mod_requires: mod_metadata.sdk_version.clone(),
            game_provides: game_sdk_version.clone(),
            suggestion: if game_sdk_version.major > mod_metadata.sdk_version.major() {
                "This mod was built for an older version of Megacity. Check for an updated version."
            } else {
                "This mod requires a newer version of Megacity. Please update the game."
            }.to_string(),
        };
    }

    // Check for deprecation warnings
    let warnings = check_deprecated_api_usage(&mod_metadata);
    if warnings.is_empty() {
        CompatibilityResult::FullyCompatible
    } else {
        CompatibilityResult::CompatibleWithWarnings(warnings)
    }
}
```

**API deprecation process:**

1. In SDK 1.2: Function `get_traffic_level()` is marked `#[deprecated(since = "1.2.0", note = "Use get_congestion() instead")]`
2. In SDK 1.x: The deprecated function continues to work, just logs a warning
3. In SDK 2.0: The deprecated function is removed

**Feature flags for incremental API expansion:**

```rust
// In mod-sdk/src/lib.rs

/// Features available in SDK 1.0
pub mod v1_0 {
    pub use super::core::*;
    pub use super::buildings::*;
    pub use super::traffic::*;
}

/// Features added in SDK 1.1
pub mod v1_1 {
    pub use super::v1_0::*;
    pub use super::vehicles::*;
    pub use super::districts::*;
}

/// Features added in SDK 1.2
pub mod v1_2 {
    pub use super::v1_1::*;
    pub use super::public_transit::*;
    pub use super::custom_zones::*;
}

/// Latest stable API
pub mod prelude {
    pub use super::v1_2::*;
}
```

### 8.2 Evolving Internal Systems Without Breaking Mods

The SDK facade pattern (Section 1.2) is the primary mechanism. But there are additional strategies:

**Strategy 1: Adapter layers for system refactors**

When we refactor an internal system, we add an adapter that translates between the old SDK API and the new internal implementation:

```rust
// Example: we refactor TrafficGrid from flat Vec<u16> to chunked storage

// Old internal API (removed):
// pub struct TrafficGrid { pub density: Vec<u16>, ... }

// New internal API:
pub struct ChunkedTrafficGrid {
    chunks: Vec<TrafficChunk>,
    chunk_size: usize,
}

// Adapter in mod-host/src/bridge.rs:
impl TrafficApi for ChunkedTrafficGridAdapter {
    fn get_density(&self, x: usize, y: usize) -> u16 {
        // Translate SDK call to new internal API
        let chunk_x = x / self.grid.chunk_size;
        let chunk_y = y / self.grid.chunk_size;
        let local_x = x % self.grid.chunk_size;
        let local_y = y % self.grid.chunk_size;
        self.grid.chunks[chunk_y * self.chunks_x + chunk_x]
            .density[local_y * self.grid.chunk_size + local_x]
    }
    // ... etc
}
```

**Strategy 2: Event-driven APIs age better than polling APIs**

Instead of exposing `get_traffic_density(x, y)` which assumes a grid structure, expose events like `on_congestion_changed(segment_id, old_level, new_level)`. Events don't encode implementation details; they describe what happened, not how it was stored.

**Strategy 3: Capability-based feature detection**

```rust
// Mods can check what features are available at runtime
pub trait GameCapabilities {
    fn supports_feature(&self, feature: &str) -> bool;
    fn sdk_version(&self) -> &semver::Version;
    fn game_version(&self) -> &semver::Version;
}

// Usage in a mod:
fn on_game_start(caps: &dyn GameCapabilities) {
    if caps.supports_feature("custom_zones") {
        // Use custom zone API
    } else {
        // Fallback for older game versions
    }
}
```

### 8.3 Save File Compatibility with Mods

Save file compatibility is one of the hardest problems in modded games. CS1 players routinely lost saves when mods updated or were removed. We must do better.

**Problem 1: Mods add new components to entities**

A mod adds a `CustomBehavior` component to some citizens. When the mod is removed, entities with `CustomBehavior` exist in the save but the component type is unknown.

**Solution: Tagged component storage for mod data**

```rust
// Instead of mods adding arbitrary ECS components, mod data is stored
// in a generic container that survives mod removal:

#[derive(Component, Serialize, Deserialize)]
pub struct ModData {
    /// mod_id -> serialized data blob
    entries: HashMap<String, Vec<u8>>,
}

impl ModData {
    pub fn set<T: Serialize>(&mut self, mod_id: &str, data: &T) {
        let bytes = bitcode::encode(data);
        self.entries.insert(mod_id.to_string(), bytes);
    }

    pub fn get<T: DeserializeOwned>(&self, mod_id: &str) -> Option<T> {
        self.entries.get(mod_id)
            .and_then(|bytes| bitcode::decode(bytes).ok())
    }

    pub fn remove(&mut self, mod_id: &str) {
        self.entries.remove(mod_id);
    }
}
```

When a mod is removed, its entries in `ModData` are preserved but ignored. If the mod is re-enabled, its data is still there.

**Problem 2: Mods modify base game data (building capacities, road speeds)**

A mod changes all residential building capacities to 2x. The save stores the modified values. When the mod is removed, buildings have inflated capacities.

**Solution: Save the delta, not the final value**

```rust
#[derive(Serialize, Deserialize)]
pub struct ModOverrides {
    /// Ordered list of overrides, applied in order
    overrides: Vec<SavedOverride>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedOverride {
    pub mod_id: String,
    pub target: String,          // "building/residential_high/capacity"
    pub operation: OverrideOp,
}

#[derive(Serialize, Deserialize)]
pub enum OverrideOp {
    Set(f64),                    // Absolute value
    Multiply(f64),               // Multiplier
    Add(f64),                    // Additive
}
```

When loading a save, only overrides from currently-enabled mods are applied. Disabled mods' overrides are stored but skipped.

**Problem 3: Mods add custom entity types**

A mod adds "Tram" entities that don't exist in the base game. Removing the mod leaves orphaned tram entities.

**Solution: Mod entity registry with cleanup hooks**

```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct ModEntityRegistry {
    /// mod_id -> list of entities created by this mod
    entities: HashMap<String, Vec<Entity>>,
}

impl ModEntityRegistry {
    pub fn register_entity(&mut self, mod_id: &str, entity: Entity) {
        self.entities.entry(mod_id.to_string())
            .or_default()
            .push(entity);
    }

    /// Called when a mod is disabled: despawn all its entities
    pub fn cleanup_mod(&mut self, mod_id: &str, commands: &mut Commands) {
        if let Some(entities) = self.entities.remove(mod_id) {
            for entity in entities {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
```

**Save file format with mod metadata:**

```rust
#[derive(Serialize, Deserialize)]
pub struct SaveFileV3 {
    pub version: u32,
    pub game_version: String,

    // Mod information embedded in save
    pub active_mods: Vec<SavedModInfo>,

    // Base game state
    pub grid: SavedGrid,
    pub clock: SavedClock,
    pub budget: SavedBudget,
    // ... etc

    // Mod-specific state (opaque blobs keyed by mod ID)
    pub mod_state: HashMap<String, Vec<u8>>,

    // Mod overrides (so we know what was modified and by whom)
    pub mod_overrides: ModOverrides,

    // Mod entity registry
    pub mod_entities: HashMap<String, Vec<u64>>,  // mod_id -> entity IDs
}

#[derive(Serialize, Deserialize)]
pub struct SavedModInfo {
    pub mod_id: String,
    pub version: String,
    pub sdk_version: String,
}
```

**Load behavior with mod mismatches:**

| Scenario | Behavior |
|----------|----------|
| Save has mod A, mod A is still installed | Load normally, restore mod A's state |
| Save has mod A, mod A is removed | Load base game state, skip mod A's state, warn user |
| Save lacks mod B, mod B is newly installed | Load base game state, let mod B initialize fresh |
| Save has mod A v1.0, mod A v2.0 is installed | Attempt load, let mod A's migration hook run |
| Save has mod A v1.0, mod A v3.0 is installed (major ver change) | Warn user: "Mod A has been significantly updated. Mod data from save may be lost." |

**Mod migration hooks:**

```rust
pub trait MegacityMod: Plugin {
    // ... existing methods ...

    /// Called when loading a save that was created with an older version of this mod.
    /// Allows the mod to migrate its saved state to the current format.
    fn migrate_save(
        &self,
        old_version: &semver::Version,
        old_data: &[u8],
    ) -> Result<Vec<u8>, ModError> {
        // Default: return old data unchanged (no migration needed)
        Ok(old_data.to_vec())
    }
}
```

---

## 9. Implementation Roadmap

The modding architecture should be built incrementally, with each phase delivering value to modders:

### Phase 1: Data-Driven Foundation (Months 1-3)

**Goal**: Make the game moddable without any code by converting hardcoded values to data files.

1. Create `data/` directory with TOML files for all Priority 1 items (Section 7.4)
2. Implement TOML asset loaders for building templates, road types, zone params, policy definitions
3. Replace hardcoded `match` statements with registry lookups
4. Implement override hierarchy (base game -> mod data files)
5. Create the `RoadTypeRegistry` and `ZoneTypeRegistry`
6. Make `GRID_WIDTH`/`GRID_HEIGHT` runtime-configurable

**Deliverable**: Modders can create building packs, custom road types, and gameplay tweaks with TOML files only. No code needed.

### Phase 2: Mod SDK and Package Format (Months 3-5)

1. Create the `megacity-mod-sdk` crate with stable API types
2. Create the `megacity-mod-host` crate with mod loader
3. Define the `.megamod` package format
4. Implement `mod.toml` manifest parsing
5. Implement dependency resolution (topological sort)
6. Implement asset validation
7. Build basic mod manager UI (enable/disable, load order)

**Deliverable**: Mods can be packaged, shared, installed, and managed. Data-only mods work end-to-end.

### Phase 3: Lua Scripting (Months 5-7)

1. Integrate `mlua` with sandboxing
2. Implement the complete Lua API surface (Section 2.5)
3. Build the Lua bridge layer (connecting API functions to game resources)
4. Implement the command queue pattern for mutations
5. Implement event dispatch to Lua callbacks
6. Add instruction counting and memory limits
7. Support hot-reloading of Lua scripts

**Deliverable**: Modders can write Lua scripts for custom events, gameplay tweaks, and UI additions.

### Phase 4: Native Plugins and Steam Workshop (Months 7-9)

1. Implement native plugin loading via `libloading`
2. Implement hot-reloading for native plugins (dev mode)
3. Implement the TrafficApi, BuildingSpawnApi, EntityManipulationApi
4. Integrate with Steam Workshop (upload, download, subscribe)
5. Add conflict detection and resolution UI
6. Implement undo/redo system for entity manipulation

**Deliverable**: Power users can create TM:PE-class mods. Steam Workshop integration is live.

### Phase 5: WASM and Advanced Features (Months 9-12)

1. Integrate `wasmtime` with sandboxing
2. Implement WASM host functions (mirror of Lua API)
3. Build AssemblyScript SDK bindings
4. Implement per-mod resource budgets and performance monitoring
5. Build self-hosted mod repository API
6. Build in-game asset editor prototype
7. Implement save file compatibility with mods

**Deliverable**: Full modding ecosystem with multiple runtime options, distribution channels, and development tools.

### Phase 6: Community and Polish (Months 12+)

1. Modding documentation site with tutorials and API reference
2. Example mods (building pack, gameplay tweak, traffic mod)
3. Modding SDK templates (cargo-generate templates for each mod type)
4. Community mod showcase and curation
5. Mod compatibility database (crowd-sourced)
6. Performance profiler for mod developers

---

## 10. Appendix: Reference Implementations

### A. Games with exemplary modding architectures

**Factorio (Lua scripting, data-driven)**
- All game content (items, recipes, technologies, entities) defined in Lua data files
- Mods can add, modify, or remove any data definition
- Three mod loading stages: settings -> data -> data-final-fixes
- Deterministic mod loading order based on dependencies
- Built-in mod portal with API
- Exemplary backward compatibility (mods rarely break between updates)
- Key lesson: the data-stage/runtime-stage separation prevents many conflict classes

**Minecraft (Java plugins, Forge/Fabric)**
- Forge: heavy, event-driven modding with extensive hooks into game systems
- Fabric: lightweight, mixin-based modding with minimal overhead
- Mod loader handles dependency resolution and version conflicts
- Save files store mod data via NBT compound tags (similar to our `ModData` approach)
- Key lesson: Forge's heavy API surface caused breakage on every game update. Fabric's lightweight approach is more resilient.

**Rimworld (C# modding, XML data, Harmony patches)**
- Game data defined in XML (defs for items, buildings, jobs, etc.)
- Mods can add new XML defs or patch existing ones with XPath operations
- C# mods use Harmony library for runtime method patching (IL manipulation)
- Mod load order is user-configurable with auto-sort by dependencies
- Key lesson: XML patching (add/modify/remove fields) is extremely accessible to non-programmers and covers 80% of mod needs.

**Kerbal Space Program (Unity/C# modding, ModuleManager patches)**
- ModuleManager: config file patching system with regex-like selectors
- Example: `@PART[mk1pod] { @maxTemp = 2400 }` modifies a specific part's temperature
- Patches are applied in order, creating a pipeline
- Key lesson: declarative patching syntax is more accessible than imperative scripting for data modifications.

### B. Example mod implementations for Megacity

**Example 1: Pure data mod (building pack)**

```
tel-aviv-heritage-pack/
  mod.toml
  assets/
    buildings/
      ottoman-house/
        building.toml
        mesh_lod0.glb
        textures/diffuse.png
      templar-house/
        building.toml
        mesh_lod0.glb
        textures/diffuse.png
      brutalist-tower/
        building.toml
        mesh_lod0.glb
        mesh_lod1.glb
        textures/diffuse.png
        textures/normal.png
  preview.png
```

No scripts. No code. Just meshes and TOML metadata. A 3D artist can create this mod.

**Example 2: Lua gameplay mod (seasonal events)**

```
seasonal-events/
  mod.toml
  scripts/
    main.lua
  data/
    events.toml
  preview.png
```

```lua
-- scripts/main.lua

local events = {}

function events.on_day_change(day)
    local season = math.floor((day % 365) / 91)  -- 0=spring, 1=summer, 2=autumn, 3=winter

    if season == 1 and day % 7 == 0 then
        -- Summer weekend beach party
        ui.show_notification("Beach Party Weekend",
            "Citizens flock to the coast! Commercial demand +20%")
        city.modify_demand("commercial", 0.20)
    end

    if season == 3 and day % 30 == 0 then
        -- Winter storm event
        local storm_severity = math.random()
        if storm_severity > 0.7 then
            ui.show_notification("Winter Storm Warning",
                "A severe storm is approaching! Services may be disrupted.")
            -- Reduce service effectiveness temporarily
        end
    end

    -- Independence Day (day 120 = May 1st roughly)
    if day % 365 == 120 then
        ui.show_notification("Independence Day",
            "Fireworks and celebrations across the city! +30 happiness for all citizens.")
        citizens.modify_happiness_all(30)
        timer.after(24, "end_independence_day")
    end
end

function events.end_independence_day()
    citizens.modify_happiness_all(-30)  -- Remove the temporary boost
end

return events
```

**Example 3: WASM performance mod (traffic AI)**

Written in Rust, compiled to WASM:

```rust
// traffic-ai-mod/src/lib.rs
// Compiled with: cargo build --target wasm32-wasip1 --release

#[link(wasm_import_module = "megacity")]
extern "C" {
    fn traffic_get_density(x: i32, y: i32) -> i32;
    fn traffic_set_speed_override(segment_id: i32, speed: f32);
    fn get_segment_count() -> i32;
    fn get_segment_density(segment_id: i32) -> f32;
    fn get_segment_capacity(segment_id: i32) -> i32;
    fn city_get_hour() -> f32;
}

#[no_mangle]
pub extern "C" fn on_tick() {
    unsafe {
        let hour = city_get_hour();
        let segment_count = get_segment_count();

        // Dynamic speed management: reduce speed limits on congested segments
        // during rush hour to prevent gridlock (counterintuitive but effective)
        let is_rush_hour = (hour >= 7.0 && hour <= 9.5) || (hour >= 16.5 && hour <= 19.0);

        for seg_id in 0..segment_count {
            let density = get_segment_density(seg_id);
            let capacity = get_segment_capacity(seg_id) as f32;

            if capacity <= 0.0 {
                continue;
            }

            let utilization = density / capacity;

            if is_rush_hour && utilization > 0.8 {
                // Reduce speed to prevent stop-and-go waves
                // Metering: controlled flow is better than uncontrolled congestion
                let speed_factor = 1.0 - (utilization - 0.8) * 2.5;  // Linear reduction
                let min_speed = 10.0;
                let new_speed = (speed_factor * 60.0).max(min_speed);
                traffic_set_speed_override(seg_id, new_speed);
            } else if utilization < 0.3 {
                // Low utilization: remove any speed override (restore default)
                traffic_set_speed_override(seg_id, -1.0);  // -1 = remove override
            }
        }
    }
}
```

**Example 4: Native plugin (Move It tool)**

```rust
// move-it-mod/src/lib.rs
// crate-type = ["cdylib"]

use megacity_mod_sdk::prelude::*;

pub struct MoveItMod {
    undo_stack: Vec<UndoGroup>,
    selected_entities: Vec<Entity>,
    drag_origin: Option<WorldPos>,
}

impl Plugin for MoveItMod {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveItState>()
            .add_systems(Update, (
                handle_selection,
                handle_drag,
                handle_keyboard_shortcuts,
                render_selection_overlay,
                render_move_preview,
            ));
    }
}

impl MegacityMod for MoveItMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "com.community.move-it".to_string(),
            name: "Move It!".to_string(),
            version: semver::Version::new(3, 0, 0),
            sdk_version: "^1.0".parse().unwrap(),
            dependencies: vec![],
            conflicts: vec![],
            load_order: LoadOrder::Any,
        }
    }
}

#[no_mangle]
pub fn megacity_mod_entry() -> Box<dyn MegacityMod> {
    Box::new(MoveItMod {
        undo_stack: Vec::new(),
        selected_entities: Vec::new(),
        drag_origin: None,
    })
}

fn handle_selection(
    input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    mut state: ResMut<MoveItState>,
    entity_api: Res<EntityManipulationApiImpl>,
) {
    if input.just_pressed(MouseButton::Left) && state.tool_active {
        // Query entities near cursor
        let nearby = entity_api.query_entities_in_radius(
            cursor_pos.world_pos,
            16.0,  // Selection radius
            EntityFilter::default(),
        );

        if let Some(closest) = nearby.first() {
            if input.pressed(KeyCode::ShiftLeft) {
                // Add to selection
                state.selected.push(closest.entity);
            } else {
                // Replace selection
                state.selected.clear();
                state.selected.push(closest.entity);
            }
        }
    }
}

fn handle_drag(
    input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPos>,
    mut state: ResMut<MoveItState>,
    mut entity_api: ResMut<EntityManipulationApiImpl>,
) {
    if input.pressed(MouseButton::Left) && !state.selected.is_empty() {
        if let Some(origin) = state.drag_origin {
            let delta = WorldPos {
                x: cursor_pos.world_pos.x - origin.x,
                y: cursor_pos.world_pos.y - origin.y,
            };

            // Move all selected entities
            for &entity in &state.selected {
                if let Some(info) = entity_api.get_entity_info(entity) {
                    let new_pos = WorldPos {
                        x: info.position.x + delta.x,
                        y: info.position.y + delta.y,
                    };
                    entity_api.move_entity(entity, new_pos).ok();
                }
            }
        } else {
            state.drag_origin = Some(cursor_pos.world_pos);
            // Begin undo group
            state.current_batch = Some(entity_api.begin_batch());
        }
    }

    if input.just_released(MouseButton::Left) {
        if let Some(batch) = state.current_batch.take() {
            entity_api.commit_batch(batch);
        }
        state.drag_origin = None;
    }
}
```

### C. Dependency graph for modding crate implementation

```
megacity-mod-sdk (public crate, published to crates.io)
  depends on: bevy (re-exported subset), semver, serde

megacity-mod-sdk-derive (proc macros for mod authors)
  depends on: syn, quote, proc-macro2

megacity-mod-host (internal crate)
  depends on: megacity-mod-sdk, simulation, rendering, ui
  depends on: libloading (native plugin loading)
  depends on: mlua (Lua scripting)
  depends on: wasmtime (WASM runtime)
  depends on: zip (mod packaging)
  depends on: toml (manifest parsing)
  depends on: semver (version resolution)
  depends on: notify (file watching for hot-reload)
  depends on: steamworks (Steam Workshop, optional)
  depends on: reqwest (mod repository API, optional)

megacity-app (binary)
  depends on: megacity-mod-host (mod loading at startup)
  depends on: simulation, rendering, ui, save
```

### D. Key metrics for modding ecosystem health

Track these metrics to gauge modding ecosystem success:

| Metric | Target (6 months) | Target (1 year) | Target (2 years) |
|--------|-------------------|-----------------|------------------|
| Published mods (all platforms) | 100 | 1,000 | 10,000 |
| Mods with >1K subscribers | 10 | 100 | 500 |
| Unique mod authors | 50 | 500 | 2,000 |
| Data-only mods (% of total) | 60% | 50% | 40% |
| Lua script mods (% of total) | 30% | 35% | 30% |
| WASM/Native mods (% of total) | 10% | 15% | 30% |
| Average mods per player | 2 | 5 | 10 |
| Mod-related crash reports | <5% of total | <3% | <1% |
| Save files broken by mods | <2% of loads | <1% | <0.5% |
| Time from SDK release to first community mod | <1 week | - | - |
| Mod API breaking changes per year | 0 | 0-1 | 0-1 |

---

## Summary of Critical Architectural Decisions

1. **SDK crate as stable API facade**: Internal systems can evolve freely; mods only see versioned abstractions.

2. **Dual-track scripting (Lua + WASM)**: Lua for accessibility, WASM for performance. Skip Rhai.

3. **Data-driven first**: Convert all hardcoded enums and match statements to data registries. This is the highest-leverage change -- it makes 70% of mods possible without any scripting.

4. **Command queue pattern**: Mods never directly mutate game state. All writes are queued and validated.

5. **Per-mod resource budgets**: CPU, memory, entity count, and command limits prevent any single mod from degrading game performance.

6. **Save file resilience**: Mod data stored as tagged blobs that survive mod enable/disable. Overrides stored as deltas, not absolute values.

7. **RoadType and ZoneType as registries, not enums**: The single most impactful refactor for modding. Without this, the most popular mod categories (custom roads, custom zones) are impossible.

8. **Grid dimensions as runtime config, not compile-time constants**: Enables map size mods (the 81 Tiles pattern) which are consistently among the most popular mods in city builders.

9. **Hot-reloading everything**: Data files, scripts, meshes, textures, and (in dev mode) native plugins. Fast iteration is what makes modding communities thrive.

10. **Steam Workshop as primary, self-hosted as backup**: Workshop handles 90% of distribution. Self-hosted repository serves non-Steam platforms and provides API access for tools.
