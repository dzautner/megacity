# INFRA-135: Remove or Implement Contraction Hierarchy
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** none
**Source:** MEMORY.md known issues

## Description
The `contraction_hierarchy.rs` module is dead code (exists but is never used). Either implement contraction hierarchies for faster long-distance pathfinding, or remove the module to reduce code complexity. If keeping: CH precomputes shortcuts for hierarchical routing, O(log n) queries vs O(n) for A*. If removing: delete module and remove from lib.rs.

## Definition of Done
- [ ] Decision: implement or remove
- [ ] If remove: module deleted, no references remain
- [ ] If implement: CH builds from CSR graph, query function works, benchmarked faster than A*
- [ ] Tests pass

## Test Plan
- Unit (if implementing): CH path matches A* path for same OD pair
- Unit (if removing): cargo build succeeds without module

## Pitfalls
- CH requires full graph rebuild when topology changes (road placement)
- For 256x256 grid, A* may be fast enough; CH benefits larger maps
- If removing, check no other module references it

## Relevant Code
- `crates/simulation/src/contraction_hierarchy.rs` -- dead code module
- `crates/simulation/src/lib.rs` -- module registration
