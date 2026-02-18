# POLL-032: Environment Plugin System Registration and Scheduling

## Priority: T1 (Core)

## Description
Create the `EnvironmentPlugin` that registers all environmental systems with correct update frequencies, parallel groups, and run conditions. This is the top-level organizational ticket for integrating all environmental systems into Bevy's ECS scheduler.

## Current State
- Environmental systems are registered individually in `SimulationPlugin`.
- No dedicated `EnvironmentPlugin`.
- No frequency-based scheduling (systems run every tick or on slow tick timer).

## Definition of Done
- [ ] `EnvironmentPlugin` struct implementing `Plugin` for Bevy.
- [ ] System groups by frequency: every-tick (fire, disaster), every-4-ticks (air pollution, energy), every-8-ticks (water pollution, noise), every-30-ticks (soil, UHI), daily (waste, water demand), yearly (recycling market, climate).
- [ ] Run conditions: `every_n_ticks(n)`, `on_game_hour`, `on_game_day`, `on_game_year`, event-driven conditions.
- [ ] Resource initialization: all grids, weather, energy, waste, water resources initialized with defaults.
- [ ] Parallel system groups correctly annotated for Bevy scheduler.
- [ ] Plugin added to app in `main.rs`.

## Test Plan
- [ ] Unit test: all resources initialized to correct defaults.
- [ ] Integration test: systems run at their specified frequencies.
- [ ] Integration test: parallel groups don't cause data races.
- [ ] Performance test: system scheduling overhead is negligible.

## Pitfalls
- Must not break existing system registration in `SimulationPlugin`.
- Transition from existing pollution/weather systems to new plugin must be gradual.
- Run conditions must be deterministic for replay/save consistency.

## Code References
- `crates/simulation/src/lib.rs`: `SimulationPlugin`
- `crates/app/src/main.rs`: plugin registration
- Research: `environment_climate.md` section 8.2
