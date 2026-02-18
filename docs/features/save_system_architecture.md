# Save System Architecture

## Table of Contents

1. [ECS World Serialization in Bevy](#ecs-world-serialization-in-bevy)
2. [Megacity's Current Implementation](#megacitys-current-implementation)
3. [Save File Format Design](#save-file-format-design)
4. [Versioning and Migration](#versioning-and-migration)
5. [What to Serialize — Complete Inventory](#what-to-serialize--complete-inventory)
6. [What NOT to Serialize — Reconstructed State](#what-not-to-serialize--reconstructed-state)
7. [Known Issues and Missing Serialization](#known-issues-and-missing-serialization)
8. [Delta and Incremental Saves](#delta-and-incremental-saves)
9. [Autosave Design](#autosave-design)
10. [Cloud Save Integration](#cloud-save-integration)
11. [Save File Security and Integrity](#save-file-security-and-integrity)
12. [Performance Optimization](#performance-optimization)
13. [Testing Strategy](#testing-strategy)
14. [Entity Reference Remapping](#entity-reference-remapping)
15. [Future Architecture Recommendations](#future-architecture-recommendations)

---

## ECS World Serialization in Bevy

### The Fundamental Challenge

Serializing an Entity Component System world is categorically different from serializing a traditional
object graph. In a conventional OOP game, you might have a `City` object containing a list of `Building`
objects, each containing a list of `Citizen` objects — a tree that maps directly to JSON or any
hierarchical format. ECS inverts this: entities are featureless integer IDs, components are loose bags
of data stored in archetype tables, and the relationships between entities exist only as component
data (an `Entity` field inside a `HomeLocation` component, for example).

This inversion creates three fundamental problems for serialization:

1. **Archetype dynamism**: An entity's "shape" (which components it has) is determined at runtime.
   Two citizens might have different component sets if one has a `PathCache` and the other does not.
   The serializer needs to know how to discover and iterate all component types on all entities.

2. **Entity ID instability**: Bevy's `Entity` type is a generational index — a 32-bit index plus a
   32-bit generation counter. These IDs are assigned by the allocator at runtime and are not stable
   across sessions. If Building A is `Entity(42, gen=1)` in the running game, it will almost certainly
   not be `Entity(42, gen=1)` after a load. Any component that stores an `Entity` reference
   (like `HomeLocation.building` or `WorkLocation.building`) must be *remapped* on load.

3. **Component heterogeneity**: Some components are pure data (position, age, happiness), some
   reference GPU resources (mesh handles, material handles), some are caches derived from other data
   (spatial indices, pathfinding graphs), and some contain function pointers or trait objects that
   cannot be serialized at all. The serializer must distinguish between these categories.

### Bevy's Built-in Scene Serialization

Bevy provides `DynamicScene` and its associated `DynamicSceneBuilder` as a first-party solution.
The architecture is built on Bevy's reflection system (`bevy_reflect`):

**What DynamicScene gives you:**

- Automatic discovery of all reflected components on entities via the `TypeRegistry`
- Serialization to RON format using `bevy_reflect` (no manual struct-per-component needed)
- Entity remapping on load: `DynamicScene::write_to_world` returns an `EntityMap` that maps old
  Entity IDs to newly spawned Entity IDs
- Handles component addition/removal between versions: unrecognized components are skipped,
  missing components simply are not spawned

**What DynamicScene does NOT give you:**

- **Resource serialization**: Only handles entities/components, not Resources (`WorldGrid`,
  `CityBudget`, etc.). This is a major gap when the bulk of state lives in resources.
- **Performance**: Reflection-based serialization is significantly slower than direct binary
  encoding. For 100K citizens with 10+ components, this means millions of reflection lookups.
- **Format control**: RON output is verbose (500MB+ for a 100K city). No built-in compression
  or binary format support.
- **Selective serialization**: No built-in way to say "serialize Position but not Velocity."
  Must manually curate via `allow_component::<T>()` calls.
- **Post-load reconstruction**: No concept of rebuilding derived state (road graph, spatial indices).
- **Entity references in resources**: `WorldGrid.cells[i].building_id` will not be remapped.
  Only references inside scene-captured components get remapped.

### The bevy_save Crate Approach

The community crate `bevy_save` (now somewhat unmaintained as of Bevy 0.14+) attempted to solve
the full-world serialization problem. It registers "saveable" component and resource types via a
plugin, serializes both entities and resources in a single pass, handles entity remapping globally,
supports rollback/undo via world snapshots, and supports multiple backends (RON, binary, custom).

**Why Megacity does not use bevy_save:**

- **Maintenance lag**: `bevy_save` historically lags behind Bevy releases. With Bevy 0.15's
  required_components and observer-based architecture changes, compatibility is uncertain.
- **Granularity**: We need fine-grained control over what gets serialized vs reconstructed.
- **Performance**: For 100K-1M citizens, we need sub-second save times. Generic
  reflection-based approaches are too slow. We need bitcode-level binary encoding.
- **Dependency weight**: Adds complexity and potential breakage on every Bevy version bump.

### Custom Serialization: Megacity's Approach

Megacity uses a fully custom serialization layer. The architecture separates the problem into:

1. **Save-side structs** (`SaveData`, `SaveGrid`, `SaveCitizen`, etc.) — flat, `#[derive(Encode, Decode)]`
   structs that mirror the game state but contain only primitive types, no Entity references
2. **Conversion functions** (`create_save_data`, `restore_*`) — explicit mapping between live ECS
   state and save structs
3. **Binary encoding** via `bitcode` — extremely compact, zero-copy-friendly encoding
4. **Post-load reconstruction** — systems that rebuild derived state from loaded source-of-truth data

This approach trades development effort (manual struct definitions, manual conversion code) for:
- Full control over save file size and format
- Maximum serialization performance
- Clean entity remapping (grid-coordinate-based rather than Entity-ID-based)
- Version migration as explicit code paths
- Zero dependency on Bevy's reflection system

---

## Megacity's Current Implementation

### Crate Layout

The save system lives in `crates/save/` with two source files:

```
crates/save/
  Cargo.toml          # depends on bevy, bitcode, serde, simulation, rendering
  src/
    lib.rs            # Plugin, events, system functions (handle_save, handle_load, handle_new_game)
    serialization.rs  # Save structs, encoding helpers, create_save_data(), restore_*() functions
```

### Dependencies

```toml
[dependencies]
bevy = { workspace = true }
bitcode = { workspace = true }       # v0.6 — binary encoding
serde = { workspace = true }         # serde derives on save structs (used by bitcode)
simulation = { path = "../simulation" }
rendering = { path = "../rendering" }  # only for BuildingMesh3d/CitizenSprite despawn queries
```

The dependency on `rendering` is unfortunate — it exists solely because `handle_load` and
`handle_new_game` need to despawn entities with `BuildingMesh3d` and `CitizenSprite` components
before respawning the world. Ideally, the rendering crate would own the cleanup of its own
entities through Bevy observers or removal hooks.

### Plugin Architecture

Three events drive the system: `SaveGameEvent`, `LoadGameEvent`, and `NewGameEvent`. Each triggers
a corresponding system (`handle_save`, `handle_load`, `handle_new_game`) in the `Update` schedule.
All are synchronous and block the main thread for the duration of serialization.

Bevy limits system parameter count to 16. The save system works around this with `#[derive(SystemParam)]`
bundles (e.g., `V2ResourcesRead` groups policies, weather, unlocks, extended budget, and loan book
into one parameter). As more resources are added, more bundles will be needed.

### Save Flow (handle_save)

```
1. Read events
2. Query all buildings, citizens, utilities, services from ECS
3. Collect into Vec<(Component, ...)> tuples
4. Read all resources (grid, roads, segments, clock, budget, demand, V2 resources)
5. Call create_save_data() — converts ECS state to SaveData struct
6. Call save.encode() — bitcode serialization to Vec<u8>
7. Write bytes to "megacity_save.bin" via std::fs::write()
```

Key observations:
- Save is **synchronous** — the entire encode + write happens on the main thread during Update
- **Single save slot** — hardcoded to `"megacity_save.bin"` with no multiple slots or naming
- **No compression** — raw bitcode bytes are written directly
- **No atomic writes** — if the process crashes during `fs::write`, the save file is corrupted
- **No error propagation** — errors are `eprintln!`'d and swallowed

### Load Flow (handle_load)

```
1. Read events
2. Read "megacity_save.bin" from disk
3. Decode bytes via SaveData::decode() (bitcode)
4. Despawn ALL existing game entities (meshes, sprites, buildings, citizens, utilities, services)
5. Reconstruct WorldGrid from save cells
6. Reconstruct RoadNetwork: place_road() for each saved position, then restore road types
7. Reconstruct RoadSegmentStore if present, then rasterize_all()
8. Restore GameClock, CityBudget, ZoneDemand
9. Spawn Building entities, link building_id into grid cells
10. Spawn UtilitySource entities
11. Spawn ServiceBuilding entities, link building_id into grid cells
12. Spawn Citizen entities with all components
13. Restore V2 resources (policies, weather, unlocks, budget, loans) with fallback to defaults
```

Key observations:
- Load is a **full world replacement** — everything is despawned and respawned
- Entity remapping is done **implicitly** via grid coordinates: buildings are spawned first, their
  new Entity IDs are stored in `grid.cells[i].building_id`, and then citizens look up their
  home/work building entity from the grid
- Citizens that reference out-of-bounds or empty grid cells get `Entity::PLACEHOLDER`
- Many citizen fields are **not roundtripped**: gender is derived from `age % 2`, salary is derived
  from education level, savings is hardcoded to `salary * 2.0`, personality is hardcoded to all 0.5,
  Needs is default, Family is default
- `PathCache` is loaded as empty (`PathCache::new(vec![])`) — commuting citizens lose their paths
- `Velocity` is loaded as zero — citizens freeze until the movement system recalculates
- `ActivityTimer` is loaded as default — activity progress is lost

### Encoding Helpers

The serialization module uses manual enum-to-u8 conversions for every game enum:

```rust
pub fn zone_type_to_u8(z: ZoneType) -> u8 { ... }
pub fn u8_to_zone_type(v: u8) -> ZoneType { ... }
pub fn road_type_to_u8(r: RoadType) -> u8 { ... }
pub fn u8_to_road_type(v: u8) -> RoadType { ... }
// ... service_type, utility_type, policy, weather_event, season, unlock_node
```

Each function is a match statement mapping variants to fixed u8 discriminants. This is robust
against Rust's `#[repr(u8)]` changes and allows explicit versioning of discriminant values, but it
is verbose — the file is over 900 lines, with roughly half devoted to these match arms.

An alternative approach would be `#[repr(u8)]` on the enums with `as u8` / `unsafe transmute`,
but the explicit match approach is safer and more maintainable when enum variants are added,
removed, or reordered.

---

## Save File Format Design

### Binary Format: Why Bitcode

Megacity uses the `bitcode` crate (v0.6) for binary serialization. This is a deliberate choice
among several options:

**Format comparison for a hypothetical 50K-citizen city:**

| Format     | Estimated Size | Encode Speed | Decode Speed | Human Readable |
|------------|---------------|-------------|-------------|----------------|
| RON        | ~200-400 MB   | Slow        | Slow        | Yes            |
| JSON       | ~250-500 MB   | Moderate    | Moderate    | Yes            |
| bincode    | ~15-30 MB     | Fast        | Fast        | No             |
| MessagePack| ~12-25 MB     | Fast        | Fast        | No (but tools) |
| bitcode    | ~5-15 MB      | Very Fast   | Very Fast   | No             |
| protobuf   | ~10-20 MB     | Fast        | Fast        | No (but tools) |

**Why bitcode wins for Megacity:**

1. **Size**: bitcode uses variable-length integer encoding and bit-packing. A `u8` field that
   always stores values 0-6 (zone types) costs roughly 3 bits, not 8. Over 65,536 grid cells,
   this saves significant space. bitcode produces the smallest output of any general-purpose Rust
   serialization format.

2. **Speed**: bitcode is designed for zero-allocation decoding where possible. Benchmarks
   consistently show it as the fastest Rust serialization library, typically 2-5x faster than
   bincode for both encode and decode.

3. **Derive macros**: `#[derive(Encode, Decode)]` works alongside `#[derive(Serialize, Deserialize)]`,
   allowing the same structs to support both bitcode (for production saves) and serde-based formats
   (for debugging/testing) if needed.

4. **No schema overhead**: Unlike protobuf or FlatBuffers, bitcode does not embed field names or
   type descriptors in the output. The schema is defined by the Rust struct layout at compile time.
   This minimizes file size but means the decoder must match the encoder's struct definitions exactly
   — hence the importance of versioning.

**Risks of bitcode:**

- **No self-describing format**: If you lose the struct definitions, the bytes are meaningless.
  Unlike JSON or MessagePack, you cannot inspect a bitcode file without the exact Rust types.
- **Struct layout sensitivity**: Adding, removing, or reordering fields changes the binary layout.
  Forward compatibility requires explicit versioning (see Migration section).
- **Ecosystem size**: bitcode is less widely used than bincode or serde_json. Bus factor considerations.
- **No random access**: You must decode the entire blob; you cannot seek to "just the grid data"
  without fully parsing everything before it.

### Current File Structure

The current save format is extremely simple — a single bitcode-encoded `SaveData` blob:

```
[raw bitcode bytes of SaveData struct]
```

There is no header, no version number, no magic bytes, no checksum. The entire file is a single
bitcode encoding of the `SaveData` struct.

**Problems with this approach:**

1. **No version detection**: If the `SaveData` struct changes, old saves silently fail to decode.
   There is no way to determine the save format version before attempting decode.
2. **No integrity check**: A partially written or corrupted file will produce a decode error, but
   the error message will be opaque ("unexpected end of input" rather than "file corrupted").
3. **No metadata**: You cannot display save metadata (city name, population, play time, screenshot)
   without fully decoding the entire save file.
4. **No partial loading**: The entire save must be decoded into memory at once.

### Proposed File Structure

A production-ready save file should have this layout:

```
Offset  | Size     | Field
--------|----------|----------------------------------------------
0x00    | 4 bytes  | Magic bytes: "MEGA" (0x4D454741)
0x04    | 4 bytes  | Format version (u32, little-endian)
0x08    | 4 bytes  | Flags (u32: bit 0 = compressed, bit 1 = delta save, etc.)
0x0C    | 8 bytes  | Timestamp (Unix epoch, u64)
0x14    | 4 bytes  | Uncompressed data size (u32, for allocation hint)
0x18    | 4 bytes  | Checksum of compressed data (xxHash32)
0x1C    | 4 bytes  | Metadata section length (u32)
0x20    | variable | Metadata (bitcode-encoded SaveMetadata)
        | variable | World data (bitcode-encoded SaveData, optionally compressed)
```

The `SaveMetadata` section would contain quick-access information for the load screen: city name,
population, treasury, game day/hour, play time, mod list, and a screenshot thumbnail (max 64KB PNG).

### Compression

The current implementation writes raw bitcode bytes with no compression. For large cities, this
is wasteful:

**Estimated uncompressed save sizes (bitcode encoding):**

| City Size       | Grid     | Citizens  | Buildings | Roads    | Resources | Total     |
|-----------------|----------|-----------|-----------|----------|-----------|-----------|
| Empty (new)     | ~400 KB  | 0         | 0         | 0        | ~1 KB     | ~400 KB   |
| Small (5K pop)  | ~400 KB  | ~200 KB   | ~50 KB    | ~20 KB   | ~5 KB     | ~675 KB   |
| Medium (50K)    | ~400 KB  | ~2 MB     | ~500 KB   | ~100 KB  | ~10 KB    | ~3 MB     |
| Large (200K)    | ~400 KB  | ~8 MB     | ~2 MB     | ~200 KB  | ~20 KB    | ~10 MB    |
| Megacity (1M)   | ~400 KB  | ~40 MB    | ~10 MB    | ~500 KB  | ~50 KB    | ~50 MB    |

The grid is always 256x256 = 65,536 cells. Each `SaveCell` contains: elevation (f32=4B),
cell_type (u8=1B), zone (u8=1B), road_type (u8=1B), has_power (bool=1B), has_water (bool=1B) =
9 bytes raw, probably ~6 bytes with bitcode's bit-packing. So the grid is approximately
65,536 * 6 = ~384 KB.

Each `SaveCitizen` contains: age (u8), happiness (f32), education (u8), state (u8), home_x (usize),
home_y (usize), work_x (usize), work_y (usize). On 64-bit, usize is 8 bytes each, giving ~37 bytes
raw, maybe ~20 bytes bitcode-packed (coordinates are 0-255, so 8 bits each suffice). For 50K
citizens: ~1 MB. For 1M citizens: ~20 MB.

**Compression format choices:**

| Algorithm | Compression Ratio | Encode Speed | Decode Speed | Best For           |
|-----------|-------------------|-------------|-------------|---------------------|
| LZ4       | 2-3x              | ~3 GB/s     | ~5 GB/s     | Autosave (speed)    |
| Zstd      | 3-5x              | ~500 MB/s   | ~1.5 GB/s   | Manual save (ratio) |
| DEFLATE   | 3-4x              | ~200 MB/s   | ~500 MB/s   | Compatibility       |
| Brotli    | 4-6x              | ~50 MB/s    | ~400 MB/s   | Cloud upload        |

**Recommendation:** Use LZ4 for autosaves (speed priority — the save must not cause a visible
hitch) and Zstd level 3 for manual saves (better compression, player expects a brief pause).
A 50 MB uncompressed save becomes ~15 MB with LZ4 or ~10 MB with Zstd.

In Rust, use `lz4_flex` for LZ4 (`compress_prepend_size` / `decompress_size_prepended`) and
the `zstd` crate for Zstandard (`encode_all` / `decode_all` with configurable level).

### Chunk-Based Saving

For very large cities (500K+ population), even compressed saves can be tens of megabytes. A
chunk-based approach enables:

1. **Partial loading**: Load the grid and camera position first (instant playable state),
   then stream in citizen data chunk by chunk
2. **Parallel encoding**: Each chunk can be compressed independently on a separate thread
3. **Incremental saves**: Only re-encode chunks whose data changed

The world grid naturally divides into 32x32 = 1024 chunks of 8x8 cells each (matching the
existing `CHUNK_SIZE=8` constant). Entities can be assigned to chunks by their grid position.

A chunked file layout would have: header, metadata, global resources, chunk index table (1024
entries: chunk_id + offset + size + checksum), per-chunk data (grid cells + buildings + citizens),
and a global road segments section. This is an advanced optimization not needed until the game
supports cities significantly larger than the current 256x256 grid.

---

## Versioning and Migration

### The Versioning Problem

Save format changes are inevitable during development. Every time a new feature adds a component,
resource, or field, the save format must evolve. Without versioning, save compatibility breaks
silently — players lose their cities.

### Current Versioning Approach

Megacity currently uses an **implicit versioning** strategy via `#[serde(default)]`. New fields
in `SaveData` are wrapped in `Option<T>` with `#[serde(default)]`. When loading an old save:
- Old saves decode to `None` for new fields
- The load handler falls back to `Default::default()` for the corresponding resource
- The game proceeds with default state for any missing V2+ data

**This works, but has limitations:**

1. **Append-only**: You can only add new `Option<T>` fields at the end of the struct. You cannot
   remove fields, rename fields, or change field types without breaking old saves.
2. **No removal**: If you deprecate a feature and remove its save struct, old saves with that data
   will fail to decode. You must keep dead fields forever or add explicit migration.
3. **Binary format sensitivity**: `#[serde(default)]` works with self-describing formats (RON, JSON,
   MessagePack) but bitcode is NOT self-describing. Bitcode encodes fields positionally. Adding a
   field changes the layout for ALL subsequent fields. The current code derives both `Serialize` +
   `Encode`, but the `encode()` method uses bitcode, which means `#[serde(default)]` annotations
   do NOT apply to bitcode encoding. **This is a latent bug**: the serde defaults give a false sense
   of backward compatibility that bitcode does not actually provide.
4. **No structural migration**: You cannot transform data during load (e.g., "split the old `tax_rate`
   field into per-zone tax rates").

### Recommended Versioning Strategy

**Explicit version number with migration chain.** The file header contains a monotonic version
number. Each save version has its own `SaveData` struct (`mod v1`, `mod v2`, etc.) and a pure
migration function (e.g., `migrate_v1_to_v2`). On load, the version is read from the header,
the appropriate struct is decoded, and migration functions are chained sequentially to reach the
current version. Future versions are detected and rejected gracefully.

**Advantages:**

- Clean separation between save versions
- Each migration is a pure function, trivially testable
- Old saves always work (migration chain runs sequentially)
- Structural transformations are possible (rename, split, merge fields)
- Future saves from newer game versions are detected and rejected gracefully

**Costs:**

- Code bloat: each version's structs remain in the binary forever
- Compile time: many struct definitions to derive-macro
- Maintenance: migration functions must be kept correct forever

In practice, most shipped games use this approach because the alternative — breaking player saves —
is unacceptable. The code bloat is modest (dead code from old struct definitions compiles but is
rarely instantiated).

### Handling Specific Migration Scenarios

Each migration function is a pure transformation from one struct to the next. Common patterns:

- **Adding a component**: Map old citizens to new struct, filling new fields with defaults
  (e.g., `ambition: 0.5` for a new Personality field)
- **Removing a feature**: Simply do not copy the deprecated field to the new struct
- **Renaming a field**: Copy `old.field_a` to `new.field_b` in the migration function
- **Changing a field type**: Convert in the migration (e.g., `treasury_cents = (old.treasury * 100.0) as i64`)

**The nuclear option -- when migration is impossible:**

Sometimes a save format change is so fundamental that automatic migration is impractical. For
example, if the grid changes from 256x256 to variable size, or if the citizen model is completely
redesigned. Options:

1. **Partial load**: Load what you can (grid, buildings), discard what you cannot (citizens),
   and let the simulation naturally repopulate. Show a warning: "This save is from an older
   version. Some citizens may have been reset."
2. **Parallel support**: Keep the old load path alongside the new one, but flag the loaded game
   as "legacy" with certain features disabled.
3. **Version cutoff**: Document that saves from before version X are not supported. This is
   acceptable for an early access game but not for a shipped product.
4. **Export/import**: Provide a separate tool that reads the old format and exports to a neutral
   format (JSON), which can then be hand-edited or imported into the new format.

---

## What to Serialize — Complete Inventory

The fundamental rule is: **Serialize the source of truth. Reconstruct derived state.**

This principle resolves every ambiguous case. If a piece of data is computed from other data, it
is derived and should not be serialized. If it is the authoritative, canonical representation,
it is the source of truth and must be serialized.

### Tier 1: World Foundation (must be first to load)

| Data                        | Component/Resource      | Source/Save Struct      | Notes                                    |
|-----------------------------|------------------------|------------------------|------------------------------------------|
| Grid cell types             | `WorldGrid`            | `SaveGrid.cells`       | CellType (Grass/Water/Road)              |
| Grid elevations             | `WorldGrid`            | `SaveGrid.cells`       | f32 per cell                             |
| Grid zones                  | `WorldGrid`            | `SaveGrid.cells`       | ZoneType enum per cell                   |
| Grid road types             | `WorldGrid`            | `SaveGrid.cells`       | RoadType enum per cell                   |
| Grid utility flags          | `WorldGrid`            | `SaveGrid.cells`       | has_power, has_water bools               |
| Grid dimensions             | `WorldGrid`            | `SaveGrid.width/height`| Currently fixed 256x256                  |
| Road segment geometry       | `RoadSegmentStore`     | `SaveRoadSegmentStore` | Bezier curves (p0-p3), node connections  |
| Road network positions      | `RoadNetwork`          | `SaveRoadNetwork`      | Legacy: list of (x,y) road positions     |

**WorldGrid is the single largest serialized object** — 65,536 cells, each with 6 fields. It must
be loaded first because buildings and citizens reference grid coordinates. Road segments must also
be loaded early because they rasterize onto the grid and populate the road network.

### Tier 2: Entities (buildings, citizens, services)

| Data                        | Component              | Save Struct             | Notes                                    |
|-----------------------------|------------------------|-------------------------|------------------------------------------|
| Building zone type          | `Building.zone_type`   | `SaveBuilding.zone_type`| u8 discriminant                          |
| Building level              | `Building.level`       | `SaveBuilding.level`    | u8 (0-3)                                |
| Building position           | `Building.grid_x/y`   | `SaveBuilding.grid_x/y` | Grid coordinates                        |
| Building capacity           | `Building.capacity`    | `SaveBuilding.capacity` | u32                                     |
| Building occupants          | `Building.occupants`   | `SaveBuilding.occupants`| u32                                     |
| Citizen age                 | `CitizenDetails.age`   | `SaveCitizen.age`       | u8 (0-100)                              |
| Citizen happiness           | `CitizenDetails.happiness` | `SaveCitizen.happiness` | f32 (0-100)                         |
| Citizen education           | `CitizenDetails.education` | `SaveCitizen.education` | u8 (0-3)                            |
| Citizen state               | `CitizenStateComp`     | `SaveCitizen.state`     | u8 discriminant                          |
| Citizen home position       | `HomeLocation.grid_x/y`| `SaveCitizen.home_x/y` | Grid coordinates (building looked up)    |
| Citizen work position       | `WorkLocation.grid_x/y`| `SaveCitizen.work_x/y` | Grid coordinates (building looked up)    |
| Utility sources             | `UtilitySource`        | `SaveUtilitySource`     | type, position, range                    |
| Service buildings           | `ServiceBuilding`      | `SaveServiceBuilding`   | type, position, radius                   |

### Tier 3: Simulation Resources

| Data                        | Resource               | Save Struct             | Notes                                    |
|-----------------------------|------------------------|-------------------------|------------------------------------------|
| Game clock                  | `GameClock`            | `SaveClock`             | day, hour, speed                         |
| City budget                 | `CityBudget`           | `SaveBudget`            | treasury, tax_rate, last_collection_day  |
| Zone demand                 | `ZoneDemand`           | `SaveDemand`            | R/C/I/O demand floats                    |
| Policies                    | `Policies`             | `SavePolicies`          | Active policy list                       |
| Weather                     | `Weather`              | `SaveWeather`           | Season, temp, event, days remaining      |
| Unlock state                | `UnlockState`          | `SaveUnlockState`       | Dev points, unlocked node list           |
| Extended budget             | `ExtendedBudget`       | `SaveExtendedBudget`    | Per-zone taxes, per-service budgets      |
| Loan book                   | `LoanBook`             | `SaveLoanBook`          | Active loans, credit rating              |

### Tier 4: Not Yet Serialized (gaps in current implementation)

These exist in the simulation but are not currently saved:

| Data                        | Resource/Component     | Impact of Not Saving                     |
|-----------------------------|------------------------|------------------------------------------|
| `LifecycleTimer`            | Resource               | All life events fire immediately on load  |
| `PathCache`                 | Component on Citizens  | Commuting citizens lose paths, freeze     |
| `Velocity`                  | Component on Citizens  | Citizens stop moving until recalculated   |
| `ActivityTimer`             | Component on Citizens  | Activity duration progress lost           |
| `CitizenDetails.gender`     | Component              | Reconstructed from `age % 2` (incorrect)  |
| `CitizenDetails.health`     | Component              | Hardcoded to 80.0 on load                |
| `CitizenDetails.salary`     | Component              | Recalculated from education (loses mods)  |
| `CitizenDetails.savings`    | Component              | Hardcoded to `salary * 2.0` on load       |
| `Personality`               | Component on Citizens  | All 0.5 on load (personality lost)        |
| `Needs`                     | Component on Citizens  | Default on load (hunger/energy lost)      |
| `Family`                    | Component on Citizens  | Default on load (family ties lost)        |
| `DestinationCache`          | Resource               | Rebuilt automatically on next frame       |
| `LandValueGrid`             | Resource               | Recalculated from buildings/services      |
| `CrimeGrid`                 | Resource               | Recalculated from police coverage         |
| `PollutionGrid`             | Resource               | Recalculated from industrial/traffic      |
| `Districts`                 | Resource               | Not serialized if newly added             |
| `Tourism`                   | Resource               | Not serialized if newly added             |
| `VirtualPopulation`         | Resource               | Not serialized, recalculated on load      |

### Serialization Priority Assessment

**Critical (game-breaking if lost):**
- WorldGrid, RoadSegmentStore, Buildings, Citizens (core identity), Budget, Clock

**Important (noticeable if lost, but game continues):**
- Citizen state, home/work locations, happiness, education
- Policies, weather, unlocks, loans

**Moderate (slight inaccuracy on load, self-corrects over time):**
- Citizen health, savings, salary (recalculated but wrong initially)
- LifecycleTimer (causes burst of events on load)
- Activity progress (citizens restart current activity)

**Low (reconstructed automatically within one game tick):**
- DestinationCache, LandValueGrid, spatial indices, pathfinding graph
- Velocity (recalculated by movement system)

**Not applicable (GPU/engine state):**
- Meshes, materials, sprites, camera entities, UI panels

---

## What NOT to Serialize — Reconstructed State

### The Reconstruction Principle

For every piece of derived state, there must be a reconstruction path — code that runs after load
and rebuilds the derived data from the serialized source of truth. This code often already exists
as initialization code (used when the game starts fresh), but load may need to re-invoke it
explicitly.

### Render State

`BuildingMesh3d`, `CitizenSprite`, and material handles are spawned by the rendering systems
automatically when they detect entities with `Building` or `Citizen` components lacking visual
representations. The load handler must explicitly despawn these render entities first because they
are on separate entities from the gameplay data.

### Spatial Indices

`DestinationCache` (shops, leisure, schools position lists) rebuilds automatically via the
`refresh_destination_cache` system when `Added<Building>` or `Added<ServiceBuilding>` triggers.
Since load spawns all buildings fresh, the cache rebuilds on the next frame. `SpatialIndex` is
similarly rebuilt from the cache.

### Pathfinding Graph

`CsrGraph` and `RoadNetwork` are rebuilt from `RoadSegmentStore` segments after load. The load
handler calls `restore_road_segment_store` then `rasterize_all` to rebuild grid cells and the road
network. The CSR graph builder runs on the next tick to construct the A* graph. The CSR graph is
never serialized.

### Other Reconstructed State

- **Chunk dirty flags**: All chunks implicitly dirty after load; mesh regeneration rebuilds them.
- **Coverage grids** (`CrimeGrid`, `PollutionGrid`, `LandValueGrid`, `DeathCareGrid`): Recalculated
  from building/service positions within one calculation cycle.
- **UI state**: Panel positions, selections, overlay modes reset to defaults. Players expect this.
- **Audio state**: Music, effects, ambient sounds reconstructed from game state (time, weather).

---

## Known Issues and Missing Serialization

### Issue 1: LifecycleTimer Not Serialized

**Source:** `crates/simulation/src/lifecycle.rs` -- `LifecycleTimer { last_aging_day: u32, last_emigration_tick: u32 }`

**Symptom:** On load, both fields default to 0. If the loaded game is on day 400, the aging check
(`clock.day < timer.last_aging_day + 365`) passes immediately, and every citizen ages by one year.
On day 800, it fires twice. Similarly, emigration fires immediately.

**Fix:** Add `lifecycle_last_aging_day: Option<u32>` and `lifecycle_last_emigration_tick: Option<u32>`
to `SaveData`. On load, use `unwrap_or(clock.day)` so old saves default to the current day,
preventing immediate firing.

### Issue 2: PathCache and Velocity Not Serialized

**Source:** `crates/save/src/lib.rs`, lines 376-378

**Symptom:** On load, Position is set to home coordinates, Velocity to zero, PathCache to empty.
A citizen who was mid-commute loads at home with no path. The movement system may request a new
path (brief delay), get stuck in commuting state, or teleport to the destination.

**Fix options:**

**Option A: Reset commuting citizens to AtHome (simple, lossy).** On load, map all commuting
states to `AtHome`. The movement system naturally re-dispatches them. Visual effect: commuting
citizens "teleport home" on load, barely noticeable.

**Option B: Serialize path waypoints (complex, accurate).** Add `path_x/path_y: Vec<u16>` and
exact position/velocity to `SaveCitizen`. Preserves exact state but adds ~120 bytes per commuting
citizen (~6 MB for 50K commuters). Fragile if road layout changes between versions.

**Option C: Serialize position only, re-path on load (balanced).** Add `pos_x/pos_y: f32` to
`SaveCitizen`. Citizens resume at saved position, movement system recalculates paths on next tick.
No teleportation, minimal save bloat.

**Recommendation:** Option A for v1 (simple, correct enough), Option C for v2 (better UX).

### Issue 3: Citizen Details Data Loss

Multiple `CitizenDetails` fields are not roundtripped:

- **Gender**: Reconstructed from `age % 2` -- changes on birthday, incorrect half the time
- **Health**: Hardcoded to 80.0 -- a citizen at 20% health loads as healthy
- **Salary**: Recalculated from education level -- loses job-match modifiers
- **Savings**: Hardcoded to `salary * 2.0` -- a citizen with $50K savings loads with $3K

**Fix:** Add `gender: u8`, `health: f32`, `salary: f32`, `savings: f32` to `SaveCitizen`.
Adds ~13 bytes per citizen (~1.3 MB for 100K citizens, acceptable).

### Issue 4: Personality, Needs, and Family Not Serialized

These components contain meaningful simulation state that is lost on load:

- `Personality { ambition, sociability, materialism, resilience }` — determines citizen behavior
  patterns. Currently hardcoded to 0.5 on load, making all citizens identical in personality.
- `Needs { hunger, energy, social, entertainment }` — determines citizen urgency for various
  activities. Reset to default on load.
- `Family { partner: Option<Entity>, children: Vec<Entity> }` — family relationships. Reset to
  default on load, destroying all family bonds.

The `Family` component is particularly problematic because it stores `Entity` references. These
would need remapping on load. One approach: serialize family relationships as citizen indices
(position in the citizens array) rather than Entity IDs, then resolve to entities after all
citizens are spawned.

---

## Delta and Incremental Saves

### Full Save vs Delta Save

A **full save** captures the complete world state. A **delta save** captures only what changed
since the last full save (the "base"). Loading a delta save requires: load the base, then
apply the delta.

### Why Delta Saves Matter for City Builders

In a mature city with 100K+ citizens:
- The grid (65K cells) rarely changes in bulk — most ticks, only a few cells change zone/building
- Most buildings are static — only a handful are being constructed or upgraded per minute
- Most citizens change only their state (AtHome/Commuting/Working) and position
- Resources change every tick (budget, clock) but are tiny

A delta save can capture just the changed entities and resource fields, skipping the 99% of
the world that has not changed. This dramatically reduces save time and file size.

### Change Detection Mechanisms

**Bevy's `Changed<T>` filter:** Tracks which components were accessed mutably each frame. Run a
system every frame that copies `Changed<T>` entity IDs into a dirty set, then at delta save time,
serialize only dirty entities. Limitations: cleared each frame, tracks `DerefMut` access rather
than actual value changes, and does not report which fields changed.

**Grid dirty flags:** Add a `dirty: bool` per cell. On delta save, serialize only dirty cells
and clear flags. On delta load, apply cell data to specific grid positions.

**Resource change tracking:** Bevy does not provide `Changed<T>` for resources. Options: manual
dirty flags, hash comparison, or simply always including resources in deltas (they are small).

### Journaling Approach

Instead of snapshotting deltas, log every mutation as it happens (cell changes, entity spawns/despawns,
resource updates) in a `MutationJournal`. Advantages: perfect change tracking, enables undo/redo
and replays. Disadvantages: high memory overhead (100K citizens moving = 100K mutations/tick),
unbounded journal growth, and every mutation point must be instrumented.

**Recommendation:** Journaling is overkill for Megacity. Use snapshot-based delta saves with
`Changed<T>` tracking and grid dirty flags.

### Delta Save Usage Pattern

```
Time 0:00  — Full save (base)                    → full_save_001.bin
Time 5:00  — Autosave (delta from base)           → delta_001_001.bin
Time 10:00 — Autosave (delta from base)           → delta_001_002.bin
Time 15:00 — Full autosave (new base, compact)    → full_save_002.bin
Time 20:00 — Autosave (delta from new base)       → delta_002_001.bin
Time 25:00 — Manual save (always full)            → manual_save_001.bin
```

**Full save triggers:**
- Manual save (always full for reliability)
- Every 15 minutes (periodic compaction to prevent long delta chains)
- On game exit (ensure a clean base exists for next session)

**Delta save triggers:**
- Autosave every 5 minutes
- Quicksave (Ctrl+S / F5)

**Maximum delta chain length:** 3 deltas before a forced full save. Longer chains increase load
time and risk of corruption (if any delta in the chain is corrupted, all subsequent deltas are
unusable).

---

## Autosave Design

### Requirements

1. **Frequency**: Every 5 minutes by default, configurable (1-30 minutes)
2. **Performance**: Must not freeze the game for more than 1 frame (16ms at 60fps)
3. **Reliability**: Must not corrupt the save file if the game crashes during autosave
4. **Slots**: Rotating autosave slots (autosave_1, autosave_2, autosave_3)
5. **Visibility**: Brief "Saving..." indicator in the UI corner, no blocking modal

### The Main Thread Problem

Bevy's ECS `World` is not `Send` or `Sync` — it cannot be accessed from a background thread
(without `unsafe` and significant complexity). This means you cannot simply "serialize in the
background" while the game continues simulating.

**The naive approach (current implementation):**

```rust
fn handle_save(/* system params */) {
    let save = create_save_data(/* ... */);   // reads from World — BLOCKING
    let bytes = save.encode();                // CPU work — BLOCKING
    std::fs::write(&path, &bytes);            // I/O — BLOCKING
}
```

Everything is synchronous on the main thread. For a small city, this takes <10ms and is
imperceptible. For a 100K city, serialization alone might take 50-200ms (visible hitch), and
disk I/O adds another 10-50ms.

### Double-Buffered Async Save

The correct architecture is a **snapshot-then-serialize** pipeline:

```
Frame N:    [Snapshot world state into SaveData]  ← must be on main thread
Frame N+1:  [Game continues simulating]
Background: [Encode SaveData to bytes]            ← can be on background thread
Background: [Compress bytes]                      ← can be on background thread
Background: [Write to temp file]                  ← can be on background thread
Background: [Rename temp file to save file]       ← atomic, can be on background thread
Frame N+K:  [Notify UI "Save complete"]
```

The critical insight: only the **snapshot** step needs main-thread access. Once `create_save_data()`
has produced a `SaveData` struct (which contains only owned data, no references to the World),
the rest can happen on a background thread.

**Implementation in Bevy:** Use `AsyncComputeTaskPool` to spawn the encode/compress/write work.
A `SaveInProgress` resource tracks the active task. The `start_autosave` system snapshots the world
into a `SaveData` (main thread), then spawns a background task for encoding, compression, and
atomic file writing. A `poll_save_completion` system checks each frame whether the task is done.

**Performance budget for snapshot step:**

The snapshot step (main thread) must complete within one frame (16ms). This means:
- Grid serialization (65K cells, ~6 bytes each): ~0.5ms (memcpy-like)
- Building iteration (10K buildings): ~1ms
- Citizen iteration (100K citizens, 10 components each): ~5-10ms
- Resource collection: <0.5ms
- **Total: ~7-12ms** — fits within one frame for 100K citizens

For 1M citizens, the snapshot step might take 50-100ms. At that scale, you would need:
- Snapshot only the delta (changed entities)
- Or split the snapshot across multiple frames (snapshot citizens in batches of 200K/frame)
- Or accept a single-frame hitch for autosave and document it

### Snapshot Consistency

The snapshot reads world state at frame N; by the time the background thread writes it, the world
has advanced. This is fine -- the save is self-consistent within the snapshot moment. However, if
the snapshot spans multiple frames (batched citizen collection), cross-frame inconsistency is
possible. For Megacity, single-frame snapshots are recommended up to 500K entities.

### Rotating Autosave Slots

An `AutosaveConfig` resource tracks `interval_minutes` (default 5.0), `slot_count` (default 3),
`current_slot` (cycles 0, 1, 2, 0, ...), and `last_save_time` (real-world clock). Each autosave
writes to `autosave_{slot+1}.bin` and advances the slot counter.

With 3 rotating slots, the player always has at least 2 good autosaves if the latest one is
corrupted (crash during write). The oldest autosave is at most 15 minutes old (3 slots * 5 min).

### Crash Recovery

On startup, iterate autosave slots in reverse order (newest first). Remove any `.tmp` files
(incomplete writes from crashes). For each slot, read the file, parse the header, and verify
the checksum. Return the first valid autosave found. If a crash is detected (`.tmp` file existed),
prompt the player: "A crash was detected. Load the most recent autosave?"

---

## Cloud Save Integration

### Steam Cloud (Steamworks SDK)

Steam Cloud via `ISteamRemoteStorage` (accessed through the `steamworks-rs` crate) provides
`file_write`, `file_read`, and `file_timestamp` operations.

**Key constraints:**
- **Per-file size limit**: 100 MB (comfortable for compressed city saves)
- **Total quota**: Configured in Steamworks partner settings, typically 1-5 GB
- **Sync behavior**: Steam syncs on game launch and game exit. Mid-session syncs are not automatic.
- **Conflict resolution**: Steam does not automatically resolve conflicts. If local and cloud
  timestamps differ, the game must present a choice to the player.

**File structure for Steam Cloud:**

```
saves/
  manual/
    city_springfield.bin       # manual save with city name
    city_shelbyville.bin
  autosave/
    autosave_1.bin
    autosave_2.bin
    autosave_3.bin
  meta/
    save_index.json            # list of saves with metadata for load screen
```

**Compression is critical for cloud saves.** A 50 MB uncompressed save takes ~10 seconds to
upload on a 40 Mbps connection. Compressed to 10 MB with Zstd, it takes ~2 seconds. Players
expect save operations to be near-instant.

### GOG Galaxy

GOG Galaxy Cloud Saves use a different API but similar concepts:
- File-based storage (similar to Steam)
- Quota-limited
- Sync on launch/exit
- The game must declare which files to sync in the GOG Galaxy configuration

### Epic Games Store

Epic's cloud save system:
- Files in a designated directory are automatically synced
- No API calls needed — just write to the correct directory
- Directory specified in `.egstore` configuration

### Platform-Agnostic Design

Abstract save I/O behind a `SaveBackend` trait with methods: `write_save`, `read_save`,
`list_saves`, `delete_save`, and `resolve_conflict`. Implementations include `LocalSaveBackend`,
`SteamSaveBackend`, and `GogSaveBackend`. The save system uses `dyn SaveBackend`, selected at
initialization based on the detected storefront.

### Conflict Resolution UI

When a cloud/local conflict is detected, show a dialog comparing both saves (game day, population,
timestamp) with three options: "Use Local," "Use Cloud," or "Keep Both" (which copies the losing
version with a timestamp suffix for manual comparison later).

---

## Save File Security and Integrity

### Preventing Save Corruption

**Atomic writes (current gap):**

The current implementation uses a bare `std::fs::write`:

```rust
if let Err(e) = std::fs::write(&path, &bytes) {
    eprintln!("Failed to save: {}", e);
}
```

If the process crashes during `write`, the file is partially written and corrupted. The previous
save is destroyed.

**Solution: write-rename pattern.** Write data to `{path}.tmp`, call `sync_all()` to flush to
disk, then atomically `rename` the temp file to the final path. On POSIX, `rename` is atomic if
both paths are on the same filesystem. On Windows/NTFS, `MoveFileEx` with
`MOVEFILE_REPLACE_EXISTING` provides the same guarantee.

### Checksums

Add a checksum to the file header. On save, compute the hash of the data section and store it in
the header. On load, recompute and compare before decoding.

**CRC32 vs xxHash:**
- CRC32: Standard, well-understood, hardware-accelerated on x86 (SSE4.2). ~3 GB/s.
- xxHash32: Faster in software (~6 GB/s), comparable quality. Good for non-cryptographic use.
- xxHash64: Even faster on 64-bit systems, overkill for save file integrity.

For save files (typically <50 MB), the checksum computation takes <10ms regardless of algorithm.
xxHash32 is recommended for simplicity and speed.

### Save Scumming

City builders generally do not prevent save scumming -- there is little benefit to "cheating" in
a sandbox game, and preventing it frustrates legitimate players. Provide ample save slots and easy
save/load to encourage experimentation.

### Modded Saves

The save file header should record which mods were active (`mod_id: String`, `version: String`,
`save_version: u32` per mod). On load, check compatibility: all required mods present loads
normally; missing mod shows a warning dialog; wrong mod version warns about possible migration
needs; extra mods (not in save) load normally.

Mod data is stored in a `HashMap<String, Vec<u8>>` section keyed by mod ID. Each mod serializes/
deserializes its own data; the core save system provides only the storage mechanism.

---

## Performance Optimization

### Performance Targets

| City Size   | Population | Save Time | Load Time | File Size (compressed) |
|-------------|-----------|-----------|-----------|------------------------|
| Small       | 5K        | <100ms    | <200ms    | <200 KB                |
| Medium      | 50K       | <500ms    | <1s       | <2 MB                  |
| Large       | 200K      | <1s       | <3s       | <10 MB                 |
| Megacity    | 1M        | <3s       | <10s      | <30 MB                 |

### Profiling Save/Load Times

Insert `Instant::now()` timing instrumentation at each phase boundary (snapshot, encode, compress,
write) and log the breakdown. This immediately identifies which phase dominates.

**Expected bottlenecks:**

1. **Citizen iteration** (snapshot phase): Iterating 100K+ citizens with 10+ components is the
   slowest part of the snapshot. Each citizen requires reading from multiple archetype tables.
2. **Grid copy** (snapshot phase): Copying 65K cells is fast (memcpy-like) but can be further
   optimized if cells are `repr(C)` and can be copied as a byte slice.
3. **Bitcode encoding** (encode phase): Generally fast, but Vec<SaveCitizen> encoding involves
   N small allocations for citizen data.
4. **Compression** (compress phase): LZ4 is fast enough to be negligible. Zstd at level 3 is
   noticeable for >10 MB inputs.
5. **Disk I/O** (write phase): SSD writes are fast, HDD writes can stall. Async I/O helps.

### Parallel Serialization

The snapshot phase could use `rayon::join3` to serialize grid, citizens, and buildings in parallel.
However, Bevy queries borrow the `World` and cannot be sent to other threads. The workaround is:
collect query results into owned `Vec` on the main thread, then convert to save structs in parallel.
This only helps if the per-entity conversion is complex; for simple struct mapping, thread
synchronization overhead may exceed the savings.

### Memory-Mapped Files

For very large saves (>100 MB), `memmap2::MmapOptions` can map the file into memory without a
full read. This is most beneficial when reading only part of the file (e.g., just the metadata
section for the load screen). For full-file decoding, it is roughly equivalent to `std::fs::read`.

### Lazy Loading

Load the game in stages to get the player into a playable state faster:

```
Phase 1 (instant):   Load header + metadata, display load screen
Phase 2 (100ms):     Load grid + roads + camera position → render terrain
Phase 3 (200ms):     Load buildings → spawn building entities + meshes
Phase 4 (500ms):     Load citizens → spawn citizen entities
Phase 5 (100ms):     Load resources (budget, clock, weather, etc.)
Phase 6 (50ms):      Reconstruct derived state (pathfinding graph, spatial indices)
Phase 7:             Resume simulation
```

After Phase 2, the player sees the terrain and can pan the camera. After Phase 3, buildings are
visible. Citizens stream in during Phase 4. This progressive loading prevents the "staring at a
black screen for 5 seconds" experience.

This can be implemented with Bevy's `States` enum (`NotLoading`, `LoadingGrid`, `LoadingBuildings`,
`LoadingCitizens`, `LoadingResources`, `Reconstructing`, `Complete`). Each state transition
processes one phase, then advances. The rendering system runs throughout, showing progressive results.

### Progress Bar

A `SaveProgress` resource tracks `phase` (string), `current`, and `total` counts. The UI renders
a progress bar from this resource. In a single-threaded save, the progress bar cannot update
mid-save because the main thread is blocked. For the async save approach, use a
`crossbeam_channel` to send progress updates from the background thread, polled each frame by
the UI system. For synchronous saves, show an indeterminate indicator (spinning icon) instead.

---

## Testing Strategy

### Round-Trip Tests

The most fundamental test: save a world, load it, save again, and verify the bytes are identical.

```rust
#[test]
fn test_save_load_roundtrip_binary_identical() {
    // Create a world with known state
    let grid = create_test_grid();
    let buildings = create_test_buildings(100);
    let citizens = create_test_citizens(1000);
    // ...

    // Save
    let save1 = create_save_data(&grid, /* ... */);
    let bytes1 = save1.encode();

    // Load
    let loaded = SaveData::decode(&bytes1).unwrap();

    // Save again (from loaded data, not original)
    let save2 = create_save_data_from_loaded(&loaded);
    let bytes2 = save2.encode();

    // Binary compare
    assert_eq!(bytes1, bytes2, "round-trip should produce identical bytes");
}
```

**Caveat:** This test only works if the save-load-save path is truly lossless. Currently, Megacity's
load path loses data (gender, health, salary, savings, personality, etc.), so this test would fail.
Fixing the data loss issues is a prerequisite for this test.

The existing test suite in `serialization.rs` tests individual component roundtrips:

```rust
#[test] fn test_roundtrip_serialization()    // full grid + encode + decode
#[test] fn test_zone_type_roundtrip()         // all ZoneType variants
#[test] fn test_utility_type_roundtrip()      // all UtilityType variants
#[test] fn test_service_type_roundtrip()      // all ServiceType variants (0..=49)
#[test] fn test_policy_roundtrip()            // all Policy variants
#[test] fn test_weather_roundtrip()           // Weather fields
#[test] fn test_unlock_state_roundtrip()      // UnlockState fields
#[test] fn test_unlock_node_roundtrip()       // all UnlockNode variants
#[test] fn test_policies_serialize_roundtrip() // Policies with active list
#[test] fn test_extended_budget_roundtrip()   // ExtendedBudget fields
#[test] fn test_loan_book_roundtrip()         // LoanBook with loans
#[test] fn test_v2_full_roundtrip()           // all V2 fields together
#[test] fn test_backward_compat_v1_defaults() // V1 save (no V2 fields) loads correctly
```

These tests are valuable but incomplete. Missing coverage:
- Citizens with various states and component combinations
- Buildings with entity references
- Road segments with node/edge connectivity
- Large-scale tests (10K+ entities)
- Error cases (corrupted data, truncated files)

### Fuzz Testing

Feed corrupted save files to the decoder to ensure graceful failure (no panics). A basic fuzz
test creates a valid save, then applies random corruption strategies in a loop (10K iterations):
truncation at random position, random bit flips, zeroing sections, inserting garbage bytes, and
empty file. The decoder must either succeed or return `Err` — never panic.

For continuous fuzzing, use `cargo-fuzz` with `libfuzzer`. Create a fuzz target that calls
`SaveData::decode(data)` on arbitrary byte slices. Run with:
`cargo fuzz run fuzz_save_decode -- -max_total_time=3600`

### Migration Tests

Maintain test save files from every released format version in `tests/fixtures/saves/`
(e.g., `v1_small_city.bin`, `v2_medium_city.bin`). Tests use `include_bytes!` to load each
fixture and verify it passes through the migration chain correctly — V1 saves get default V2
fields, V2 saves retain their V2 data, etc.

**Critical:** These fixture files must be committed to version control and never regenerated.
They represent the exact binary output of each historical version. Regenerating them with current
code would defeat the purpose of migration testing.

### Benchmark Tests

Use `criterion` to benchmark each phase (`create_save_data`, `encode`, `decode`, `compress_lz4`,
`compress_zstd`) at city sizes of 1K, 10K, 50K, and 100K citizens. Each size creates a test city,
then benchmarks each phase independently using `BenchmarkId` for labeled comparison.

**Expected benchmark results (estimated, 2024 hardware):**

| Operation      | 1K citizens | 10K citizens | 50K citizens | 100K citizens |
|---------------|------------|-------------|-------------|---------------|
| Snapshot       | <1ms       | ~5ms        | ~25ms       | ~50ms         |
| Encode         | <1ms       | ~3ms        | ~15ms       | ~30ms         |
| Decode         | <1ms       | ~3ms        | ~15ms       | ~30ms         |
| LZ4 compress   | <1ms       | ~1ms        | ~5ms        | ~10ms         |
| Zstd compress  | ~1ms       | ~5ms        | ~20ms       | ~40ms         |
| File write     | <1ms       | <1ms        | ~5ms        | ~10ms         |
| **Total save** | <5ms       | ~15ms       | ~70ms       | ~140ms        |
| **Total load** | <5ms       | ~15ms       | ~70ms       | ~140ms        |

These estimates assume SSD storage, single-threaded, no compression. With LZ4 compression,
add ~10% to save time and subtract ~5% from load time (smaller I/O).

### Property-Based Testing

Use `proptest` to generate arbitrary save data and verify invariants. Key properties to test:
- Roundtrip preserves citizen count for arbitrary `num_citizens in 0..10000`
- Roundtrip preserves grid dimensions for arbitrary `width/height in 16..512`
- All citizen fields survive roundtrip (age, happiness, education, state, positions)
- All resource values survive roundtrip (treasury, tax rates, etc.)

### Integration Testing

Full integration tests exercise the Bevy systems directly: create a `App` with `MinimalPlugins`,
`SimulationPlugin`, and `SavePlugin`. Set up a city (grid zones, buildings, citizens), trigger
`SaveGameEvent` and update, then modify the world, trigger `LoadGameEvent` and update, and verify
the restored state matches the original. This catches issues in the full system pipeline that
unit tests on `SaveData` alone would miss.

---

## Entity Reference Remapping

### The Problem in Detail

Bevy's `Entity` type is a generational index:

```rust
pub struct Entity {
    index: u32,      // slot in the entity allocator
    generation: u32, // reuse counter (incremented when slot is recycled)
}
```

When a building is spawned, Bevy assigns it the next available `Entity`. On a fresh world, the
first building might be `Entity(5, gen=0)`. On load, after despawning everything and respawning,
the same building might be `Entity(127, gen=3)` depending on what entities were allocated and
freed during the load process.

Components that store `Entity` references become invalid across save/load boundaries:

```rust
pub struct HomeLocation {
    pub grid_x: usize,
    pub grid_y: usize,
    pub building: Entity,  // THIS becomes invalid on load
}

pub struct Family {
    pub partner: Option<Entity>,    // THIS becomes invalid on load
    pub children: Vec<Entity>,      // THESE become invalid on load
}
```

### Current Remapping Strategy

Megacity uses **grid-coordinate-based remapping** for building references. Buildings are spawned
first, their new Entity IDs stored in `grid.cells[x][y].building_id`. When citizens are spawned,
`HomeLocation.building` is looked up from the grid at the saved coordinates. This works because
buildings have unique grid positions, making `(grid_x, grid_y)` a stable identifier.

**Limitations:**
- Only works for entity references that map to grid positions (buildings, services)
- Does not work for citizen-to-citizen references (Family.partner, Family.children)
- `Entity::PLACEHOLDER` is used for unresolvable references, which may cause crashes if
  systems dereference it without checking

### Alternative Remapping Strategies

**Strategy 1: Save-local entity ID mapping.** Assign each entity a sequential integer during save
(buildings 0..N, citizens 0..M). Serialize all entity references as these local IDs. On load,
build a `Vec<Entity>` mapping save-local IDs to newly spawned entities, then resolve references
via index lookup. Works for any entity-to-entity reference. Requires two passes (spawn first,
resolve references second).

**Strategy 2: Position-based hashing.** Use composite keys like
`(home_x, home_y, work_x, work_y, age, education)` as probabilistically unique identifiers.
Fragile — not recommended for production use.

**Strategy 3: Stable entity IDs.** Add a `StableId(u64)` component to every referenceable entity,
with a `StableIdAllocator` resource assigning monotonic IDs. Serialize references as `u64`. On
load, build a `HashMap<u64, Entity>` to resolve. Clean and explicit, but adds overhead per entity.

**Recommendation:** For Megacity, use save-local IDs (Strategy 1) for building references and
stable IDs (Strategy 3) for citizen-to-citizen references (Family). The grid-based approach is
fine for building references but does not generalize to non-spatial relationships.

---

## Future Architecture Recommendations

### Short-Term Fixes (before next release)

1. **Add file header** with magic bytes, version number, and checksum
2. **Fix atomic writes** (write-rename pattern)
3. **Serialize missing citizen fields**: gender, health, salary, savings
4. **Serialize LifecycleTimer**: prevent aging burst on load
5. **Reset commuting citizens to AtHome on load** (Option A from Issue 2)
6. **Add error types** instead of `eprintln!` (proper `Result` propagation)

### Medium-Term Improvements (next 2-3 releases)

1. **Explicit version numbers** with migration chain
2. **LZ4 compression** for all saves
3. **Async autosave** with double-buffered snapshot
4. **Rotating autosave slots** (3 slots, 5-minute interval)
5. **Multiple named save slots** (city_name_timestamp.bin)
6. **Serialize Personality, Needs** (citizen richness)
7. **Save-local entity IDs** for Family references
8. **Crash recovery detection** (check for .tmp files on startup)
9. **Save metadata section** for load screen display
10. **Remove rendering dependency** from save crate (use observers/hooks)

### Long-Term Architecture (1.0 release)

1. **Cloud save support** (Steam Cloud, GOG Galaxy, Epic)
2. **Delta saves** for fast autosave
3. **Mod save data** (extensible mod data section)
4. **Progressive loading** (phased load with progress bar)
5. **Save file browser** with city thumbnails and statistics
6. **Backward compatibility guarantee** (support saves from 1.0 forward indefinitely)
7. **Benchmark suite** in CI (save/load performance regression detection)
8. **Fuzz testing** in CI (continuous fuzzing of save decoder)

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Save System                                  │
│                                                                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐           │
│  │  SaveEvent   │───>│  Snapshot    │───>│  Background  │           │
│  │  (UI/auto)   │    │  (main thr.) │    │  Thread      │           │
│  └──────────────┘    └──────────────┘    │              │           │
│                             │            │  ┌────────┐  │           │
│                             v            │  │ Encode │  │           │
│                      ┌──────────────┐    │  │(bitcode)│ │           │
│                      │   SaveData   │───>│  └────────┘  │           │
│                      │ (owned data) │    │       │      │           │
│                      └──────────────┘    │  ┌────────┐  │           │
│                                          │  │Compress│  │           │
│                                          │  │ (LZ4)  │  │           │
│                                          │  └────────┘  │           │
│                                          │       │      │           │
│                                          │  ┌────────┐  │           │
│                                          │  │ Write  │  │           │
│                                          │  │(atomic)│  │           │
│                                          │  └────────┘  │           │
│                                          └──────────────┘           │
│                                                                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐           │
│  │  LoadEvent   │───>│   Read +     │───>│  Reconstruct │           │
│  │  (UI)        │    │   Decode     │    │  (main thr.) │           │
│  └──────────────┘    └──────────────┘    │              │           │
│                                          │  Grid        │           │
│                                          │  Buildings   │           │
│                                          │  Citizens    │           │
│                                          │  Resources   │           │
│                                          │  ↓           │           │
│                                          │  Rebuild:    │           │
│                                          │  - Road graph│           │
│                                          │  - Spatial   │           │
│                                          │  - Meshes    │           │
│                                          └──────────────┘           │
└─────────────────────────────────────────────────────────────────────┘
```

### File Format Evolution

```
V1 (current):  [raw bitcode SaveData]
               No header, no version, no checksum, no compression

V2 (proposed): [MEGA][version=2][flags][timestamp][size][checksum]
               [metadata section]
               [LZ4-compressed bitcode SaveData]
               Header + metadata + compression + integrity

V3 (future):   [MEGA][version=3][flags][timestamp][size][checksum]
               [metadata section with thumbnail]
               [global resources section]
               [chunk index table]
               [chunk 0..1023 data, individually compressed]
               [road segments section]
               [mod data section]
               Chunk-based, mod-aware, progressive loading
```

---

## Appendix A: SaveData Struct Reference

See `crates/save/src/serialization.rs` for the complete struct definitions. The root `SaveData`
struct contains: `SaveGrid` (cells + dimensions), `SaveRoadNetwork` (positions), `SaveClock`,
`SaveBudget`, `SaveDemand`, `Vec<SaveBuilding>`, `Vec<SaveCitizen>`, `Vec<SaveUtilitySource>`,
`Vec<SaveServiceBuilding>`, and optional V2 fields: `SaveRoadSegmentStore`, `SavePolicies`,
`SaveWeather`, `SaveUnlockState`, `SaveExtendedBudget`, `SaveLoanBook`.

## Appendix B: Enum Discriminant Registry

All enum-to-u8 mappings are defined in `crates/save/src/serialization.rs` (functions like
`zone_type_to_u8`, `u8_to_zone_type`, etc.). These discriminant values are part of the save
format contract and must never change without a version bump and migration. Key ranges:

- CellType: 0-2 (Grass, Water, Road)
- ZoneType: 0-6 (None through Office)
- RoadType: 0-5 (Local through Path)
- UtilityType: 0-8 (PowerPlant through WaterTreatment)
- ServiceType: 0-49 (FireStation through WellPump)
- CitizenState: 0-9 (AtHome through AtSchool)
- Policy: 0-14 (FreePublicTransport through IndustrialZoningRestriction)
- WeatherEvent: 0-4 (Clear through Storm)
- Season: 0-3 (Spring through Winter)
- UnlockNode: 0-37 (BasicRoads through InternationalAirports)

## Appendix C: Derived State Reconstruction Order

After loading source-of-truth data, derived state must be reconstructed in dependency order:

```
1. WorldGrid (loaded from save)
     │
     ├─> 2. RoadSegmentStore.rasterize_all()
     │        Writes CellType::Road into grid cells
     │        Populates RoadNetwork adjacency list
     │        │
     │        └─> 3. CsrGraph::build_from_road_network()
     │                Pathfinding graph for A* queries
     │
     ├─> 4. Building entities spawned
     │        grid.cells[x][y].building_id = new Entity
     │        │
     │        ├─> 5. DestinationCache rebuilt (Added<Building> triggers)
     │        │        shops, leisure, schools position lists
     │        │
     │        └─> 6. SpatialIndex rebuilt from DestinationCache
     │
     ├─> 7. ServiceBuilding entities spawned
     │        grid.cells[x][y].building_id = new Entity
     │        │
     │        ├─> 8. Coverage grids recalculated
     │        │        CrimeGrid, PollutionGrid, LandValueGrid
     │        │
     │        └─> 9. Service coverage areas updated
     │
     ├─> 10. Citizen entities spawned
     │         HomeLocation.building resolved from grid
     │         WorkLocation.building resolved from grid
     │         │
     │         └─> 11. Citizen visual representations spawned
     │                  CitizenSprite/mesh attached by rendering systems
     │
     └─> 12. Building visual representations spawned
              BuildingMesh3d attached by rendering systems
```

Steps 5-6, 8-9, 11-12 happen automatically in subsequent frames — the rendering and cache
systems detect newly spawned entities and react. Steps 1-4, 7, 10 are explicit in `handle_load`.
Step 3 (CSR graph rebuild) happens via a system that detects changes to `RoadNetwork`.

---

*Document generated from analysis of `crates/save/src/lib.rs` and `crates/save/src/serialization.rs`
in the Megacity codebase. All code references are to the current implementation as of the latest
commit on main.*
