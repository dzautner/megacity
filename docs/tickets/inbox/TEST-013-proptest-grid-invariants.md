# TEST-013: Property-Based Tests for Grid Invariants

## Priority: T2 (Depth)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 2.4: Property-Based Testing

## Description
Use proptest to verify grid indices always in bounds, world_to_grid produces valid indices, neighbors produce only in-bounds results.

## Acceptance Criteria
- [ ] For any world coordinates, world_to_grid produces valid index
- [ ] For any valid grid index, neighbors are all in bounds
- [ ] For any cell, building_id references are valid or None
