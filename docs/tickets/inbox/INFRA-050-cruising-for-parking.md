# INFRA-050: Cruising for Parking Simulation
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-049
**Source:** transportation_simulation.md, Section 6.2

## Description
Model cruising for parking behavior. When parking occupancy > 85% in an area, search time increases exponentially. At 95%+, nearly impossible to find parking. Cruising vehicles add traffic volume (30% of downtown traffic can be cruising). Formula: `effective_search_time = 0.5 + avg_search_time * ((occupancy - 0.85) / 0.15)^2` for occupancy > 0.85. Add cruising vehicle count to road traffic volumes near destinations.

## Definition of Done
- [ ] Per-area parking occupancy tracking
- [ ] Cruising traffic computation when occupancy > 85%
- [ ] Cruising vehicles added to road traffic volumes
- [ ] 85% occupancy threshold as the critical point
- [ ] Tests pass

## Test Plan
- Unit: At 90% occupancy, search time increases significantly
- Unit: At 70% occupancy, minimal cruising traffic

## Pitfalls
- "Area" needs definition (per block? per district?)
- Cruising traffic is a positive feedback loop (more cruising = more congestion = more cruising)
- Double-parking as alternative when no spots found

## Relevant Code
- `crates/simulation/src/traffic.rs` -- additional traffic volume
- `crates/simulation/src/districts.rs` -- area-based aggregation
