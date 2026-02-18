# TEST-053: Unit Tests for SpatialIndex (DestinationCache)

## Priority: T1 (Core)
## Effort: Small (0.5 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test SpatialIndex on DestinationCache: nearest lookup returns closest entity, empty index returns None, bounds checking.

## Acceptance Criteria
- [ ] Test nearest lookup returns closest destination
- [ ] Test empty index returns None
- [ ] Test with multiple destinations, correct closest returned
- [ ] Test all destinations within radius found
