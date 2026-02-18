# TEST-002: Unit Tests for Happiness Formula

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test happiness calculation with controlled inputs. Verify each factor (commute, services, pollution, crime, noise) contributes correctly. Verify output clamped to [0.0, 100.0].

## Acceptance Criteria
- [ ] Test happiness with all positive factors = high happiness
- [ ] Test happiness with all negative factors = low happiness
- [ ] Test each factor independently (toggle one, check delta)
- [ ] Verify output always in [0.0, 100.0]
- [ ] Test with extreme values (all services, max pollution)
