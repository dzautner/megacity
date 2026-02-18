# UI-003: Data-Driven Game Parameters
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M3

## Description
Externalize game parameters (building stats, road configs, service costs, tax defaults) into data files (TOML/JSON/RON) instead of hardcoded Rust constants. This enables easier balancing, modding preparation, and testing.

- Building capacity table: separate data file per zone type
- Road type stats: speed, cost, lanes, noise_radius, zoning_allowed
- Service building stats: radius, cost, capacity
- Tax defaults: rates per zone/bracket
- Milestone unlocks: population thresholds and unlock lists
- Hot-reload in debug builds (change data file, reload without recompile)

## Definition of Done
- [ ] Building stats loaded from data files
- [ ] Road stats loaded from data files
- [ ] Service stats loaded from data files
- [ ] Hot-reload in debug mode
- [ ] Default data files included in game assets

## Test Plan
- Unit: Changing data file value changes in-game behavior
- Integration: Hot-reload building capacity, verify building spawns with new capacity

## Pitfalls
- Must maintain compile-time defaults as fallback if data file missing
- RON format integrates well with Bevy's asset system
- Performance: data files loaded once at startup, not per-frame

## Relevant Code
- `crates/simulation/src/config.rs` -- currently just constants
- `crates/simulation/src/buildings.rs:Building::capacity_for_level` -- move to data
- `crates/simulation/src/grid.rs:RoadType` -- move stats to data
