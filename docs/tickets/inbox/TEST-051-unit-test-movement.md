# TEST-051: Unit Tests for Citizen Movement System

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test citizen movement: velocity application, path following, waypoint progression, arrival detection, position clamping to grid bounds.

## Acceptance Criteria
- [ ] Test velocity moves citizen position
- [ ] Test path following progresses through waypoints
- [ ] Test arrival detection when close to waypoint
- [ ] Test position stays within grid bounds
- [ ] Test PathCache index <= waypoints.len()
