# POLL-005: Water Pollution Downstream Flow Model (D8 Algorithm)

## Priority: T2 (Depth)

## Description
Replace the current simple diffusion-based water pollution model with a terrain-aware downstream flow model using the D8 (steepest descent) algorithm. Currently water pollution only spreads between adjacent water cells from higher to lower pollution. The research doc specifies flow direction precomputation, topological sort, and downstream propagation with transfer rates.

## Current State
- `WaterPollutionGrid` uses simple neighbor diffusion (1/8 of pollution to each cardinal neighbor).
- Diffusion only occurs between water cells (CellType::Water).
- No terrain slope consideration for flow direction.
- No topological sort for upstream-to-downstream processing order.
- No distinction between stream cells, sink cells, and river cells.

## Definition of Done
- [ ] Precompute `flow_dir[256][256]` using D8 algorithm (steepest descent to neighbors8).
- [ ] Precompute topological sort order using Kahn's algorithm on the flow DAG.
- [ ] Propagate pollution in topological order with `FLOW_TRANSFER_RATE = 0.7`.
- [ ] Sink cells (no lower neighbor) accumulate pollution with `SINK_DECAY = 0.98`.
- [ ] Natural decay: `STREAM_DECAY = 0.90` for all cells, extra `RIVER_DECAY = 0.95` for river/stream cells.
- [ ] Recompute flow directions when terrain changes (rare event).
- [ ] Clamp values to `[0.0, MAX_WATER_POLLUTION = 500.0]`.

## Test Plan
- [ ] Unit test: D8 flow direction points to steepest downhill neighbor.
- [ ] Unit test: topological sort processes upstream cells before downstream.
- [ ] Integration test: pollution placed at a high-elevation cell reaches the river downstream.
- [ ] Integration test: pollution accumulates in terrain sinks (lakes/ponds).
- [ ] Performance test: full grid propagation completes within 5ms.

## Pitfalls
- Current terrain is flat (y=0.0 rendering), but `Cell.elevation` exists in the grid data.
- Flow direction precomputation must handle flat areas (no steepest descent) gracefully.
- Topological sort must handle cycles (should not exist in D8, but guard against it).
- Requires wider type than u8 for the 0-500 range.

## Code References
- `crates/simulation/src/water_pollution.rs`: `WaterPollutionGrid`, `update_water_pollution`
- `crates/simulation/src/grid.rs`: `WorldGrid`, `Cell.elevation`
- `crates/simulation/src/groundwater.rs`: existing groundwater model
- Research: `environment_climate.md` sections 1.2.3, 1.2.4
