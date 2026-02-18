# TEST-010: Unit Tests for Grid Operations

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test grid operations: world_to_grid, grid_to_world, neighbors4, neighbors8, in_bounds. Test edge/corner cases.

## Acceptance Criteria
- [ ] Test world_to_grid roundtrip with grid_to_world
- [ ] Test neighbors4 at center (4 neighbors)
- [ ] Test neighbors4 at corner (2 neighbors)
- [ ] Test neighbors8 at center (8 neighbors)
- [ ] Test in_bounds rejects out-of-range indices
- [ ] Test boundary cells (0, 0) and (255, 255)
