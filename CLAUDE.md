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
- Systems registered in `crates/*/src/lib.rs` Plugin impls
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
- All new components/resources MUST be added to the save system in `crates/save/`

## PR Requirements
- All code must compile: `cargo build --workspace`
- All tests must pass: `cargo test --workspace`
- No clippy warnings: `cargo clippy --workspace -- -D warnings`
- Code must be formatted: `cargo fmt --all`
- PR description must reference the issue number with "Closes #N"
