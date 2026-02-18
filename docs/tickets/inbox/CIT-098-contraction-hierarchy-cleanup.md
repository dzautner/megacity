# CIT-098: Remove Dead Code -- Contraction Hierarchy

**Priority:** T1 (Core -- cleanup)
**Complexity:** Low (0.25 person-weeks)
**Dependencies:** None
**Source:** MEMORY.md Known Issues

## Description

`contraction_hierarchy.rs` is dead code -- module exists but is never used (never registered, never called). Remove or integrate. If the contraction hierarchy was intended for faster pathfinding, either implement it properly or remove it to reduce code maintenance burden.

## Definition of Done

- [ ] Evaluate if contraction hierarchy provides value
- [ ] If yes: integrate into pathfinding system, benchmark vs CSR A*
- [ ] If no: remove module entirely
- [ ] Remove any unused imports/references
- [ ] Compile without warnings

## Test Plan

- Compile test: no dead code warnings
- If integrated: pathfinding benchmark comparison

## Pitfalls

- May have been left intentionally for future use; verify with codebase history

## Relevant Code

- `crates/simulation/src/contraction_hierarchy.rs`
- `crates/simulation/src/lib.rs` (module registration)
