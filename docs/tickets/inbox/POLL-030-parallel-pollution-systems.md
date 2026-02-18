# POLL-030: Parallel System Scheduling for Independent Pollution Grids

## Priority: T2 (Depth)

## Description
Configure Bevy's system scheduler to run independent pollution systems in parallel. Air, water, noise, and soil pollution systems all read from source lists and write to separate grids -- they can run simultaneously.

## Current State
- Systems run sequentially in the Update schedule.
- No explicit parallel grouping.
- Bevy can auto-parallelize if resources don't conflict.

## Definition of Done
- [ ] Pollution systems in parallel group A: `air_pollution_update`, `water_pollution_propagation`, `noise_grid_update`, `soil_contamination_update`.
- [ ] Each system only writes to its own grid (no cross-grid writes in phase A).
- [ ] Infrastructure systems in parallel group B: `energy_dispatch`, `water_supply`, `waste_collection` (reads pollution).
- [ ] Sequential group C: health effects, happiness (reads all pollution grids).
- [ ] `.add_systems(Update, (air, water, noise, soil).run_if(every_n_ticks(4)))` with correct resource annotations.
- [ ] Benchmark: measure parallel speedup with 4+ cores.

## Test Plan
- [ ] Unit test: all systems produce identical results in parallel vs sequential.
- [ ] Benchmark: 4 pollution systems in parallel complete in ~1x time of 1 system (vs 4x).
- [ ] Integration test: no race conditions or data corruption.

## Pitfalls
- Bevy requires correct `Res`/`ResMut` annotations for automatic parallelization.
- Systems that share mutable resources cannot run in parallel.
- Must verify no hidden shared state between pollution systems.

## Code References
- `crates/simulation/src/lib.rs`: system registration
- Research: `environment_climate.md` section 9.4
