# TEST-062: Integration Test: Negative Budget Consequences

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 3: Integration Testing

## Description
Test that negative treasury triggers consequences: service degradation, potential building abandonment, inability to place new infrastructure.

## Acceptance Criteria
- [ ] Set treasury to large negative value
- [ ] Run ticks
- [ ] Verify service quality degrades or buildings placed are rejected
- [ ] Verify game does not crash with negative treasury
