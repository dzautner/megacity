# TEST-038: Unit Tests for Road Segment (Bezier) Operations

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test Bezier segment operations: rasterization to grid, intersection detection between segments, segment splitting at intersection points.

## Acceptance Criteria
- [ ] Test linear Bezier rasterizes to expected cells
- [ ] Test curved Bezier rasterizes within expected bounds
- [ ] Test intersection detection between crossing segments
- [ ] Test segment splitting produces valid sub-segments
- [ ] Test segment length calculation
