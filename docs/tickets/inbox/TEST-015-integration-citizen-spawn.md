# TEST-015: Integration Test: Citizens Spawn in Completed Buildings

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 3.2: Bevy Integration Tests

## Description
Set up completed residential and commercial buildings. Run 50 ticks. Verify citizens spawn with HomeLocation and WorkLocation.

## Acceptance Criteria
- [ ] Completed ResidentialLow building with capacity > 0
- [ ] Completed CommercialLow building nearby
- [ ] ZoneDemand.residential = 1.0
- [ ] After 50 ticks, citizen_count > 0
- [ ] Citizens have valid HomeLocation
