# TEST-054: Integration Test: Economy Tax Collection

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 3: Integration Testing

## Description
Full integration test: citizens exist and are employed -> run simulation through tax collection day -> verify treasury increases.

## Acceptance Criteria
- [ ] Set up city with employed citizens
- [ ] Run ticks through at least one tax collection cycle
- [ ] Verify treasury increased
- [ ] Verify monthly_income > 0
- [ ] Verify expenses deducted for active services
