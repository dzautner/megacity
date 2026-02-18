# TEST-004: Unit Tests for CSR Pathfinding

## Priority: T1 (Core)
## Effort: Small (1-2 days)
## Source: testing_strategy.md -- Section 2: Unit Testing

## Description
Test A* pathfinding on CSR graph: simple paths, no-path cases, single-node graph, disconnected components. Verify path cost monotonicity and correctness.

## Acceptance Criteria
- [ ] Test straight-line path returns correct waypoints
- [ ] Test path around obstacle
- [ ] Test no-path-exists returns None
- [ ] Test single-node graph
- [ ] Test disconnected components
- [ ] Test path cost is non-decreasing along path
- [ ] Test traffic-aware path prefers less congested routes
