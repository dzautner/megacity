# MISC-001: Remove or Implement Contraction Hierarchy
**Priority:** T1
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section 5.1 (known issue)

## Description
contraction_hierarchy.rs is dead code (module exists but never used, per MEMORY.md). Either implement it as a pathfinding optimization or remove it to reduce maintenance burden.

Options:
A) Remove: delete module, clean up any references
B) Implement: use CH as preprocessing step for long-distance queries, keeping CSR A* for short paths

Recommendation: Remove for now, add back when pathfinding becomes a bottleneck.

## Definition of Done
- [ ] Dead code removed or functional implementation complete
- [ ] No compilation warnings related to unused module
- [ ] Pathfinding performance unchanged (if removed) or improved (if implemented)

## Test Plan
- Build: project compiles cleanly without dead code warnings
- Benchmark: pathfinding performance unchanged after removal

## Pitfalls
- Check for any imports or references before removal
- If implementing, CH preprocessing is expensive (minutes for large graphs)

## Relevant Code
- `crates/simulation/src/contraction_hierarchy.rs` -- the dead code
- `crates/simulation/src/lib.rs` -- module registration
