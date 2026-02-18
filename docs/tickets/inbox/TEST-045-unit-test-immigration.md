# TEST-045: Unit Tests for Immigration/Emigration

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test immigration attractiveness calculation. Verify high happiness + jobs + services = positive immigration. Low happiness = emigration.

## Acceptance Criteria
- [ ] Test high attractiveness -> positive immigration rate
- [ ] Test low attractiveness -> emigration
- [ ] Test immigration rate scales with available housing
- [ ] Test no immigration when no housing available
