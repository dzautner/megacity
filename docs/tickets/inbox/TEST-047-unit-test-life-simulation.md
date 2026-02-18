# TEST-047: Unit Tests for Life Simulation (Aging, Death, Education)

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test life events: aging increments age, death at old age, education level progression, marriage, children. Verify all rates are within expected ranges.

## Acceptance Criteria
- [ ] Test aging: age increments by 1 per aging tick
- [ ] Test death probability increases with age
- [ ] Test education progression: child -> student -> graduated
- [ ] Test citizens with homes reference valid buildings
