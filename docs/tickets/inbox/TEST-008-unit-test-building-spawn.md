# TEST-008: Unit Tests for Building Spawner Logic

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test building spawner: only spawns on zoned road-adjacent cells with utilities, respects zone type, does not spawn when demand <= 0.

## Acceptance Criteria
- [ ] Test building spawns only on zoned cells
- [ ] Test building requires road adjacency
- [ ] Test building requires power and water
- [ ] Test no spawn when demand <= 0
- [ ] Test correct zone type matches building zone type
