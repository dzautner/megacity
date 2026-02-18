# SVC-001: Hybrid Service Coverage Model

**Priority:** T2 (Depth)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 6.4, master_architecture.md Section 1.10

## Description

Replace binary Euclidean radius coverage with hybrid model combining: (1) road-network distance BFS for static coverage (not crow-flies), (2) capacity constraints (staff, equipment per station), (3) quality factor from funding level, (4) demand pressure (demand / capacity ratio). Effective service at cell = proximity * capacity * quality * demand_ratio. Fire station across river with no bridge should provide zero coverage. Over-capacity service degrades quality.

## Definition of Done

- [ ] `ServiceCoverageQuality` grid (f32 per cell per service category)
- [ ] Road-network BFS distance from each service building (not Euclidean)
- [ ] Distance decay: coverage = 1.0 at station, 0.0 at max road distance
- [ ] Capacity tracking per service building (beds, vehicles, staff)
- [ ] Quality factor from budget level (0.5 to 1.5)
- [ ] Demand/capacity ratio tracked per building
- [ ] Over-capacity degrades quality proportionally
- [ ] Effective service = proximity * capacity * quality * demand_ratio
- [ ] Backward compatible: bitflag grid still available for binary checks

## Test Plan

- Unit test: station across river with no bridge = zero coverage
- Unit test: over-capacity hospital has degraded quality
- Unit test: budget reduction lowers quality factor
- Integration test: road-network coverage visibly different from Euclidean
- Performance test: BFS coverage computation < 50ms for 50 service buildings

## Pitfalls

- Road-network BFS is O(V+E) per service building; cache results, only recompute on road change
- f32 coverage grid per service category = 8 * 256 * 256 * 4 = 2MB; acceptable
- Must maintain bitflag grid for happiness system compatibility during transition

## Relevant Code

- `crates/simulation/src/happiness.rs` (ServiceCoverageGrid, lines 19-88)
- `crates/simulation/src/services.rs` (ServiceBuilding, coverage_radius)
