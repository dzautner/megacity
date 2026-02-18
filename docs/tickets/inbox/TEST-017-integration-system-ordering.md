# TEST-017: Integration Test: System Ordering Dependencies

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 3.4: System Ordering

## Description
Verify that system ordering constraints are correct: traffic updates before happiness reads congestion, service coverage updates before happiness reads coverage.

## Acceptance Criteria
- [ ] Test: set traffic density, run ticks, happiness reflects congestion penalty
- [ ] Test: place hospital, run ticks, coverage grid has health flag
- [ ] Test: service coverage available to happiness system same tick
