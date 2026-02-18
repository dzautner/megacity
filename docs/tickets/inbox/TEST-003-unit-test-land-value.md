# TEST-003: Unit Tests for Land Value Calculation

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test land value with controlled inputs: road accessibility, services, pollution, noise, crime. Verify output in [0, 255]. Test that each input factor affects value in correct direction.

## Acceptance Criteria
- [ ] Test base land value on grass cell
- [ ] Test road proximity bonus
- [ ] Test service coverage bonus
- [ ] Test pollution penalty
- [ ] Test noise penalty
- [ ] Test crime penalty
- [ ] Verify output clamped to [0, 255]
