# TEST-029: Performance Budget Tests (Fail on Regression)

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 5.4: Performance Budgets

## Description
Integration tests that fail if performance budgets are exceeded: full tick < 16ms at 100K, pathfinding < 1ms, save < 1s, load < 3s.

## Acceptance Criteria
- [ ] Test asserts full tick < 16ms at 100K citizens
- [ ] Test asserts single A* pathfinding < 1ms
- [ ] Test asserts save to disk < 1s
- [ ] Test asserts load from disk < 3s
- [ ] Tests tagged `#[ignore]` for CI-only (slow)
