# TEST-066: Remove or Integrate Dead Code: contraction_hierarchy.rs

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: MEMORY.md -- Known Remaining Issues

## Description
`contraction_hierarchy.rs` is dead code (module exists but is never used). Either remove it or integrate it as a pathfinding optimization. Dead code increases maintenance burden.

## Acceptance Criteria
- [ ] Module either removed or integrated
- [ ] If removed: no references remain in codebase
- [ ] If integrated: tests added for correctness
- [ ] No unused code warnings
