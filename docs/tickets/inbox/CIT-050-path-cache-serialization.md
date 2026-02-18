# CIT-050: Fix PathCache/Velocity Serialization

**Priority:** T1 (Core -- bugfix)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.17, MEMORY.md Known Issues

## Description

`PathCache` and `Velocity` are not serialized, causing commuting citizens to lose their paths on save/load. Citizens mid-commute when the game is saved will be stuck at their current position without a path after load. Fix by either: (a) serializing PathCache waypoints and Velocity, or (b) on load, resetting all commuting citizens to AtHome state and re-requesting paths. Option (b) is simpler and avoids serializing large waypoint vectors.

## Definition of Done

- [ ] Commuting citizens reset to AtHome on load (option b)
- [ ] Or: PathCache waypoints serialized (option a)
- [ ] Velocity reset to zero on load
- [ ] Citizens re-request paths from their current state
- [ ] No citizens stuck mid-road after load

## Test Plan

- Integration test: save while citizens commuting -> load -> all citizens resume normal behavior
- Integration test: no citizens permanently stuck at road positions

## Pitfalls

- Resetting to AtHome may cause a brief traffic spike as everyone re-commutes
- PathCache contains Entity references (RoadNode might have Entity); check

## Relevant Code

- `crates/simulation/src/citizen.rs` (PathCache, Velocity)
- `crates/save/src/serialization.rs`
- `crates/simulation/src/movement.rs`
