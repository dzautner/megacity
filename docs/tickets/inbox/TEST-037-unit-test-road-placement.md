# TEST-037: Unit Tests for Road Placement

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test road placement: cell type changes to Road, road type set correctly, neighbors updated, road network updated, CSR graph rebuilt.

## Acceptance Criteria
- [ ] Test place_road changes cell_type to Road
- [ ] Test road_type matches requested type
- [ ] Test road network has edge for placed road
- [ ] Test adjacent cells become road-adjacent
- [ ] Test place road on water fails
