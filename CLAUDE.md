# Megacity - Bevy ECS City Builder

## Build & Test Commands
- Build: `cargo build --workspace`
- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace -- -D warnings`
- Format: `cargo fmt --all`
- Format check: `cargo fmt --all -- --check`

## Architecture
- Bevy ECS game engine (Rust), workspace at `crates/` with: simulation, rendering, save, ui, app
- 256x256 grid (CELL_SIZE=16.0, CHUNK_SIZE=8) with road segments overlay
- Road system: `RoadSegmentStore` (Bezier curves) is source of truth; grid is derived cache
- CSR graph for pathfinding, built from segments when available, grid fallback otherwise
- LOD system: Full/Simplified/Abstract tiers for citizens

## Key Conventions
- Each feature module defines its own `Plugin` struct with `init_resource`/`add_systems` -- do NOT add these to `simulation/src/lib.rs`
- Grid types in `simulation/src/grid.rs` (WorldGrid, Cell, RoadType, ZoneType)
- Road segments in `simulation/src/road_segments.rs` (Bezier curves, intersection detection)
- Pathfinding in `simulation/src/road_graph_csr.rs` (CSR A* with traffic-aware routing)
- Citizen state machine in `simulation/src/movement.rs`
- Economy in `simulation/src/economy.rs` + `simulation/src/budget.rs`
- `RoadType::half_width()` centralizes road widths (never hardcode)
- `neighbors8()` exists for 8-connectivity (diagonal support)
- `RoadSegmentStore` rasterizes to grid AND adds to RoadNetwork
- Traffic-aware pathfinding via `csr_find_path_with_traffic()`
- `SpatialIndex` on `DestinationCache` for O(1) nearest lookups

## Auto-Module Discovery
Modules in `simulation`, `rendering`, and `ui` crates are auto-discovered at compile time using `automod_dir::dir!()`. This means:
- **You do NOT need to add `pub mod my_feature;` to `lib.rs`** — just create the file and it's automatically included
- **You do NOT need to add `mod my_test;` to `integration_tests/mod.rs`** — just create the test file
- Only `plugin_registration.rs` (private), `integration_tests` (cfg-gated), and `test_harness` (cfg-gated) are declared manually
- The `save` crate still uses manual `mod` declarations due to mixed visibility and conditional compilation

## Adding New Features (Conflict-Free Pattern)
New features should NOT touch shared files. Follow this pattern:
1. Create your module file (e.g., `simulation/src/my_feature.rs`) — it is auto-discovered, NO lib.rs edit needed
2. Define a `Plugin` struct in your module with all `init_resource`/`add_systems` calls
3. Add your plugin to `simulation/src/plugin_registration.rs` (one plugin per line, no tuples)
4. For saveable state, implement the `Saveable` trait and call `app.register_saveable::<MyState>()` in your plugin -- do NOT modify `save_types.rs`, `serialization.rs`, `save_restore.rs`, or `save_helpers.rs`

## Code Modularity (Treat This as a 100+ Contributor Project)
This project is structured for high parallelism — many agents/contributors working simultaneously on separate features. **Every file should be small, focused, and conflict-resistant.**

### File Size Limits
- **Hard limit: 500 lines per file.** If a file exceeds 500 lines, split it before adding more code.
- **Target: 200–400 lines.** This is the sweet spot for readability, reviewability, and merge-conflict avoidance.
- **Shared files (lib.rs, mod.rs, plugin_registration.rs):** Keep as thin as possible — just `pub mod`, `pub use`, and delegation calls. No logic.

### Single Responsibility
- Each `.rs` file should do ONE thing. If you can't describe the file's purpose in one sentence, it's too big.
- Feature logic, tests, and registration are separated: feature code in `my_feature.rs`, tests in `integration_tests/my_feature.rs`, plugin registration in `plugin_registration.rs`.
- Never put unrelated functionality in the same file just because "it's small" or "it's convenient."

### Avoiding Merge Conflicts
- **Module declarations are auto-discovered** — `lib.rs` and `integration_tests/mod.rs` use `automod_dir::dir!()` so new modules never cause conflicts.
- **Registration files** (`plugin_registration.rs`, `saveable_keys.rs`) use one entry per line — never group into tuples or multi-item blocks.
- **Test files:** ONE file per feature domain. Never add tests to someone else's test file — create a new one.
- **Large structs or enums** that many features extend: put each variant/field on its own line.

### When to Split a File
Split a file if any of these are true:
1. It exceeds 500 lines
2. It has more than one "reason to change" (e.g., it handles both simulation logic AND rendering)
3. Multiple PRs frequently conflict on it
4. It contains both public API and private implementation details that could be separated
5. It has more than 3 logically distinct sections separated by comment headers

### How to Split
- Extract into a sibling file in the same directory (e.g., `economy.rs` -> `economy.rs` + `economy_taxes.rs`)
- Or use a subdirectory with `mod.rs` (e.g., `economy/mod.rs` + `economy/taxes.rs` + `economy/budget.rs`)
- Re-export from the parent so callers don't need to change their imports
- The original file should become a thin facade: just `pub mod` + `pub use` statements

## PR Requirements
- All code must compile: `cargo build --workspace`
- All tests must pass: `cargo test --workspace`
- No clippy warnings: `cargo clippy --workspace -- -D warnings`
- Code must be formatted: `cargo fmt --all`
- PR description must reference the issue number with "Closes #N"

## Integration Test Plans
- Every PR that adds or modifies simulation behavior MUST include integration tests using `TestCity`
- Test harness: `simulation/src/test_harness.rs` provides `TestCity` builder for headless Bevy App tests
- Integration test files live in: `simulation/src/integration_tests/` (one file per feature domain)
- New tests go in a NEW file: `simulation/src/integration_tests/<feature>_tests.rs` — it is auto-discovered, NO mod.rs edit needed
- Do NOT append tests to existing test files — create a new file to avoid merge conflicts
- Test pattern:
  1. Set up city state using `TestCity::new()` builder methods (roads, zones, buildings, citizens, etc.)
  2. Run simulation ticks with `tick()`, `tick_slow_cycle()`, or `tick_slow_cycles()`
  3. Assert expected outcomes using query methods and assertion helpers
- Name tests descriptively: `test_<feature>_<scenario>_<expected_outcome>`
- Use `TestCity::with_tel_aviv()` for smoke/regression tests against the full map
