# INFRA-066: Fix PathCache/Velocity Serialization
**Priority:** T0
**Complexity:** S (hours)
**Dependencies:** none
**Source:** master_architecture.md, M2; MEMORY.md known issues

## Description
Fix the known bug where `PathCache` and `Velocity` components are not serialized. Commuting citizens lose their paths on save/load and teleport or get stuck. Either serialize the full path data, or mark citizens as "needs repath" on load so they recalculate their route on the next tick.

## Definition of Done
- [ ] Either serialize PathCache+Velocity OR reset citizen to "needs repath" state on load
- [ ] No stuck citizens after load
- [ ] No teleporting after load
- [ ] Tests pass

## Test Plan
- Unit: Save while citizen is mid-commute, load, citizen continues or re-paths
- Integration: Load a city with 10K citizens, none are stuck

## Pitfalls
- Full path serialization increases save file size significantly
- "Needs repath" approach causes a spike in pathfinding load on first tick after load
- Some citizen states (on bus, in building) may not need path restoration

## Relevant Code
- `crates/simulation/src/movement.rs` -- PathCache, Velocity components
- `crates/save/src/serialization.rs` -- citizen serialization
