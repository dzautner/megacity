# TEST-055: Integration Test: Utility Loss -> Abandonment Chain

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 3: Integration Testing

## Description
Test the chain: remove power/water from building -> abandonment triggers -> citizens evicted -> building marked abandoned.

## Acceptance Criteria
- [ ] Building with citizens has power and water
- [ ] Remove power and water
- [ ] Run sufficient ticks for abandonment to trigger
- [ ] Building becomes abandoned
- [ ] Citizens evicted (occupants = 0)
