# TEST-009: Unit Tests for Citizen State Machine

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test citizen state transitions: AtHome -> CommutingToWork -> AtWork -> CommutingHome -> AtHome. Verify transitions follow time-of-day rules.

## Acceptance Criteria
- [ ] Test AtHome transitions to CommutingToWork at work hour
- [ ] Test CommutingToWork transitions to AtWork on arrival
- [ ] Test AtWork transitions to CommutingHome at end of day
- [ ] Test CommutingHome transitions to AtHome on arrival
- [ ] Test citizen without job stays AtHome
- [ ] Test citizen without home enters Wandering
